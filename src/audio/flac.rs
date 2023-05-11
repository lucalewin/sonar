use crossbeam_channel::{unbounded, Receiver, Sender};
use flac_bound::{FlacEncoder, WriteWrapper};
#[cfg(feature = "NOISE")]
use rand::{distributions::Uniform, rngs::StdRng, Rng, SeedableRng};
use std::{
    io::Write,
    sync::{
        atomic::{AtomicBool, Ordering::Relaxed},
        Arc,
    },
    time::Duration,
};

const NOISE_PERIOD: u64 = 250;

// the flacwriter receives the data from the encoder
// and writes them to the flac output channel
#[derive(Clone)]
pub struct FlacWriter {
    flac_out: Sender<Vec<u8>>,
}

impl FlacWriter {
    pub fn new(flac_out: Sender<Vec<u8>>) -> FlacWriter {
        FlacWriter { flac_out }
    }
}

impl Write for FlacWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self.flac_out.send(buf.to_vec()) {
            Ok(()) => Ok(buf.len()),
            Err(_e) => Err(std::io::Error::new(
                std::io::ErrorKind::ConnectionAborted,
                "FlacWriter channel SendError",
            )),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

// a FlacChannel is set up by the channelstream
// the ChannelStream writes the captured f32 samples
// to the samples_in channel for encoding
#[derive(Clone)]
pub struct FlacChannel {
    samples_in: Receiver<Vec<f32>>,
    pub flac_in: Receiver<Vec<u8>>,
    active: Arc<AtomicBool>,
    writer: FlacWriter,
    sample_rate: u32,
    bits_per_sample: u32,
    channels: u32,
}

impl FlacChannel {
    pub fn new(
        samples_in: Receiver<Vec<f32>>,
        sample_rate: u32,
        bits_per_sample: u32,
        channels: u32,
    ) -> FlacChannel {
        let (flac_out, flac_in): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = unbounded();
        FlacChannel {
            samples_in,
            flac_in,
            active: Arc::new(AtomicBool::new(false)),
            writer: FlacWriter::new(flac_out),
            sample_rate,
            bits_per_sample,
            channels,
        }
    }

    pub fn run(&self) {
        // copy instance data for thread
        let samples_in = self.samples_in.clone();
        let mut writer = self.writer.clone();
        let ch = self.channels;
        let bps = self.bits_per_sample;
        let sr = self.sample_rate;
        let l_active = self.active.clone();
        // fire up thread
        self.active.store(true, Relaxed);
        let _thr = std::thread::Builder::new()
            .name("flac_encoder".into())
            .stack_size(4 * 1024 * 1024)
            .spawn(move || {
                // we're running
                // setup the encoder
                let mut outw = WriteWrapper(&mut writer);
                let mut enc = FlacEncoder::new()
                    .unwrap()
                    .channels(ch)
                    .bits_per_sample(bps)
                    .sample_rate(sr)
                    .compression_level(0)
                    .init_write(&mut outw)
                    .unwrap();
                // read captured samples and encode
                let shift = if bps == 24 { 8u8 } else { 16u8 };
                #[cfg(feature = "NOISE")]
                {
                    // create the random generator for the white noise
                    let mut rng = StdRng::seed_from_u64(79);
                    // preallocate the noise buffer
                    const DIVISOR: u64 = 1000 / NOISE_PERIOD;
                    let size = ((sr * 2) / DIVISOR as u32) as usize;
                    let mut noise: Vec<f32> = Vec::with_capacity(size);
                    noise.resize(size, 0.0);
                    let mut sending_silence = false;
                }
                // read and FLAC encode samples
                let time_out = Duration::from_millis(NOISE_PERIOD);
                while l_active.load(Relaxed) {
                    #[cfg(feature = "NOISE")]
                    {
                        let time_out = if sending_silence {
                            Duration::from_millis(NOISE_PERIOD)
                        } else {
                            Duration::from_millis(NOISE_PERIOD * 4)
                        };
                    }
                    if let Ok(f32_samples) = samples_in.recv_timeout(time_out) {
                        #[cfg(feature = "NOISE")]
                        {
                            sending_silence = false;
                        }
                        let samples = f32_samples
                            .iter()
                            .map(|s| to_i32_sample(*s) >> shift)
                            .collect::<Vec<i32>>();
                        enc.process_interleaved(samples.as_slice(), (samples.len() / 2) as u32)
                            .unwrap();
                    } else {
                        #[cfg(feature = "NOISE")]
                        {
                            // if no samples for a certain time: send a faint white noise
                            sending_silence = true;
                            if l_active.load(Relaxed) {
                                fill_noise_buffer(&mut rng, &mut noise);
                                let samples = noise
                                    .iter()
                                    .map(|s| to_i32_sample(*s) >> shift)
                                    .collect::<Vec<i32>>();
                                let res = enc.process_interleaved(
                                    samples.as_slice(),
                                    (samples.len() / 2) as u32,
                                );
                                if let Err(e) = res {
                                    ui_log(format!(
                                        "Flac encoding error caused by silence {:?}",
                                        e
                                    ));
                                    break;
                                }
                            }
                        }
                    }
                }
                let _ = enc.finish();
            })
            .unwrap();
    }

    pub fn stop(&self) {
        self.active.store(false, Relaxed);
    }
}

fn to_i32_sample(mut f32_sample: f32) -> i32 {
    f32_sample = f32_sample.clamp(-1.0, 1.0);
    if f32_sample >= 0.0 {
        ((f32_sample as f64 * i32::MAX as f64) + 0.5) as i32
    } else {
        ((-f32_sample as f64 * i32::MIN as f64) - 0.5) as i32
    }
}

#[cfg(feature = "NOISE")]
///
/// fille the pre-allocated noise buffer with a very faint white noise (-60db)
///
fn fill_noise_buffer(rng: &mut StdRng, noise: &mut [f32]) {
    let amplitude: f32 = 0.001;
    for sample in noise.iter_mut() {
        *sample = ((rng.sample(Uniform::new(0.0, 1.0)) * 2.0) - 1.0) * amplitude;
    }
}
