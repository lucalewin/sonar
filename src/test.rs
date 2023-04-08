use cpal::traits::DeviceTrait;
use log::{debug, error};

pub fn capture_output_audio(
    device: &cpal::Device
) -> Option<cpal::Stream> {
    let audio_cfg = device
        .default_output_config()
        .expect("No default output config found");
    debug!("Default audio {audio_cfg:?}");
    let mut f32_samples: Vec<f32> = Vec::with_capacity(16384);
    match audio_cfg.sample_format() {
        cpal::SampleFormat::F32 => match device.build_input_stream(
            &audio_cfg.config(),
            move |data, _: &_| wave_reader::<f32>(data, &mut f32_samples),
            capture_err_fn,
            None,
        ) {
            Ok(stream) => Some(stream),
            Err(e) => {
                error!("Error capturing f32 audio stream: {e}");
                None
            }
        },
        cpal::SampleFormat::I16 => {
            match device.build_input_stream(
                &audio_cfg.config(),
                move |data, _: &_| wave_reader::<i16>(data, &mut f32_samples),
                capture_err_fn,
                None,
            ) {
                Ok(stream) => Some(stream),
                Err(e) => {
                    error!("Error capturing i16 audio stream: {e}");
                    None
                }
            }
        }
        cpal::SampleFormat::U16 => {
            match device.build_input_stream(
                &audio_cfg.config(),
                move |data, _: &_| wave_reader::<u16>(data, &mut f32_samples),
                capture_err_fn,
                None,
            ) {
                Ok(stream) => Some(stream),
                Err(e) => {
                    error!("Error capturing u16 audio stream: {e}");
                    None
                }
            }
        }
        _ => None,
    }
}

/// capture_err_fn - called whan it's impossible to build an audio input stream
fn capture_err_fn(err: cpal::StreamError) {
    error!("Error {err} building audio input stream");
}

/// wave_reader - the captured audio input stream reader
///
/// writes the captured samples to all registered clients in the
/// CLIENTS ChannnelStream hashmap
/// also feeds the RMS monitor channel if the RMS option is set
fn wave_reader<T>(samples: &[T], f32_samples: &mut Vec<f32>) {
    debug!("new data");
}

