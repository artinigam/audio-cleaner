use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoudnessMetrics {
    pub integrated: f64,     // Integrated loudness (LUFS)
    pub true_peak: f64,      // True peak (dBTP)
    pub lra: f64,            // Loudness range (LU)
    pub threshold: f64,      // Gating threshold
}

/// Measure loudness using FFmpeg's ebur128 filter (ITU-R BS.1770-4)
pub async fn measure_loudness(audio_path: &Path) -> Result<LoudnessMetrics, String> {
    if !audio_path.exists() {
        return Err(format!("Audio file does not exist: {}", audio_path.display()));
    }

    // Run FFmpeg with ebur128 filter to measure loudness
    // peak=true ensures we get true peak measurements
    let output = Command::new("ffmpeg")
        .args([
            "-i",
            audio_path.to_str().unwrap(),
            "-af",
            "ebur128=peak=true",
            "-f",
            "null",
            "-",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("Failed to run ffmpeg for loudness measurement: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "FFmpeg loudness measurement failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    // Parse the ebur128 output from stderr
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Debug: print the output to see what we're getting
    println!("=== ebur128 output ===");
    for line in stderr.lines().rev().take(15).collect::<Vec<_>>().iter().rev() {
        println!("{}", line);
    }
    println!("======================");

    parse_ebur128_output(&stderr)
}

/// Parse ebur128 filter output to extract loudness metrics
fn parse_ebur128_output(output: &str) -> Result<LoudnessMetrics, String> {
    let mut integrated = None;
    let mut true_peak = None;
    let mut lra = None;
    let mut threshold = None;
    let mut in_integrated_section = false;
    let mut in_lra_section = false;
    let mut in_peak_section = false;

    // Parse the Summary section at the end
    // Format:
    //   Integrated loudness:
    //     I:         -31.5 LUFS
    //     Threshold: -41.5 LUFS
    //   Loudness range:
    //     LRA:         0.0 LU
    //     Threshold: -51.5 LUFS
    //   True peak:
    //     Peak:      -23.9 dBFS

    for line in output.lines() {
        let line = line.trim();

        // Track which section we're in
        if line.contains("Integrated loudness:") {
            in_integrated_section = true;
            in_lra_section = false;
            in_peak_section = false;
        } else if line.contains("Loudness range:") {
            in_integrated_section = false;
            in_lra_section = true;
            in_peak_section = false;
        } else if line.contains("True peak:") {
            in_integrated_section = false;
            in_lra_section = false;
            in_peak_section = true;
        }

        // Parse values based on current section
        if line.starts_with("I:") && in_integrated_section {
            if let Some(value) = extract_float_value(line, "LUFS") {
                integrated = Some(value);
            }
        } else if line.starts_with("Threshold:") && in_integrated_section {
            if let Some(value) = extract_float_value(line, "LUFS") {
                threshold = Some(value);
            }
        } else if line.starts_with("LRA:") && in_lra_section {
            if let Some(value) = extract_float_value(line, "LU") {
                lra = Some(value);
            }
        } else if line.starts_with("Peak:") && in_peak_section {
            if let Some(value) = extract_float_value(line, "dBFS") {
                true_peak = Some(value);
            } else if let Some(value) = extract_float_value(line, "dBTP") {
                true_peak = Some(value);
            }
        }
    }

    // Verify we got required metrics (true_peak is optional)
    match (integrated, lra, threshold) {
        (Some(i), Some(l), Some(t)) => Ok(LoudnessMetrics {
            integrated: i,
            true_peak: true_peak.unwrap_or(0.0), // Default to 0.0 if not available
            lra: l,
            threshold: t,
        }),
        _ => Err(format!(
            "Failed to parse loudness metrics from FFmpeg output. Found: I={:?}, TP={:?}, LRA={:?}, T={:?}",
            integrated, true_peak, lra, threshold
        )),
    }
}

/// Extract a float value from a line like "  I:         -23.0 LUFS"
fn extract_float_value(line: &str, unit: &str) -> Option<f64> {
    // Remove the unit suffix
    let without_unit = line.replace(unit, "");

    // Split by whitespace and find the numeric value
    for part in without_unit.split_whitespace() {
        if let Ok(value) = part.parse::<f64>() {
            return Some(value);
        }
    }
    None
}

/// Normalize audio to target LUFS using FFmpeg's loudnorm filter
pub async fn normalize_loudness(
    input_path: &Path,
    output_path: &Path,
    target_lufs: f64,
) -> Result<(), String> {
    if !input_path.exists() {
        return Err(format!("Input audio file does not exist: {}", input_path.display()));
    }

    // Use FFmpeg's loudnorm filter with two-pass mode for accurate normalization
    // First pass: analyze
    let analyze_output = Command::new("ffmpeg")
        .args([
            "-i",
            input_path.to_str().unwrap(),
            "-af",
            &format!("loudnorm=I={}:TP=-1.5:LRA=11:print_format=json", target_lufs),
            "-f",
            "null",
            "-",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("Failed to run ffmpeg loudnorm analysis: {}", e))?;

    let stderr = String::from_utf8_lossy(&analyze_output.stderr);

    // Extract measured values from JSON output
    let (measured_i, measured_lra, measured_tp, measured_thresh) = parse_loudnorm_json(&stderr)?;

    // Second pass: apply normalization with measured values
    let normalize_output = Command::new("ffmpeg")
        .args([
            "-i",
            input_path.to_str().unwrap(),
            "-af",
            &format!(
                "loudnorm=I={}:TP=-1.5:LRA=11:measured_I={}:measured_LRA={}:measured_TP={}:measured_thresh={}:linear=true:print_format=summary",
                target_lufs, measured_i, measured_lra, measured_tp, measured_thresh
            ),
            "-ar",
            "48000",
            "-y",
            output_path.to_str().unwrap(),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("Failed to run ffmpeg loudnorm normalization: {}", e))?;

    if !normalize_output.status.success() {
        return Err(format!(
            "FFmpeg normalization failed: {}",
            String::from_utf8_lossy(&normalize_output.stderr)
        ));
    }

    Ok(())
}

/// Parse loudnorm JSON output to extract measured values
fn parse_loudnorm_json(output: &str) -> Result<(f64, f64, f64, f64), String> {
    // Find the JSON block in the output (between { and })
    let json_start = output.rfind('{').ok_or("No JSON found in loudnorm output")?;
    let json_end = output.rfind('}').ok_or("No JSON found in loudnorm output")?;

    let json_str = &output[json_start..=json_end];

    // Parse JSON manually (simple approach, could use serde_json)
    let measured_i = extract_json_value(json_str, "input_i")?;
    let measured_lra = extract_json_value(json_str, "input_lra")?;
    let measured_tp = extract_json_value(json_str, "input_tp")?;
    let measured_thresh = extract_json_value(json_str, "input_thresh")?;

    Ok((measured_i, measured_lra, measured_tp, measured_thresh))
}

/// Extract a numeric value from JSON string (simple parser)
fn extract_json_value(json: &str, key: &str) -> Result<f64, String> {
    let search = format!("\"{}\"", key);

    if let Some(pos) = json.find(&search) {
        let after_key = &json[pos + search.len()..];

        // Find the colon
        if let Some(colon_pos) = after_key.find(':') {
            let after_colon = &after_key[colon_pos + 1..];

            // Extract until comma or closing brace
            let value_str = after_colon
                .trim()
                .trim_start_matches('"')
                .split(|c| c == ',' || c == '}' || c == '"')
                .next()
                .unwrap_or("")
                .trim();

            value_str
                .parse::<f64>()
                .map_err(|_| format!("Failed to parse {} value: {}", key, value_str))
        } else {
            Err(format!("No colon found after key {}", key))
        }
    } else {
        Err(format!("Key {} not found in JSON", key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_float_value() {
        assert_eq!(
            extract_float_value("  I:         -23.0 LUFS", "LUFS"),
            Some(-23.0)
        );
        assert_eq!(
            extract_float_value("  True peak:  -0.3 dBTP", "dBTP"),
            Some(-0.3)
        );
    }

    #[test]
    fn test_extract_json_value() {
        let json = r#"{"input_i":"-23.5","input_lra":"5.2","input_tp":"-1.0","input_thresh":"-33.8"}"#;
        assert_eq!(extract_json_value(json, "input_i").unwrap(), -23.5);
        assert_eq!(extract_json_value(json, "input_lra").unwrap(), 5.2);
    }
}
