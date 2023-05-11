use std::{net::{TcpListener, TcpStream}, io::{BufReader, BufRead, Write}, thread};

use crossbeam::channel::Receiver;
use log::{debug, info, error, warn};

const HEADERS: &str = concat!(
    "HTTP/1.1 200 OK\r\n",
    "Server: UPnP/1.0 DLNADOC/1.50 LAB/1.0\r\n",
    "icy-name: Sonar\r\n",
    "Connection: close\r\n",
    "Accept-Ranges: none\r\n",
    "Content-Type: audio/vnd.wave;codec=1\r\n",
    "TransferMode.DLNA.ORG: Streaming\r\n",
    "Transfer-Encoding: chunked\r\n",
    "\r\n"
);

pub fn start_server(audio_stream: Receiver<Vec<f32>>) {
    let listener = TcpListener::bind("0.0.0.0:8764").unwrap();

    // let mut clients = Vec::new();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        // if clients.contains(&stream.peer_addr().unwrap().ip().to_string()) {
        //     continue;
        // }

        // clients.push(stream.peer_addr().unwrap().ip().to_string());

        let audio = audio_stream.clone();

        thread::spawn(move || handle_connection(stream, audio));
    }
}

fn handle_connection(mut stream: TcpStream, audio_stream: Receiver<Vec<f32>>) {
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    debug!("Request ({}): {:#?}", stream.peer_addr().unwrap(), http_request);

    if !http_request.iter().any(|line| line.contains("GET /test.wav")) {
        warn!("close connection!");
        return;
    }
    
    // stream.write_all(&b"HTTP/1.1 200 OK\r\n"[..]).unwrap();
    stream.write_all(HEADERS.as_bytes()).unwrap();
    // stream.write_all(format!("Content-Lenght: {}", usize::MAX).as_bytes()).unwrap();
    
    // send wav_hdr
    stream.write_all(&create_wav_header(48000, 16)).unwrap(); // FIXME: remove constant sample rate
    
    stream.flush().unwrap();
//     let buf_reader = BufReader::new(&mut stream);
//     let http_request: Vec<_> = buf_reader
//     .lines()
//     .map(|result| result.unwrap())
//     .take_while(|line| !line.is_empty())
//     .collect();
    
// debug!("res: {:#?}", http_request);
//     // if http_request.is_empty() {
//     //     return;
//     // }
    info!("sending audio stream");
    
    loop {
        match audio_stream.recv() {
            Ok(data) => {
                if data.is_empty() {
                    continue;
                }
                let converted = convert_f32_to_i16(data);

                let mut vec_u8: Vec<u8> = Vec::new();
                for &x in &converted {
                    vec_u8.extend_from_slice(&x.to_le_bytes());
                }

                stream.write_all(&vec_u8).unwrap();
                stream.flush().unwrap();
            },
            Err(e) => error!("{e}")
        }
    };
}

fn convert_f32_to_i16(x: Vec<f32>) -> Vec<i16> {
    let mut y = Vec::with_capacity(x.len());
    (0..x.len()).for_each(|i| {
        y.push((x[i] * 32_768.0) as i16);
    });
    y
}

// fn convert_f32_to_i32(x: Vec<f32>) -> Vec<i32> {
//     let mut y = Vec::with_capacity(x.len());
//     for i in 0..x.len() {
//         let sample = x[i].clamp(-1.0, 1.0);
//         y[i] = if sample >= 0.0 {
//             (sample * i32::MAX as f32 + 0.5) as i32
//         } else {
//             (-sample * i32::MIN as f32 - 0.5) as i32
//         };
//     }
//     y
// }

// create an "infinite size" wav hdr
// note this may not work when streaming to a "libsndfile" based renderer
// as libsndfile insists on a seekable WAV file depending on the open mode used
fn create_wav_header(sample_rate: u32, bits_per_sample: u16) -> Vec<u8> {
    let mut hdr = [0u8; 44];
    let channels: u16 = 2;
    let bytes_per_sample: u16 = bits_per_sample / 8;
    let block_align: u16 = channels * bytes_per_sample;
    let byte_rate: u32 = sample_rate * block_align as u32;
    hdr[0..4].copy_from_slice(b"RIFF"); //ChunkId, little endian WAV
    let subchunksize: u32 = std::u32::MAX; // "infinite" data chunksize signal value
    let chunksize: u32 = subchunksize; // "infinite" RIFF chunksize signal value
    hdr[4..8].copy_from_slice(&chunksize.to_le_bytes()); // ChunkSize
    hdr[8..12].copy_from_slice(b"WAVE"); // File Format
    hdr[12..16].copy_from_slice(b"fmt "); // SubChunk = Format
    hdr[16..20].copy_from_slice(&16u32.to_le_bytes()); // SubChunk1Size for PCM
    hdr[20..22].copy_from_slice(&1u16.to_le_bytes()); // AudioFormat: uncompressed PCM
    hdr[22..24].copy_from_slice(&channels.to_le_bytes()); // numchannels 2
    hdr[24..28].copy_from_slice(&sample_rate.to_le_bytes()); // SampleRate
    hdr[28..32].copy_from_slice(&byte_rate.to_le_bytes()); // ByteRate (Bps)
    hdr[32..34].copy_from_slice(&block_align.to_le_bytes()); // BlockAlign
    hdr[34..36].copy_from_slice(&bits_per_sample.to_le_bytes()); // BitsPerSample
    hdr[36..40].copy_from_slice(b"data"); // SubChunk2Id
    hdr[40..44].copy_from_slice(&subchunksize.to_le_bytes()); // SubChunk2Size
    // debug!("WAV Header (l={}): \r\n{:02x?}", hdr.len(), hdr);
    hdr.to_vec()
}

// use std::thread;

// use crossbeam::channel::Receiver;
// use log::{info, warn};
// use tiny_http::{Method, Request, Response, Server};

// use crate::{config::DEFAULT_CONFIG, streaming::AudioStream};

// pub fn start_server(receiver: Receiver<Vec<f32>>) {
//     info!("Starting streaming server");

//     let http_server = Server::http(format!("0.0.0.0:{}", DEFAULT_CONFIG.port))
//         .expect("could not create HTTP Server");

//     for request in http_server.incoming_requests() {
//         let rec = receiver.clone();
//         thread::spawn(|| handle_client(request, rec));
//     }
// }

// fn handle_client(req: Request, receiver: Receiver<Vec<f32>>) {
//     info!("new client connected: {:?}", req);

//     if req.url() != "/" {
//         warn!("client did not use default url: '{}'", req.url());
//         req.respond(Response::empty(404)).unwrap();
//         return;
//     }

//     match req.method() {
//         Method::Get => start_stream(req, receiver),
//         _ => {
//             req.respond(Response::empty(200)).unwrap();
//         }
//     }
// }

// fn start_stream(req: Request, receiver: Receiver<Vec<f32>>) {
//     let stream = AudioStream::new(receiver);

//     let response = Response::empty(200)
//         .with_data(stream, Some(usize::MAX)) // fix sample length
//         .with_chunked_threshold(usize::MAX); // fix threshold size

//     req.respond(response).unwrap();
// }
