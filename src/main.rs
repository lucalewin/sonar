#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // to suppress console with debug output for release builds
use crate::{
    audio::{
        capture::capture_output_audio,
        devices::get_default_audio_output_device,
        silence::run_silence_injector,
    },
    config::Configuration,
    network::get_local_addr,
    priority::raise_priority,
};

use audio::devices::Device;
use cpal::{traits::StreamTrait, Stream};
use crossbeam_channel::Sender;
use log::{debug, info, LevelFilter};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::{net::IpAddr, thread, time::Duration};

pub mod audio;
pub mod config;
pub mod network;
pub mod openhome;
pub mod priority;
pub mod server;

/// app version
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_NAME: &str = "Sonar";

/// the HTTP server port
pub const SERVER_PORT: u16 = 5901;

pub static NEW_CLIENTS: Lazy<RwLock<Vec<Sender<Vec<f32>>>>> = Lazy::new(|| RwLock::new(Vec::new()));

// the global configuration state
pub static CONFIG: Lazy<RwLock<Configuration>> =
    Lazy::new(|| RwLock::new(Configuration::read_config()));

/// Sonar
///
/// - setup and start audio capture
/// - start the streaming webserver
fn main() {
    // init logger
    env_logger::builder()
        .filter_level(if cfg!(debug_assertions) {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        })
        .init();

    // first initialize cpal audio to prevent COM reinitialize panic on Windows
    let audio_output_device = get_default_audio_output_device().expect("No default audio device");

    // initialize config
    let config = init_config(&audio_output_device);

    info!("{} (v{})", APP_NAME, APP_VERSION);
    info!("Config: {:#?}", config);

    // get the default network that connects to the internet
    let local_addr = load_local_addr(&config);

    let wav_data = audio_output_device.wav_data();

    // raise process priority a bit to prevent audio stuttering under cpu load
    raise_priority();

    // capture system audio
    let _stream = start_audio_capture(&audio_output_device);

    // If silence injector is on, start the "silence_injector" thread
    start_silence_injector_thread(audio_output_device);

    thread::spawn(server::start_server);

    // wait for ctrl-c
    loop { std::thread::sleep(Duration::from_secs(1)); }
}

fn init_config(audio_output_device: &Device) -> Configuration {
    let mut conf = CONFIG.write();
    if conf.sound_source == "None" {
        conf.sound_source = audio_output_device.name().unwrap();
        let _ = conf.update_config();
    }
    conf.clone()
}

fn load_local_addr(config: &Configuration) -> IpAddr {
    if config.last_network == "None" {
        let addr = get_local_addr().expect("Could not obtain local address.");
        let mut conf = CONFIG.write();
        conf.last_network = addr.to_string();
        let _ = conf.update_config();
        addr
    } else {
        config.last_network.parse().unwrap()
    }
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

fn start_silence_injector_thread(audio_output_device: Device) {
    if let Some(true) = CONFIG.read().inject_silence {
        let _ = thread::Builder::new()
            .name("silence_injector".into())
            .stack_size(4 * 1024 * 1024)
            .spawn(move || run_silence_injector(&audio_output_device))
            .unwrap();
    }
}
