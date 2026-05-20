use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

/// Enhance audio using DeepFilterNet CLI tool
/// This is simpler than ONNX Runtime and uses the official pre-built binary
pub async fn enhance_audio(
    input_path: &Path,
    output_path: &Path,
    intensity: f32,
) -> Result<(), String> {
    if !input_path.exists() {
        return Err(format!("Input audio file does not exist: {}", input_path.display()));
    }

    // Validate intensity
    if !(0.0..=1.0).contains(&intensity) {
        return Err("Intensity must be between 0.0 and 1.0".to_string());
    }

    // Find the DeepFilterNet CLI binary
    let cli_paths = [
        "models/deep-filter",
        "../models/deep-filter",
        "./deep-filter",
    ];

    let cli_path = cli_paths
        .iter()
        .find(|p| Path::new(p).exists())
        .ok_or_else(|| {
            format!(
                "DeepFilterNet CLI not found. Expected at: {}. \
                Please download from https://github.com/Rikorose/DeepFilterNet/releases",
                cli_paths[0]
            )
        })?;

    println!("Using DeepFilterNet CLI: {}", cli_path);

    // Convert intensity (0.0-1.0) to attenuation limit in dB
    // intensity 1.0 = full attenuation (100 dB)
    // intensity 0.0 = no attenuation (0 dB)
    let atten_lim_db = intensity * 100.0;

    // Create a temporary output directory
    let temp_dir = std::env::temp_dir().join(format!("deepfilter-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("Failed to create temp directory: {}", e))?;

    println!("DeepFilterNet: intensity={:.0}%, atten_lim={:.1}dB", intensity * 100.0, atten_lim_db);

    // Run DeepFilterNet CLI
    // --output-dir: output directory
    // --atten-lim-db: control intensity (0-100 dB)
    // --compensate-delay: maintain sync
    let output = Command::new(cli_path)
        .args([
            input_path.to_str().unwrap(),
            "--output-dir",
            temp_dir.to_str().unwrap(),
            "--atten-lim-db",
            &format!("{:.1}", atten_lim_db),
            "--compensate-delay",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("Failed to run DeepFilterNet CLI: {}", e))?;

    if !output.status.success() {
        let _ = std::fs::remove_dir_all(&temp_dir);
        return Err(format!(
            "DeepFilterNet enhancement failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    // DeepFilterNet creates output with the same filename in the output directory
    let input_filename = input_path.file_name().unwrap();
    let cli_output = temp_dir.join(input_filename);

    if cli_output.exists() {
        std::fs::copy(&cli_output, output_path)
            .map_err(|e| format!("Failed to copy output file: {}", e))?;

        // Clean up temp directory
        let _ = std::fs::remove_dir_all(&temp_dir);
    } else {
        let _ = std::fs::remove_dir_all(&temp_dir);
        return Err(format!(
            "DeepFilterNet output file not found. Expected: {}",
            cli_output.display()
        ));
    }

    println!("Enhancement complete: {}", output_path.display());
    Ok(())
}

/// Check if DeepFilterNet CLI is available
pub fn is_available() -> bool {
    let cli_paths = [
        "models/deep-filter",
        "../models/deep-filter",
        "./deep-filter",
    ];

    cli_paths.iter().any(|p| Path::new(p).exists())
}

/// Get version info from DeepFilterNet CLI
pub async fn get_version() -> Result<String, String> {
    let cli_paths = [
        "models/deep-filter",
        "../models/deep-filter",
        "./deep-filter",
    ];

    let cli_path = cli_paths
        .iter()
        .find(|p| Path::new(p).exists())
        .ok_or("DeepFilterNet CLI not found")?;

    let output = Command::new(cli_path)
        .arg("--version")
        .output()
        .await
        .map_err(|e| format!("Failed to get version: {}", e))?;

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_available() {
        // Just check if the function runs without panic
        let _ = is_available();
    }

    #[tokio::test]
    #[ignore] // Requires actual CLI binary and audio file
    async fn test_enhance_audio() {
        let input = Path::new("/tmp/test_input.wav");
        let output = Path::new("/tmp/test_output.wav");

        // This test would require actual files
        // let result = enhance_audio(input, output, 0.8).await;
        // assert!(result.is_ok());
    }
}
