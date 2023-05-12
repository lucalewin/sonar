#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // to suppress console with debug output for release builds
use crate::{
    audio::{
        devices::{capture_output_audio, get_default_audio_output_device},
        silence::run_silence_injector,
        WavData
    },
    config::Configuration,
    network::get_local_addr,
    server::start,
    streaming::rwstream::ChannelStream,
    utils::priority::raise_priority,
};

use audio::devices::Device;
use cpal::{traits::StreamTrait, Stream};
use crossbeam_channel::Sender;
use log::{debug, info, LevelFilter};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::{collections::HashMap, net::IpAddr, thread::{self, JoinHandle}, time::Duration};

pub mod audio;
pub mod config;
pub mod network;
pub mod openhome;
pub mod server;
pub mod streaming;
pub mod utils;
pub mod tcp;

/// app version
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_NAME: &str = "Sonar";

/// the HTTP server port
pub const SERVER_PORT: u16 = 5901;

// streaming clients of the webserver
pub static CLIENTS: Lazy<RwLock<HashMap<String, ChannelStream>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

pub static NEW_CLIENTS: Lazy<RwLock<Vec<Sender<Vec<f32>>>>> = Lazy::new(|| RwLock::new(Vec::new()));

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
    let local_addr = load_local_addr(&config);

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
    let _stream = start_audio_capture(&audio_output_device);

    // If silence injector is on, start the "silence_injector" thread
    start_silence_injector_thread(audio_output_device);

    // // finally start a webserver on the local address,
    // start_webserver_thread(config, local_addr, wav_data);

    thread::spawn(tcp::start_server);

    // wait for ctrl-c
    loop { std::thread::sleep(Duration::from_secs(1)); }
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

fn start_webserver_thread(config: Configuration, local_addr: IpAddr, wav_data: WavData) -> JoinHandle<()> {
    let server_port = config.server_port.unwrap_or_default();
    thread::Builder::new()
        .name("webserver".into())
        .stack_size(4 * 1024 * 1024)
        .spawn(move || start(&local_addr, server_port, wav_data))
        .unwrap()
}
