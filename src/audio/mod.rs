pub mod devices;
pub mod silence;
pub mod capture;
pub mod format;

/// some audio config info
#[derive(Debug, Clone, Copy)]
pub struct WavData {
    pub sample_format: cpal::SampleFormat,
    pub sample_rate: cpal::SampleRate,
    pub channels: u16,
}
