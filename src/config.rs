use std::{
    fs,
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
};

use log::LevelFilter;
use serde::{Deserialize, Serialize};
use toml::from_str;

use crate::{audio::format::StreamingFormat, APP_NAME};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub server: ServerConfig,
    pub device: Option<DeviceConfig>,
    pub renderer: Option<RendererConfig>,
    pub audio: AudioConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub log_level: LevelFilter,
    pub auto_reconnect: bool,
    pub inject_silence: bool,
    pub capture_timeout: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub network: IpAddr,
    pub port: u16,
    pub workers: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceConfig {
    pub name: String,
    pub index: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RendererConfig {
    pub name: String,
    pub ip_addr: IpAddr,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AudioConfig {
    pub format: StreamingFormat,
    pub bits_per_sample: u8,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            log_level: LevelFilter::Info,
            auto_reconnect: true,
            inject_silence: true,
            capture_timeout: 250,
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            network: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            port: 5901,
            workers: 8,
        }
    }
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            format: StreamingFormat::Wav,
            bits_per_sample: 16,
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load() -> Self {
        let home_dir = dirs::home_dir().unwrap_or_default();
        let config_dir = home_dir.join(".".to_string() + APP_NAME);
        let config_file = config_dir.join("config.toml");

        Self::check(&config_dir, &config_file);

        from_str(&fs::read_to_string(config_file).unwrap()).unwrap()
    }

    pub fn save(&self) -> std::io::Result<()> {
        let home_dir = dirs::home_dir().unwrap_or_default();
        let config_dir = home_dir.join(".".to_string() + APP_NAME);
        let config_file = config_dir.join("config.toml");

        Self::check(&config_dir, &config_file);

        fs::write(config_file, toml::to_string_pretty(&self).unwrap())
    }

    fn check(config_dir: &PathBuf, config_file: &PathBuf) {
        let config = Config::default();
        if !config_dir.exists() {
            // create config directory
            fs::create_dir_all(config_dir).unwrap();
            // create config with default values
            fs::write(config_file, toml::to_string_pretty(&config).unwrap()).unwrap();
        } else if !config_file.exists() {
            // create config with default values
            fs::write(config_file, toml::to_string_pretty(&config).unwrap()).unwrap();
        }
    }
}
