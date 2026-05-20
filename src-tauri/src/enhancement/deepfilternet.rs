use crate::enhancement::chunking::{chunk_audio, reconstruct_from_chunks};
use crate::utils::audio_io::{load_audio_f32, save_audio_f32};
use ort::{execution_providers::CUDAExecutionProvider, session::Session};
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};

const SAMPLE_RATE: u32 = 48000;
const CHUNK_DURATION_SECS: f32 = 5.0;
const OVERLAP_DURATION_SECS: f32 = 0.5;

// Model frame size (DeepFilterNet typically processes 96 or 480 samples per frame)
const MODEL_FRAME_SIZE: usize = 480; // Adjust based on actual model

// Global ONNX session (initialized once on first use) - wrapped in Mutex for interior mutability
static ONNX_SESSION: OnceLock<Arc<Mutex<Session>>> = OnceLock::new();

/// Initialize the ONNX session for DeepFilterNet
fn get_or_init_session() -> Result<Arc<Mutex<Session>>, String> {
    if let Some(session) = ONNX_SESSION.get() {
        return Ok(session.clone());
    }

    // Initialize session
        // Try to find the model in standard locations
        let model_paths = [
            "models/deepfilternet.onnx",
            "../models/deepfilternet.onnx",
            "./deepfilternet.onnx",
        ];

        let model_path = model_paths
            .iter()
            .find(|p| Path::new(p).exists())
            .ok_or_else(|| {
                format!(
                    "DeepFilterNet ONNX model not found. Expected at: {}. \
                    Please download from https://github.com/Rikorose/DeepFilterNet/releases",
                    model_paths[0]
                )
            })?;

    println!("Loading DeepFilterNet model from: {}", model_path);

    // Create ONNX session with optimizations
    let session = Session::builder()
        .map_err(|e| format!("Failed to create session builder: {}", e))?
        // Try to use CUDA if available, fallback to CPU
        .with_execution_providers([
            CUDAExecutionProvider::default().build()
        ])
        .map_err(|e| format!("Failed to configure execution providers: {}", e))?
        .commit_from_file(model_path)
        .map_err(|e| format!("Failed to load ONNX model: {}", e))?;

    let session_arc = Arc::new(Mutex::new(session));

    // Try to set the session (may fail if another thread initialized first)
    match ONNX_SESSION.set(session_arc.clone()) {
        Ok(_) => Ok(session_arc),
        Err(_) => {
            // Another thread initialized first, use that one
            Ok(ONNX_SESSION.get().unwrap().clone())
        }
    }
}

/// Enhance audio using DeepFilterNet ONNX model
pub async fn enhance_audio(
    input_path: &Path,
    output_path: &Path,
    intensity: f32, // 0.0 to 1.0 (dry/wet mix)
) -> Result<(), String> {
    // Load audio
    let (audio, sample_rate) = load_audio_f32(input_path)?;

    if sample_rate != SAMPLE_RATE {
        return Err(format!(
            "Expected sample rate {}Hz, got {}Hz. Please resample first.",
            SAMPLE_RATE, sample_rate
        ));
    }

    // Get or initialize ONNX session
    let session = get_or_init_session()?;

    // Calculate chunk sizes
    let chunk_size = (CHUNK_DURATION_SECS * sample_rate as f32) as usize;
    let overlap_size = (OVERLAP_DURATION_SECS * sample_rate as f32) as usize;

    // Split into chunks
    let chunks = chunk_audio(&audio, chunk_size, overlap_size);

    // Process each chunk
    let mut processed_chunks = Vec::new();
    for (i, chunk) in chunks.iter().enumerate() {
        let processed = {
            let mut session_guard = session
                .lock()
                .map_err(|e| format!("Failed to lock session: {}", e))?;
            run_onnx_inference(&mut session_guard, chunk)?
        };
        processed_chunks.push(processed);

        // Progress reporting
        let progress = (i + 1) as f32 / chunks.len() as f32;
        println!("Enhancement progress: {:.1}%", progress * 100.0);
    }

    // Reconstruct audio from processed chunks
    let enhanced_audio = reconstruct_from_chunks(&processed_chunks, chunk_size, overlap_size);

    // Apply dry/wet mix based on intensity
    let final_audio = if intensity < 1.0 {
        mix_audio(&audio[..enhanced_audio.len()], &enhanced_audio, intensity)
    } else {
        enhanced_audio
    };

    // Save output
    save_audio_f32(output_path, &final_audio, sample_rate)?;

    Ok(())
}

/// Run ONNX inference on an audio chunk
fn run_onnx_inference(session: &mut Session, audio: &[f32]) -> Result<Vec<f32>, String> {
    use ort::inputs;
    use ort::value::Tensor;
    use ndarray::Array;

    // DeepFilterNet expects input shape: [batch, channels, samples]
    // For mono audio: [1, 1, num_samples]
    let batch_size = 1;
    let channels = 1;
    let num_samples = audio.len();

    // Pad audio to multiple of frame size if needed
    let padded_len = ((num_samples + MODEL_FRAME_SIZE - 1) / MODEL_FRAME_SIZE) * MODEL_FRAME_SIZE;
    let mut padded_audio = audio.to_vec();
    padded_audio.resize(padded_len, 0.0);

    // Create input tensor [1, 1, samples] using from_shape_vec
    let input_array = Array::from_shape_vec(
        (batch_size, channels, padded_len),
        padded_audio
    ).map_err(|e| format!("Failed to create input array: {}", e))?;

    // Convert to Value using from_array
    let input_tensor = Tensor::from_array(input_array)
        .map_err(|e| format!("Failed to create tensor: {}", e))?;

    // Run inference - inputs! macro expects Value objects
    let outputs = session
        .run(inputs!["input" => input_tensor])
        .map_err(|e| format!("ONNX inference failed: {}", e))?;

    // Extract output tensor by name
    let output_tensor = outputs
        .get("output")
        .ok_or_else(|| "Output tensor 'output' not found".to_string())?;

    // Extract the tensor data
    let (_, output_slice) = output_tensor
        .try_extract_tensor::<f32>()
        .map_err(|e| format!("Failed to extract output tensor: {}", e))?;

    // Take only the original unpadded length
    let mut result = output_slice[..num_samples.min(output_slice.len())].to_vec();

    // Ensure result matches input length
    result.resize(num_samples, 0.0);

    Ok(result)
}

/// Dry/wet mix utility
pub fn mix_audio(dry: &[f32], wet: &[f32], intensity: f32) -> Vec<f32> {
    dry.iter()
        .zip(wet.iter())
        .map(|(&d, &w)| d * (1.0 - intensity) + w * intensity)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder_enhancement() {
        let audio = vec![0.001, 0.5, 0.002, -0.6];
        let enhanced = apply_placeholder_enhancement(&audio, 0.5);

        // Should attenuate quiet samples
        assert!(enhanced[0].abs() < audio[0].abs());
        assert!(enhanced[2].abs() < audio[2].abs());

        // Should preserve loud samples
        assert_eq!(enhanced[1], audio[1]);
        assert_eq!(enhanced[3], audio[3]);
    }

    #[test]
    fn test_mix_audio() {
        let dry = vec![1.0, 0.5, 0.0];
        let wet = vec![0.0, 0.5, 1.0];
        let mixed = mix_audio(&dry, &wet, 0.5);

        assert_eq!(mixed, vec![0.5, 0.5, 0.5]);
    }
}
