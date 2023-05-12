pub mod encoder;

use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

use crossbeam_channel::{unbounded, Receiver, Sender};
use dasp_sample::Sample;
use log::{debug, error};

use crate::{audio::format::wav::create_header, server::encoder::Encoder, CLIENTS};

const HEADERS: &str = concat!(
    "HTTP/1.1 200 OK\r\n",
    "Connection: close\r\n",
    "Content-Type: audio/vnd.wave;codec=1\r\n",
    "Transfer-Encoding: chunked\r\n",
    "\r\n"
);

pub fn start_server() {
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

    debug!(
        "Request ({}): {:#?}",
        stream.peer_addr().unwrap(),
        http_request
    );

    // http response header
    stream.write_all(HEADERS.as_bytes()).unwrap();

    let (s, r): (Sender<Vec<f32>>, Receiver<Vec<f32>>) = unbounded();
    CLIENTS.write().push(s);

    match send_audio_stream(stream, r) {
        Ok(()) => {},
        Err(_) => {
            error!("handle error: remove receiver from the client list");
        }
    }
}

fn send_audio_stream(stream: TcpStream, receiver: Receiver<Vec<f32>>) -> std::io::Result<()> {
    let mut encoder = Encoder::new(stream);

    encoder.write_all(&create_header(48000, 16))?;
    encoder.flush()?;

    // TODO: check if stream is still open
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
