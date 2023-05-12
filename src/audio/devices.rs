use cpal::traits::{DeviceTrait, HostTrait};
use log::debug;

use super::WavData;

/// A [cpal::Device] with either a default input or default output config.
pub enum Device {
    Input(cpal::Device),
    Output(cpal::Device),
}

impl Device {
    // Construct a [Device] from a [cpal::Device].
    //
    // Devices may support both input and output.
    // This defaults to output if both are present on one device.
    fn from_device(device: cpal::Device) -> Option<Self> {
        // Only use the default config for output or input
        // Prefer output if a device supports both
        if let Ok(conf) = device.default_output_config() {
            debug!("    Default output stream config:\n      {:?}", conf);
            Some(Self::Output(device))
        } else if let Ok(conf) = device.default_input_config() {
            debug!("    Default input stream config:\n      {:?}", conf);
            Some(Self::Input(device))
        } else {
            None
        }
    }

    /// Returns the default [cpal::SupportedStreamConfig] regardless of device type.
    pub fn default_config_any(
        &self,
    ) -> Result<cpal::SupportedStreamConfig, cpal::DefaultStreamConfigError> {
        match self {
            Device::Input(device) => device.default_input_config(),
            Device::Output(device) => device.default_output_config(),
        }
    }

    /// Device name
    pub fn name(&self) -> Result<String, cpal::DeviceNameError> {
        match self {
            Device::Input(device) => device.name(),
            Device::Output(device) => device.name(),
        }
    }

    pub fn wav_data(&self) -> WavData {
        let config = self.default_config_any().unwrap();

        WavData {
            sample_format: config.sample_format(),
            sample_rate: config.sample_rate(),
            channels: config.channels()
        }
    }
}

impl AsRef<cpal::Device> for Device {
    fn as_ref(&self) -> &cpal::Device {
        match self {
            Device::Input(device) => device,
            Device::Output(device) => device,
        }
    }
}

/// Log all supported stream configs for both input and output devices.
fn log_stream_configs(
    // Iterator returned by [cpal::Device::supported_input_configs] or [cpal::Device::supported_output_configs].
    configs: Result<
        impl Iterator<Item = cpal::SupportedStreamConfigRange>,
        cpal::SupportedStreamConfigsError,
    >,
    // "output" or "input"
    cfg_type: &str,
    // Device index in relation to the iterator returned by [cpal::Host::devices]
    device_index: usize,
) {
    match configs {
        Ok(configs) => {
            let mut configs = configs.peekable();
            if configs.peek().is_some() {
                debug!("    All supported {cfg_type} stream configs:");
                for (config_index, config) in configs.enumerate() {
                    debug!(
                        "      {}.{}. {:?}",
                        device_index + 1,
                        config_index + 1,
                        config
                    );
                }
            }
        }
        Err(e) => {
            debug!("Error retrieving {cfg_type} stream configs: {:?}", e);
        }
    };
}

pub fn get_output_audio_devices() -> Option<Vec<Device>> {
    let mut result = Vec::new();
    debug!("Supported hosts:\n  {:?}", cpal::ALL_HOSTS);
    let available_hosts = cpal::available_hosts();
    debug!("Available hosts:\n  {:?}", available_hosts);

    for host_id in available_hosts {
        debug!("{}", host_id.name());
        let host = cpal::host_from_id(host_id).unwrap();

        let default_out = host.default_output_device().map(|e| e.name().unwrap());
        debug!("  Default Output Device:\n    {:?}", default_out);

        let default_in = host.default_input_device().and_then(|e| e.name().ok());
        debug!("  Default Input Device:\n    {:?}", default_in);

        let devices = host.devices().unwrap();
        debug!("  Devices: ");
        for (device_index, device) in devices.enumerate() {
            debug!("  {}. \"{}\"", device_index + 1, device.name().unwrap());
            // List all of the supported stream configs per device.
            log_stream_configs(device.supported_output_configs(), "output", device_index);
            log_stream_configs(device.supported_input_configs(), "input", device_index);
            if let Some(device) = Device::from_device(device) {
                result.push(device);
            }
        }
    }

    Some(result)
}

pub fn get_default_audio_output_device() -> Option<Device> {
    cpal::default_host()
        .default_output_device()
        .map(Device::Output)
}
