mod encoder;

use std::{net::{TcpListener, TcpStream}, io::{BufReader, BufRead, Write}};

use crossbeam_channel::{Sender, Receiver, unbounded};
use dasp_sample::Sample;
use log::{debug, error};

use crate::{streaming::rwstream::create_wav_header, tcp::encoder::Encoder, NEW_CLIENTS};

const HEADERS: &str = concat!(
    "HTTP/1.1 200 OK\r\n",
    "Connection: close\r\n",
    "Content-Type: audio/vnd.wave;codec=1\r\n",
    "TransferMode.DLNA.ORG: Streaming\r\n",
    "Server: UPnP/1.0 DLNADOC/1.50 LAB/1.0\r\n",
    "Accept-Ranges: none\r\n",
    "icy-name: sonar\r\n",
    "Transfer-Encoding: chunked\r\n",
    "\r\n"
);

pub fn start_server() {
    let listener = TcpListener::bind("0.0.0.0:5901").unwrap();

    for incoming in listener.incoming() {
        std::thread::spawn(|| { handle_client(incoming.unwrap()) });
    }
}

fn handle_client(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    debug!("Request ({}): {:#?}", stream.peer_addr().unwrap(), http_request);
    
    // http response header
    stream.write_all(HEADERS.as_bytes()).unwrap();
    
    let mut encoder = Encoder::new(stream);

    encoder.write_all(&create_wav_header(48000, 16)).unwrap();
    encoder.flush().unwrap();

    let (s, r): (Sender<Vec<f32>>, Receiver<Vec<f32>>) = unbounded();

    NEW_CLIENTS.write().push(s);

    // TODO: check if stream is still open
    loop {
        match r.recv() {
            Ok(samples) => {
                let mut buffer = Vec::with_capacity(samples.len() * 2);
                for sample in samples {
                    let sample = i16::from_sample(sample);

                    buffer.extend_from_slice(&sample.to_le_bytes());

                }

                encoder.write_all(&buffer).unwrap();
                encoder.flush().unwrap();
            },
            Err(e) => error!("error occured: {e}")
        }
    }
}
