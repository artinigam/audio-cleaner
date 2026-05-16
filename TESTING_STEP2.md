# Step 2 Testing Guide - Media Probing + Audio Extraction

## Prerequisites

### 1. Install FFmpeg (Required)
```bash
# Check if FFmpeg is installed
ffmpeg -version
ffprobe -version

# If not installed on macOS:
brew install ffmpeg

# Verify installation
which ffmpeg
which ffprobe
```

### 2. Get Test Video Files
You need video files in different formats to test. Options:
- Use your own videos (MP4, MOV, MKV, WebM)
- Download sample videos from: https://sample-videos.com/
- Create a test video with audio

## Testing Methods

### Method 1: Frontend Testing (Recommended)

#### Step 1: Start the dev server
```bash
npm run tauri dev
```

#### Step 2: Open browser DevTools console (Cmd+Option+I on Mac)

#### Step 3: Test the commands
```javascript
// Test 1: Probe a video file
const mediaInfo = await window.__TAURI__.core.invoke('probe_media_file', { 
  path: '/Users/artinigam/path/to/your/video.mp4'  // Use your actual path
});
console.log('Media Info:', JSON.stringify(mediaInfo, null, 2));

// Test 2: Extract audio
await window.__TAURI__.core.invoke('extract_audio_from_media', {
  mediaPath: '/Users/artinigam/path/to/your/video.mp4',
  outputPath: '/tmp/test_extracted_audio.wav'
});
console.log('Audio extracted to /tmp/test_extracted_audio.wav');
```

### Method 2: Create a Simple Test Page

Create `src/TestPage.tsx`:
```tsx
import { invoke } from '@tauri-apps/api/core';
import { useState } from 'react';

function TestPage() {
  const [mediaInfo, setMediaInfo] = useState(null);
  const [status, setStatus] = useState('');

  const testProbe = async () => {
    try {
      setStatus('Probing...');
      const info = await invoke('probe_media_file', {
        path: '/path/to/your/video.mp4'  // Update this path
      });
      setMediaInfo(info);
      setStatus('✅ Probe successful!');
    } catch (error) {
      setStatus(`❌ Error: ${error}`);
    }
  };

  const testExtract = async () => {
    try {
      setStatus('Extracting audio...');
      await invoke('extract_audio_from_media', {
        mediaPath: '/path/to/your/video.mp4',  // Update this path
        outputPath: '/tmp/test_audio.wav'
      });
      setStatus('✅ Audio extracted to /tmp/test_audio.wav');
    } catch (error) {
      setStatus(`❌ Error: ${error}`);
    }
  };

  return (
    <div style={{ padding: 20 }}>
      <h1>Step 2 Testing</h1>
      
      <button onClick={testProbe}>Test Probe</button>
      <button onClick={testExtract}>Test Extract</button>
      
      <div style={{ marginTop: 20 }}>
        <strong>Status:</strong> {status}
      </div>
      
      {mediaInfo && (
        <pre style={{ marginTop: 20, background: '#f0f0f0', padding: 10 }}>
          {JSON.stringify(mediaInfo, null, 2)}
        </pre>
      )}
    </div>
  );
}

export default TestPage;
```

Then update `src/App.tsx` to use TestPage temporarily.

### Method 3: Command Line Testing (Advanced)

You can also test by creating a Rust integration test:

Create `src-tauri/tests/test_step2.rs`:
```rust
use std::path::PathBuf;

#[tokio::test]
async fn test_probe_media_file() {
    let test_video = PathBuf::from("/path/to/test/video.mp4");
    
    // This would require refactoring to make the probe function testable
    // For now, use Method 1 or 2 above
}
```

## What to Test

### Test Case 1: Basic Probe ✅
**Goal:** Verify media probing works

1. Run probe on a video file
2. **Verify output contains:**
   - `path`: File path
   - `format`: Format name (e.g., "mov,mp4,m4a,3gp,3g2,mj2")
   - `duration_secs`: Duration in seconds (> 0)
   - `video_streams`: Array with at least 1 video stream
   - `audio_streams`: Array with at least 1 audio stream

**Expected Video Stream Info:**
- `index`: Stream index (0, 1, 2, etc.)
- `codec`: Codec name (e.g., "h264", "hevc")
- `width` and `height`: Video dimensions
- `fps`: Frame rate (e.g., 30.0, 24.0)
- `bitrate`: Optional bitrate

**Expected Audio Stream Info:**
- `index`: Stream index
- `codec`: Codec name (e.g., "aac", "mp3", "opus")
- `sample_rate`: Sample rate (e.g., 44100, 48000)
- `channels`: Number of channels (1=mono, 2=stereo)
- `bitrate`: Optional bitrate

### Test Case 2: Different Video Formats ✅
**Goal:** Ensure compatibility across formats

Test with:
- ✅ MP4 file (.mp4)
- ✅ MOV file (.mov)
- ✅ MKV file (.mkv)
- ✅ WebM file (.webm)

All should return valid MediaFile objects.

### Test Case 3: Audio Extraction ✅
**Goal:** Verify audio is extracted in correct format

1. Run extract_audio_from_media
2. **Verify:**
   - File created at output path
   - File size > 0 bytes

3. **Validate audio format with ffprobe:**
```bash
ffprobe /tmp/test_extracted_audio.wav

# Look for these in output:
# - Sample rate: 48000 Hz
# - Channels: 1 (mono)
# - Format: pcm_s16le (16-bit PCM)
```

4. **Listen to the audio:**
```bash
# Play the extracted audio
afplay /tmp/test_extracted_audio.wav  # macOS

# Should sound the same as original (just mono)
```

### Test Case 4: Stereo to Mono Conversion ✅
**Goal:** Verify stereo audio is properly converted to mono

1. Use a video with stereo audio (2 channels)
2. Extract audio
3. Verify output is mono:
```bash
ffprobe /tmp/test_extracted_audio.wav 2>&1 | grep "channels"
# Should show: Stream #0:0: Audio: pcm_s16le ... 48000 Hz, 1 channels
```

### Test Case 5: Different Sample Rates ✅
**Goal:** Verify resampling works

Test with videos that have:
- 44.1 kHz audio (common for music)
- 48 kHz audio (common for video)
- 96 kHz audio (high quality)

All should output 48 kHz.

### Test Case 6: Multiple Audio Streams ✅
**Goal:** Verify best audio stream is selected

1. Find a video with multiple audio tracks (e.g., different languages)
2. Extract audio
3. **Verify:** The highest bitrate audio stream is selected

### Test Case 7: Error Handling ⚠️

Test these error cases:

**A. Non-existent file:**
```javascript
await invoke('probe_media_file', { path: '/nonexistent/file.mp4' });
// Expected: Error message "File does not exist: ..."
```

**B. File without audio:**
```javascript
// Use a video file with no audio track
await invoke('extract_audio_from_media', { 
  mediaPath: '/path/to/video-no-audio.mp4',
  outputPath: '/tmp/output.wav'
});
// Expected: Error message "No audio stream found in media file"
```

**C. Invalid file format:**
```javascript
await invoke('probe_media_file', { path: '/path/to/image.jpg' });
// Expected: Error message from ffprobe
```

**D. FFmpeg not installed:**
- Temporarily rename ffmpeg: `sudo mv /opt/homebrew/bin/ffmpeg /opt/homebrew/bin/ffmpeg.bak`
- Try to probe a file
- Expected: Error message "Make sure FFmpeg is installed"
- Restore: `sudo mv /opt/homebrew/bin/ffmpeg.bak /opt/homebrew/bin/ffmpeg`

### Test Case 8: Long Video Files ⏱️
**Goal:** Verify it handles large files

1. Test with a 10+ minute video
2. Extract audio (may take 10-30 seconds)
3. Verify success

### Test Case 9: Different Codecs ✅
**Goal:** Ensure codec compatibility

Test videos with different audio codecs:
- AAC (most common)
- MP3
- Opus (WebM)
- PCM (uncompressed)

All should extract successfully.

## Verification Checklist

After testing, verify:

- [ ] Probe returns correct metadata for MP4 files
- [ ] Probe returns correct metadata for MOV files
- [ ] Probe returns correct metadata for MKV files
- [ ] Probe returns correct metadata for WebM files
- [ ] Extracted audio is 48kHz sample rate
- [ ] Extracted audio is mono (1 channel)
- [ ] Extracted audio is 16-bit PCM format
- [ ] Extracted audio sounds correct (no distortion)
- [ ] Stereo → mono conversion works
- [ ] Multiple audio streams: highest bitrate selected
- [ ] Error handling works for missing files
- [ ] Error handling works for files without audio
- [ ] Duration matches original video
- [ ] Video stream info is correct (codec, dimensions, fps)

## Common Issues & Solutions

### Issue 1: "Failed to run ffprobe"
**Solution:** Install FFmpeg: `brew install ffmpeg`

### Issue 2: "No audio stream found"
**Solution:** Your video file has no audio track. Try a different file.

### Issue 3: Path issues on Windows
**Solution:** Use forward slashes or double backslashes:
- ✅ `C:/Users/name/video.mp4`
- ✅ `C:\\Users\\name\\video.mp4`
- ❌ `C:\Users\name\video.mp4`

### Issue 4: Output file not created
**Solution:** Check that output directory exists. The code creates parent dirs, but verify write permissions.

### Issue 5: "Command not found: npm"
**Solution:** Install Node.js 18+ from nodejs.org

## Performance Benchmarks

Expected performance:
- **Probe:** < 1 second for any file
- **Extract:** ~1-2x real-time duration
  - 1 min video → 1-2 min extraction time
  - 10 min video → 10-20 min extraction time

If extraction is slower, it might be CPU-limited.

## Next Steps After Testing

Once all tests pass:
1. Document any issues found
2. Test with your actual creator videos (real-world testing)
3. Collect 25+ test clips for benchmark corpus (per PRD)
4. Ready to move to Step 3: DeepFilterNet Integration

## Quick Test Script

Here's a quick test you can run in the browser console:

```javascript
// Quick comprehensive test
async function testStep2() {
  console.log('🧪 Testing Step 2...\n');
  
  const testVideo = '/path/to/your/test/video.mp4'; // UPDATE THIS
  
  try {
    // Test 1: Probe
    console.log('1️⃣ Testing probe...');
    const info = await window.__TAURI__.core.invoke('probe_media_file', { 
      path: testVideo 
    });
    console.log('✅ Probe successful!');
    console.log('   Format:', info.format);
    console.log('   Duration:', info.duration_secs, 'seconds');
    console.log('   Video streams:', info.video_streams.length);
    console.log('   Audio streams:', info.audio_streams.length);
    
    if (info.audio_streams.length > 0) {
      const audio = info.audio_streams[0];
      console.log('   Audio codec:', audio.codec);
      console.log('   Sample rate:', audio.sample_rate, 'Hz');
      console.log('   Channels:', audio.channels);
    }
    
    // Test 2: Extract
    console.log('\n2️⃣ Testing extraction...');
    await window.__TAURI__.core.invoke('extract_audio_from_media', {
      mediaPath: testVideo,
      outputPath: '/tmp/test_step2.wav'
    });
    console.log('✅ Extraction successful!');
    console.log('   Output: /tmp/test_step2.wav');
    console.log('\n🎉 All tests passed!');
    console.log('\nVerify audio format:');
    console.log('Run in terminal: ffprobe /tmp/test_step2.wav');
    
  } catch (error) {
    console.error('❌ Test failed:', error);
  }
}

// Run the test
testStep2();
```

Good luck with testing! Let me know if you encounter any issues.
