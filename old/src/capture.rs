use cpal::{
    traits::DeviceTrait, Device, SampleFormat, Stream, StreamError,
    SupportedStreamConfig,
};
use crossbeam::channel::Sender;
use log::{error, info};

#[allow(unused)] // TODO: remove attribute
pub fn capture_input_audio(device: &Device, sender: Sender<Vec<f32>>) -> Option<Stream> {
    info!(
        "starting capture of input audio of device '{}'",
        device.name().ok()?
    );
    let config = device.default_input_config().ok()?;

    capture_audio(device, config, sender)
}

pub fn capture_output_audio(device: &Device, sender: Sender<Vec<f32>>) -> Option<Stream> {
    info!(
        "starting capture of output audio of device '{}'",
        device.name().ok()?
    );
    let config = device.default_output_config().ok()?;

    capture_audio(device, config, sender)
}

fn capture_audio(device: &Device, config: SupportedStreamConfig, sender: Sender<Vec<f32>>) -> Option<Stream> {
    match config.sample_format() {
        SampleFormat::F32 => device
            .build_input_stream(
                &config.config(),
                move |data, _| capture(data, sender.clone()),
                capture_error,
                None
            ).ok(),
        sample_format => {
            error!("sample format '{}' is not supported!", sample_format);
            None
        }
    }
}

fn capture(_data: &[f32], sender: Sender<Vec<f32>>) {
    // convert f32 slice to u32 vec

    // debug!("data len: {}", _data.len());

    sender.send(_data.to_vec()).unwrap();
    // TODO: implement function
}

fn capture_error(err: StreamError) {
    error!("{err}");
}


