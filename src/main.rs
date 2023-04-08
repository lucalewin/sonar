use std::{thread, time::Duration};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use log::{info, error};

mod audio;
mod devices;
mod server;
mod volume;
mod speaker;
mod capture;
mod test;

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    // use WASAPI host (on windows)
    let host = cpal::default_host();

    // get default output device
    let default_output_device = host.default_output_device().unwrap();
    info!("default output device: {}", default_output_device.name().unwrap());

    // start audio capture
    let stream = capture::capture_output_audio(&default_output_device).unwrap();
    stream.play().unwrap();

    // // start streaming server
    // let server_thread = thread::spawn(server::start);

    thread::sleep(Duration::from_secs(5));
    // server_thread.join().unwrap();
}
