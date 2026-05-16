use crate::models::media::{MediaFile, AudioStreamInfo, VideoStreamInfo};
use serde_json::Value;
use std::path::Path;
use std::process::Command;

pub async fn probe_media_file(path: &Path) -> Result<MediaFile, String> {
    // Run ffprobe to get media information
    let output = Command::new("ffprobe")
        .args(&[
            "-v", "quiet",
            "-print_format", "json",
            "-show_format",
            "-show_streams",
            path.to_str().ok_or("Invalid path")?,
        ])
        .output()
        .map_err(|e| format!("Failed to run ffprobe: {}. Make sure FFmpeg is installed.", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("ffprobe failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse ffprobe output: {}", e))?;

    // Extract format information
    let format = json["format"]["format_name"]
        .as_str()
        .ok_or("Missing format name")?
        .to_string();

    let duration_secs = json["format"]["duration"]
        .as_str()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);

    // Extract stream information
    let streams = json["streams"]
        .as_array()
        .ok_or("Missing streams array")?;

    let mut video_streams = Vec::new();
    let mut audio_streams = Vec::new();

    for stream in streams {
        let codec_type = stream["codec_type"].as_str().unwrap_or("");

        match codec_type {
            "video" => {
                let video_info = VideoStreamInfo {
                    index: stream["index"].as_u64().unwrap_or(0) as u32,
                    codec: stream["codec_name"].as_str().unwrap_or("unknown").to_string(),
                    width: stream["width"].as_u64().unwrap_or(0) as u32,
                    height: stream["height"].as_u64().unwrap_or(0) as u32,
                    fps: stream["r_frame_rate"]
                        .as_str()
                        .and_then(|s| parse_fraction(s))
                        .unwrap_or(0.0),
                    bitrate: stream["bit_rate"]
                        .as_str()
                        .and_then(|s| s.parse::<u64>().ok()),
                };
                video_streams.push(video_info);
            }
            "audio" => {
                let audio_info = AudioStreamInfo {
                    index: stream["index"].as_u64().unwrap_or(0) as u32,
                    codec: stream["codec_name"].as_str().unwrap_or("unknown").to_string(),
                    sample_rate: stream["sample_rate"]
                        .as_str()
                        .and_then(|s| s.parse::<u32>().ok())
                        .unwrap_or(0),
                    channels: stream["channels"].as_u64().unwrap_or(0) as u32,
                    bitrate: stream["bit_rate"]
                        .as_str()
                        .and_then(|s| s.parse::<u64>().ok()),
                };
                audio_streams.push(audio_info);
            }
            _ => {}
        }
    }

    if audio_streams.is_empty() {
        return Err("No audio stream found in media file".to_string());
    }

    Ok(MediaFile {
        path: path.to_path_buf(),
        format,
        duration_secs,
        video_streams,
        audio_streams,
    })
}

fn parse_fraction(s: &str) -> Option<f64> {
    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() == 2 {
        let numerator = parts[0].parse::<f64>().ok()?;
        let denominator = parts[1].parse::<f64>().ok()?;
        if denominator != 0.0 {
            return Some(numerator / denominator);
        }
    }
    None
}
