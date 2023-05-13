pub mod encoder;

use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

use crossbeam_channel::{unbounded, Receiver};
use dasp_sample::Sample;
use log::{debug, error, info};

use crate::{audio::format::wav::create_header, server::encoder::Encoder, CLIENTS};

const HEADERS: &str = concat!(
    "HTTP/1.1 200 OK\r\n",
    "Connection: close\r\n",
    "Content-Type: audio/vnd.wave;codec=1\r\n",
    "Transfer-Encoding: chunked\r\n",
    "\r\n"
);

pub fn start_server() {
    info!("starting server on '0.0.0.0:5901'");

    let listener = TcpListener::bind("0.0.0.0:5901").unwrap();

    for incoming in listener.incoming() {
        std::thread::spawn(|| handle_client(incoming.unwrap()));
    }
}

fn handle_client(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();
    
    let ip = stream.peer_addr().unwrap().ip();
    
    info!("client '{}' connected", ip);
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
        Ok(()) => {},
        Err(_) => {
            CLIENTS.write().remove(&ip);
            info!("client '{}' disconnected", ip);
        }
    }
}

/// returns Err when the tcp stream is closed and the data cannot be flushed anymore
fn send_audio_stream(stream: &TcpStream, receiver: Receiver<Vec<f32>>) -> std::io::Result<()> {
    let mut encoder = Encoder::new(stream);

    encoder.write_all(&create_header(48000, 16))?;
    encoder.flush()?;

    loop {
        match receiver.recv() {
            Ok(samples) => {
                let mut buffer = Vec::with_capacity(samples.len() * 2);
                for sample in samples {
                    let sample = i16::from_sample(sample);

                    buffer.extend_from_slice(&sample.to_le_bytes());
                }

                encoder.write_all(&buffer)?;
                encoder.flush()?;
            }
            Err(e) => error!("error occured: {e}"),
        }
    }
}
