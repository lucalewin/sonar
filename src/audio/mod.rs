use log::debug;

pub mod devices;
pub mod flac;
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

/// create an "infinite size" wav hdr
/// note this may not work when streaming to a "libsndfile" based renderer
/// as libsndfile insists on a seekable WAV file depending on the open mode used
pub fn create_wav_header(sample_rate: u32, bits_per_sample: u16) -> [u8; 44] {
    let mut hdr = [0u8; 44];
    let channels: u16 = 2;
    let bytes_per_sample: u16 = bits_per_sample / 8;
    let block_align: u16 = channels * bytes_per_sample;
    let byte_rate: u32 = sample_rate * block_align as u32;
    hdr[0..4].copy_from_slice(b"RIFF");    // ChunkId, little endian WAV
    let subchunksize: u32 = std::u32::MAX; // "infinite" data chunksize signal value
    let chunksize: u32 = subchunksize;     // "infinite" RIFF chunksize signal value
    hdr[4..8].copy_from_slice(&chunksize.to_le_bytes()); // ChunkSize
    hdr[8..12].copy_from_slice(b"WAVE");  // File Format
    hdr[12..16].copy_from_slice(b"fmt "); // SubChunk = Format
    hdr[16..20].copy_from_slice(&16u32.to_le_bytes()); // SubChunk1Size for PCM
    hdr[20..22].copy_from_slice(&1u16.to_le_bytes());  // AudioFormat: uncompressed PCM
    hdr[22..24].copy_from_slice(&channels.to_le_bytes());    // numchannels 2
    hdr[24..28].copy_from_slice(&sample_rate.to_le_bytes()); // SampleRate
    hdr[28..32].copy_from_slice(&byte_rate.to_le_bytes());   // ByteRate (Bps)
    hdr[32..34].copy_from_slice(&block_align.to_le_bytes()); // BlockAlign
    hdr[34..36].copy_from_slice(&bits_per_sample.to_le_bytes()); // BitsPerSample
    hdr[36..40].copy_from_slice(b"data"); // SubChunk2Id
    hdr[40..44].copy_from_slice(&subchunksize.to_le_bytes()); // SubChunk2Size
    debug!("WAV Header (l={}): \r\n{:02x?}", hdr.len(), hdr);
    hdr
}
