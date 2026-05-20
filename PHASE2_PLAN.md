# Phase 2: UI & Preview (Weeks 5-7)

**Goal:** Transform the test UI into a production-ready interface with real-time feedback and A/B comparison.

---

## Current Status (End of Phase 1)

✅ **Core Pipeline Working:**
- Extract, enhance, normalize, remux all functional
- Handles both video and audio-only files
- DeepFilterNet CLI integrated
- Basic test UI for validation

⚠️ **Current Limitations:**
- No real-time progress updates (user waits blindly)
- No A/B comparison (can't hear before/after)
- No waveform visualization
- No batch processing
- Basic error handling
- No cancellation once started

---

## Phase 2 Priorities

### 🎯 Week 5: Progress & Feedback

**Goal:** Give users visibility into what's happening

#### 1. Real-Time Progress Tracking
**Current:** Silent processing, no feedback  
**Target:** Live stage updates with progress bars

**Implementation:**
- [ ] Add progress callback system in Rust
- [ ] Use Tauri events to stream progress to UI
- [ ] Show current stage (Extracting, Enhancing, etc.)
- [ ] Display percentage complete (0-100%)
- [ ] Estimate time remaining
- [ ] Show processing speed (e.g., "2.5x realtime")

**UI Components:**
```typescript
<ProgressTracker 
  stage="Enhancing audio"
  percentage={45}
  timeRemaining="2m 15s"
  speed="2.5x"
/>
```

#### 2. Better Error Handling
**Current:** Generic error messages  
**Target:** Actionable error messages with recovery

**Implementation:**
- [ ] Categorize errors (user error, system error, bug)
- [ ] Provide specific fix suggestions
- [ ] Add "Retry" button for recoverable errors
- [ ] Log detailed errors for debugging
- [ ] Show which stage failed

**Error Types:**
- File not found → "Please select a valid video file"
- FFmpeg missing → "Install FFmpeg: brew install ffmpeg"
- Out of disk space → "Free up X GB and try again"
- Corrupted file → "File may be damaged, try another"

#### 3. Process Cancellation
**Current:** Can't stop once started  
**Target:** Cancel button that works instantly

**Implementation:**
- [ ] Add cancel signal to pipeline
- [ ] Kill FFmpeg/DeepFilterNet processes
- [ ] Clean up temp files on cancel
- [ ] Return to ready state

---

### 🎯 Week 6: A/B Comparison & Preview

**Goal:** Let users hear the difference before committing

#### 1. Audio Preview Player
**Target:** Side-by-side comparison of original vs enhanced

**Implementation:**
- [ ] Extract 30-second preview segments
- [ ] Load both original and enhanced audio
- [ ] Build custom audio player with:
  - Play/pause both versions
  - Switch between A/B instantly (same position)
  - Sync playback position
  - Volume control
  - Seek bar with waveform

**UI Layout:**
```
┌────────────────────────────────────┐
│  ◄◄  ▶  ►►   [====|--------]       │
│  0:15 / 0:30                  🔊   │
├────────────────────────────────────┤
│  ○ Original    ● Enhanced          │
│  -25.3 LUFS     -14.0 LUFS         │
└────────────────────────────────────┘
```

**Features:**
- Keyboard shortcuts (Space = play/pause, A/B = switch)
- Visual indicator showing which version is playing
- Loudness meter for both versions
- Loop mode for detailed comparison

#### 2. Waveform Visualization
**Target:** Visual before/after comparison

**Implementation:**
- [ ] Generate waveform data (peak levels)
- [ ] Use Canvas or WebGL for rendering
- [ ] Show both waveforms stacked or overlaid
- [ ] Highlight differences (noise reduction visible)
- [ ] Click waveform to seek

**Libraries to Consider:**
- `wavesurfer.js` - Popular, feature-rich
- `peaks.js` - BBC's waveform library
- Custom Canvas implementation

**Visual Example:**
```
Original:  ▁▂█▃▁▃▂█▁▂▁█▃▁▂  (noisy, uneven)
Enhanced:  ▁▂█▃▁▂▂█▁▂▁█▂▁▂  (cleaner, smoother)
```

#### 3. Quick Preview Generation
**Current:** Manual preview button  
**Target:** Auto-generate preview after file selection

**Implementation:**
- [ ] Automatically create 30s preview on file drop
- [ ] Show preview while user configures settings
- [ ] Skip enhancement for instant original playback
- [ ] Cache preview for repeated comparisons

---

### 🎯 Week 7: Queue & Polish

**Goal:** Production-ready UI that feels professional

#### 1. Processing Queue
**Target:** Add multiple files and process in sequence

**Implementation:**
- [ ] Queue component showing all files
- [ ] Drag-to-reorder
- [ ] Remove files from queue
- [ ] Show status: Pending / Processing / Complete / Failed
- [ ] Process files one at a time (or configurable parallel)
- [ ] Persist queue across app restarts

**UI Layout:**
```
┌─────────────────────────────────────────┐
│  Queue (3 files)           [+ Add More] │
├─────────────────────────────────────────┤
│  ✓ video1.mp4        Complete   [View]  │
│  ⚙ video2.mp4        45%        [Stop]  │
│  ⏸ video3.mp4        Pending    [▶]     │
└─────────────────────────────────────────┘
```

#### 2. Settings Panel
**Target:** User preferences and defaults

**Implementation:**
- [ ] Default target LUFS (-14, -16, -23, custom)
- [ ] Default enhancement intensity (0-100%)
- [ ] Output directory preference
- [ ] File naming convention (suffix, replace, etc.)
- [ ] Auto-preview toggle
- [ ] Keep/delete temp files

**Settings Categories:**
- **Processing:** Defaults for LUFS, intensity
- **Output:** Where to save, naming pattern
- **Interface:** Theme, keyboard shortcuts
- **Advanced:** FFmpeg path, temp directory

#### 3. UI Polish
**Target:** Professional, intuitive interface

**Improvements:**
- [ ] Drag-and-drop file zone (prominent, visual)
- [ ] Keyboard shortcuts (Space, Enter, Esc, etc.)
- [ ] Responsive layout (window resizing)
- [ ] Loading states with animations
- [ ] Success celebrations (subtle)
- [ ] Empty states (helpful, not blank)
- [ ] Tooltips for all controls
- [ ] Dark mode support

**Design Language:**
- Clean, modern interface
- Focus on the audio waveform as hero element
- Minimal chrome, maximum content
- Clear hierarchy: File → Preview → Settings → Process

---

## Phase 2 Deliverables

By end of Week 7, the app should have:

- [x] ~~Basic test UI~~ → [ ] Production-ready interface
- [ ] Real-time progress tracking with percentage and ETA
- [ ] A/B audio comparison player
- [ ] Waveform visualization (before/after)
- [ ] Processing queue for batch operations
- [ ] Settings panel with user preferences
- [ ] Process cancellation
- [ ] Better error messages with recovery
- [ ] Keyboard shortcuts
- [ ] Professional visual design

---

## Technical Decisions for Phase 2

### Progress Tracking Approach

**Option A: Polling** (Simple)
```typescript
// Frontend polls backend every 500ms
setInterval(() => {
  invoke('get_progress').then(setProgress);
}, 500);
```

**Option B: Events** (Better) ✅ Recommended
```rust
// Backend emits events
window.emit("progress", { stage: "Enhancing", pct: 45 });
```

### Audio Player Library

**Option A: Native HTML5 Audio**
- Pros: Simple, no dependencies
- Cons: Limited control, hard to sync A/B

**Option B: Howler.js** ✅ Recommended
- Pros: Cross-browser, good API, sprite support
- Cons: 30KB bundle size

**Option C: Tone.js**
- Pros: Powerful, music production features
- Cons: Overkill for our use case

### Waveform Library

**Option A: WaveSurfer.js** ✅ Recommended
- Pros: Popular, maintained, feature-rich
- Cons: 100KB, might be slow for long files

**Option B: Peaks.js**
- Pros: BBC-maintained, designed for long audio
- Cons: Requires pre-computed waveform data

**Option C: Custom Canvas**
- Pros: Full control, lightweight
- Cons: More work, reinventing wheel

### State Management

**Current:** React useState (adequate for test UI)  
**Phase 2:** Consider Zustand or Jotai for complex state

**Why?**
- Multiple files in queue
- Progress for each file
- Settings persisted across sessions
- Undo/redo for edits (Phase 3)

---

## Success Criteria for Phase 2

Before moving to Phase 3, verify:

- [ ] User can see progress in real-time (no more blind waiting)
- [ ] User can compare original vs enhanced audio instantly
- [ ] User can cancel processing at any time
- [ ] User can queue multiple files and process in batch
- [ ] User can configure default settings (LUFS, intensity)
- [ ] Errors are clear and actionable
- [ ] UI feels responsive and professional
- [ ] Keyboard shortcuts work for common actions
- [ ] App works smoothly with 5-10 files in queue

---

## What Comes After Phase 2?

### Phase 3: Polish & Features (Weeks 8-10)
- Platform-specific presets (YouTube, LinkedIn, Instagram, Spotify)
- Quality meter (source audio assessment)
- Fine-tuned intensity control with real-time preview
- Watch folders / auto-process
- Speech/music detection for adaptive enhancement
- Export history and favorites

### Phase 4: Commercial & Distribution (Weeks 11-12)
- License system (free vs Pro)
- Feature gating (preview limits, watermarks)
- Code signing + notarization
- Installer creation (DMG, MSI)
- Auto-update system
- Analytics + crash reporting
- Landing page + marketing

---

## Recommended Focus for Week 5 (Start of Phase 2)

**Day 1-2: Progress Events System**
1. Add Tauri event emitter in Rust pipeline
2. Emit progress at each stage
3. Update UI with real-time progress bar
4. Add time remaining estimation

**Day 3-4: Error Handling**
1. Categorize all possible errors
2. Write user-friendly error messages
3. Add retry logic for transient failures
4. Test with various error conditions

**Day 5: Cancellation**
1. Add cancel signal to pipeline
2. Implement process killing (FFmpeg, DeepFilterNet)
3. Clean up temp files on cancel
4. Add "Cancel" button to UI

**Goal:** By end of Week 5, users should never be in the dark about what's happening!

---

## Questions to Answer Before Starting Phase 2

1. **Design:** Do you want to design the UI yourself, or use a component library?
   - **Component Libraries:** shadcn/ui, Chakra UI, Mantine
   - **Design System:** Build custom with Tailwind

2. **Audio Player:** Should A/B comparison be instant toggle or side-by-side players?
   - **Instant toggle:** Space bar to switch (faster comparison)
   - **Side-by-side:** Two players, manual control (more flexibility)

3. **Queue:** Should files process in parallel or sequentially?
   - **Sequential:** Safer, predictable (1 at a time)
   - **Parallel:** Faster, but high CPU usage (2-3 at a time)

4. **Platform Priority:** Which platform to focus on first?
   - **macOS:** Your current dev environment
   - **Windows:** Largest user base
   - **Linux:** Nice-to-have

---

## Ready to Start Phase 2?

Let me know:
1. Which Week 5 features you want to tackle first
2. Any design preferences (library vs custom)
3. Whether you want me to implement or just guide

Phase 1 was foundation, **Phase 2 is where the app comes alive!** 🎨🎵

---

**Current Status:** Phase 1 Complete ✅  
**Next Step:** Phase 2 Week 5 - Progress & Feedback  
**Timeline:** 3 weeks to production-ready UI
