use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream}, error::Error,
};

use crossbeam_channel::{unbounded, Receiver};
use dasp_sample::Sample;
use log::{debug, info};

use crate::{audio::format::wav::create_header, CLIENTS, CONFIG};

const HEADERS: &str = concat!(
    "HTTP/1.1 200 OK\r\n",
    "Connection: close\r\n",
    "Content-Type: audio/vnd.wave;codec=1\r\n",
    "Transfer-Encoding: chunked\r\n",
    "\r\n"
);

pub fn start_server() {
    let addr = {
        let config = CONFIG.read();
        format!("{}:{}", config.server.network, config.server.port)
    };
    info!("starting server on '{addr}'");

    let listener = TcpListener::bind(addr).unwrap();
    
    for incoming in listener.incoming() {
        std::thread::spawn(|| handle_client(incoming.unwrap()));
    }
}

fn handle_client(mut stream: TcpStream) {
    let ip = stream.peer_addr().unwrap().ip();
    info!("client '{}' connected", ip);

    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();
    
    debug!(
        "Request ({}): {:#?}",
        stream.peer_addr().unwrap(),
        http_request
    );

    // http response header
    stream.write_all(HEADERS.as_bytes()).unwrap();

    let (s, r) = unbounded();
    CLIENTS.write().insert(ip, s);

    match send_audio_stream(&stream, r) {
        Ok(()) => {}, // this function does not return OK because of the endless loop
        Err(_) => {   // it only returns ERR when the client disconnected
            CLIENTS.write().remove(&ip);
            info!("client '{}' disconnected", ip);
        }
    }
}

/// returns Err when the tcp stream is closed and the data cannot be flushed anymore
fn send_audio_stream(stream: &TcpStream, receiver: Receiver<Vec<f32>>) -> Result<(), Box<dyn Error>> {
    let bits_per_sample = CONFIG.read().audio.bits_per_sample;

    // send wav header with an "infinite size"
    send_encoded(stream, &create_header(48000, bits_per_sample))?;

    loop {
        // wait for samples from the audio capture thread
        let samples = receiver.recv()?;

        // convert f32 samples to i16 samples as bytes
        let mut buffer = Vec::with_capacity(samples.len() * 2);
        for sample in samples {
            let sample = i16::from_sample(sample);
            buffer.extend_from_slice(&sample.to_le_bytes());
        }

        // send buffer to client
        send_encoded(stream, &buffer)?;
    }
}

/// encode data and write it the the TCP stream
fn send_encoded(mut stream: &TcpStream, data: &[u8]) -> std::io::Result<()> {
    stream.write_all(format!("{:x}\r\n", data.len()).as_bytes())?;
    stream.write_all(data)?;
    stream.write_all(b"\r\n")?;
    stream.flush()
}
