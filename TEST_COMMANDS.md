# Testing Step 2 Implementation

## Prerequisites
Make sure FFmpeg is installed:
```bash
ffmpeg -version
ffprobe -version
```

If not installed on macOS:
```bash
brew install ffmpeg
```

## Testing from the Frontend

The following Tauri commands are now available:

### 1. Probe a Media File
```typescript
import { invoke } from '@tauri-apps/api/core';

// Probe a video file
const mediaInfo = await invoke('probe_media_file', { 
  path: '/path/to/video.mp4' 
});

console.log(mediaInfo);
// Returns: MediaFile object with format, duration, video/audio streams
```

### 2. Extract Audio from Media
```typescript
import { invoke } from '@tauri-apps/api/core';

// Extract audio to WAV (48kHz, mono, 16-bit PCM)
await invoke('extract_audio_from_media', {
  mediaPath: '/path/to/video.mp4',
  outputPath: '/tmp/extracted_audio.wav'
});
```

## Manual Testing

1. Start the dev server:
```bash
npm run tauri dev
```

2. Open the browser console and test the commands with a real video file

3. Verify the extracted WAV file is created and has correct format:
```bash
ffprobe /tmp/extracted_audio.wav
# Should show: 48000 Hz, mono, s16 (16-bit PCM)
```

## What Was Implemented

✅ **Data Models** (`src-tauri/src/models/`)
- MediaFile, AudioStreamInfo, VideoStreamInfo
- ProcessingJob, EnhancementPreset (for future use)

✅ **FFmpeg Integration** (`src-tauri/src/ffmpeg/`)
- probe.rs - Media probing with ffprobe (JSON parsing)
- extract.rs - Audio extraction to 48kHz/mono/16-bit WAV

✅ **Tauri Commands** (`src-tauri/src/commands/`)
- probe_media_file - Get video/audio stream information
- extract_audio_from_media - Extract audio from video

✅ **Error Handling**
- File existence validation
- FFmpeg availability checks
- Audio stream detection
- Output validation (file exists and non-empty)

## Next Steps (Step 3)
Implement DeepFilterNet integration for audio enhancement.
