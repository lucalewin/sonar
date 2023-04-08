use std::thread;

use log::{info, warn};
use tiny_http::{Method, Request, Response, Server};

use crate::config::DEFAULT_CONFIG;

pub fn start_server() {
    info!("Starting streaming server");

    let http_server = Server::http(format!("0.0.0.0:{}", DEFAULT_CONFIG.port))
        .expect("could not create HTTP Server");

    for request in http_server.incoming_requests() {
        thread::spawn(|| handle_client(request));
    }
}

fn handle_client(req: Request) {
    info!("new client connected: {:?}", req);

    if req.url() != "/" {
        warn!("client did not use default url: '{}'", req.url());
        req.respond(Response::empty(404)).unwrap();
        return;
    }

    match req.method() {
        Method::Get => {}
        _ => {
            req.respond(Response::empty(200)).unwrap();
        }
    }
}
