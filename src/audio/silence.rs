use std::time::Duration;

use cpal::{
    traits::{DeviceTrait, StreamTrait},
    SampleFormat,
};
use dasp_sample::Sample;

use super::devices::Device;

// fn start_silence_injector_thread(_audio_output_device: Device) {
//     if let Some(true) = CONFIG.read().inject_silence {
//         let _ = thread::Builder::new()
//             .name("silence_injector".into())
//             .stack_size(4 * 1024 * 1024)
//             .spawn(move || run_silence_injector(&audio_output_device))
//             .unwrap();
//     }
// }

/// inject silence into the audio stream to
/// solve problems with Sonos when pusing audio
pub fn run_silence_injector(device: &Device) {
    let config = device
        .default_config_any()
        .expect("Error while querying stream configs for the silence injector");

    let sample_format = config.sample_format();
    let err_fn = |err| eprintln!("an error occurred on the output audio stream: {err}");
    let config = config.into();

    let device = device.as_ref();
    let stream = match sample_format {
        SampleFormat::F32 => device
            .build_output_stream(&config, write_silence::<f32>, err_fn, None)
            .unwrap(),
        SampleFormat::I16 => device
            .build_output_stream(&config, write_silence::<i16>, err_fn, None)
            .unwrap(),
        SampleFormat::U16 => device
            .build_output_stream(&config, write_silence::<u16>, err_fn, None)
            .unwrap(),
        format => panic!("Unsupported sample format: {format:?}"),
    };
    stream
        .play()
        .expect("Unable to inject silence into the output stream");

    loop {
        std::thread::sleep(Duration::from_secs(1));
    }
}

fn write_silence<T: Sample>(data: &mut [T], _: &cpal::OutputCallbackInfo) {
    for sample in data.iter_mut() {
        *sample = Sample::EQUILIBRIUM;
    }
}
