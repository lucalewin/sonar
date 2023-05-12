use crate::{
    audio::WavData, streaming::rwstream::ChannelStream, streaming::StreamingFormat, CLIENTS,
    CONFIG,
};
use crossbeam_channel::{unbounded, Receiver, Sender};
use log::{debug, error, info};
use std::net::IpAddr;
use tiny_http::{Header, Method, Request, Response, Server};

/// run a tiny-http webserver to serve streaming requests from renderers
///
/// all music is sent in audio/l16 PCM format (i16) with the sample rate of the source
/// the samples are read from a crossbeam channel fed by the wave_reader
/// a ChannelStream is created for this purpose, and inserted in the array of active
/// "clients" for the wave_reader
pub fn start(local_addr: &IpAddr, server_port: u16, wav_data: WavData) {
    let addr = format!("{local_addr}:{server_port}");
    info!("The streaming server is listening on http://{addr}/stream/swyh.wav");
    let cfg = CONFIG.read();
    debug!(
        "Streaming sample rate: {}, bits per sample: {}, format: {}",
        wav_data.sample_rate.0,
        cfg.bits_per_sample.unwrap(),
        cfg.streaming_format.unwrap(),
    );

    let server = Server::http(addr).unwrap();

    for req in server.incoming_requests() {
        // start streaming in a new thread and continue serving new requests
        std::thread::spawn(move || handle_request(req, wav_data));
    }
}

fn handle_request(req: Request, wav_data: WavData) {
    if cfg!(debug_assertions) {
        debug!("[{}] Incoming {:?}", req.remote_addr().unwrap(), req);
        for hdr in req.headers() {
            debug!("[{}]  - {:?}", req.remote_addr().unwrap(), hdr);
        }
    }
    // get remote ip
    let remote_addr = format!("{}", req.remote_addr().unwrap());
    let mut remote_ip = remote_addr.clone();
    if let Some(i) = remote_ip.find(':') {
        remote_ip.truncate(i);
    }
    // default headers
    let srvr_hdr =
        Header::from_bytes(&b"Server"[..], &b"UPnP/1.0 DLNADOC/1.50 LAB/1.0"[..]).unwrap();
    let nm_hdr = Header::from_bytes(&b"icy-name"[..], &b"swyh-rs"[..]).unwrap();
    let cc_hdr = Header::from_bytes(&b"Connection"[..], &b"close"[..]).unwrap();
    let acc_rng_hdr = Header::from_bytes(&b"Accept-Ranges"[..], &b"none"[..]).unwrap();

    // check url
    if req.url() != "/stream/swyh.wav" {
        error!(
            "Unrecognized request '{}' from {}'",
            req.url(),
            req.remote_addr().unwrap()
        );
        let response = Response::empty(404)
            .with_header(cc_hdr)
            .with_header(srvr_hdr)
            .with_header(nm_hdr);
        if let Err(e) = req.respond(response) {
            error!("HTTP POST connection with {remote_addr} terminated [{e}]");
        }
        return;
    }

    // prepare streaming headers
    let conf = CONFIG.read().clone();
    let format = conf.streaming_format.unwrap();

    let ct_text = match format {
        StreamingFormat::Flac => "audio/flac",
        StreamingFormat::Wav => "audio/vnd.wave;codec=1",
        StreamingFormat::Lpcm => todo!("implement the code below!"), // if conf.bits_per_sample == Some(16) {
                                                                     //     format!("audio/L16;rate={};channels=2", wav_data.sample_rate.0).as_str()
                                                                     // } else {
                                                                     //     format!("audio/L24;rate={};channels=2", wav_data.sample_rate.0).as_str()
                                                                     // }
    };
    let ct_hdr = Header::from_bytes(&b"Content-Type"[..], ct_text.as_bytes()).unwrap();
    let tm_hdr = Header::from_bytes(&b"TransferMode.DLNA.ORG"[..], &b"Streaming"[..]).unwrap();

    match req.method() {
        Method::Get => {
            debug!(
                "Received request {} from {}",
                req.url(),
                req.remote_addr().unwrap()
            );
            // set transfer encoding chunked unless disabled
            let (streamsize, chunked_threshold) = {
                if conf.disable_chunked {
                    (Some(usize::MAX - 1), usize::MAX)
                } else {
                    (None, 8192)
                }
            };
            let (tx, rx): (Sender<Vec<f32>>, Receiver<Vec<f32>>) = unbounded();
            let channel_stream = ChannelStream::new(
                tx,
                rx,
                remote_ip.clone(),
                conf.use_wave_format,
                wav_data.sample_rate.0,
                conf.bits_per_sample.unwrap(),
                conf.streaming_format.unwrap(),
            );
            let nclients = {
                let mut clients = CLIENTS.write();
                clients.insert(remote_addr.clone(), channel_stream.clone());
                clients.len()
            };
            debug!("Now have {} streaming clients", nclients);

            let streaming_format = match format {
                StreamingFormat::Flac => "audio/FLAC",
                StreamingFormat::Wav => "audio/wave;codec=1 (WAV)",
                StreamingFormat::Lpcm => {
                    if conf.bits_per_sample == Some(16) {
                        "audio/L16 (LPCM)"
                    } else {
                        "audio/L24 (LPCM)"
                    }
                }
            };

            debug!(
                "Streaming {streaming_format}, input sample format {:?}, channels=2, rate={}, disable chunked={} to {}",
                wav_data.sample_format,
                wav_data.sample_rate.0,
                conf.disable_chunked,
                req.remote_addr().unwrap()
            );
            let response = Response::empty(200)
                .with_data(channel_stream, streamsize)
                .with_chunked_threshold(chunked_threshold)
                .with_header(cc_hdr)
                .with_header(ct_hdr)
                .with_header(tm_hdr)
                .with_header(srvr_hdr)
                .with_header(acc_rng_hdr)
                .with_header(nm_hdr);
            let e = req.respond(response);
            if e.is_err() {
                error!("HTTP connection with {remote_addr} terminated [{e:?}]");
            }
            let nclients = {
                let mut clients = CLIENTS.write();
                if let Some(chs) = clients.remove(&remote_addr) {
                    chs.stop_flac_encoder()
                };
                clients.len()
            };
            debug!("Now have {} streaming clients left", nclients);
            info!("Streaming to {remote_addr} has ended");
        }
        Method::Post => {
            debug!("POST request from {}", remote_addr);
            let response = Response::empty(200)
                .with_header(cc_hdr)
                .with_header(srvr_hdr)
                .with_header(nm_hdr);
            if let Err(e) = req.respond(response) {
                error!("HTTP POST connection with {remote_addr} terminated [{e}]");
            }
        }
        Method::Head => {
            debug!("HEAD request from {}", remote_addr);
            let response = Response::empty(200)
                .with_header(cc_hdr)
                .with_header(ct_hdr)
                .with_header(tm_hdr)
                .with_header(srvr_hdr)
                .with_header(acc_rng_hdr)
                .with_header(nm_hdr);
            if let Err(e) = req.respond(response) {
                error!("HTTP HEAD connection with {remote_addr} terminated [{e}]");
            }
        }
        _ => error!("TODO: respond with 404 reponse"),
    }
}
