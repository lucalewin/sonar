use std::{thread, time::Duration};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam::channel::{unbounded, Receiver, Sender};
use log::info;

mod capture;
mod config;
mod devices;
mod streaming;

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    // use WASAPI host (on windows)
    let host = cpal::default_host();

    // get default output device
    let default_output_device = host.default_output_device().unwrap();
    info!(
        "default output device: {}",
        default_output_device.name().unwrap()
    );

    let (sender, receiver): (Sender<Vec<f32>>, Receiver<Vec<f32>>) = unbounded();

    // start audio capture
    // We the following variant to start the audio capture stream because it ensures that the `Stream` object
    // returned by the `capture_output_audio` function is kept in scope for the duration of the audio capture.
    // This is important because if the `Stream` object is dropped, the audio capture will stop working.
    let stream = capture::capture_output_audio(&default_output_device, sender).unwrap();
    stream.play().unwrap();

    // // start streaming server
    let server_thread = thread::spawn(move || streaming::start_server());

    thread::sleep(Duration::from_secs(5));

    server_thread.join().unwrap();
}
