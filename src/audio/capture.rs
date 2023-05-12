use cpal::{traits::{DeviceTrait, StreamTrait}, Stream};
use dasp_sample::{Sample, ToSample};
use log::{debug, error, info};

use crate::CLIENTS;

use super::devices::Device;

pub fn start_audio_capture(audio_output_device: &Device) -> Stream {
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

/// capture_audio_output - capture the audio stream from the default audio output device
///
/// sets up an input stream for the wave_reader in the appropriate format (f32/i16/u16)
pub fn capture_output_audio(
    device_wrap: &Device,
    // rms_sender: Sender<Vec<f32>>,
) -> Option<cpal::Stream> {
    let device = device_wrap.as_ref();
    info!(
        "Capturing audio from: {}",
        device
            .name()
            .expect("Could not get default audio device name")
    );
    let audio_cfg = device_wrap
        .default_config_any()
        .expect("No default stream config found");
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
fn wave_reader<T>(samples: &[T], f32_samples: &mut Vec<f32>)
where
    T: Sample + ToSample<f32>,
{
    f32_samples.clear();
    f32_samples.extend(samples.iter().map(|x: &T| T::to_sample::<f32>(*x)));
    for s in CLIENTS.read().iter() {
        s.send(f32_samples.clone()).unwrap();
    }
}
