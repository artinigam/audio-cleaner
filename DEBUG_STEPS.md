# Debugging Steps for Failed Probe/Extract

## Step 1: Check Browser Console

1. Run `npm run tauri dev`
2. When the app opens, press **Cmd+Option+I** (Mac) or **F12** (Windows/Linux)
3. Go to the **Console** tab
4. Try clicking "Test Probe" or "Test Extract"
5. **Copy the exact error message** that appears in red

## Step 2: Check Terminal Output

Look at the terminal where you ran `npm run tauri dev` for any Rust panic messages or errors.

## Step 3: Common Issues & Solutions

### Issue 1: "Failed to run ffprobe"
**Cause:** FFmpeg not in PATH when Tauri runs
**Solution:** Need to use absolute path to ffprobe

### Issue 2: "File does not exist"
**Cause:** Invalid file path
**Solution:** Make sure you're using an absolute path like `/Users/artinigam/Movies/video.mp4`

### Issue 3: CORS or permission errors
**Cause:** Tauri security restrictions
**Solution:** May need to update tauri.conf.json permissions

### Issue 4: Command not registered
**Cause:** Tauri commands not properly exposed
**Solution:** Check main.rs has commands in invoke_handler

## Step 4: Manual FFmpeg Test

Try running ffprobe manually to verify it works:

```bash
# Test ffprobe with a real video file
ffprobe -v quiet -print_format json -show_format -show_streams /path/to/your/video.mp4

# If this works, the issue is with Tauri integration
# If this fails, FFmpeg is not properly installed
```

## Step 5: Test with Simple File

Create a small test video:
```bash
# Create a 5-second test video with audio
ffmpeg -f lavfi -i testsrc=duration=5:size=1280x720:rate=30 \
       -f lavfi -i sine=frequency=1000:duration=5 \
       -pix_fmt yuv420p /tmp/test_video.mp4
```

Then test with: `/tmp/test_video.mp4`

## What to Share for Help

Please provide:
1. **Exact error message** from browser console
2. **Error message** from terminal (if any)
3. **The file path** you're trying to probe
4. **Screenshot** of the error (optional but helpful)
