use crate::models::media::MediaFile;
use std::path::Path;
use std::process::Command;

pub async fn extract_audio(media: &MediaFile, output: &Path) -> Result<(), String> {
    // Select the best audio stream (highest bitrate)
    let audio_stream = media
        .audio_streams
        .iter()
        .max_by_key(|s| s.bitrate.unwrap_or(0))
        .ok_or("No audio stream available")?;

    // Create parent directory if it doesn't exist
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;
    }

    // Use absolute path to ffmpeg (Homebrew location on macOS)
    let ffmpeg_path = if std::path::Path::new("/opt/homebrew/bin/ffmpeg").exists() {
        "/opt/homebrew/bin/ffmpeg"
    } else if std::path::Path::new("/usr/local/bin/ffmpeg").exists() {
        "/usr/local/bin/ffmpeg"
    } else {
        "ffmpeg" // fallback to PATH
    };

    // Extract audio to WAV format: 48kHz, mono, 16-bit PCM
    let output_result = Command::new(ffmpeg_path)
        .args(&[
            "-i",
            media.path.to_str().ok_or("Invalid input path")?,
            "-map",
            &format!("0:{}", audio_stream.index),
            "-ar",
            "48000", // 48kHz sample rate
            "-ac",
            "1", // Mono
            "-sample_fmt",
            "s16", // 16-bit PCM
            "-y", // Overwrite output file
            output.to_str().ok_or("Invalid output path")?,
        ])
        .output()
        .map_err(|e| format!("Failed to run ffmpeg ({}): {}. Make sure FFmpeg is installed.", ffmpeg_path, e))?;

    if !output_result.status.success() {
        let stderr = String::from_utf8_lossy(&output_result.stderr);
        return Err(format!("Audio extraction failed: {}", stderr));
    }

    // Validate output file exists and is non-empty
    if !output.exists() {
        return Err("Output file was not created".to_string());
    }

    let metadata = std::fs::metadata(output)
        .map_err(|e| format!("Failed to read output file metadata: {}", e))?;

    if metadata.len() == 0 {
        return Err("Output file is empty".to_string());
    }

    Ok(())
}
