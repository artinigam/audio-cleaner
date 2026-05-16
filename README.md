# Audio Cleanup Desktop Application

**Desktop-first audio enhancement tool for YouTube creators**

Transform "decent" creator recordings into "studio-quality" upload-ready videos through automated AI-driven audio enhancement.

---

## 📚 Documentation

This repository contains comprehensive design documentation for the Audio Cleanup application:

| File | Size | Purpose |
|------|------|---------|
| **DESIGN.md** | 168KB | **Primary Reference** - Complete system design including architecture, components, data models, API contracts, and implementation details |
| **PHASE1_READINESS.md** | 27KB | **Implementation Guide** - Phase 1 (Weeks 1-4) readiness checklist with all specifications validated |
| **Audio-cleaner.pdf** | - | **Product Requirements Document (PRD)** - Original product vision and requirements |

---

## 🎯 Quick Start

### **For Implementation:**
1. **Read DESIGN.md** - Your implementation bible
   - Sections 1-2: Requirements & Architecture
   - Section 3-4: Infrastructure & Component Design
   - Section 5-7: Low-level design, NFR mapping, traceability
   - Section 8: Appendices (decision log, risks, glossary)

2. **Check PHASE1_READINESS.md** - Validate all Phase 1 specs are complete

3. **Start Coding** - No blockers, all specifications ready

### **For Understanding Architecture:**
- Read DESIGN.md Executive Summary (first 3 pages)
- Review Section 2: High-Level Architecture
- Review Section 3: Infrastructure Decisions

---

## 🏗️ Architecture Overview

**Chosen Stack:** Tauri + Rust + React/TypeScript + FFmpeg + ONNX Runtime

**Key Design Decisions:**
1. **Local-first processing** - Privacy, offline capability, zero marginal costs
2. **Monolithic architecture** - Solo developer, rapid iteration, simpler deployment
3. **Desktop-first** - Windows/macOS with native performance
4. **Open-source ML models** - DeepFilterNet (Apache 2.0) for denoise/dereverb

**Processing Pipeline:**
1. File ingestion (drag & drop MP4/MOV/MKV/WebM)
2. Audio extraction (FFmpeg → 48kHz mono PCM)
3. Quality analysis (noise floor, clipping, bandwidth)
4. ML enhancement (DeepFilterNet via ONNX Runtime)
5. DSP post-processing (EQ, compression, de-esser)
6. Loudness normalization (ITU-R BS.1770-4)
7. Preview generation (A/B comparison)
8. Video remux (FFmpeg stream copy)
9. Export (platform-specific presets: YouTube, LinkedIn, Instagram)

---

## 📊 Requirements Coverage

**Functional Requirements:** 16/16 designed (100%)
- ✅ FR-1 to FR-11: MVP features (drag-drop, one-click cleanup, A/B preview, export, loudness, local processing, presets, batch queue, speech/music awareness, quality meter, intensity control)
- 🔄 FR-12 to FR-16: v2 features (watch folders, transcript-assisted cleanup, team presets, cloud salvage mode, API/CLI)

**Non-Functional Requirements:** 23/23 designed (100%)
- ✅ Performance: 10-20 min processing, <30s preview, <40ms A/V sync
- ✅ Scalability: Support 100MB-10GB files, 5 min-3 hour audio, 1-2 concurrent jobs
- ✅ Reliability: >95% export success, >99% crash-free, 100% offline
- ✅ Security: Local-only processing, license validation, code signing, LGPL compliance
- ✅ Compatibility: Windows 10+, macOS 11+, all major video/audio codecs

**Traceability:** 95% coverage (37/39 scoped requirements designed in DESIGN.md Section 7)

---

## 🚀 Implementation Phases

### **Phase 1: Core Pipeline (Weeks 1-4)** ✅ READY
1. Tauri app scaffold + FFmpeg integration
2. Media probing + audio extraction
3. DeepFilterNet integration (ONNX Runtime)
4. Basic loudness normalization
5. Video remuxing

**Status:** All specifications complete in DESIGN.md. No blockers.

### **Phase 2: UI & Preview (Weeks 5-7)**
1. React UI shell + file drop zone
2. Processing queue component
3. Preview generation + A/B player
4. Waveform visualization
5. Progress tracking

### **Phase 3: Polish & Features (Weeks 8-10)**
1. Preset system (YouTube, LinkedIn, Instagram)
2. Intensity control (dry/wet mix)
3. Quality meter + warnings
4. Batch queue
5. Settings panel

### **Phase 4: Commercial & Distribution (Weeks 11-12)**
1. License system + feature gating
2. Code signing + notarization
3. Installer creation
4. Auto-update system
5. Analytics + crash reporting

---

## 🎨 Technology Stack

| Layer | Technology | Why |
|-------|------------|-----|
| **Desktop Shell** | Tauri 2 | Smaller bundle than Electron, native performance, secure by default |
| **Backend** | Rust | Memory safety, FFmpeg interop, excellent audio processing performance |
| **Frontend** | React + TypeScript | Fast iteration, strong component ecosystem, type safety |
| **Media I/O** | FFmpeg | Universal codec support, battle-tested, LGPL-compliant |
| **ML Inference** | ONNX Runtime | Cross-platform, hardware acceleration options (CPU/GPU) |
| **Enhancement Model** | DeepFilterNet | Open-source (Apache 2.0), real-time capable, CPU-friendly |
| **Payments** | Paddle | Merchant of record (handles tax), SaaS subscriptions |

---

## 📦 Project Structure

```
audio-cleaner/
├── src-tauri/                  # Rust backend
│   ├── src/
│   │   ├── main.rs            # Entry point
│   │   ├── commands/          # Tauri command handlers
│   │   ├── processing/        # Core processing pipeline
│   │   ├── enhancement/       # ML inference (ONNX)
│   │   ├── dsp/               # Signal processing
│   │   ├── ffmpeg/            # FFmpeg wrapper
│   │   ├── models/            # Data models
│   │   ├── state/             # App state management
│   │   └── persistence/       # Settings/license/cache
│   └── Cargo.toml
├── src/                       # React frontend
│   ├── components/            # UI components
│   ├── hooks/                 # Custom React hooks
│   ├── services/              # Tauri API wrappers
│   ├── types/                 # TypeScript types
│   └── App.tsx
├── public/
│   └── models/                # Bundled ML models
├── DESIGN.md                  # Complete system design
├── PHASE1_READINESS.md        # Implementation readiness checklist
└── Audio-cleaner.pdf          # Product Requirements Document
```

---

## 🎯 Target Market

**Primary Users:**
- YouTubers, tutorial/screencast creators, course creators
- Indie commentators, coaches, consultants
- Small content agencies

**Market Size:**
- TAM: 2.25M users (desktop-first spoken-video creators)
- SAM: 540K users (English-first, YouTube-first, solo/small-team)
- SOM: 4,000-10,000 paying users (24-month target)

**Revenue Model:**
- Free: Preview unlimited, capped export (3-5/month)
- Pro: $99/year or $15/month (watermark-free, batch processing)
- Lifetime: $149 (launch accelerator)

---

## 📋 Key Features

### **MVP (v1)**
- ✅ Drag-and-drop video file ingestion (MP4, MOV, MKV, WebM)
- ✅ One-click speech cleanup preset (YouTube, LinkedIn, Instagram)
- ✅ A/B preview comparison (original vs enhanced)
- ✅ Export remuxed video without re-editing (preserves video stream)
- ✅ Loudness consistency (platform-specific targets: -14 to -16 LUFS)
- ✅ Local-only processing (100% offline capability)
- ✅ Batch queue for multiple files
- ✅ Speech/music awareness (adaptive enhancement)
- ✅ Quality meter (source quality assessment with warnings)
- ✅ Dry/wet intensity control (0-100% enhancement strength)

### **v2 (Future)**
- 🔄 Watch folders / auto-process
- 🔄 Transcript-assisted cleanup (jump to problem segments)
- 🔄 Team presets and shared profiles
- 🔄 Optional cloud "salvage mode" (heavy dereverb)
- 🔄 API / CLI for workflow automation

---

## 🔐 Licensing & Compliance

**Application License:** TBD (Commercial closed-source)

**Dependencies:**
- FFmpeg: LGPL v2.1+ (compliant build, no GPL codecs)
- DeepFilterNet: Apache 2.0 (commercial use allowed)
- Silero VAD: MIT (commercial use allowed)
- ONNX Runtime: MIT (commercial use allowed)
- Tauri: MIT/Apache 2.0 (commercial use allowed)

**Compliance:**
- ✅ LGPL-compliant FFmpeg build documented
- ✅ All ML models use permissive licenses (Apache 2.0 / MIT)
- ✅ Code signing + notarization for macOS/Windows trust

---

## 🛠️ Development Setup

*(To be added during implementation - Phase 1)*

Prerequisites:
- Rust 1.70+ (via rustup)
- Node.js 18+ (for React frontend)
- FFmpeg 5.x+ (bundled in release builds)
- ONNX Runtime (bundled via Rust crate)

---

## 📞 Support & Feedback

- **Issues:** TBD (GitHub Issues or support email)
- **Documentation:** See DESIGN.md for complete technical documentation
- **PRD:** See Audio-cleaner.pdf for product vision and requirements

---

## 📅 Timeline

**12-week MVP timeline** (670 hours at 55-60 hours/week):
- Weeks 1-4: Core Pipeline ← **Current Phase**
- Weeks 5-7: UI & Preview
- Weeks 8-10: Polish & Features
- Weeks 11-12: Commercial & Distribution

**Current Status:** Phase 1 specifications complete, ready for implementation.

---

**Built with ❤️ for creators who want better audio without the complexity.**
