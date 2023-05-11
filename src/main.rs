#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // to suppress console with debug output for release builds
use crate::{
    audio::{
        devices::{capture_output_audio, get_default_audio_output_device},
        silence::run_silence_injector,
    },
    config::Configuration,
    network::get_local_addr,
    openhome::WavData,
    server::start,
    streaming::rwstream::ChannelStream,
    utils::priority::raise_priority,
};

use cpal::traits::StreamTrait;
use log::{debug, error, info, LevelFilter};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::{collections::HashMap, net::IpAddr, thread, time::Duration};

pub mod audio;
pub mod config;
pub mod network;
pub mod openhome;
pub mod server;
pub mod streaming;
pub mod utils;

/// app version
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_NAME: &str = "Sonar";

/// the HTTP server port
pub const SERVER_PORT: u16 = 5901;

// streaming clients of the webserver
pub static CLIENTS: Lazy<RwLock<HashMap<String, ChannelStream>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));
// the global configuration state
pub static CONFIG: Lazy<RwLock<Configuration>> =
    Lazy::new(|| RwLock::new(Configuration::read_config()));

/// swyh-rs
///
/// - set up the fltk GUI
/// - setup and start audio capture
/// - start the streaming webserver
/// - start ssdp discovery of media renderers thread
/// - run the GUI, and show any renderers found in the GUI as buttons (to start/stop playing)
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
    let config = {
        let mut conf = CONFIG.write();
        if conf.sound_source == "None" {
            conf.sound_source = audio_output_device.name().unwrap();
            let _ = conf.update_config();
        }
        conf.clone()
    };

    info!("{} (v{})", APP_NAME, APP_VERSION);

    info!("Config: {:#?}", config);

    // get the default network that connects to the internet
    let local_addr: IpAddr = {
        if config.last_network == "None" {
            let addr = get_local_addr().expect("Could not obtain local address.");
            let mut conf = CONFIG.write();
            conf.last_network = addr.to_string();
            let _ = conf.update_config();
            addr
        } else {
            config.last_network.parse().unwrap()
        }
    };

    // we need to pass some audio config data to the play function
    let audio_cfg = &audio_output_device
        .default_config_any()
        .expect("No default input or output config found");
    let wav_data = WavData {
        sample_format: audio_cfg.sample_format(),
        sample_rate: audio_cfg.sample_rate(),
        channels: audio_cfg.channels(),
    };

    // raise process priority a bit to prevent audio stuttering under cpu load
    raise_priority();

    // capture system audio
    debug!("Try capturing system audio");
    let stream: cpal::Stream;
    match capture_output_audio(&audio_output_device) {
        Some(s) => {
            stream = s;
            stream.play().unwrap();
        }
        None => {
            error!("could not start audio capture!");
        }
    }

    // If silence injector is on, start the "silence_injector" thread
    if let Some(true) = CONFIG.read().inject_silence {
        let _ = thread::Builder::new()
            .name("silence_injector".into())
            .stack_size(4 * 1024 * 1024)
            .spawn(move || run_silence_injector(&audio_output_device))
            .unwrap();
    }

    // finally start a webserver on the local address,
    let server_port = config.server_port.unwrap_or_default();
    let _ = thread::Builder::new()
        .name("webserver".into())
        .stack_size(4 * 1024 * 1024)
        .spawn(move || start(&local_addr, server_port, wav_data))
        .unwrap();

    // wait for ctrl-c
    loop {
        std::thread::sleep(Duration::from_secs(1));
    }
}
