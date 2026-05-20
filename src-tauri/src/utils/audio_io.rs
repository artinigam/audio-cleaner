use hound::{WavReader, WavWriter, WavSpec, SampleFormat};
use std::path::Path;

/// Load audio from WAV file as f32 samples normalized to [-1.0, 1.0]
pub fn load_audio_f32(path: &Path) -> Result<(Vec<f32>, u32), String> {
    let reader = WavReader::open(path)
        .map_err(|e| format!("Failed to open WAV file: {}", e))?;

    let spec = reader.spec();
    let sample_rate = spec.sample_rate;

    let samples = match spec.sample_format {
        SampleFormat::Float => {
            reader.into_samples::<f32>()
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| format!("Failed to read samples: {}", e))?
        }
        SampleFormat::Int => {
            let bits = spec.bits_per_sample;
            let max_value = (1 << (bits - 1)) as f32;

            reader.into_samples::<i32>()
                .map(|s| s.map(|sample| sample as f32 / max_value))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| format!("Failed to read samples: {}", e))?
        }
    };

    Ok((samples, sample_rate))
}

/// Save f32 audio samples to WAV file (16-bit PCM)
pub fn save_audio_f32(path: &Path, samples: &[f32], sample_rate: u32) -> Result<(), String> {
    let spec = WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };

    let mut writer = WavWriter::create(path, spec)
        .map_err(|e| format!("Failed to create WAV file: {}", e))?;

    // Convert f32 [-1.0, 1.0] to i16
    for &sample in samples {
        let clamped = sample.clamp(-1.0, 1.0);
        let i16_sample = (clamped * 32767.0) as i16;
        writer.write_sample(i16_sample)
            .map_err(|e| format!("Failed to write sample: {}", e))?;
    }

    writer.finalize()
        .map_err(|e| format!("Failed to finalize WAV file: {}", e))?;

    Ok(())
}
