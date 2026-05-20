use crate::enhancement::deepfilternet_cli;
use crate::ffmpeg::{extract, loudness, probe, remux};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineProgress {
    pub stage: String,
    pub percentage: f32,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineResult {
    pub output_path: String,
    pub original_loudness: f64,
    pub final_loudness: f64,
    pub processing_time_seconds: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineOptions {
    pub target_lufs: f64,
    pub enhancement_intensity: f32,
}

impl Default for PipelineOptions {
    fn default() -> Self {
        Self {
            target_lufs: -14.0, // YouTube standard
            enhancement_intensity: 0.8,
        }
    }
}

/// Run the complete audio enhancement pipeline
/// 1. Probe media file
/// 2. Extract audio
/// 3. Measure original loudness
/// 4. Enhance audio (DeepFilterNet)
/// 5. Normalize loudness
/// 6. Remux video with processed audio
#[tauri::command]
pub async fn process_video_file(
    video_path: String,
    output_path: String,
    options: Option<PipelineOptions>,
) -> Result<PipelineResult, String> {
    let start_time = std::time::Instant::now();
    let video_path = PathBuf::from(video_path);
    let output_path = PathBuf::from(output_path);

    let options = options.unwrap_or_default();

    // Validate input
    if !video_path.exists() {
        return Err(format!("Video file does not exist: {}", video_path.display()));
    }

    // Create temporary directory for intermediate files
    let temp_dir = std::env::temp_dir().join(format!("audio-cleaner-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("Failed to create temp directory: {}", e))?;

    // Define temporary file paths
    let extracted_audio = temp_dir.join("extracted.wav");
    let enhanced_audio = temp_dir.join("enhanced.wav");
    let normalized_audio = temp_dir.join("normalized.wav");

    // Stage 1: Probe media file (5%)
    println!("Stage 1: Probing media file...");
    let media_info = probe::probe_media_file(&video_path).await?;

    // Stage 2: Extract audio (15%)
    println!("Stage 2: Extracting audio...");
    extract::extract_audio(&media_info, &extracted_audio).await?;

    // Stage 3: Measure original loudness (5%)
    println!("Stage 3: Measuring original loudness...");
    let original_metrics = loudness::measure_loudness(&extracted_audio).await?;
    println!(
        "Original loudness: {:.1} LUFS (target: {:.1} LUFS)",
        original_metrics.integrated, options.target_lufs
    );

    // Stage 4: Enhance audio with DeepFilterNet (50%)
    println!("Stage 4: Enhancing audio with DeepFilterNet CLI...");
    deepfilternet_cli::enhance_audio(
        &extracted_audio,
        &enhanced_audio,
        options.enhancement_intensity,
    )
    .await?;

    // Stage 5: Normalize loudness (15%)
    println!("Stage 5: Normalizing loudness to {:.1} LUFS...", options.target_lufs);
    loudness::normalize_loudness(&enhanced_audio, &normalized_audio, options.target_lufs).await?;

    // Measure final loudness
    let final_metrics = loudness::measure_loudness(&normalized_audio).await?;
    println!("Final loudness: {:.1} LUFS", final_metrics.integrated);

    // Stage 6: Remux video with processed audio (10%)
    println!("Stage 6: Remuxing video with processed audio...");
    remux::remux_video_with_audio(&video_path, &normalized_audio, &output_path).await?;

    // Clean up temporary files
    let _ = std::fs::remove_dir_all(&temp_dir);

    let processing_time = start_time.elapsed().as_secs_f64();
    println!("Processing complete in {:.1}s", processing_time);

    Ok(PipelineResult {
        output_path: output_path.to_string_lossy().to_string(),
        original_loudness: original_metrics.integrated,
        final_loudness: final_metrics.integrated,
        processing_time_seconds: processing_time,
    })
}

/// Quick preview: extract and enhance a short segment (first 30 seconds)
#[tauri::command]
pub async fn generate_preview(
    video_path: String,
    output_path: String,
    duration_seconds: Option<f32>,
) -> Result<PipelineResult, String> {
    let video_path = PathBuf::from(video_path);
    let output_path = PathBuf::from(output_path);
    let duration = duration_seconds.unwrap_or(30.0);

    if !video_path.exists() {
        return Err(format!("Video file does not exist: {}", video_path.display()));
    }

    // Create temporary directory
    let temp_dir = std::env::temp_dir().join(format!("audio-cleaner-preview-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("Failed to create temp directory: {}", e))?;

    let extracted_audio = temp_dir.join("preview_extracted.wav");
    let enhanced_audio = temp_dir.join("preview_enhanced.wav");
    let normalized_audio = temp_dir.join("preview_normalized.wav");

    // Extract short segment
    println!("Generating {:.0}s preview...", duration);

    // Use FFmpeg to extract first N seconds of audio
    extract_audio_segment(&video_path, &extracted_audio, duration).await?;

    // Measure original loudness
    let original_metrics = loudness::measure_loudness(&extracted_audio).await?;

    // Enhance
    deepfilternet_cli::enhance_audio(&extracted_audio, &enhanced_audio, 0.8).await?;

    // Normalize to -14 LUFS (YouTube)
    loudness::normalize_loudness(&enhanced_audio, &normalized_audio, -14.0).await?;

    // Measure final
    let final_metrics = loudness::measure_loudness(&normalized_audio).await?;

    // Copy normalized audio to output path for preview playback
    std::fs::copy(&normalized_audio, &output_path)
        .map_err(|e| format!("Failed to copy preview audio: {}", e))?;

    // Clean up
    let _ = std::fs::remove_dir_all(&temp_dir);

    Ok(PipelineResult {
        output_path: output_path.to_string_lossy().to_string(),
        original_loudness: original_metrics.integrated,
        final_loudness: final_metrics.integrated,
        processing_time_seconds: 0.0,
    })
}

/// Extract audio segment using FFmpeg
async fn extract_audio_segment(
    video_path: &Path,
    output_path: &Path,
    duration_seconds: f32,
) -> Result<(), String> {
    use std::process::Stdio;
    use tokio::process::Command;

    let output = Command::new("ffmpeg")
        .args([
            "-i",
            video_path.to_str().unwrap(),
            "-t",
            &duration_seconds.to_string(),
            "-vn",
            "-ar",
            "48000",
            "-ac",
            "1",
            "-c:a",
            "pcm_s16le",
            "-y",
            output_path.to_str().unwrap(),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("Failed to extract audio segment: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "FFmpeg audio segment extraction failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}
