use cpal::{traits::DeviceTrait, Device, Stream, SupportedStreamConfig, SampleFormat, InputCallbackInfo, StreamError};
use log::{info, error};

#[allow(unused)] // TODO: remove attribute
pub fn capture_input_audio(device: &Device) -> Option<Stream> {
    info!("starting capture of input audio of device '{}'", device.name().ok()?);
    let config = device.default_input_config().ok()?;

    capture_audio(device, config)
}

pub fn capture_output_audio(device: &Device) -> Option<Stream> {
    info!("starting capture of output audio of device '{}'", device.name().ok()?);
    let config = device.default_output_config().ok()?;

    capture_audio(device, config)
}

fn capture_audio(device: &Device, config: SupportedStreamConfig) -> Option<Stream> {
    match config.sample_format() {
        SampleFormat::F32 => device.build_input_stream(&config.config(), capture::<f32>, capture_error, None).ok(),
        sample_format => {
            error!("sample format '{}' is not supported!", sample_format);
            None
        }
    }
}

fn capture<T>(_data: &[T], _: &InputCallbackInfo) {
    // TODO: implement function
}

fn capture_error(err: StreamError) {
    error!("{err}");
}
