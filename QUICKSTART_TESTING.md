# Quick Start - Testing Step 2

## 🚀 5-Minute Testing Guide

### Step 1: Install FFmpeg (if not already installed)
```bash
brew install ffmpeg
```

### Step 2: Start the app
```bash
npm run tauri dev
```

### Step 3: Use the Test UI
The app will open with a testing interface showing:
- Input field for video path
- Buttons to test probe and extraction
- Real-time status updates
- Detailed media information display

### Step 4: Test with your video
1. **Update the video path** in the input field to point to a real video file on your Mac:
   - Example: `/Users/artinigam/Movies/test-video.mp4`
   - Or drag a video file to get its path

2. **Click "Test Probe"** to see video metadata:
   - Format, duration
   - Video streams (codec, resolution, fps)
   - Audio streams (codec, sample rate, channels)

3. **Click "Test Extract"** to extract audio:
   - Converts to 48kHz, mono, 16-bit PCM WAV
   - Saves to `/tmp/test_audio.wav` by default
   - May take 1-2 minutes for longer videos

### Step 5: Verify the extracted audio
```bash
# Check the audio format
ffprobe /tmp/test_audio.wav

# Look for:
# - Sample rate: 48000 Hz ✅
# - Channels: mono (1 channel) ✅
# - Format: pcm_s16le (16-bit PCM) ✅

# Listen to the audio
afplay /tmp/test_audio.wav
```

## ✅ What to Look For

### Probe Test Success Criteria:
- ✅ Shows correct video format (MP4, MOV, etc.)
- ✅ Shows correct duration in seconds
- ✅ Lists video streams with resolution and codec
- ✅ Lists audio streams with sample rate and channels
- ✅ No error messages

### Extract Test Success Criteria:
- ✅ Status shows "✅ Audio extracted successfully"
- ✅ File exists at output path
- ✅ File size > 0 bytes
- ✅ Audio plays correctly
- ✅ Format verified with ffprobe shows:
  - 48000 Hz sample rate
  - 1 channel (mono)
  - pcm_s16le format

## 🧪 Test Different Scenarios

Try testing with:
1. **Different formats**: MP4, MOV, MKV, WebM
2. **Different codecs**: H.264, H.265, VP9
3. **Stereo audio**: Verify it converts to mono
4. **Different sample rates**: 44.1kHz, 48kHz should all output 48kHz
5. **Long videos**: 10+ minutes to test performance

## ❌ Common Issues

### Issue: "Failed to run ffprobe"
**Fix:** Install FFmpeg: `brew install ffmpeg`

### Issue: "File does not exist"
**Fix:** Make sure the path is correct. Use absolute path, not relative.

### Issue: "No audio stream found"
**Fix:** Your video has no audio. Try a different file.

### Issue: App won't start
**Fix:** 
```bash
# Clean and rebuild
npm install
npm run tauri dev
```

## 📊 Expected Performance

- **Probe**: < 1 second (instant)
- **Extract**: ~1-2x real-time
  - 1 min video → 1-2 min
  - 5 min video → 5-10 min
  - 10 min video → 10-20 min

## 🎯 What's Next?

Once testing is complete and all tests pass:
1. ✅ Step 2 is validated
2. → Ready for Step 3: DeepFilterNet Integration
3. → 5-7 days estimated for ML inference implementation

## 💡 Pro Tips

1. **Use short test videos first** (< 1 min) for quick iteration
2. **Keep extracted WAV files** to use for Step 3 testing
3. **Test with your actual creator videos** for real-world validation
4. **Build a test corpus** of 25+ clips (per PRD requirements)

Happy testing! 🎉
