use crate::enhancement::deepfilternet;
use crate::ffmpeg::{extract, probe};
use crate::models::media::MediaFile;
use std::path::PathBuf;

#[tauri::command]
pub async fn probe_media_file(path: String) -> Result<MediaFile, String> {
    let path = PathBuf::from(path);

    if !path.exists() {
        return Err(format!("File does not exist: {}", path.display()));
    }

    probe::probe_media_file(&path).await
}

#[tauri::command]
pub async fn extract_audio_from_media(
    media_path: String,
    output_path: String,
) -> Result<(), String> {
    let media_path = PathBuf::from(media_path);

    // First probe the media file
    let media = probe::probe_media_file(&media_path).await?;

    // Then extract the audio
    let output_path = PathBuf::from(output_path);
    extract::extract_audio(&media, &output_path).await
}

#[tauri::command]
pub async fn enhance_audio_file(
    input_path: String,
    output_path: String,
    intensity: f32,
) -> Result<(), String> {
    let input_path = PathBuf::from(input_path);
    let output_path = PathBuf::from(output_path);

    // Validate intensity range
    if !(0.0..=1.0).contains(&intensity) {
        return Err("Intensity must be between 0.0 and 1.0".to_string());
    }

    deepfilternet::enhance_audio(&input_path, &output_path, intensity).await
}
