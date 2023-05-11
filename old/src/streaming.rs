// use crossbeam::channel::Receiver;
// use log::info;

// /// TODO: add documentation
// pub struct AudioStream {
//     receiver: Receiver<Vec<f32>>
// }

// impl AudioStream {
//     pub fn new(receiver: Receiver<Vec<f32>>) -> Self {
//         Self { receiver }
//     }
// }

// // `Read` needs to be implemented because the client
// // tries to read from this stream.
// // We do not write to the client explicitly!
// impl std::io::Read for AudioStream {
//     fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
//         info!("tried to read from audio stream");
//         todo!()
//     }
// }

// fn to_i32_sample(mut f32_sample: f32) -> i32 {
//     f32_sample = f32_sample.clamp(-1.0, 1.0);
//     if f32_sample >= 0.0 {
//         ((f32_sample as f64 * i32::MAX as f64) + 0.5) as i32
//     } else {
//         ((-f32_sample as f64 * i32::MIN as f64) - 0.5) as i32
//     }
// }
