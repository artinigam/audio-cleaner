use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

/// Remux video with new audio track (no re-encoding of video)
/// Handles both video files and audio-only files
pub async fn remux_video_with_audio(
    video_path: &Path,
    audio_path: &Path,
    output_path: &Path,
) -> Result<(), String> {
    if !video_path.exists() {
        return Err(format!("Video file does not exist: {}", video_path.display()));
    }

    if !audio_path.exists() {
        return Err(format!("Audio file does not exist: {}", audio_path.display()));
    }

    // First, check if the input has a video stream
    let has_video = check_has_video_stream(video_path).await?;

    if has_video {
        // Input has video: use stream copy for video, encode audio
        println!("Remuxing video file with new audio track...");
        remux_with_video(video_path, audio_path, output_path).await
    } else {
        // Input is audio-only: just encode the processed audio
        println!("Input is audio-only, encoding processed audio...");
        encode_audio_only(audio_path, output_path).await
    }
}

/// Check if a media file has a video stream
async fn check_has_video_stream(path: &Path) -> Result<bool, String> {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=codec_type",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            path.to_str().unwrap(),
        ])
        .output()
        .await
        .map_err(|e| format!("Failed to probe for video stream: {}", e))?;

    // If output contains "video", the file has a video stream
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.trim() == "video")
}

/// Remux video file with new audio (stream copy for video)
async fn remux_with_video(
    video_path: &Path,
    audio_path: &Path,
    output_path: &Path,
) -> Result<(), String> {
    let output = Command::new("ffmpeg")
        .args([
            "-i",
            video_path.to_str().unwrap(),
            "-i",
            audio_path.to_str().unwrap(),
            "-c:v",
            "copy", // Don't re-encode video
            "-c:a",
            "aac", // Encode audio to AAC
            "-b:a",
            "192k",
            "-ar",
            "48000",
            "-map",
            "0:v:0", // Video from first input
            "-map",
            "1:a:0", // Audio from second input
            "-shortest",
            "-y", // Overwrite output
            output_path.to_str().unwrap(),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("Failed to run ffmpeg for remuxing: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "FFmpeg remuxing failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

/// Encode audio-only output (for files without video)
async fn encode_audio_only(
    audio_path: &Path,
    output_path: &Path,
) -> Result<(), String> {
    let output = Command::new("ffmpeg")
        .args([
            "-i",
            audio_path.to_str().unwrap(),
            "-c:a",
            "aac", // Encode to AAC
            "-b:a",
            "192k",
            "-ar",
            "48000",
            "-y", // Overwrite output
            output_path.to_str().unwrap(),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("Failed to run ffmpeg for audio encoding: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "FFmpeg audio encoding failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

/// Remux video with new audio track, preserving original audio codec if possible
/// Falls back to AAC if original codec is not supported
pub async fn remux_video_with_audio_preserve_codec(
    video_path: &Path,
    audio_path: &Path,
    output_path: &Path,
    audio_codec: Option<&str>,
) -> Result<(), String> {
    if !video_path.exists() {
        return Err(format!("Video file does not exist: {}", video_path.display()));
    }

    if !audio_path.exists() {
        return Err(format!("Audio file does not exist: {}", audio_path.display()));
    }

    // Determine audio encoding settings
    let (codec_arg, bitrate_arg) = match audio_codec {
        Some("aac") => ("aac", "192k"),
        Some("mp3") => ("libmp3lame", "192k"),
        Some("opus") => ("libopus", "128k"),
        _ => ("aac", "192k"), // Default to AAC
    };

    let output = Command::new("ffmpeg")
        .args([
            "-i",
            video_path.to_str().unwrap(),
            "-i",
            audio_path.to_str().unwrap(),
            "-c:v",
            "copy",
            "-c:a",
            codec_arg,
            "-b:a",
            bitrate_arg,
            "-ar",
            "48000",
            "-map",
            "0:v:0",
            "-map",
            "1:a:0",
            "-shortest",
            "-y",
            output_path.to_str().unwrap(),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("Failed to run ffmpeg for remuxing: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "FFmpeg remuxing failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires FFmpeg and test media files
    async fn test_remux_video_with_audio() {
        // This test is ignored by default as it requires actual media files
        // To run: cargo test -- --ignored
        let video_path = Path::new("/tmp/test_video.mp4");
        let audio_path = Path::new("/tmp/test_audio.wav");
        let output_path = Path::new("/tmp/test_output.mp4");

        // Test would require actual files
        // let result = remux_video_with_audio(video_path, audio_path, output_path).await;
        // assert!(result.is_ok());
    }
}
