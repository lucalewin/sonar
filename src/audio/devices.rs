use cpal::traits::DeviceTrait;
use log::debug;

use super::WavData;

/// A [cpal::Device] with either a default input or default output config.
pub enum Device {
    Input(cpal::Device),
    Output(cpal::Device),
}

impl Device {
    /// Construct a [Device] from a [cpal::Device].
    ///
    /// Devices may support both input and output.
    /// This defaults to output if both are present on one device.
    pub fn from_device(device: cpal::Device) -> Option<Self> {
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
            channels: config.channels(),
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
