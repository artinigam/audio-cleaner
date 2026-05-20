# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

---

## Project Overview

**Audio Cleanup Desktop Application** - A local-first desktop app for YouTube creators to enhance audio in video files through automated noise reduction, dereverberation, and loudness normalization.

**Tech Stack:**
- **Desktop Framework:** Tauri 2 (Rust backend + React frontend)
- **Backend:** Rust (audio processing, FFmpeg interop)
- **Frontend:** React + TypeScript + Vite
- **Audio Processing:** FFmpeg (media I/O), DeepFilterNet CLI (ML enhancement)
- **Target Platforms:** macOS 11+ (ARM/Intel), Windows 10+ (future)

**Current Status:** Phase 1 Complete (core pipeline), Phase 2 planned (UI & preview)

---

## Build & Development Commands

### Development
```bash
# Start Tauri dev server (hot reload for both Rust and React)
npm run tauri dev

# Frontend only (no Tauri)
npm run dev

# Build frontend (TypeScript + Vite)
npm run build
```

### Rust Backend
```bash
# Check Rust code (fast, no build)
cargo check --manifest-path=src-tauri/Cargo.toml

# Build Rust backend
cargo build --manifest-path=src-tauri/Cargo.toml

# Build release (optimized)
cargo build --release --manifest-path=src-tauri/Cargo.toml

# Run Rust tests
cargo test --manifest-path=src-tauri/Cargo.toml
```

### Production Build
```bash
# Build distributable app (DMG for Mac, MSI for Windows)
npm run tauri:build
```

---

## Architecture

### Processing Pipeline (src-tauri/src/)

The core audio processing follows this flow:

```
Input Media → Probe → Extract → Measure → Enhance → Normalize → Remux → Output
```

**Key Modules:**

1. **commands/** - Tauri command handlers (frontend ↔ backend interface)
   - `media.rs` - Individual operations (probe, extract, enhance)
   - `pipeline.rs` - **Main entry point** for end-to-end processing
     - `process_video_file()` - Full pipeline
     - `generate_preview()` - Quick 30s preview

2. **ffmpeg/** - FFmpeg wrapper for media operations
   - `probe.rs` - Media metadata extraction (codec, duration, streams)
   - `extract.rs` - Audio extraction (converts to 48kHz mono WAV)
   - `loudness.rs` - **Critical:** ITU-R BS.1770-4 LUFS measurement & normalization
   - `remux.rs` - Video remuxing (replaces audio track, preserves video quality)
     - **Important:** Auto-detects video vs audio-only files

3. **enhancement/** - Audio enhancement
   - `deepfilternet_cli.rs` - **Active implementation:** Calls DeepFilterNet CLI binary
   - `deepfilternet.rs` - Legacy ONNX implementation (not used)
   - `chunking.rs` - Audio chunking utilities

4. **models/** - Data structures
   - `media.rs` - MediaFile, AudioStream, VideoStream types
   - `processing.rs` - Processing job types (unused, for Phase 2)

5. **utils/** - Shared utilities
   - `audio_io.rs` - WAV file I/O (load/save f32 arrays)

### Frontend Structure (src/)

- `App.tsx` - Main entry point, renders TestPipeline
- `TestPipeline.tsx` - **Phase 1 test UI** (file picker, options, logging)
- Future: Production UI components in `src/components/` (Phase 2)

### External Dependencies

**Critical System Requirements:**
- **FFmpeg** must be in PATH (probe, extract, loudness, remux all fail without it)
- **DeepFilterNet CLI** binary at `models/deep-filter` (27MB, macOS ARM currently)
  - Download from: https://github.com/Rikorose/DeepFilterNet/releases/tag/v0.5.6
  - Other platforms: Replace with x86_64-apple-darwin, x86_64-pc-windows-msvc.exe, etc.

---

## Important Implementation Details

### Audio Processing Flow

**Standard Pipeline** (`commands/pipeline.rs::process_video_file`):
1. **Probe:** Get media metadata, validate streams exist
2. **Extract:** Convert audio to 48kHz mono WAV (temp file)
3. **Measure:** Get original LUFS (ebur128 filter)
4. **Enhance:** DeepFilterNet CLI for noise reduction (intensity 0.0-1.0)
5. **Normalize:** Two-pass loudnorm to target LUFS (default: -14 for YouTube)
6. **Remux:** Replace audio in video (stream copy for video, AAC for audio)

**Preview Pipeline** (`commands/pipeline.rs::generate_preview`):
- Extracts only first 30 seconds
- Same enhancement pipeline
- Outputs WAV file (no video remux)

### Key Technical Decisions

**Why CLI instead of ONNX?**
- DeepFilterNet doesn't provide pre-built ONNX models
- CLI is official, pre-compiled, simpler integration
- See CHANGELOG.md "DeepFilterNet: ONNX → CLI" for details

**Why 48kHz Mono?**
- 48kHz is broadcast standard
- Mono sufficient for speech (primary use case)
- Reduces processing time vs stereo
- Upgradeable to stereo in Phase 3

**Audio-Only File Handling** (remux.rs):
- Many podcasts/music files use .mp4 extension but have no video stream
- `check_has_video_stream()` uses ffprobe to detect
- Branches to `encode_audio_only()` if no video (prevents "stream not found" errors)

**Loudness Parsing** (loudness.rs):
- ebur128 output format requires section-aware parsing
- "True peak:" section contains "Peak:" values (not "True peak:" label directly)
- Parser tracks sections: Integrated loudness / Loudness range / True peak
- See CHANGELOG.md "Loudness Parsing Fix" for bug history

### Temporary Files

All intermediate files go to system temp directory:
```rust
let temp_dir = std::env::temp_dir().join(format!("audio-cleaner-{}", uuid::Uuid::new_v4()));
```

**Important:** Always clean up temp directories on success OR failure
- Success: `std::fs::remove_dir_all(&temp_dir)`
- Failure: Same cleanup in error handling

---

## Common Patterns

### Adding a New FFmpeg Operation

1. Create function in `src-tauri/src/ffmpeg/<module>.rs`
2. Use `tokio::process::Command` for async execution
3. Pattern:
```rust
pub async fn operation(input: &Path, output: &Path) -> Result<(), String> {
    let output = Command::new("ffmpeg")
        .args([/* args */])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("Failed to run ffmpeg: {}", e))?;
    
    if !output.status.success() {
        return Err(format!("FFmpeg failed: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    Ok(())
}
```

### Adding a New Tauri Command

1. Add function to `src-tauri/src/commands/<module>.rs`:
```rust
#[tauri::command]
pub async fn command_name(param: String) -> Result<ReturnType, String> {
    // Implementation
}
```

2. Register in `src-tauri/src/main.rs`:
```rust
.invoke_handler(tauri::generate_handler![
    existing_command,
    command_name,  // Add here
])
```

3. Call from frontend:
```typescript
import { invoke } from "@tauri-apps/api/core";

const result = await invoke<ReturnType>("command_name", { param: value });
```

### Progress Tracking (Phase 2)

Currently: No progress updates (UI waits blindly)

**To implement (Phase 2):**
- Use Tauri events: `window.emit("progress", { stage, percentage })`
- Emit at each pipeline stage
- Frontend listens: `listen("progress", (event) => ...)`

---

## Testing

### Manual Testing

**Phase 1 Test UI:**
```bash
npm run tauri dev
```
1. Click "Select Video"
2. Choose MP4/MOV/M4A file
3. Configure target LUFS (-14 to -23) and intensity (0-100%)
4. Click "⚡ Quick Preview" (30s test) or "🎯 Full Pipeline"

**Create Test Video:**
```bash
ffmpeg -f lavfi -i testsrc=duration=30:size=1280x720:rate=30 \
       -f lavfi -i "sine=frequency=1000:duration=30,volume=0.3" \
       -pix_fmt yuv420p test_video.mp4
```

### Rust Tests

Currently minimal (marked with `#[ignore]`). To run:
```bash
cargo test --manifest-path=src-tauri/Cargo.toml -- --ignored
```

**Note:** Tests require actual FFmpeg and media files, so they're ignored by default.

---

## Common Issues

### "FFmpeg not found"
**Cause:** FFmpeg not in PATH  
**Fix:** `brew install ffmpeg` (macOS) or add FFmpeg to system PATH

### "DeepFilterNet CLI not found"
**Cause:** Binary missing or wrong platform  
**Fix:** Download from GitHub releases, place at `models/deep-filter`, `chmod +x`

### "Stream map matches no streams"
**Cause:** Trying to map video stream on audio-only file  
**Fixed in:** remux.rs now auto-detects and handles audio-only files

### "Failed to parse loudness metrics"
**Cause:** ebur128 output format mismatch  
**Fixed in:** loudness.rs now uses section-aware parsing

### Processing hangs
**Cause:** FFmpeg waiting for input or DeepFilterNet CLI hung  
**Debug:** Check terminal for FFmpeg/CLI output, verify file isn't corrupted

---

## File Locations

**Documentation:**
- `DESIGN.md` - Complete system design (4,500+ lines)
- `PHASE1_COMPLETE.md` - Phase 1 completion report
- `PHASE2_PLAN.md` - Roadmap for UI & Preview features
- `CHANGELOG.md` - Bug fixes and technical decisions
- `README.md` - Quick start guide

**Configuration:**
- `src-tauri/tauri.conf.json` - Tauri app configuration
- `src-tauri/Cargo.toml` - Rust dependencies
- `package.json` - Node dependencies and scripts

**Build Artifacts:**
- `src-tauri/target/` - Rust build output (gitignored)
- `dist/` - Frontend build output (gitignored)

---

## Phase 2 Context (Next Steps)

**Current Phase 1 Limitations:**
1. No real-time progress updates
2. No cancellation once started
3. No A/B audio comparison UI
4. No waveform visualization
5. Single file only (no batch queue)

**Phase 2 Goals (Weeks 5-7):**
- Real-time progress tracking (Tauri events)
- A/B audio player with instant toggle
- Waveform visualization (WaveSurfer.js or custom Canvas)
- Processing queue for batch operations
- Settings panel for user preferences
- Process cancellation

See `PHASE2_PLAN.md` for detailed week-by-week breakdown.

---

## Performance Expectations

**10-minute video (typical tutorial):**
- Probe: <1s
- Extract: ~10s
- Measure: ~5s
- Enhance: ~2-4s (DeepFilterNet CLI is 400x realtime!)
- Normalize: ~20s
- Remux: ~10s
- **Total: ~50-60s** (6x faster than realtime)

**Bottlenecks:**
- Loudness normalization (two-pass, CPU-bound)
- Audio extraction for long videos
- Enhancement is surprisingly fast (CLI optimized)

---

## Contributing Guidelines

**When adding features:**
1. Check `DESIGN.md` for requirements and architecture decisions
2. Follow existing patterns (async/await, error handling)
3. Update `CHANGELOG.md` for notable changes
4. Clean up temp files in all code paths (success AND error)

**When fixing bugs:**
1. Document fix in `CHANGELOG.md` with before/after
2. Add test case if feasible (even if `#[ignore]`)
3. Check if fix applies to other similar code

**Commit messages:**
- Prefix: `feat:`, `fix:`, `docs:`, `refactor:`, `test:`
- Include Co-Authored-By for Claude contributions
- Reference issue numbers if applicable

---

## Resources

- **FFmpeg Documentation:** https://ffmpeg.org/documentation.html
- **DeepFilterNet:** https://github.com/Rikorose/DeepFilterNet
- **Tauri Guides:** https://tauri.app/v2/guides/
- **ITU-R BS.1770-4:** Loudness standard (Google for spec)
- **Project PRD:** `Audio-cleaner.pdf` (not in repo)
