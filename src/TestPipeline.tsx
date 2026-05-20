import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

interface PipelineOptions {
  target_lufs: number;
  enhancement_intensity: number;
}

interface PipelineResult {
  output_path: string;
  original_loudness: number;
  final_loudness: number;
  processing_time_seconds: number;
}

function TestPipeline() {
  const [videoPath, setVideoPath] = useState<string>("");
  const [outputPath, setOutputPath] = useState<string>("");
  const [targetLufs, setTargetLufs] = useState<number>(-14);
  const [intensity, setIntensity] = useState<number>(0.8);
  const [isProcessing, setIsProcessing] = useState<boolean>(false);
  const [result, setResult] = useState<PipelineResult | null>(null);
  const [error, setError] = useState<string>("");
  const [log, setLog] = useState<string[]>([]);

  const addLog = (message: string) => {
    setLog((prev) => [...prev, `[${new Date().toLocaleTimeString()}] ${message}`]);
  };

  const selectVideoFile = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: "Video",
            extensions: ["mp4", "mov", "mkv", "webm"],
          },
        ],
      });

      if (selected) {
        setVideoPath(selected);
        addLog(`Selected video: ${selected}`);

        // Auto-generate output path
        const outputPath = selected.replace(/\.[^/.]+$/, "_enhanced.mp4");
        setOutputPath(outputPath);
        addLog(`Output will be saved to: ${outputPath}`);
      }
    } catch (err) {
      setError(`Failed to select file: ${err}`);
      addLog(`ERROR: ${err}`);
    }
  };

  const processVideo = async () => {
    if (!videoPath) {
      setError("Please select a video file first");
      return;
    }

    setIsProcessing(true);
    setError("");
    setResult(null);
    addLog("Starting full pipeline processing...");

    try {
      const options: PipelineOptions = {
        target_lufs: targetLufs,
        enhancement_intensity: intensity,
      };

      addLog(`Options: ${targetLufs} LUFS, ${intensity * 100}% intensity`);
      const startTime = Date.now();

      const pipelineResult = await invoke<PipelineResult>("process_video_file", {
        videoPath,
        outputPath,
        options,
      });

      const elapsed = ((Date.now() - startTime) / 1000).toFixed(1);
      addLog(`✅ Processing complete in ${elapsed}s`);
      addLog(`Original loudness: ${pipelineResult.original_loudness.toFixed(1)} LUFS`);
      addLog(`Final loudness: ${pipelineResult.final_loudness.toFixed(1)} LUFS`);
      addLog(`Output saved to: ${pipelineResult.output_path}`);

      setResult(pipelineResult);
    } catch (err) {
      const errorMsg = `Processing failed: ${err}`;
      setError(errorMsg);
      addLog(`❌ ${errorMsg}`);
    } finally {
      setIsProcessing(false);
    }
  };

  const generatePreview = async () => {
    if (!videoPath) {
      setError("Please select a video file first");
      return;
    }

    setIsProcessing(true);
    setError("");
    setResult(null);
    addLog("Generating 30-second preview...");

    try {
      const previewOutputPath = videoPath.replace(/\.[^/.]+$/, "_preview.wav");
      const startTime = Date.now();

      const previewResult = await invoke<PipelineResult>("generate_preview", {
        videoPath,
        outputPath: previewOutputPath,
        durationSeconds: 30,
      });

      const elapsed = ((Date.now() - startTime) / 1000).toFixed(1);
      addLog(`✅ Preview generated in ${elapsed}s`);
      addLog(`Original loudness: ${previewResult.original_loudness.toFixed(1)} LUFS`);
      addLog(`Final loudness: ${previewResult.final_loudness.toFixed(1)} LUFS`);
      addLog(`Preview saved to: ${previewResult.output_path}`);

      setResult(previewResult);
    } catch (err) {
      const errorMsg = `Preview generation failed: ${err}`;
      setError(errorMsg);
      addLog(`❌ ${errorMsg}`);
    } finally {
      setIsProcessing(false);
    }
  };

  const clearLog = () => {
    setLog([]);
    setResult(null);
    setError("");
  };

  return (
    <div style={{ padding: "20px", fontFamily: "monospace" }}>
      <h1>🎬 Phase 1 Pipeline Test</h1>

      {/* File Selection */}
      <div style={{ marginBottom: "20px", padding: "15px", border: "1px solid #ccc" }}>
        <h3>1. Select Video File</h3>
        <button onClick={selectVideoFile} style={{ padding: "10px", fontSize: "14px" }}>
          📁 Select Video
        </button>
        {videoPath && (
          <div style={{ marginTop: "10px", fontSize: "12px" }}>
            <div>📹 Input: {videoPath}</div>
            <div>💾 Output: {outputPath}</div>
          </div>
        )}
      </div>

      {/* Pipeline Options */}
      <div style={{ marginBottom: "20px", padding: "15px", border: "1px solid #ccc" }}>
        <h3>2. Configure Options</h3>

        <div style={{ marginBottom: "10px" }}>
          <label style={{ display: "block", marginBottom: "5px" }}>
            Target Loudness (LUFS): {targetLufs}
          </label>
          <input
            type="range"
            min="-23"
            max="-10"
            step="0.5"
            value={targetLufs}
            onChange={(e) => setTargetLufs(parseFloat(e.target.value))}
            style={{ width: "300px" }}
          />
          <div style={{ fontSize: "11px", color: "#666" }}>
            YouTube: -14, Spotify: -14, Instagram: -16
          </div>
        </div>

        <div>
          <label style={{ display: "block", marginBottom: "5px" }}>
            Enhancement Intensity: {(intensity * 100).toFixed(0)}%
          </label>
          <input
            type="range"
            min="0"
            max="1"
            step="0.05"
            value={intensity}
            onChange={(e) => setIntensity(parseFloat(e.target.value))}
            style={{ width: "300px" }}
          />
        </div>
      </div>

      {/* Actions */}
      <div style={{ marginBottom: "20px", padding: "15px", border: "1px solid #ccc" }}>
        <h3>3. Process</h3>
        <div style={{ display: "flex", gap: "10px" }}>
          <button
            onClick={generatePreview}
            disabled={!videoPath || isProcessing}
            style={{
              padding: "10px 20px",
              fontSize: "14px",
              backgroundColor: "#4CAF50",
              color: "white",
              border: "none",
              cursor: videoPath && !isProcessing ? "pointer" : "not-allowed",
            }}
          >
            ⚡ Quick Preview (30s)
          </button>

          <button
            onClick={processVideo}
            disabled={!videoPath || isProcessing}
            style={{
              padding: "10px 20px",
              fontSize: "14px",
              backgroundColor: "#2196F3",
              color: "white",
              border: "none",
              cursor: videoPath && !isProcessing ? "pointer" : "not-allowed",
            }}
          >
            🎯 Full Pipeline
          </button>

          <button
            onClick={clearLog}
            style={{
              padding: "10px 20px",
              fontSize: "14px",
              backgroundColor: "#f44336",
              color: "white",
              border: "none",
              cursor: "pointer",
            }}
          >
            🗑️ Clear Log
          </button>
        </div>

        {isProcessing && (
          <div style={{ marginTop: "15px", color: "#FF9800", fontSize: "14px" }}>
            ⏳ Processing... This may take several minutes depending on video length.
          </div>
        )}
      </div>

      {/* Results */}
      {result && (
        <div style={{ marginBottom: "20px", padding: "15px", border: "2px solid #4CAF50", backgroundColor: "#E8F5E9" }}>
          <h3>✅ Results</h3>
          <div style={{ fontSize: "13px" }}>
            <div>📊 Original Loudness: <strong>{result.original_loudness.toFixed(1)} LUFS</strong></div>
            <div>📊 Final Loudness: <strong>{result.final_loudness.toFixed(1)} LUFS</strong></div>
            <div>⏱️ Processing Time: <strong>{result.processing_time_seconds.toFixed(1)}s</strong></div>
            <div>💾 Output: <strong>{result.output_path}</strong></div>
          </div>
        </div>
      )}

      {/* Error */}
      {error && (
        <div style={{ marginBottom: "20px", padding: "15px", border: "2px solid #f44336", backgroundColor: "#FFEBEE" }}>
          <h3>❌ Error</h3>
          <pre style={{ fontSize: "12px", whiteSpace: "pre-wrap" }}>{error}</pre>
        </div>
      )}

      {/* Log */}
      <div style={{ padding: "15px", border: "1px solid #ccc", backgroundColor: "#f5f5f5" }}>
        <h3>📝 Log</h3>
        <div
          style={{
            maxHeight: "300px",
            overflowY: "auto",
            fontSize: "12px",
            fontFamily: "monospace",
            whiteSpace: "pre-wrap",
          }}
        >
          {log.length === 0 ? (
            <div style={{ color: "#999" }}>No log entries yet...</div>
          ) : (
            log.map((entry, idx) => (
              <div key={idx} style={{ marginBottom: "5px" }}>
                {entry}
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}

export default TestPipeline;
