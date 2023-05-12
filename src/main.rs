#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // to suppress console with debug output for release builds
use crate::{audio::capture::capture_output_audio, config::Config, priority::raise_priority};

use audio::devices::Device;
use cpal::{
    traits::{HostTrait, StreamTrait},
    Stream,
};
use crossbeam_channel::Sender;
use log::{debug, info, LevelFilter};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::thread;

pub mod audio;
pub mod config;
pub mod network;
pub mod priority;
pub mod server;

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_NAME: &str = env!("CARGO_PKG_NAME");

pub static NEW_CLIENTS: Lazy<RwLock<Vec<Sender<Vec<f32>>>>> = Lazy::new(|| RwLock::new(Vec::new()));
pub static CONFIG: Lazy<RwLock<Config>> = Lazy::new(|| RwLock::new(Config::load()));

/// Sonar
///
/// - setup and start audio capture
/// - start the streaming webserver
fn main() {
    env_logger::builder()
        .filter_level(if cfg!(debug_assertions) {
            LevelFilter::Debug
        } else {
            CONFIG.read().app.log_level
        })
        .init();

    info!("{} (v{})", APP_NAME, APP_VERSION);
    info!("Config: {:?}", CONFIG.read());

    // first initialize cpal audio to prevent
    // COM reinitialize panic on Windows
    let audio_device = cpal::default_host()
        .default_output_device()
        .map(Device::Output)
        .expect("No default audio device found!");

    // raise process priority a bit to prevent
    // audio stuttering under cpu load
    raise_priority();

    // start the capture of the system audio
    // this variable needs to be keept in scope
    // otherwise the audio capture would stop
    let _stream = start_audio_capture(&audio_device);

    // start the http webserver
    thread::spawn(server::start_server).join().unwrap();
}

fn start_audio_capture(audio_output_device: &Device) -> Stream {
    debug!("Try capturing system audio");
    match capture_output_audio(audio_output_device) {
        Some(s) => {
            s.play().unwrap();
            s
        }
        None => {
            panic!("could not start audio capture!");
        }
    }
}

// fn start_silence_injector_thread(_audio_output_device: Device) {
//     if let Some(true) = CONFIG.read().inject_silence {
//         let _ = thread::Builder::new()
//             .name("silence_injector".into())
//             .stack_size(4 * 1024 * 1024)
//             .spawn(move || run_silence_injector(&audio_output_device))
//             .unwrap();
//     }
// }
