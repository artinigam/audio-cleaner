# SYSTEM DESIGN DOCUMENT
## Audio Cleanup Desktop Application for YouTube Creators

**Version:** 2.0  
**Document Type:** Comprehensive System Design (Engineering + Architecture)  
**Target Platform:** Windows & macOS Desktop  
**Architecture:** Tauri + Rust + React/TypeScript  
**Processing Model:** Local-first with offline capability  
**Source:** Audio-cleaner.pdf PRD

---

## EXECUTIVE SUMMARY

This document presents the complete system design for an **Audio Cleanup Desktop Application** targeting YouTube creators, tutorial/screencast creators, course creators, indie commentators, coaches, and small content agencies. The application addresses a critical workflow gap: transforming "decent" creator recordings into "studio-quality" upload-ready videos through automated AI-driven audio enhancement.

**Problem Statement**: Creators face a resource-heavy, multi-step, artifact-prone workflow when cleaning spoken audio in video files. Current solutions are either embedded in heavy NLE software (DaVinci Resolve, Adobe), cloud-dependent with usage metering (Descript, Auphonic), or require plugin/host knowledge (iZotope, Waves).

**Market Opportunity**: TAM of ~2.25M users, SAM of 540K users, achievable SOM of 4,000-10,000 paying users within 24 months. Revenue potential: $316K-$1.49M SOM at ARPU $79-$149.

**Chosen Architecture**: **Desktop-first Monolithic Application** with Tauri framework, Rust backend for audio processing, React/TypeScript frontend for UI, FFmpeg for media I/O, and ONNX Runtime for ML inference. This architecture prioritizes:
- **Local-first processing**: Privacy, offline capability, no cloud dependency
- **Single-purpose utility**: Fast iteration, lower operational overhead vs microservices
- **Cross-platform desktop**: Windows/macOS native performance with web UI flexibility
- **Embedded ML models**: DeepFilterNet (Apache 2.0) for denoise/dereverb without cloud costs

**Key Design Decisions**:
1. **Local-first over cloud**: Aligns with whitespace in market, creator privacy concerns, predictable unit economics
2. **Monolith over microservices**: Single developer, rapid iteration priority, simpler deployment
3. **Rust for performance**: Memory-safe audio pipeline, FFmpeg interop, 10-20 min processing target for typical creator videos
4. **Freemium model**: Preview unlimited, export limited (free) vs watermark-free batch export (Pro $99/year)

**Success Metrics**: 
- Processing time: 10-20 minutes per video on modern laptops
- Quality: Output preferred over original in 60%+ of blind tests
- A/V sync drift: <40ms tolerance
- Export compatibility: YouTube/LinkedIn/Instagram loudness standards

---

## 1. REQUIREMENTS ANALYSIS

### 1.1 Functional Requirements (Source: Audio-cleaner.pdf)

| ID | Requirement | Priority | Source (PRD Section) | Acceptance Criteria |
|----|-------------|----------|----------------------|---------------------|
| FR-1 | Drag-and-drop video file ingestion (MP4/MOV/MKV/WebM) | High | MVP Features (p.10) | File probe succeeds on common creator formats; unsupported files fail with clear message |
| FR-2 | One-click speech cleanup preset | High | MVP Features (p.10) | Output preferred over original in internal listening tests on a majority of noisy creator clips |
| FR-3 | A/B preview comparison (original vs enhanced) | High | MVP Features (p.10) | Toggle between versions in two clicks or fewer |
| FR-4 | Export remuxed video without re-editing | High | MVP Features (p.10) | Original video stream preserved; enhanced audio replaced; sync drift under 40ms |
| FR-5 | Loudness consistency (platform-specific targets) | High | MVP Features (p.10) | Output stays within defined loudness/peak guardrails (YouTube: -14 LUFS, -1 dBTP) |
| FR-6 | Local-only processing (offline capability) | High | MVP Features (p.10) | Entire cleanup path works offline after install |
| FR-7 | Platform presets (YouTube, LinkedIn, Instagram) | Medium | v1 Features (p.11) | Users can export with explainable presets, not hidden magic |
| FR-8 | Batch queue for multiple files | Medium | v1 Features (p.11) | Queue survives multiple files and partial failures |
| FR-9 | Speech/music awareness (avoid crushing music beds) | Medium | v1 Features (p.11) | Non-speech sections are affected less aggressively |
| FR-10 | Quality meter (source quality assessment) | Medium | v1 Features (p.11) | App warns when source is too degraded for "studio" result |
| FR-11 | Dry/wet intensity control | Medium | v1 Features (p.11) | User can back off enhancement if voice sounds overprocessed |
| FR-12 | Watch folders / auto-process (v2) | Low | v2 Features (p.11) | Strong agency and team use case |
| FR-13 | Transcript-assisted cleanup (v2) | Low | v2 Features (p.11) | Lets users jump to problem segments |
| FR-14 | Team presets and shared profiles (v2) | Low | v2 Features (p.11) | Useful for agencies and small studios |
| FR-15 | Optional cloud "salvage mode" (v2) | Low | v2 Features (p.11) | For difficult dereverb or severe source repair |
| FR-16 | API / CLI (v2) | Low | v2 Features (p.11) | Opens B2B and workflow automation paths |

### 1.2 Non-Functional Requirements (Source: Audio-cleaner.pdf + Assumptions)

| ID | Category | Requirement | Target | Source (PRD Section) | Rationale |
|----|----------|-------------|--------|----------------------|-----------|
| NFR-1 | Performance | Processing time per video | 10-20 minutes | Executive Summary (p.1), UX Flow (p.11) | Core promise: "save creators 10-20 minutes per video" |
| NFR-2 | Performance | Preview generation latency | <30 seconds for 5-10s clip | UX Flow (p.11) - "hear the difference" step | Fast feedback loop for A/B comparison |
| NFR-3 | Performance | A/V sync accuracy | <40ms drift | MVP Features (p.10) | Imperceptible to viewers; critical for credibility |
| NFR-4 | Scalability | Concurrent processing jobs | 1-2 simultaneous | Module Breakdown (p.14) - "solo developer", batch queue | Desktop app, single user; limit to prevent resource exhaustion |
| NFR-5 | Scalability | Supported video file sizes | 100MB - 10GB | Assumption based on creator workflows | YouTube uploads range from phone clips to 4K recordings |
| NFR-6 | Scalability | Audio duration support | 5 min - 3 hours | Assumption based on tutorial/course content | Handles typical tutorial (10-30 min) and long-form content |
| NFR-7 | Reliability | Export success rate | >95% for supported formats | Milestone Plan (p.15) - "remux success on 95% of test files" | Critical for trust; failures must be rare |
| NFR-8 | Reliability | Crash-free sessions | >99% (no unexpected crashes) | Analytics plumbing (p.14) | Desktop app; crashes destroy trust |
| NFR-9 | Reliability | Uptime (local processing) | 100% offline after install | Core promise: "local-only processing" (p.10) | No cloud dependency for core workflow |
| NFR-10 | Usability | Onboarding time | <5 minutes (first export) | UX Flow (p.11) - "Drop → hear → export" | One-sentence UX: simplicity is core differentiator |
| NFR-11 | Usability | Learning curve | Non-technical creators can use without training | Target Users (p.15) - tutorial/course creators | "No audio expertise" required |
| NFR-12 | Security | Client data privacy | Videos never leave local machine (MVP) | Offline architecture (p.13) | Privacy and trust for client/course footage |
| NFR-13 | Security | License validation | Online check every 24h (graceful offline fallback) | Licensing posture (p.12-13) | Balance piracy prevention with offline usability |
| NFR-14 | Maintainability | Code signing & notarization | Signed from alpha stage | Distribution tactics (p.9), Implementation risks (p.14) | macOS/Windows trust; reduces "unverified app" warnings |
| NFR-15 | Maintainability | Auto-update system | Background updates, user-initiated install | Technology Stack (p.12) | Model upgrades, codec support, bug fixes |
| NFR-16 | Maintainability | Crash reporting & analytics | Opt-in telemetry for diagnostics | Analytics plumbing (p.14) | Solo developer needs failure visibility |
| NFR-17 | Compatibility | Supported platforms | Windows 10+ (64-bit), macOS 11+ (Intel & Apple Silicon) | Technology Stack (p.12), Distribution (p.9) | Cover 95%+ of creator desktop base |
| NFR-18 | Compatibility | Video codec support | H.264, H.265/HEVC, VP9, AV1 (read-only, stream copy) | Assumption based on YouTube/platform standards | Handle all major creator upload formats |
| NFR-19 | Compatibility | Audio codec support | AAC, MP3, Opus, PCM (input); AAC (output) | Assumption based on platform standards | AAC for export (universal platform compatibility) |
| NFR-20 | Quality | Audio output quality | Perceived improvement in 60%+ of blind tests | Milestone Plan (p.15) - "60% of noisy clips" | Subjective but measurable; benchmark corpus needed |
| NFR-21 | Quality | Loudness standard compliance | ITU-R BS.1770-4 (LUFS measurement) | Industry standard for broadcast loudness | YouTube/streaming platform requirement |
| NFR-22 | Licensing | FFmpeg compliance | LGPL-compliant build, document dependencies | Implementation risks (p.14) | Can block distribution if mishandled |
| NFR-23 | Licensing | ML model licensing | Apache 2.0 / MIT (DeepFilterNet baseline) | Model options (p.12) | Commercial use without royalties |

### 1.3 Assumptions and Constraints

**Assumptions** (stated explicitly):
1. **Target hardware**: Modern laptops (2020+) with 8GB+ RAM, quad-core CPU, SSD storage - *Source: "modern laptops" in Milestone Plan (p.15)*
2. **Creator workflows**: Publish video often enough that audio cleanup pain is recurring - *Source: "frequent-publishing share" sizing (p.4)*
3. **Desktop-first preference**: 60% of spoken-word creators value Windows/Mac local processing over mobile-only editing - *Source: Sizing model (p.4)*
4. **English-first scope**: Initial serviceable scope focuses on English-first, YouTube-first, solo/small-team, local desktop app - *Source: Assumptions and sizing model (p.4)*
5. **Conversion benchmarks**: Conservative assumption: convert below generic B2B trial benchmarks due to solo developer, strong niche GTM, no paid brand budget - *Source: Conversion assumptions (p.8)*
6. **Processing quality modes**: Fast (preview), Standard (export), Maximum (future) to balance latency vs quality - *Assumed based on solo developer reality*
7. **Free tier limits**: Export count limit for free tier (e.g., 3 exports/month) to drive Pro conversion - *Implied by freemium model (p.7-8)*
8. **Model download**: Larger models (20MB+) downloaded on first use; smaller models (<10MB) bundled in installer - *Assumed for installer size*

**Constraints**:
1. **Solo developer**: No DevOps/SRE team; limits operational complexity (rules out microservices, Kubernetes) - *Source: "solo developer" throughout PRD*
2. **No paid brand budget**: Organic launch channels only (Reddit/forums for acquisition, Product Hunt for early adopters) - *Source: Distribution tactics (p.9)*
3. **12-week MVP timeline**: 670 hours at 55-60 hours/week focused pace - *Source: Module breakdown and milestone plan (p.14-15)*
4. **LGPL compliance**: FFmpeg is LGPL; must use compliant build, document dependencies - *Source: Implementation risks (p.14)*
5. **No cloud infrastructure (MVP)**: Processing must work 100% offline; optional cloud tier is v2 - *Source: Architecture (p.13)*
6. **No GPU acceleration (MVP)**: Stick to CPU inference for compatibility; GPU acceleration is optimization - *Assumed based on timeline*

---

## 2. HIGH-LEVEL ARCHITECTURE

### 2.1 Architecture Pattern Selection

**Chosen Pattern**: **Monolithic Desktop Application** (Tauri framework)

**Decision Framework**:

| Pattern | Evaluation | Fit for Audio Cleanup App |
|---------|------------|--------------------------|
| **Microservices** | ✗ | **Rejected**: Overkill for single developer; no independent scaling needs; shared transactions (audio + video sync) |
| **Monolith** | ✓✓✓ | **Selected**: Solo dev, rapid iteration, simpler deployment, shared audio/video processing state |
| **Event-Driven** | ○ | **Partial fit**: Use internally for progress updates (Tauri event system), but not primary architecture |
| **Layered Architecture** | ✓ | **Adopted within monolith**: Clear separation: Presentation (React) → Business (Rust processing) → Data (FFmpeg/ONNX) |

**Rationale for Monolithic Tauri Architecture**:
1. **Solo developer constraint**: No team to manage distributed services; operational simplicity critical *(NFR-15, NFR-16)*
2. **Desktop-first requirement**: Tauri provides native performance with web UI flexibility; smaller bundle than Electron *(PRD p.12: "smaller bundle size")*
3. **Shared transaction scope**: Audio extraction, enhancement, and video remuxing must maintain A/V sync - single process reduces coordination failures *(NFR-3: <40ms sync drift)*
4. **Offline-first constraint**: No network dependency for core workflow; all processing local *(NFR-9: 100% offline)*
5. **Rapid iteration priority**: "10-12 weeks" MVP timeline favors tight coupling over microservice interfaces *(PRD p.1: "buildable but not trivial")*

**Trade-offs Accepted**:
- **Less horizontal scalability**: Desktop app = one user; acceptable for target market *(NFR-4: 1-2 concurrent jobs sufficient)*
- **Tighter coupling**: Changes to audio pipeline may affect UI; mitigated by clean Tauri command API layer
- **Single language dominance**: Rust backend limits contributors to Rust developers; acceptable for solo dev

**Alternatives Rejected**:
- **Cloud-first web app**: Rejected due to upload friction, privacy concerns, cloud costs eating margins *(PRD p.13: "local-first is right default")*
- **Electron**: Rejected due to larger bundle size vs Tauri; Rust backend more suitable for audio pipelines than Node.js *(PRD p.12: Tauri advantages)*
- **Native Swift/C++**: Rejected due to cross-platform effort (Windows + macOS); Tauri provides write-once, deploy-twice

### 2.2 System Components Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           USER (Creator)                                     │
│                     Drag video → Preview → Export                            │
└────────────────────────────────┬────────────────────────────────────────────┘
                                 │
┌────────────────────────────────▼────────────────────────────────────────────┐
│                        PRESENTATION LAYER (React/TypeScript)                 │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │ FileDropZone│  │ MediaPreview │  │ ProcessingQueue│ │ ExportDialog │    │
│  │ Component   │  │ (A/B Player) │  │ Manager       │  │              │    │
│  └──────┬──────┘  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘    │
│         │                │                   │                 │             │
│         └────────────────┴───────────────────┴─────────────────┘             │
│                                   │                                           │
│                          Tauri IPC Commands                                   │
│                         (invoke, event listeners)                             │
└───────────────────────────────────┼───────────────────────────────────────────┘
                                    │
┌───────────────────────────────────▼───────────────────────────────────────────┐
│                      BUSINESS LOGIC LAYER (Rust Backend)                      │
│  ┌─────────────────────────────────────────────────────────────────────┐     │
│  │                    Command Handlers (Tauri)                          │     │
│  │  probe_media │ create_job │ start_processing │ generate_preview │   │     │
│  │  analyze_quality │ export_enhanced_video │ get_license_info       │     │
│  └──────────────────────────────┬────────────────────────────────────┘     │
│                                 │                                             │
│  ┌─────────────────────────────▼────────────────────────────────────┐       │
│  │              Processing Pipeline Orchestrator                     │       │
│  │  (Coordinates: Extract → Analyze → Enhance → Normalize → Remux)  │       │
│  └──────────────────────────────┬────────────────────────────────────┘       │
│                                 │                                             │
│       ┌─────────────────────────┼─────────────────────────┐                 │
│       │                         │                         │                   │
│  ┌────▼─────┐          ┌────────▼──────┐         ┌──────▼────────┐         │
│  │ Media    │          │ Enhancement   │         │ DSP           │         │
│  │ Module   │          │ Engine        │         │ Module        │         │
│  │          │          │ (ML Inference)│         │ (Filters,     │         │
│  │ - Probe  │          │ - DeepFilterNet│         │  Compression, │         │
│  │ - Extract│          │ - Chunking    │         │  Loudness)    │         │
│  │ - Remux  │          │ - ONNX Runtime│         │               │         │
│  └────┬─────┘          └────────┬──────┘         └──────┬────────┘         │
│       │                         │                         │                   │
│       └─────────────────────────┼─────────────────────────┘                 │
│                                 │                                             │
│  ┌─────────────────────────────▼────────────────────────────────────┐       │
│  │                     State & Persistence                           │       │
│  │  - AppState (jobs, license, settings)                            │       │
│  │  - Temp file manager                                             │       │
│  │  - Settings/License/Preset storage                               │       │
│  └───────────────────────────────────────────────────────────────────┘       │
└───────────────────────────────────────┬───────────────────────────────────────┘
                                        │
┌───────────────────────────────────────▼───────────────────────────────────────┐
│                      EXTERNAL DEPENDENCIES / SYSTEM LAYER                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │    FFmpeg    │  │ ONNX Runtime │  │ File System  │  │ OS Services  │    │
│  │              │  │              │  │              │  │              │    │
│  │ - Probe      │  │ - DeepFilterNet│ - Temp dirs  │  │ - Sandboxing │    │
│  │ - Decode     │  │   inference  │  │ - App data   │  │ - Keychain   │    │
│  │ - Encode     │  │ - Silero VAD │  │ - User prefs │  │ - Signing    │    │
│  │ - Remux      │  │              │  │              │  │              │    │
│  └──────────────┘  └──────────────┘  └──────────────┘  └──────────────┘    │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### 2.3 Component Overview

| Component | Responsibilities | Technology | Rationale |
|-----------|------------------|------------|-----------|
| **Presentation Layer** | User interaction, drag-drop, A/B preview, progress visualization | React + TypeScript + Tailwind CSS | Fast iteration, strong component ecosystem, type safety |
| **Tauri IPC Layer** | Command invocation, event streaming (progress updates), security boundary | Tauri framework (Rust ↔ JS bridge) | Smaller bundle than Electron, native performance, secure by default |
| **Command Handlers** | Validate inputs, orchestrate backend services, emit progress events | Rust async (Tokio) | Memory-safe, async I/O for long-running operations |
| **Processing Pipeline** | End-to-end orchestration: probe → extract → enhance → remux | Rust | Single-threaded coordination; parallelism within stages (chunking) |
| **Media Module** | FFmpeg wrapper for probe/decode/encode/remux | Rust + FFmpeg CLI | FFmpeg = industry standard; CLI avoids C++ bindings complexity |
| **Enhancement Engine** | ML model inference (denoise, dereverb), audio chunking, overlap-add | Rust + ONNX Runtime | ONNX = cross-platform; Rust for memory-safe audio buffers |
| **DSP Module** | Traditional signal processing (EQ, compression, de-esser, loudness normalization) | Rust DSP libraries (biquad filters, ITU-R BS.1770) | Complement ML; loudness normalization is non-ML |
| **State Management** | AppState (jobs, license, settings), temp file lifecycle | Rust (Tokio Mutex, Arc) | Shared mutable state across async tasks |
| **Persistence** | Settings, license cache, preset storage, model cache | JSON files in OS app data directory | Simple, human-readable, no DB overhead |
| **FFmpeg** | Media container manipulation, codec detection, A/V stream handling | FFmpeg 5.x+ (LGPL build) | Universal media Swiss Army knife |
| **ONNX Runtime** | Cross-platform ML inference engine | ONNX Runtime 1.15+ | Hardware acceleration options (CPU, CoreML, DirectML future) |

---

## 3. INFRASTRUCTURE DECISIONS

### 3.1 Database Selection

**Decision: No Traditional Database (File-Based Persistence)**

**Options Considered**:
1. SQLite (embedded relational database)
2. JSON files in app data directory
3. IndexedDB (browser storage via Tauri)
4. No persistence (in-memory only)

**Decision Criteria Evaluation**:
- **Data complexity**: ○ Simple key-value (settings, license) and flat lists (presets, job queue history)
- **Query requirements**: ✓ No complex queries; all access by primary key (job ID, preset name)
- **Transaction requirements**: ○ No multi-table transactions; atomic file writes sufficient
- **Concurrent access**: ✓ Single user, single process (desktop app)
- **Portability**: ✓ Human-readable format for debugging/support
- **Setup complexity**: ✓ Zero schema migrations; version upgrades = file format migration scripts

**Chosen Option**: **JSON files in OS app data directory**

**Rationale**:
1. **Simplicity**: No schema migrations, no query language, no ORM - critical for solo developer maintaining codebase *(Constraint: solo developer)*
2. **Debuggability**: Support can ask users to send `settings.json` for troubleshooting; plain text format
3. **Portability**: Users can manually edit presets or back up settings by copying files
4. **Sufficient for scale**: Desktop app = single user; no concurrent writes; max ~100 presets, ~10 active jobs = KB not MB
5. **Tauri ecosystem fit**: Tauri apps commonly use file-based config (Rust `serde_json` is first-class)

**Trade-offs Accepted**:
- **No complex queries**: Can't do "find all jobs with status=Failed in last 7 days"; acceptable - we don't need this
- **Manual file locking**: Must implement atomic writes (write temp → rename); Rust stdlib provides this
- **No transactions**: Can't rollback multi-file changes; mitigated by writing to temp first, validating, then moving

**Alternatives Rejected**:
- **SQLite**: Rejected - overkill for key-value + small lists; adds migration complexity *(no NFR requires relational queries)*
- **IndexedDB**: Rejected - browser storage in desktop app is awkward; less portable than files
- **No persistence**: Rejected - users expect settings/presets to persist across sessions *(NFR-11: usability)*

**Requirements Addressed**: NFR-10 (usability), NFR-15 (maintainability), NFR-16 (diagnostics)

---

### 3.2 Caching Strategy

**Decision: Multi-Tier Caching (In-Memory + Filesystem + Model Cache)**

**Options Considered**:
1. No caching (regenerate everything)
2. In-memory cache only (lost on app restart)
3. Filesystem cache (persistent across sessions)
4. Hybrid: In-memory + Filesystem
5. External cache (Redis/Memcached) - N/A for desktop app

**Decision Criteria Evaluation**:
- **Performance impact**: ✓✓✓ Waveform generation (3-5s), preview clips (30s), model loading (2-3s) = high cache value
- **Storage cost**: ✓ Desktop SSD storage is abundant; 100MB-500MB cache is negligible
- **Invalidation complexity**: ✓ Invalidation is simple: file mtime for source video, preset hash for previews
- **User benefit**: ✓✓ Instant preview scrubbing, instant preset switching = UX wins

**Chosen Option**: **Hybrid: In-Memory (runtime) + Filesystem (persistent) + Model Cache**

**Caching Tiers**:

| Tier | What | TTL | Rationale | Invalidation |
|------|------|-----|-----------|--------------|
| **L1: In-Memory** | Loaded ONNX models, decoded audio chunks | Process lifetime | Avoid redundant model loads (2-3s overhead), reuse decoded audio for preview region scrubbing | App restart |
| **L2: Filesystem** | Waveform visualizations, preview clips (original + enhanced) | 24 hours or file mtime change | Instant re-open of same video; waveform regeneration is 3-5s overhead | Delete on source file mtime change or cache size > 500MB |
| **L3: Model Cache** | Downloaded ONNX models (`deepfilternet_*.onnx`, `silero_vad.onnx`) | Permanent until app update | Bundled models in installer; larger models downloaded once | Delete only on app uninstall or manual cache clear |

**Rationale**:
1. **Model loading overhead**: ONNX Runtime model init is 2-3s; cache in memory after first load *(NFR-2: preview latency <30s)*
2. **Waveform reuse**: Waveform generation (FFT + downsampling) is 3-5s; cache to filesystem for instant scrubbing on reopen
3. **Preview clips**: 5-10s preview generation is 30s; cache both original + enhanced for instant A/B toggling
4. **Disk space is cheap**: Modern SSDs have 256GB+; 500MB cache limit is 0.2% of typical storage *(NFR-5: support 100MB-10GB video files)*

**Trade-offs Accepted**:
- **Cache invalidation bugs**: Stale cache if file modified externally; mitigated by checking file mtime before cache hit
- **Disk space usage**: 500MB cache limit; auto-cleanup when exceeded (LRU eviction)

**Alternatives Rejected**:
- **No caching**: Rejected - every preview regenerates waveform (3-5s) and processes audio (30s), killing UX *(NFR-2: <30s preview)*
- **In-memory only**: Rejected - cache lost on app restart; users expect instant reopen of recent files
- **External cache (Redis)**: Rejected - desktop app, single user; no need for networked cache

**Requirements Addressed**: NFR-2 (preview latency <30s), NFR-10 (usability), NFR-11 (non-technical users)

---

### 3.3 Compute Infrastructure

**Decision: Native Desktop Process (No Cloud Compute)**

**Options Considered**:
1. Local desktop process (CPU-based)
2. Local desktop with GPU acceleration (DirectML/CoreML)
3. Hybrid: Local preview + Cloud export
4. Cloud-only (serverless functions or VMs)

**Decision Criteria Evaluation**:
- **Privacy**: ✓✓✓ Creators uploading client/course footage; local-only = differentiator *(PRD p.13: "privacy and trust")*
- **Offline capability**: ✓✓✓ Core promise: works without internet *(FR-6: local-only processing, NFR-9: 100% offline)*
- **Unit economics**: ✓✓ Local compute = zero marginal cost; cloud = $0.10-0.50/video (kills margins) *(PRD p.13: "predictable unit economics")*
- **Latency**: ✓✓ Local CPU: 10-20 min; cloud upload (5-10 min) + process (5 min) + download (5 min) = 15-30 min *(worse UX)*
- **Performance target**: ○ 10-20 min per video on "modern laptops" (2020+, quad-core, 8GB RAM) *(NFR-1)*

**Chosen Option**: **Local desktop process (CPU-based), optional GPU acceleration (future optimization)**

**Compute Breakdown**:
- **Audio extraction**: FFmpeg (CPU-bound), ~30s for 30-min video
- **Enhancement inference**: ONNX Runtime + DeepFilterNet (CPU), ~5-10 min for 30-min video (chunked processing)
- **DSP chain**: Rust native (CPU), ~1-2 min for 30-min video
- **Remuxing**: FFmpeg (CPU-bound), ~1-2 min for 30-min video
- **Total**: 10-15 min typical, 20 min worst-case (4K video, older hardware)

**Rationale**:
1. **Privacy & trust**: Client footage, course videos never leave local machine = key differentiator vs cloud editors *(PRD p.13)*
2. **Offline capability**: Works on planes, in areas with poor internet, after internet outage *(FR-6)*
3. **Unit economics**: Zero marginal cost per export vs $0.10-0.50/video cloud costs = healthier margins for freemium model *(PRD p.8)*
4. **Simplicity**: No cloud infrastructure, no API keys, no rate limits = solo developer can ship faster *(Constraint: solo developer)*
5. **Acceptable performance**: 10-20 min on modern hardware meets "save creators 10-20 min per video" promise *(NFR-1)*

**Trade-offs Accepted**:
- **Slower than server-grade GPUs**: Cloud with A100 GPU could process in 2-3 min; acceptable tradeoff for privacy/offline
- **Hardware variability**: 2018 laptops may take 30-40 min; mitigated by quality meter warning + "Fast" mode
- **No elastic scaling**: Can't offload to cloud on-demand; v2 feature ("salvage mode") can address edge cases

**Alternatives Rejected**:
- **Cloud-only**: Rejected - violates core promise (local-first), kills margins, requires internet *(PRD p.13)*
- **Hybrid (local preview + cloud export)**: Rejected - adds complexity, splits user experience, requires cloud infra *(solo developer constraint)*
- **GPU-required**: Rejected - limits addressable market (not all creators have discrete GPUs); CPU-based is baseline, GPU is optimization

**Requirements Addressed**: FR-6 (local processing), NFR-1 (10-20 min processing), NFR-9 (100% offline), NFR-12 (privacy)

---

### 3.4 Message Queue / Event System

**Decision: Tauri Event System (Built-in IPC)**

**Options Considered**:
1. Tauri event system (built-in)
2. External message queue (RabbitMQ, Kafka, Redis Pub/Sub)
3. Polling (frontend polls backend for status)
4. WebSockets (custom implementation)

**Decision Criteria Evaluation**:
- **Use case**: ✓ Progress updates (processing:progress, export:completed), not distributed systems
- **Throughput**: ✓ Low volume (~10 events/sec during processing); no high-throughput requirement
- **Latency**: ✓ Sub-100ms latency sufficient for progress bar updates
- **Complexity**: ✓✓✓ Tauri provides event system out-of-box; zero additional dependencies
- **Desktop app context**: ✓✓✓ Single process, single user; no need for external queue

**Chosen Option**: **Tauri Event System (emit_all + listen)**

**Event Design**:
```rust
// Backend → Frontend events
app_handle.emit_all("processing:progress", ProgressEvent {
    job_id: String,
    progress: f32,        // 0.0-1.0
    stage: String,        // "Extracting", "Enhancing", "Remuxing"
    eta_seconds: Option<u32>,
});

app_handle.emit_all("processing:completed", CompletionEvent {
    job_id: String,
    output_path: String,
});

app_handle.emit_all("processing:failed", ErrorEvent {
    job_id: String,
    error: String,
});
```

**Rationale**:
1. **Built-in**: Tauri event system is zero-config, zero-dependency; perfect for desktop app *(Constraint: solo developer)*
2. **Type-safe**: Rust events serialize to JSON, TypeScript deserializes with type safety
3. **Sufficient throughput**: Processing emits progress every 1-2 seconds; Tauri handles this easily
4. **Desktop-appropriate**: Single process, single user; external queue is overkill *(NFR-4: 1-2 concurrent jobs)*

**Trade-offs Accepted**:
- **No persistence**: Events are fire-and-forget; if frontend misses event, state is lost; mitigated by backend maintaining job status that frontend can poll on reconnect
- **No replay**: Can't replay event history; acceptable - users don't need to see past progress updates

**Alternatives Rejected**:
- **External queue (RabbitMQ/Kafka)**: Rejected - massive overkill for desktop app; adds deployment complexity *(solo developer)*
- **Polling**: Rejected - inefficient, adds latency to progress updates; Tauri events are push-based
- **Custom WebSockets**: Rejected - reinventing Tauri's built-in IPC; no benefit

**Requirements Addressed**: FR-3 (A/B preview updates), NFR-10 (usability - responsive UI), NFR-15 (maintainability - simple architecture)

---

### 3.5 Media Processing (FFmpeg)

**Decision: FFmpeg CLI Wrapper (LGPL-compliant build)**

**Options Considered**:
1. FFmpeg CLI (subprocess calls)
2. FFmpeg Rust bindings (e.g., `rust-ffmpeg` crate)
3. Pure Rust media libraries (e.g., `symphonia`)
4. GStreamer

**Decision Criteria Evaluation**:
- **Codec coverage**: ✓✓✓ FFmpeg supports 100+ codecs; pure Rust libraries lack HEVC, VP9, AV1 encoders
- **Maintenance burden**: ✓ CLI wrapper is simpler than maintaining C bindings across Windows/macOS
- **Licensing**: ✓ LGPL-compliant FFmpeg build allows commercial use; GPL components excluded
- **Performance**: ○ CLI subprocess overhead is ~50-100ms per call; acceptable for batch operations (extract, remux)
- **Cross-platform**: ✓✓✓ FFmpeg binaries available for Windows/macOS/Linux

**Chosen Option**: **FFmpeg CLI Wrapper (LGPL build, bundled in installer)**

**FFmpeg Integration Pattern**:
```rust
// Wrapper struct for type-safe FFmpeg commands
pub struct FFmpegCommand {
    input: PathBuf,
    output: PathBuf,
    args: Vec<String>,
}

impl FFmpegCommand {
    pub fn new(input: PathBuf, output: PathBuf) -> Self { ... }
    
    pub fn map_stream(mut self, spec: &str) -> Self { ... }
    pub fn audio_codec(mut self, codec: &str) -> Self { ... }
    pub fn video_codec(mut self, codec: &str) -> Self { ... }
    
    pub async fn execute(self) -> Result<(), String> {
        let status = Command::new("ffmpeg")
            .args(&["-i", &self.input.to_string_lossy()])
            .args(&self.args)
            .arg("-y")  // Overwrite
            .arg(&self.output)
            .status()
            .await?;
        
        if status.success() { Ok(()) } else { Err("FFmpeg failed".into()) }
    }
}
```

**Rationale**:
1. **Universal codec support**: FFmpeg handles all creator video formats (H.264, HEVC, VP9, AV1, MOV, MP4, MKV, WebM) *(FR-1: drag-and-drop support)*
2. **Battle-tested**: FFmpeg is industry standard (YouTube uses it, DaVinci Resolve uses it); proven stability
3. **LGPL compliance**: Using LGPL build (no GPL codecs like x264) allows commercial distribution *(NFR-22: LGPL compliance)*
4. **Simpler than bindings**: CLI wrapper avoids C++ ABI nightmares across platforms; easier to debug *(Constraint: solo developer)*
5. **Subprocess overhead acceptable**: Extraction (once per video), remuxing (once per export) = 2 calls; 50ms overhead is negligible vs 10-20 min total processing

**Trade-offs Accepted**:
- **Subprocess overhead**: 50-100ms per call vs in-process bindings (10ms); acceptable for infrequent operations
- **No streaming**: Must write intermediate files (extracted audio, enhanced audio) vs streaming pipes; acceptable given disk space (SSD speeds make temp files fast)
- **Dependency bundling**: Must ship FFmpeg binary (~50-80MB) in installer; increases installer size from ~20MB to ~100MB; acceptable given creator download speeds

**Alternatives Rejected**:
- **FFmpeg Rust bindings**: Rejected - cross-platform compilation is fragile (Windows MSVC vs MinGW, macOS SDK versions); CLI is simpler *(solo developer constraint)*
- **Pure Rust (`symphonia`)**: Rejected - lacks encoder support for AAC, HEVC, VP9; demuxer-only is insufficient *(FR-4: export remuxed video)*
- **GStreamer**: Rejected - larger dependency footprint, LGPL + plugin complications, less familiar than FFmpeg *(maintainability)*

**Requirements Addressed**: FR-1 (format support), FR-4 (export remuxed video), NFR-3 (A/V sync <40ms), NFR-18/19 (codec compatibility), NFR-22 (LGPL compliance)

---

### 3.6 ML Inference Engine

**Decision: ONNX Runtime (DeepFilterNet model)**

**Options Considered**:
1. ONNX Runtime (cross-platform)
2. PyTorch Mobile (bundled LibTorch)
3. TensorFlow Lite
4. Native Rust ML (e.g., `burn`, `tract`)
5. Cloud API (Dolby.io, Krisp, AudioShake)

**Decision Criteria Evaluation**:
- **Model availability**: ✓✓✓ DeepFilterNet provides official ONNX exports (Apache 2.0 license)
- **Cross-platform**: ✓✓✓ ONNX Runtime supports Windows/macOS/Linux, CPU + GPU
- **Performance**: ✓✓ CPU inference for 30-min video in 5-10 min (acceptable per NFR-1)
- **Hardware acceleration**: ✓ Future GPU support via DirectML (Windows), CoreML (macOS)
- **Licensing**: ✓✓✓ ONNX Runtime is MIT-licensed; DeepFilterNet is Apache 2.0 *(NFR-23: Apache/MIT models)*
- **Deployment size**: ✓ ONNX Runtime adds ~10-15MB to bundle; acceptable

**Chosen Option**: **ONNX Runtime + DeepFilterNet (quantized model for MVP, full-precision for Pro)**

**Model Strategy**:
| Model | Size | Quality | Use Case | Bundled? |
|-------|------|---------|----------|----------|
| `deepfilternet_3_quant.onnx` | ~8MB | Good (quantized INT8) | MVP, fast preview | Yes (in installer) |
| `deepfilternet_3_full.onnx` | ~20MB | Excellent (FP32) | Pro export, maximum quality | Download on first use |
| `silero_vad_v4.onnx` | ~1MB | N/A (VAD only) | Speech detection | Yes (in installer) |

**Rationale**:
1. **Model licensing**: DeepFilterNet (Apache 2.0) allows commercial use without royalties *(PRD p.12: "legitimate commercial posture")*
2. **Cross-platform**: ONNX Runtime runs on Windows/macOS/Linux with single codebase; avoids platform-specific frameworks *(NFR-17: Windows 10+, macOS 11+)*
3. **Performance**: Quantized model (INT8) is 2x faster than FP32 with minimal quality loss; acceptable for MVP *(NFR-1: 10-20 min processing)*
4. **Future GPU acceleration**: ONNX Runtime supports DirectML (Windows), CoreML (macOS) for 5-10x speedup; opt-in optimization path
5. **Proven stack**: ONNX is industry standard (Microsoft, Meta, AWS use it); better long-term support than niche Rust ML libs

**Trade-offs Accepted**:
- **Larger bundle**: ONNX Runtime adds 10-15MB vs pure Rust; acceptable tradeoff for cross-platform + GPU future
- **External dependency**: Runtime must be bundled or statically linked; increases deployment complexity slightly
- **Not cutting-edge**: ONNX lags latest PyTorch by 3-6 months; acceptable - stability > bleeding edge for desktop app

**Alternatives Rejected**:
- **PyTorch Mobile**: Rejected - LibTorch is 100-200MB (huge); C++ bindings are fragile across platforms *(bundle size concern)*
- **TensorFlow Lite**: Rejected - worse speech model ecosystem than PyTorch; TFLite adoption declining
- **Native Rust ML (`tract`)**: Rejected - immature ecosystem, fewer pre-trained models, no hardware acceleration *(risk: solo developer can't debug low-level ML issues)*
- **Cloud API (Dolby.io, Krisp)**: Rejected - violates local-first promise, adds recurring costs ($0.10-0.50/video), requires internet *(FR-6: local-only)*

**Requirements Addressed**: FR-2 (one-click cleanup), NFR-1 (10-20 min processing), NFR-9 (offline capability), NFR-12 (privacy), NFR-23 (Apache/MIT licensing)

---

### 3.7 License Management & Payments

**Decision: Paddle (Merchant of Record) + Local License Validation**

**Options Considered**:
1. Paddle (merchant of record)
2. Stripe + manual tax handling
3. Gumroad (simple, creator-focused)
4. LemonSqueezy (merchant of record, developer-friendly)
5. No payments (free-only MVP)

**Decision Criteria Evaluation**:
- **Tax compliance**: ✓✓✓ Paddle handles EU VAT, US sales tax (merchant of record) *(solo developer can't handle tax in 50+ jurisdictions)*
- **Developer experience**: ✓✓ Paddle SDK for desktop apps; webhook-based license activation
- **SaaS/subscription support**: ✓✓✓ Native support for monthly/annual subscriptions + lifetime licenses *(PRD p.7-8: freemium + annual pricing)*
- **Fees**: ○ Paddle 5% + payment fees vs Stripe 2.9% + manual tax; acceptable for tax offloading
- **Creator-focused**: ○ Paddle used by many dev tools (not creator-specific like Gumroad); acceptable

**Chosen Option**: **Paddle (merchant of record) + Local license key validation**

**License Architecture**:
```rust
pub struct LicenseInfo {
    pub tier: LicenseTier,      // Free, ProMonthly, ProAnnual, Lifetime
    pub key: Option<String>,     // Encrypted license key
    pub expires_at: Option<SystemTime>,
    pub features: Vec<Feature>,  // ExportWatermarkFree, BatchProcessing, etc.
}

// License validation flow:
// 1. User enters license key → call Paddle API (HTTPS)
// 2. Paddle validates → returns license metadata (tier, expiry)
// 3. App encrypts + caches license locally (AES-256-GCM, machine-bound key)
// 4. Startup: load cached license, validate signature
// 5. Every 24h: ping Paddle to check revocation (if online)
// 6. Offline grace period: 7 days (then require online validation)
```

**Rationale**:
1. **Tax compliance**: Paddle is merchant of record; handles VAT/sales tax in 50+ jurisdictions *(solo developer constraint: can't handle tax)*
2. **SaaS support**: Native monthly/annual subscriptions + lifetime licenses match PRD pricing model *(PRD p.7-8)*
3. **Fraud prevention**: Paddle handles payment fraud, chargebacks; app only validates license keys *(NFR-13: license validation)*
4. **Offline grace period**: 7-day offline validation grace period balances piracy prevention with usability *(NFR-9: offline capability)*
5. **Desktop-appropriate**: Paddle supports desktop app licensing (vs Gumroad which is file-delivery-focused)

**Trade-offs Accepted**:
- **Higher fees**: Paddle 5%+ vs Stripe 2.9%; acceptable tradeoff for tax offloading + subscription management
- **Vendor lock-in**: Paddle-specific API; migration to Stripe would require rewrite; mitigated by encapsulating in `LicenseService` abstraction

**Alternatives Rejected**:
- **Stripe + manual tax**: Rejected - solo developer can't handle EU VAT, US sales tax across states *(legal risk, time sink)*
- **Gumroad**: Rejected - file-delivery-focused, not SaaS subscriptions; license validation is DIY *(PRD requires subscriptions)*
- **LemonSqueezy**: Rejected - newer player, less proven than Paddle; similar fees; Paddle has stronger desktop app ecosystem
- **No payments (free MVP)**: Rejected - PRD explicitly requires monetization path (freemium model) *(PRD p.7-8)*

**Requirements Addressed**: PRD pricing model (p.7-8), NFR-13 (license validation), NFR-14 (code signing), Distribution tactics (p.9)

---

## 4. COMPONENT DEEP DIVE (SOLID Principles & Design Patterns)

### 4.1 Component: Media Module (FFmpeg Wrapper)

**Responsibilities** (SRP):
- Probe video/audio container metadata (codecs, streams, duration)
- Extract audio streams to PCM for processing
- Remux enhanced audio with original video

**Design Patterns Applied**:
- **Command Pattern**: `FFmpegCommand` encapsulates FFmpeg CLI invocations as objects, allowing queuing and logging
- **Builder Pattern**: Fluent API for constructing FFmpeg commands (`FFmpegCommand::new().map_stream().audio_codec().execute()`)
- **Adapter Pattern**: `MediaProber` adapts FFmpeg's JSON output (`ffprobe -print_format json`) to Rust `MediaFile` struct

**SOLID Principles Verification**:
- **SRP**: Media module only handles FFmpeg interactions; doesn't know about ML models or UI ✓
- **OCP**: New FFmpeg operations (e.g., subtitle extraction) add new methods without modifying existing probe/extract/remux logic ✓
- **LSP**: N/A (no inheritance hierarchy)
- **ISP**: Separate traits: `IMediaProber`, `IAudioExtractor`, `IVideoRemuxer` (clients depend only on needed interface) ✓
- **DIP**: Processing pipeline depends on `IMediaProber` trait, not concrete `FFmpegProber` struct ✓

**API Contract**:
```rust
#[tauri::command]
async fn probe_media_file(path: String) -> Result<MediaFile, String>

// Request: { path: "/path/to/video.mp4" }
// Response: MediaFile {
//   id: "uuid",
//   format: "MP4",
//   duration_ms: 180000,
//   video_streams: [{ codec: "h264", width: 1920, height: 1080, fps: 30.0 }],
//   audio_streams: [{ codec: "aac", sample_rate: 48000, channels: 2 }]
// }
// Errors:
//   400: "Unsupported format" (file is not video)
//   404: "File not found" (path invalid)
//   500: "FFmpeg probe failed" (corrupt file)
```

**Class Structure**:
```
MediaModule
├── IMediaProber (Trait)
│   └── FFmpegProber (Impl)
├── IAudioExtractor (Trait)
│   └── FFmpegExtractor (Impl)
├── IVideoRemuxer (Trait)
│   └── FFmpegRemuxer (Impl)
└── FFmpegCommand (Builder)
```

**Dependencies** (Loose Coupling):
- `std::process::Command` (FFmpeg subprocess)
- `serde_json` (parse ffprobe JSON)
- No dependency on enhancement or DSP modules ✓

**Scaling Strategy**:
- Horizontal: N/A (desktop app, single user)
- Vertical: FFmpeg is CPU-bound; scales with core count (multi-threaded decoding)
- Caching: Probe results cached in-memory by file path + mtime

**Security Considerations**:
- **Input validation**: Sanitize file paths to prevent command injection (`ffmpeg -i "$(cat /etc/passwd)"`)
- **Resource limits**: Timeout FFmpeg calls (30s for probe, 10 min for extract) to prevent hang on corrupt files
- **Subprocess isolation**: FFmpeg runs in subprocess; crashes don't take down app

**Error Handling**:
- **FFmpeg failures**: Parse stderr for specific errors ("No such file", "Invalid data"), provide user-friendly messages
- **Timeout**: Kill FFmpeg process after timeout, return "Processing timed out" error
- **Graceful degradation**: If probe fails, allow manual codec selection (future v2 feature)

**Monitoring & Observability**:
- Metrics: FFmpeg call duration (histogram), failure rate by operation (probe/extract/remux)
- Logging: Log FFmpeg command + stderr on failure for support debugging
- Tracing: Span per FFmpeg operation with file path (PII-scrubbed in telemetry)

---

### 4.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    FRONTEND (React/TS)                       │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐   │
│  │  File Drop  │  │  Preview     │  │  Export Queue    │   │
│  │  Component  │  │  Controller  │  │  Manager         │   │
│  └─────────────┘  └──────────────┘  └──────────────────┘   │
│         │                │                    │              │
│         └────────────────┴────────────────────┘              │
│                          │                                   │
│                    Tauri IPC Layer                           │
└──────────────────────────┼───────────────────────────────────┘
                           │
┌──────────────────────────┼───────────────────────────────────┐
│                    BACKEND (Rust)                            │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              Core Processing Engine                  │    │
│  │  ┌──────────┐  ┌──────────┐  ┌───────────────┐     │    │
│  │  │ Media    │  │ Audio    │  │ Enhancement   │     │    │
│  │  │ Handler  │  │ Extractor│  │ Pipeline      │     │    │
│  │  └──────────┘  └──────────┘  └───────────────┘     │    │
│  └─────────────────────────────────────────────────────┘    │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              ML/DSP Processing Layer                 │    │
│  │  ┌──────────┐  ┌──────────┐  ┌───────────────┐     │    │
│  │  │ ONNX     │  │ DSP      │  │ Loudness      │     │    │
│  │  │ Runtime  │  │ Chain    │  │ Normalizer    │     │    │
│  │  └──────────┘  └──────────┘  └───────────────┘     │    │
│  └─────────────────────────────────────────────────────┘    │
│  ┌─────────────────────────────────────────────────────┐    │
│  │                  System Layer                        │    │
│  │  ┌──────────┐  ┌──────────┐  ┌───────────────┐     │    │
│  │  │ FFmpeg   │  │ File     │  │ License       │     │    │
│  │  │ Wrapper  │  │ System   │  │ Manager       │     │    │
│  │  └──────────┘  └──────────┘  └───────────────┘     │    │
│  └─────────────────────────────────────────────────────┘    │
└──────────────────────────────────────────────────────────────┘
```

### 1.2 Rust Backend Modules

#### **Module: `media`**
**Responsibility:** Video/audio container parsing, codec detection, stream extraction
**Key Components:**
- `MediaProber`: Uses FFmpeg to probe file metadata
- `StreamInfo`: Holds video/audio stream information
- `ContainerValidator`: Validates supported formats (MP4, MOV, MKV, WebM)

#### **Module: `audio_extraction`**
**Responsibility:** Extract audio streams to PCM format for processing
**Key Components:**
- `AudioExtractor`: FFmpeg-based audio extraction to WAV/PCM
- `AudioDecoder`: Handles various audio codecs (AAC, MP3, Opus, etc.)
- `StreamSynchronizer`: Maintains A/V sync metadata

#### **Module: `enhancement`**
**Responsibility:** ML model inference and audio enhancement pipeline
**Key Components:**
- `EnhancementEngine`: Orchestrates enhancement stages
- `ModelRunner`: ONNX Runtime integration for DeepFilterNet
- `ChunkProcessor`: Splits audio into overlapping chunks for processing
- `ArtifactDetector`: Monitors for over-processing artifacts

#### **Module: `dsp`**
**Responsibility:** Traditional DSP operations (EQ, compression, normalization)
**Key Components:**
- `EQFilter`: Frequency shaping (high-pass, presence boost)
- `Compressor`: Gentle dynamics control
- `DeEsser`: Sibilance reduction
- `LoudnessNormalizer`: ITU-R BS.1770 compliance for YouTube/streaming

#### **Module: `analysis`**
**Responsibility:** Audio quality assessment and speech detection
**Key Components:**
- `SpeechDetector`: VAD (Voice Activity Detection) using Silero VAD
- `QualityAnalyzer`: Estimates source quality (noise floor, clipping, bandwidth)
- `MusicDetector`: Distinguishes speech from music sections

#### **Module: `remux`**
**Responsibility:** Combine enhanced audio with original video
**Key Components:**
- `VideoRemuxer`: FFmpeg-based stream copy remuxing
- `SyncValidator`: Ensures A/V sync within 40ms tolerance
- `MetadataPreserver`: Keeps original video metadata

#### **Module: `export`**
**Responsibility:** Export queue management and preset handling
**Key Components:**
- `ExportQueue`: Multi-file batch processing
- `PresetManager`: Platform-specific presets (YouTube, LinkedIn, Instagram)
- `ProgressTracker`: Per-file processing progress

#### **Module: `license`**
**Responsibility:** License validation and feature gating
**Key Components:**
- `LicenseValidator`: Validates license keys
- `FeatureGate`: Controls free vs. pro features
- `UsageTracker`: Tracks export counts for free tier limits

#### **Module: `state`**
**Responsibility:** Application state management
**Key Components:**
- `AppState`: Global application state (Tokio Mutex-wrapped)
- `ProjectState`: Current processing project state
- `SettingsManager`: User preferences and config

### 1.3 React Frontend Components

#### **Component: `FileDropZone`**
**Responsibility:** Drag-and-drop file ingestion
**Props:**
- `onFilesDropped: (files: File[]) => void`
- `acceptedFormats: string[]`
- `disabled: boolean`

#### **Component: `MediaPreview`**
**Responsibility:** A/B comparison player
**State:**
- `currentVersion: 'original' | 'enhanced'`
- `playbackPosition: number`
- `isPlaying: boolean`
**Key Methods:**
- `toggleVersion()`: Switch between original/enhanced
- `syncPlayback()`: Maintain position when switching

#### **Component: `ProcessingQueue`**
**Responsibility:** Display batch processing status
**Props:**
- `queueItems: QueueItem[]`
- `onCancel: (id: string) => void`
- `onRetry: (id: string) => void`

#### **Component: `PresetSelector`**
**Responsibility:** Platform preset selection
**Props:**
- `presets: Preset[]`
- `selectedPreset: string`
- `onPresetChange: (preset: string) => void`

#### **Component: `QualityMeter`**
**Responsibility:** Source quality visualization
**Props:**
- `qualityScore: number (0-100)`
- `warnings: string[]`
- `recommendations: string[]`

#### **Component: `IntensitySlider`**
**Responsibility:** Dry/wet mix control
**Props:**
- `value: number (0-100)`
- `onChange: (value: number) => void`
- `preset: 'gentle' | 'standard' | 'aggressive'`

#### **Component: `ExportDialog`**
**Responsibility:** Export settings and confirmation
**State:**
- `outputPath: string`
- `selectedPreset: string`
- `exportOptions: ExportOptions`

---

## 2. DATA MODELS & TYPES

### 2.1 Rust Structs

```rust
// src/models/media.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaFile {
    pub id: String,
    pub path: PathBuf,
    pub format: ContainerFormat,
    pub duration_ms: u64,
    pub video_streams: Vec<VideoStreamInfo>,
    pub audio_streams: Vec<AudioStreamInfo>,
    pub metadata: MediaMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContainerFormat {
    MP4,
    MOV,
    MKV,
    WebM,
    AVI,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioStreamInfo {
    pub index: u32,
    pub codec: String,
    pub sample_rate: u32,
    pub channels: u8,
    pub bitrate: Option<u64>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoStreamInfo {
    pub index: u32,
    pub codec: String,
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub bitrate: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub date: Option<String>,
    pub creation_time: Option<String>,
}

// src/models/processing.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingJob {
    pub id: String,
    pub media_file: MediaFile,
    pub preset: EnhancementPreset,
    pub intensity: f32, // 0.0 to 1.0
    pub status: JobStatus,
    pub progress: f32, // 0.0 to 1.0
    pub error: Option<String>,
    pub created_at: SystemTime,
    pub started_at: Option<SystemTime>,
    pub completed_at: Option<SystemTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobStatus {
    Queued,
    Analyzing,
    Extracting,
    Enhancing,
    Normalizing,
    Remuxing,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancementPreset {
    pub name: String,
    pub denoise_strength: f32,
    pub dereverb_strength: f32,
    pub eq_profile: EQProfile,
    pub compression_ratio: f32,
    pub target_loudness_lufs: f32,
    pub true_peak_max_dbfs: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EQProfile {
    Natural,
    Bright,
    Warm,
    Podcast,
    YouTube,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioQualityMetrics {
    pub noise_floor_db: f32,
    pub has_clipping: bool,
    pub dynamic_range_db: f32,
    pub bandwidth_hz: u32,
    pub estimated_quality: QualityRating,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityRating {
    Excellent,
    Good,
    Fair,
    Poor,
    Critical, // Warn user it may not improve much
}

// src/models/export.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    pub output_path: PathBuf,
    pub platform: Platform,
    pub audio_codec: AudioCodec,
    pub audio_bitrate: u32,
    pub preserve_video_codec: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Platform {
    YouTube,
    LinkedIn,
    Instagram,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioCodec {
    AAC,
    Opus,
    MP3,
}

// src/models/license.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseInfo {
    pub tier: LicenseTier,
    pub key: Option<String>,
    pub expires_at: Option<SystemTime>,
    pub features: Vec<Feature>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LicenseTier {
    Free,
    ProMonthly,
    ProAnnual,
    Lifetime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Feature {
    PreviewUnlimited,
    ExportWatermarkFree,
    BatchProcessing,
    AdvancedPresets,
    PrioritySupport,
}

// src/models/settings.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub default_preset: String,
    pub default_output_directory: Option<PathBuf>,
    pub auto_open_export_folder: bool,
    pub processing_quality: ProcessingQuality,
    pub theme: Theme,
    pub analytics_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessingQuality {
    Fast,     // Lower latency for preview
    Standard, // Balanced
    Maximum,  // Highest quality, slower
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
    System,
}
```

### 2.2 TypeScript Interfaces

```typescript
// src/types/media.ts

export interface MediaFile {
  id: string;
  path: string;
  format: ContainerFormat;
  durationMs: number;
  videoStreams: VideoStreamInfo[];
  audioStreams: AudioStreamInfo[];
  metadata: MediaMetadata;
}

export enum ContainerFormat {
  MP4 = 'MP4',
  MOV = 'MOV',
  MKV = 'MKV',
  WebM = 'WebM',
  AVI = 'AVI',
}

export interface AudioStreamInfo {
  index: number;
  codec: string;
  sampleRate: number;
  channels: number;
  bitrate?: number;
  durationMs: number;
}

export interface VideoStreamInfo {
  index: number;
  codec: string;
  width: number;
  height: number;
  fps: number;
  bitrate?: number;
}

export interface MediaMetadata {
  title?: string;
  artist?: string;
  date?: string;
  creationTime?: string;
}

// src/types/processing.ts

export interface ProcessingJob {
  id: string;
  mediaFile: MediaFile;
  preset: EnhancementPreset;
  intensity: number; // 0-100
  status: JobStatus;
  progress: number; // 0-100
  error?: string;
  createdAt: number;
  startedAt?: number;
  completedAt?: number;
}

export enum JobStatus {
  Queued = 'Queued',
  Analyzing = 'Analyzing',
  Extracting = 'Extracting',
  Enhancing = 'Enhancing',
  Normalizing = 'Normalizing',
  Remuxing = 'Remuxing',
  Completed = 'Completed',
  Failed = 'Failed',
  Cancelled = 'Cancelled',
}

export interface EnhancementPreset {
  name: string;
  denoiseStrength: number;
  dereverbStrength: number;
  eqProfile: EQProfile;
  compressionRatio: number;
  targetLoudnessLufs: number;
  truePeakMaxDbfs: number;
}

export enum EQProfile {
  Natural = 'Natural',
  Bright = 'Bright',
  Warm = 'Warm',
  Podcast = 'Podcast',
  YouTube = 'YouTube',
}

export interface AudioQualityMetrics {
  noiseFloorDb: number;
  hasClipping: boolean;
  dynamicRangeDb: number;
  bandwidthHz: number;
  estimatedQuality: QualityRating;
  warnings: string[];
}

export enum QualityRating {
  Excellent = 'Excellent',
  Good = 'Good',
  Fair = 'Fair',
  Poor = 'Poor',
  Critical = 'Critical',
}

// src/types/preview.ts

export interface PreviewState {
  jobId: string;
  currentVersion: 'original' | 'enhanced';
  isPlaying: boolean;
  currentTime: number;
  duration: number;
  waveformData?: Float32Array;
}

// src/types/export.ts

export interface ExportConfig {
  outputPath: string;
  platform: Platform;
  audioCodec: AudioCodec;
  audioBitrate: number;
  preserveVideoCodec: boolean;
}

export enum Platform {
  YouTube = 'YouTube',
  LinkedIn = 'LinkedIn',
  Instagram = 'Instagram',
  Custom = 'Custom',
}

export enum AudioCodec {
  AAC = 'AAC',
  Opus = 'Opus',
  MP3 = 'MP3',
}

// src/types/license.ts

export interface LicenseInfo {
  tier: LicenseTier;
  key?: string;
  expiresAt?: number;
  features: Feature[];
}

export enum LicenseTier {
  Free = 'Free',
  ProMonthly = 'ProMonthly',
  ProAnnual = 'ProAnnual',
  Lifetime = 'Lifetime',
}

export enum Feature {
  PreviewUnlimited = 'PreviewUnlimited',
  ExportWatermarkFree = 'ExportWatermarkFree',
  BatchProcessing = 'BatchProcessing',
  AdvancedPresets = 'AdvancedPresets',
  PrioritySupport = 'PrioritySupport',
}

// src/types/settings.ts

export interface UserSettings {
  defaultPreset: string;
  defaultOutputDirectory?: string;
  autoOpenExportFolder: boolean;
  processingQuality: ProcessingQuality;
  theme: Theme;
  analyticsEnabled: boolean;
}

export enum ProcessingQuality {
  Fast = 'Fast',
  Standard = 'Standard',
  Maximum = 'Maximum',
}

export enum Theme {
  Light = 'Light',
  Dark = 'Dark',
  System = 'System',
}
```

---

## 3. API CONTRACTS (Tauri Commands)

### 3.1 Media Inspection Commands

```rust
// Command: probe_media_file
#[tauri::command]
async fn probe_media_file(
    path: String,
) -> Result<MediaFile, String>

// Usage from React:
const mediaFile = await invoke<MediaFile>('probe_media_file', { 
  path: '/path/to/video.mp4' 
});

// Error Handling:
- Returns Err("Unsupported format") for non-video files
- Returns Err("No audio stream found") if video has no audio
- Returns Err("File not found") if path is invalid
```

### 3.2 Processing Commands

```rust
// Command: create_processing_job
#[tauri::command]
async fn create_processing_job(
    state: State<'_, AppState>,
    media_file: MediaFile,
    preset_name: String,
    intensity: f32,
) -> Result<ProcessingJob, String>

// Usage:
const job = await invoke<ProcessingJob>('create_processing_job', {
  mediaFile,
  presetName: 'YouTube',
  intensity: 0.7,
});

// Errors:
- "Invalid preset name"
- "License tier does not support this feature"
- "Processing queue is full"

// Command: start_processing
#[tauri::command]
async fn start_processing(
    state: State<'_, AppState>,
    job_id: String,
) -> Result<(), String>

// Usage:
await invoke('start_processing', { jobId: job.id });

// Emits progress events via Tauri event system:
// - 'processing:progress' { jobId, progress: 0.0-1.0, stage }
// - 'processing:completed' { jobId, outputPath }
// - 'processing:failed' { jobId, error }

// Command: cancel_processing
#[tauri::command]
async fn cancel_processing(
    state: State<'_, AppState>,
    job_id: String,
) -> Result<(), String>

// Command: get_job_status
#[tauri::command]
async fn get_job_status(
    state: State<'_, AppState>,
    job_id: String,
) -> Result<ProcessingJob, String>
```

### 3.3 Analysis Commands

```rust
// Command: analyze_audio_quality
#[tauri::command]
async fn analyze_audio_quality(
    path: String,
) -> Result<AudioQualityMetrics, String>

// Usage:
const quality = await invoke<AudioQualityMetrics>('analyze_audio_quality', {
  path: mediaFile.path,
});

if (quality.estimatedQuality === 'Critical') {
  showWarning('Source quality is very poor. Enhancement may be limited.');
}

// Command: detect_speech_regions
#[tauri::command]
async fn detect_speech_regions(
    path: String,
) -> Result<Vec<TimeRange>, String>

// Returns array of { startMs: number, endMs: number }
```

### 3.4 Preview Commands

```rust
// Command: generate_preview
#[tauri::command]
async fn generate_preview(
    state: State<'_, AppState>,
    job_id: String,
    start_ms: u64,
    duration_ms: u64,
) -> Result<PreviewPaths, String>

// Returns: { originalPath: string, enhancedPath: string }
// Generates 5-10 second preview clips for A/B comparison

// Usage:
const { originalPath, enhancedPath } = await invoke('generate_preview', {
  jobId: job.id,
  startMs: 30000, // 30 seconds in
  durationMs: 5000, // 5 second preview
});

// Command: get_waveform_data
#[tauri::command]
async fn get_waveform_data(
    path: String,
    width_pixels: u32,
) -> Result<Vec<f32>, String>

// Returns normalized waveform for visualization
```

### 3.5 Export Commands

```rust
// Command: export_enhanced_video
#[tauri::command]
async fn export_enhanced_video(
    state: State<'_, AppState>,
    job_id: String,
    config: ExportConfig,
) -> Result<String, String>

// Returns output path on success
// Emits events:
// - 'export:progress' { jobId, progress: 0.0-1.0 }
// - 'export:completed' { jobId, outputPath }
// - 'export:failed' { jobId, error }

// Usage:
const outputPath = await invoke<string>('export_enhanced_video', {
  jobId: job.id,
  config: {
    outputPath: '/path/to/output.mp4',
    platform: 'YouTube',
    audioCodec: 'AAC',
    audioBitrate: 192000,
    preserveVideoCodec: true,
  },
});

// Errors:
- "Insufficient disk space"
- "Output path not writable"
- "License does not allow export" (for free tier limits)
```

### 3.6 Preset Commands

```rust
// Command: get_presets
#[tauri::command]
async fn get_presets() -> Result<Vec<EnhancementPreset>, String>

// Command: save_custom_preset
#[tauri::command]
async fn save_custom_preset(
    preset: EnhancementPreset,
) -> Result<(), String>

// Command: delete_preset
#[tauri::command]
async fn delete_preset(
    name: String,
) -> Result<(), String>
```

### 3.7 License Commands

```rust
// Command: get_license_info
#[tauri::command]
async fn get_license_info(
    state: State<'_, AppState>,
) -> Result<LicenseInfo, String>

// Command: activate_license
#[tauri::command]
async fn activate_license(
    state: State<'_, AppState>,
    license_key: String,
) -> Result<LicenseInfo, String>

// Command: check_feature_access
#[tauri::command]
async fn check_feature_access(
    state: State<'_, AppState>,
    feature: Feature,
) -> Result<bool, String>
```

### 3.8 Settings Commands

```rust
// Command: get_settings
#[tauri::command]
async fn get_settings() -> Result<UserSettings, String>

// Command: update_settings
#[tauri::command]
async fn update_settings(
    settings: UserSettings,
) -> Result<(), String>

// Command: open_settings_directory
#[tauri::command]
async fn open_settings_directory() -> Result<(), String>
```

---

## 4. STATE MANAGEMENT

### 4.1 Frontend State (React)

#### **Global State Context**

```typescript
// src/context/AppContext.tsx

interface AppContextValue {
  // License & User
  license: LicenseInfo | null;
  settings: UserSettings;
  updateSettings: (settings: Partial<UserSettings>) => Promise<void>;

  // Media & Jobs
  currentMediaFile: MediaFile | null;
  processingJobs: Map<string, ProcessingJob>;
  activeJobId: string | null;
  
  // Actions
  loadMediaFile: (path: string) => Promise<void>;
  createJob: (preset: string, intensity: number) => Promise<string>;
  startJob: (jobId: string) => Promise<void>;
  cancelJob: (jobId: string) => Promise<void>;
  exportJob: (jobId: string, config: ExportConfig) => Promise<string>;
}

// Usage in components:
const { createJob, activeJobId } = useAppContext();
```

#### **Component State Patterns**

```typescript
// FileDropZone Component
const [isDragActive, setIsDragActive] = useState(false);
const [isProcessing, setIsProcessing] = useState(false);

// MediaPreview Component
const [currentVersion, setCurrentVersion] = useState<'original' | 'enhanced'>('original');
const [playbackState, setPlaybackState] = useState({
  isPlaying: false,
  currentTime: 0,
  duration: 0,
});
const [waveformCache, setWaveformCache] = useState<Map<string, Float32Array>>();

// ProcessingQueue Component
const [queueFilter, setQueueFilter] = useState<'all' | 'active' | 'completed'>('all');
const [sortOrder, setSortOrder] = useState<'newest' | 'oldest'>('newest');

// IntensitySlider Component
const [localValue, setLocalValue] = useState(intensity); // Debounced updates
const debouncedUpdate = useMemo(
  () => debounce((value) => onIntensityChange(value), 300),
  []
);
```

### 4.2 Backend State (Rust)

#### **Application State Structure**

```rust
// src/state/app_state.rs

pub struct AppState {
    pub license: Arc<Mutex<LicenseInfo>>,
    pub jobs: Arc<Mutex<HashMap<String, ProcessingJob>>>,
    pub settings: Arc<Mutex<UserSettings>>,
    pub processing_pool: Arc<ProcessingPool>,
    pub temp_dir: PathBuf,
}

impl AppState {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            license: Arc::new(Mutex::new(LicenseInfo::load()?)),
            jobs: Arc::new(Mutex::new(HashMap::new())),
            settings: Arc::new(Mutex::new(UserSettings::load()?)),
            processing_pool: Arc::new(ProcessingPool::new(2)), // Max 2 concurrent jobs
            temp_dir: Self::create_temp_dir()?,
        })
    }
}

// src/state/processing_pool.rs

pub struct ProcessingPool {
    max_concurrent: usize,
    active_jobs: Arc<Mutex<HashSet<String>>>,
    workers: Arc<Mutex<HashMap<String, JoinHandle<()>>>>,
}

impl ProcessingPool {
    pub async fn submit_job(
        &self,
        job_id: String,
        job: ProcessingJob,
        app_handle: tauri::AppHandle,
    ) -> Result<(), String> {
        // Check capacity
        let active = self.active_jobs.lock().await;
        if active.len() >= self.max_concurrent {
            return Err("Processing queue is full".into());
        }
        
        // Spawn processing task
        let handle = tokio::spawn(async move {
            process_job(job_id.clone(), job, app_handle).await;
        });
        
        self.workers.lock().await.insert(job_id.clone(), handle);
        Ok(())
    }
}
```

### 4.3 Error Recovery & Checkpointing

#### **Job Checkpoint System**

```rust
// src/state/checkpoint.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobCheckpoint {
    pub job_id: String,
    pub stage: ProcessingStage,
    pub completed_stages: Vec<ProcessingStage>,
    pub temp_files: HashMap<ProcessingStage, PathBuf>,
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Eq)]
pub enum ProcessingStage {
    Queued,
    AudioExtracted,      // Checkpoint 1: original_audio.wav exists
    QualityAnalyzed,     // Checkpoint 2: quality metrics saved
    Enhanced,            // Checkpoint 3: enhanced_audio.wav exists
    DSPProcessed,        // Checkpoint 4: enhanced_dsp.wav exists
    Normalized,          // Checkpoint 5: enhanced_final.wav exists
    PreviewGenerated,    // Checkpoint 6: preview files exist
    Remuxed,             // Checkpoint 7: output video exists
    Completed,
}

impl JobCheckpoint {
    pub fn save(&self) -> Result<(), String> {
        let checkpoint_path = self.get_checkpoint_path()?;
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize checkpoint: {}", e))?;
        std::fs::write(checkpoint_path, json)
            .map_err(|e| format!("Failed to save checkpoint: {}", e))
    }
    
    pub fn load(job_id: &str) -> Result<Option<Self>, String> {
        let checkpoint_path = Self::checkpoint_path_for_job(job_id)?;
        if !checkpoint_path.exists() {
            return Ok(None);
        }
        
        let json = std::fs::read_to_string(&checkpoint_path)
            .map_err(|e| format!("Failed to read checkpoint: {}", e))?;
        let checkpoint: Self = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse checkpoint: {}", e))?;
        
        Ok(Some(checkpoint))
    }
    
    pub fn can_resume(&self) -> bool {
        // Check if temp files still exist
        for (stage, path) in &self.temp_files {
            if !path.exists() {
                return false;
            }
        }
        true
    }
    
    fn get_checkpoint_path(&self) -> Result<PathBuf, String> {
        Self::checkpoint_path_for_job(&self.job_id)
    }
    
    fn checkpoint_path_for_job(job_id: &str) -> Result<PathBuf, String> {
        let checkpoint_dir = dirs::data_dir()
            .ok_or("Failed to find data directory")?
            .join("audio-cleaner")
            .join("checkpoints");
        std::fs::create_dir_all(&checkpoint_dir)
            .map_err(|e| format!("Failed to create checkpoint dir: {}", e))?;
        Ok(checkpoint_dir.join(format!("{}.json", job_id)))
    }
}
```

#### **Resume-from-Failure Logic**

```rust
// src/processing/pipeline.rs

pub async fn process_job_with_resume(
    job_id: String,
    job: ProcessingJob,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    // 1. Check for existing checkpoint
    let mut checkpoint = JobCheckpoint::load(&job_id)?
        .unwrap_or_else(|| JobCheckpoint {
            job_id: job_id.clone(),
            stage: ProcessingStage::Queued,
            completed_stages: vec![],
            temp_files: HashMap::new(),
            timestamp: SystemTime::now(),
        });
    
    // 2. Verify checkpoint is still valid
    if !checkpoint.can_resume() {
        // Temp files missing, start from scratch
        checkpoint.stage = ProcessingStage::Queued;
        checkpoint.completed_stages.clear();
        checkpoint.temp_files.clear();
    }
    
    // 3. Resume from last successful stage
    match checkpoint.stage {
        ProcessingStage::Queued => {
            // Start from beginning
            process_from_start(&mut checkpoint, &job, &app_handle).await?;
        }
        ProcessingStage::AudioExtracted => {
            // Skip extraction, start from analysis
            process_from_analysis(&mut checkpoint, &job, &app_handle).await?;
        }
        ProcessingStage::Enhanced => {
            // Skip enhancement, start from DSP
            process_from_dsp(&mut checkpoint, &job, &app_handle).await?;
        }
        ProcessingStage::Completed => {
            // Already done, nothing to do
            return Ok(());
        }
        _ => {
            // For other stages, resume from appropriate point
            resume_from_stage(&mut checkpoint, &job, &app_handle).await?;
        }
    }
    
    // 4. Clean up checkpoint on success
    let checkpoint_path = JobCheckpoint::checkpoint_path_for_job(&job_id)?;
    std::fs::remove_file(checkpoint_path).ok();
    
    Ok(())
}

async fn process_from_start(
    checkpoint: &mut JobCheckpoint,
    job: &ProcessingJob,
    app_handle: &tauri::AppHandle,
) -> Result<(), String> {
    // Stage 1: Extract audio
    let audio_path = extract_audio(&job.media_file).await?;
    checkpoint.temp_files.insert(ProcessingStage::AudioExtracted, audio_path.clone());
    checkpoint.stage = ProcessingStage::AudioExtracted;
    checkpoint.completed_stages.push(ProcessingStage::AudioExtracted);
    checkpoint.save()?;
    
    // Continue with analysis
    process_from_analysis(checkpoint, job, app_handle).await
}

async fn process_from_analysis(
    checkpoint: &mut JobCheckpoint,
    job: &ProcessingJob,
    app_handle: &tauri::AppHandle,
) -> Result<(), String> {
    // Get audio file from checkpoint
    let audio_path = checkpoint.temp_files.get(&ProcessingStage::AudioExtracted)
        .ok_or("Audio file not found in checkpoint")?;
    
    // Stage 2: Analyze quality
    let quality = analyze_audio_quality(audio_path).await?;
    checkpoint.stage = ProcessingStage::QualityAnalyzed;
    checkpoint.completed_stages.push(ProcessingStage::QualityAnalyzed);
    checkpoint.save()?;
    
    // Stage 3: Enhancement
    let enhanced_path = enhance_audio(audio_path, &job.preset, job.intensity).await?;
    checkpoint.temp_files.insert(ProcessingStage::Enhanced, enhanced_path.clone());
    checkpoint.stage = ProcessingStage::Enhanced;
    checkpoint.completed_stages.push(ProcessingStage::Enhanced);
    checkpoint.save()?;
    
    // Continue with DSP
    process_from_dsp(checkpoint, job, app_handle).await
}

async fn process_from_dsp(
    checkpoint: &mut JobCheckpoint,
    job: &ProcessingJob,
    app_handle: &tauri::AppHandle,
) -> Result<(), String> {
    // Get enhanced audio from checkpoint
    let enhanced_path = checkpoint.temp_files.get(&ProcessingStage::Enhanced)
        .ok_or("Enhanced audio file not found in checkpoint")?;
    
    // Stage 4: DSP processing
    let dsp_path = apply_dsp_chain(enhanced_path, &job.preset).await?;
    checkpoint.temp_files.insert(ProcessingStage::DSPProcessed, dsp_path.clone());
    checkpoint.stage = ProcessingStage::DSPProcessed;
    checkpoint.completed_stages.push(ProcessingStage::DSPProcessed);
    checkpoint.save()?;
    
    // Stage 5: Normalization
    let normalized_path = normalize_loudness(&dsp_path, &job.preset).await?;
    checkpoint.temp_files.insert(ProcessingStage::Normalized, normalized_path.clone());
    checkpoint.stage = ProcessingStage::Normalized;
    checkpoint.completed_stages.push(ProcessingStage::Normalized);
    checkpoint.save()?;
    
    // Stage 6: Remuxing
    let output_path = remux_video(&job.media_file, &normalized_path).await?;
    checkpoint.stage = ProcessingStage::Completed;
    checkpoint.completed_stages.push(ProcessingStage::Completed);
    checkpoint.save()?;
    
    // Emit completion event
    app_handle.emit_all("processing:completed", CompletionEvent {
        job_id: checkpoint.job_id.clone(),
        output_path: output_path.to_string_lossy().to_string(),
    })?;
    
    Ok(())
}
```

#### **Queue Persistence Across App Restarts**

```rust
// src/state/queue_persistence.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueSnapshot {
    pub jobs: Vec<ProcessingJob>,
    pub timestamp: SystemTime,
}

impl QueueSnapshot {
    pub fn save(jobs: &[ProcessingJob]) -> Result<(), String> {
        let snapshot_path = Self::get_snapshot_path()?;
        let snapshot = QueueSnapshot {
            jobs: jobs.to_vec(),
            timestamp: SystemTime::now(),
        };
        
        let json = serde_json::to_string_pretty(&snapshot)
            .map_err(|e| format!("Failed to serialize queue: {}", e))?;
        std::fs::write(snapshot_path, json)
            .map_err(|e| format!("Failed to save queue: {}", e))
    }
    
    pub fn load() -> Result<Option<Self>, String> {
        let snapshot_path = Self::get_snapshot_path()?;
        if !snapshot_path.exists() {
            return Ok(None);
        }
        
        // Check if snapshot is stale (>24 hours old)
        if let Ok(metadata) = std::fs::metadata(&snapshot_path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(elapsed) = modified.elapsed() {
                    if elapsed.as_secs() > 24 * 3600 {
                        // Stale snapshot, discard
                        std::fs::remove_file(snapshot_path).ok();
                        return Ok(None);
                    }
                }
            }
        }
        
        let json = std::fs::read_to_string(&snapshot_path)
            .map_err(|e| format!("Failed to read queue: {}", e))?;
        let snapshot: Self = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse queue: {}", e))?;
        
        Ok(Some(snapshot))
    }
    
    fn get_snapshot_path() -> Result<PathBuf, String> {
        let data_dir = dirs::data_dir()
            .ok_or("Failed to find data directory")?
            .join("audio-cleaner");
        std::fs::create_dir_all(&data_dir)
            .map_err(|e| format!("Failed to create data dir: {}", e))?;
        Ok(data_dir.join("queue_snapshot.json"))
    }
}

// On app startup
pub async fn restore_queue_on_startup(state: &AppState) -> Result<(), String> {
    if let Some(snapshot) = QueueSnapshot::load()? {
        let mut jobs = state.jobs.lock().await;
        
        for job in snapshot.jobs {
            // Check if job can be resumed
            if let Some(checkpoint) = JobCheckpoint::load(&job.id)? {
                if checkpoint.can_resume() {
                    // Resume job
                    jobs.insert(job.id.clone(), job.clone());
                    
                    // Restart processing in background
                    tokio::spawn(async move {
                        if let Err(e) = process_job_with_resume(job.id.clone(), job, state.app_handle.clone()).await {
                            eprintln!("Failed to resume job {}: {}", job.id, e);
                        }
                    });
                }
            }
        }
    }
    
    Ok(())
}

// On app shutdown
pub async fn save_queue_on_shutdown(state: &AppState) -> Result<(), String> {
    let jobs = state.jobs.lock().await;
    let pending_jobs: Vec<_> = jobs.values()
        .filter(|job| matches!(job.status, JobStatus::Queued | JobStatus::Analyzing | JobStatus::Enhancing))
        .cloned()
        .collect();
    
    QueueSnapshot::save(&pending_jobs)
}
```

#### **Retry Strategies by Failure Type**

```rust
// src/processing/retry.rs

pub enum RetryStrategy {
    Immediate,           // Retry right away (transient errors)
    AfterDelay(Duration), // Wait before retry (rate limits, network)
    UserIntervention,    // Requires user action (permissions, disk space)
    NoRetry,             // Fatal error, can't recover
}

pub fn determine_retry_strategy(error: &str) -> RetryStrategy {
    if error.contains("disk space") || error.contains("No space left") {
        RetryStrategy::UserIntervention
    } else if error.contains("Permission denied") {
        RetryStrategy::UserIntervention
    } else if error.contains("Network") || error.contains("timeout") {
        RetryStrategy::AfterDelay(Duration::from_secs(30))
    } else if error.contains("FFmpeg") && error.contains("temporary") {
        RetryStrategy::Immediate
    } else if error.contains("Model") || error.contains("ONNX") {
        RetryStrategy::NoRetry // Model loading issues are fatal
    } else {
        RetryStrategy::AfterDelay(Duration::from_secs(5))
    }
}

pub async fn retry_with_strategy<F, T>(
    operation: F,
    max_attempts: u32,
) -> Result<T, String>
where
    F: Fn() -> Pin<Box<dyn Future<Output = Result<T, String>> + Send>>,
{
    let mut attempts = 0;
    let mut last_error = String::new();
    
    while attempts < max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = e.clone();
                attempts += 1;
                
                match determine_retry_strategy(&e) {
                    RetryStrategy::Immediate => {
                        // Retry immediately
                        continue;
                    }
                    RetryStrategy::AfterDelay(delay) => {
                        tokio::time::sleep(delay).await;
                        continue;
                    }
                    RetryStrategy::UserIntervention | RetryStrategy::NoRetry => {
                        // Don't retry, return error
                        return Err(e);
                    }
                }
            }
        }
    }
    
    Err(format!("Max retries ({}) exceeded: {}", max_attempts, last_error))
}
```

---

### 4.4 State Synchronization

#### **Frontend → Backend Updates**

```typescript
// User changes intensity slider
const handleIntensityChange = async (value: number) => {
  // Optimistic update
  setLocalIntensity(value);
  
  // Backend update
  try {
    await invoke('update_job_intensity', { 
      jobId: activeJobId, 
      intensity: value / 100 
    });
  } catch (error) {
    // Rollback on error
    setLocalIntensity(previousValue);
    showError(error);
  }
};
```

#### **Backend → Frontend Events**

```rust
// Emit progress updates
app_handle.emit_all("processing:progress", ProgressEvent {
    job_id: job.id.clone(),
    progress: current_progress,
    stage: current_stage,
    eta_seconds: estimated_remaining,
})?;

// Emit completion
app_handle.emit_all("processing:completed", CompletionEvent {
    job_id: job.id.clone(),
    output_path: output.to_string_lossy().to_string(),
})?;
```

```typescript
// React listener setup
useEffect(() => {
  const unlisten = listen<ProgressEvent>('processing:progress', (event) => {
    updateJobProgress(event.payload.jobId, event.payload.progress);
  });
  
  return () => {
    unlisten.then(fn => fn());
  };
}, []);
```

### 4.4 State Triggers

| **Trigger** | **Action** | **State Update** |
|------------|-----------|------------------|
| User drops file | `probe_media_file` called | `currentMediaFile` updated, quality analysis triggered |
| User clicks "Enhance" | `create_processing_job` + `start_processing` | Job added to `processingJobs`, backend emits progress events |
| Processing completes | Backend emits `processing:completed` | Job status → `Completed`, preview paths updated |
| User toggles A/B | Local state only | `currentVersion` switches, audio element source swapped |
| User changes preset | `update_job_preset` called | Job preset updated, preview regenerated |
| User exports | `export_enhanced_video` called | Export progress tracked, output path saved on completion |

---

## 5. STORAGE & PERSISTENCE

### 5.1 Directory Structure

```
~/.audio-cleaner/                    (macOS: ~/Library/Application Support/audio-cleaner)
├── config/
│   ├── settings.json               # User settings
│   ├── presets.json                # Custom presets
│   └── license.json                # License info (encrypted)
├── cache/
│   ├── models/                     # Downloaded ML models
│   │   ├── deepfilternet_3.onnx
│   │   └── silero_vad.onnx
│   ├── waveforms/                  # Cached waveform visualizations
│   └── thumbnails/                 # Video thumbnails
├── temp/                           # Temporary processing files
│   ├── <job_id>/
│   │   ├── original_audio.wav
│   │   ├── enhanced_audio.wav
│   │   ├── preview_original.wav
│   │   └── preview_enhanced.wav
└── logs/
    ├── app.log
    └── crashes/
```

### 5.2 Settings Persistence

```rust
// src/persistence/settings.rs

impl UserSettings {
    pub fn load() -> Result<Self, String> {
        let path = Self::settings_path()?;
        if path.exists() {
            let content = fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read settings: {}", e))?;
            serde_json::from_str(&content)
                .map_err(|e| format!("Invalid settings format: {}", e))
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::settings_path()?;
        fs::create_dir_all(path.parent().unwrap())
            .map_err(|e| format!("Failed to create config dir: {}", e))?;
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;
        fs::write(&path, content)
            .map_err(|e| format!("Failed to write settings: {}", e))
    }

    fn settings_path() -> Result<PathBuf, String> {
        let config_dir = dirs::config_dir()
            .ok_or("Failed to find config directory")?;
        Ok(config_dir.join("audio-cleaner").join("config").join("settings.json"))
    }
}
```

**Default Settings:**
```json
{
  "defaultPreset": "YouTube",
  "defaultOutputDirectory": null,
  "autoOpenExportFolder": true,
  "processingQuality": "Standard",
  "theme": "System",
  "analyticsEnabled": true
}
```

### 5.3 License Validation Cache

```rust
// src/persistence/license.rs

pub struct LicenseCache {
    path: PathBuf,
}

impl LicenseCache {
    pub fn load(&self) -> Result<LicenseInfo, String> {
        let encrypted = fs::read(&self.path)
            .map_err(|_| "No license found")?;
        let decrypted = self.decrypt(&encrypted)?;
        serde_json::from_slice(&decrypted)
            .map_err(|e| format!("Invalid license: {}", e))
    }

    pub fn save(&self, license: &LicenseInfo) -> Result<(), String> {
        let serialized = serde_json::to_vec(license)
            .map_err(|e| format!("Serialization failed: {}", e))?;
        let encrypted = self.encrypt(&serialized)?;
        fs::write(&self.path, encrypted)
            .map_err(|e| format!("Failed to save license: {}", e))
    }

    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        // Use machine-specific key derivation + AES-256-GCM
        // Implementation details omitted
    }

    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        // Corresponding decryption
    }
}
```

**Validation Strategy:**
- On startup: Load cached license, validate signature
- Every 24h: Ping license server to check revocation (if online)
- On export: Revalidate license, check usage limits for free tier

### 5.4 Model Caching

```rust
// src/ml/model_cache.rs

pub struct ModelCache {
    cache_dir: PathBuf,
}

impl ModelCache {
    pub async fn get_model(&self, model_name: &str) -> Result<PathBuf, String> {
        let model_path = self.cache_dir.join(format!("{}.onnx", model_name));
        
        if model_path.exists() {
            // Verify checksum
            if self.verify_checksum(&model_path, model_name)? {
                return Ok(model_path);
            }
        }
        
        // Download model
        self.download_model(model_name).await?;
        Ok(model_path)
    }

    async fn download_model(&self, model_name: &str) -> Result<(), String> {
        // Download from CDN/GitHub releases
        // Show progress to user
        // Verify checksum after download
    }
}
```

**Bundled Models (shipped with installer):**
- `deepfilternet_3_quant.onnx` (~8MB, quantized for faster inference)
- `silero_vad_v4.onnx` (~1MB)

**Downloadable Models (premium features):**
- `deepfilternet_3_full.onnx` (~20MB, higher quality)
- Future: commercial dereverb models

### 5.5 Temporary File Management

```rust
// src/processing/temp_manager.rs

pub struct TempManager {
    temp_root: PathBuf,
}

impl TempManager {
    pub fn create_job_temp_dir(&self, job_id: &str) -> Result<PathBuf, String> {
        let job_dir = self.temp_root.join(job_id);
        fs::create_dir_all(&job_dir)
            .map_err(|e| format!("Failed to create temp dir: {}", e))?;
        Ok(job_dir)
    }

    pub fn cleanup_job(&self, job_id: &str) -> Result<(), String> {
        let job_dir = self.temp_root.join(job_id);
        if job_dir.exists() {
            fs::remove_dir_all(&job_dir)
                .map_err(|e| format!("Failed to cleanup temp dir: {}", e))?;
        }
        Ok(())
    }

    pub fn cleanup_old_temps(&self, max_age_hours: u64) -> Result<(), String> {
        // Delete temp directories older than max_age_hours
        let cutoff = SystemTime::now() - Duration::from_secs(max_age_hours * 3600);
        
        for entry in fs::read_dir(&self.temp_root)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            if let Ok(modified) = metadata.modified() {
                if modified < cutoff {
                    fs::remove_dir_all(entry.path())?;
                }
            }
        }
        Ok(())
    }
}

// Cleanup strategy:
// - On job completion: Delete temp files immediately
// - On app startup: Delete temp dirs older than 24 hours
// - On disk space warning: Aggressively cleanup all temps
```

---

## 6. AUDIO PROCESSING PIPELINE

### 6.1 End-to-End Flow

```
┌─────────────────────────────────────────────────────────────────┐
│ 1. FILE INGESTION                                               │
│    - User drops video file                                      │
│    - FFmpeg probe: codecs, streams, duration                    │
│    - Validate: Has audio? Supported format?                     │
└────────────────────┬────────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────────────┐
│ 2. AUDIO EXTRACTION                                             │
│    - FFmpeg: Extract audio stream → 48kHz/16-bit/mono PCM      │
│    - Save to temp: <job_id>/original_audio.wav                 │
└────────────────────┬────────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────────────┐
│ 3. QUALITY ANALYSIS                                             │
│    - Noise floor estimation                                     │
│    - Clipping detection                                         │
│    - Bandwidth analysis                                         │
│    - VAD: Detect speech vs silence regions                      │
│    - Output: QualityMetrics + TimeRanges                       │
└────────────────────┬────────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────────────┐
│ 4. CHUNKED ENHANCEMENT                                          │
│    For each 5-second chunk (with 0.5s overlap):                │
│      a. High-pass filter (80Hz)                                │
│      b. DeepFilterNet inference (denoise + dereverb)           │
│      c. Overlap-add with previous chunk                        │
│    - Save to temp: <job_id>/enhanced_raw.wav                   │
└────────────────────┬────────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────────────┐
│ 5. DSP POST-PROCESSING                                          │
│    - Gentle EQ (presence boost ~4kHz, mud reduction ~300Hz)    │
│    - Soft compression (2:1 ratio, slow attack/release)         │
│    - De-esser (reduce harsh sibilance 6-8kHz)                  │
│    - Output: <job_id>/enhanced_dsp.wav                         │
└────────────────────┬────────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────────────┐
│ 6. LOUDNESS NORMALIZATION                                       │
│    - Measure integrated loudness (LUFS)                         │
│    - Target: -16 LUFS for YouTube, -14 for Instagram           │
│    - Apply gain, limit true peak to -1.0 dBFS                  │
│    - Save: <job_id>/enhanced_audio.wav                         │
└────────────────────┬────────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────────────┐
│ 7. PREVIEW GENERATION                                           │
│    - Extract 5s preview from middle (or user-selected region)  │
│    - Generate original + enhanced previews                      │
│    - Create waveform visualization data                         │
│    - Frontend: Load both into audio elements for A/B           │
└────────────────────┬────────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────────────┐
│ 8. USER REVIEW & ADJUSTMENT                                     │
│    - User toggles between original/enhanced                     │
│    - Adjust intensity slider (0-100%)                           │
│    - If adjusted: Re-run step 5-6 with new intensity           │
└────────────────────┬────────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────────────┐
│ 9. VIDEO REMUX                                                  │
│    - FFmpeg: Copy original video stream (no re-encode)          │
│    - Encode enhanced audio: AAC 192kbps (or user preset)       │
│    - Remux into container: output.mp4                           │
│    - Validate A/V sync (<40ms drift)                            │
└────────────────────┬────────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────────────┐
│ 10. EXPORT & CLEANUP                                            │
│     - Save to user-selected output path                         │
│     - Delete temp files for this job                            │
│     - Update job status: Completed                              │
│     - Open output folder (if setting enabled)                   │
└─────────────────────────────────────────────────────────────────┘
```

### 6.2 Algorithmic Details

#### **Stage 1-2: Extraction Strategy**

```rust
// src/processing/extractor.rs

pub async fn extract_audio(media: &MediaFile, output: &Path) -> Result<(), String> {
    // Select best audio stream (prefer highest bitrate if multiple)
    let audio_stream = media.audio_streams.iter()
        .max_by_key(|s| s.bitrate.unwrap_or(0))
        .ok_or("No audio stream")?;
    
    // FFmpeg command
    let status = Command::new("ffmpeg")
        .args(&[
            "-i", media.path.to_str().unwrap(),
            "-map", &format!("0:{}", audio_stream.index),
            "-ar", "48000",        // 48kHz sample rate
            "-ac", "1",            // Mono (sum stereo to mono for speech)
            "-sample_fmt", "s16",  // 16-bit PCM
            "-y",                  // Overwrite
            output.to_str().unwrap(),
        ])
        .status()
        .await
        .map_err(|e| format!("FFmpeg execution failed: {}", e))?;
    
    if !status.success() {
        return Err("Audio extraction failed".into());
    }
    
    Ok(())
}
```

**Rationale:**
- 48kHz: Standard for video production, high enough for full bandwidth
- Mono: Most creator speech is center-panned; mono simplifies processing
- 16-bit: Sufficient dynamic range, smaller file size than 24/32-bit

#### **Stage 3: Quality Analysis**

```rust
// src/analysis/quality.rs

pub fn analyze_quality(audio_path: &Path) -> Result<AudioQualityMetrics, String> {
    let audio = load_audio(audio_path)?;
    
    // 1. Noise floor estimation (RMS of quietest 10% of frames)
    let sorted_rms = calculate_frame_rms(&audio);
    let noise_floor_db = percentile(&sorted_rms, 10.0);
    
    // 2. Clipping detection
    let has_clipping = audio.iter().any(|&s| s.abs() > 0.99);
    
    // 3. Dynamic range
    let peak_db = 20.0 * audio.iter().map(|s| s.abs()).max().log10();
    let dynamic_range_db = peak_db - noise_floor_db;
    
    // 4. Bandwidth estimation (FFT, find -3dB rolloff point)
    let bandwidth_hz = estimate_bandwidth(&audio, 48000)?;
    
    // 5. Quality rating
    let estimated_quality = match (noise_floor_db, has_clipping, dynamic_range_db) {
        (n, false, d) if n < -60.0 && d > 40.0 => QualityRating::Excellent,
        (n, false, d) if n < -50.0 && d > 30.0 => QualityRating::Good,
        (n, _, d) if n < -40.0 && d > 20.0 => QualityRating::Fair,
        (n, _, d) if n < -30.0 => QualityRating::Poor,
        _ => QualityRating::Critical,
    };
    
    // 6. Generate warnings
    let warnings = generate_warnings(noise_floor_db, has_clipping, bandwidth_hz);
    
    Ok(AudioQualityMetrics {
        noise_floor_db,
        has_clipping,
        dynamic_range_db,
        bandwidth_hz,
        estimated_quality,
        warnings,
    })
}
```

#### **Stage 4: Chunked Enhancement**

```rust
// src/enhancement/pipeline.rs

pub async fn enhance_audio(
    input_path: &Path,
    output_path: &Path,
    preset: &EnhancementPreset,
    intensity: f32,
    progress_tx: Sender<f32>,
) -> Result<(), String> {
    let audio = load_audio_f32(input_path)?;
    let sample_rate = 48000;
    
    // Chunk parameters
    let chunk_duration_s = 5.0;
    let overlap_duration_s = 0.5;
    let chunk_samples = (chunk_duration_s * sample_rate as f32) as usize;
    let overlap_samples = (overlap_duration_s * sample_rate as f32) as usize;
    
    // Load ONNX model
    let model = load_deepfilternet_model()?;
    
    let mut output_audio = Vec::with_capacity(audio.len());
    let mut position = 0;
    
    while position < audio.len() {
        let chunk_end = (position + chunk_samples).min(audio.len());
        let chunk = &audio[position..chunk_end];
        
        // 1. Pre-filter: High-pass at 80Hz
        let filtered = highpass_filter(chunk, 80.0, sample_rate);
        
        // 2. DeepFilterNet inference
        let enhanced_chunk = run_deepfilternet(&model, &filtered, sample_rate)?;
        
        // 3. Apply intensity (dry/wet mix)
        let mixed = mix_audio(&filtered, &enhanced_chunk, intensity);
        
        // 4. Overlap-add with previous chunk
        if position > 0 {
            crossfade_overlap(&mut output_audio, &mixed, overlap_samples);
        } else {
            output_audio.extend_from_slice(&mixed);
        }
        
        position += chunk_samples - overlap_samples;
        
        // Report progress
        let progress = position as f32 / audio.len() as f32;
        let _ = progress_tx.send(progress * 0.6).await; // 60% for enhancement stage
    }
    
    save_audio_f32(output_path, &output_audio, sample_rate)?;
    Ok(())
}

fn mix_audio(dry: &[f32], wet: &[f32], intensity: f32) -> Vec<f32> {
    dry.iter()
        .zip(wet.iter())
        .map(|(&d, &w)| d * (1.0 - intensity) + w * intensity)
        .collect()
}
```

**Key Design Decisions:**
- **5-second chunks:** Balance between model efficiency and memory usage
- **0.5s overlap:** Smooth transitions, prevents edge artifacts
- **Intensity control:** Linear dry/wet mix (simple, predictable)

#### **Stage 4.5: Music-Aware Processing**

**Challenge:** Aggressive speech enhancement can damage intentional music beds or background music in creator content.

**Solution:** Detect music sections and apply reduced enhancement.

```rust
// src/analysis/music_detector.rs

use onnxruntime as ort;

pub struct MusicDetector {
    model: ort::Session,
}

impl MusicDetector {
    pub fn new() -> Result<Self, String> {
        // Use Silero VAD or similar model to distinguish speech from music
        let model = ort::Session::builder()?
            .with_model_from_file("models/music_speech_classifier.onnx")?;
        
        Ok(Self { model })
    }
    
    pub fn detect_sections(&self, audio: &[f32], sample_rate: u32) -> Result<Vec<AudioSection>, String> {
        let mut sections = Vec::new();
        let frame_duration_ms = 500; // Analyze in 500ms frames
        let frame_samples = (sample_rate as f32 * frame_duration_ms as f32 / 1000.0) as usize;
        
        let mut current_type = SectionType::Speech;
        let mut section_start = 0;
        
        for (i, chunk) in audio.chunks(frame_samples).enumerate() {
            let features = extract_audio_features(chunk, sample_rate)?;
            let prediction = self.classify(&features)?;
            
            // Smooth transitions: require 2+ consecutive frames to change type
            if prediction != current_type && i > 0 {
                // Check next frame to avoid flutter
                if let Some(next_chunk) = audio.get((i + 1) * frame_samples..(i + 2) * frame_samples) {
                    let next_features = extract_audio_features(next_chunk, sample_rate)?;
                    let next_prediction = self.classify(&next_features)?;
                    
                    if next_prediction == prediction {
                        // Confirmed type change
                        sections.push(AudioSection {
                            start_sample: section_start,
                            end_sample: i * frame_samples,
                            section_type: current_type,
                        });
                        
                        section_start = i * frame_samples;
                        current_type = prediction;
                    }
                }
            }
        }
        
        // Final section
        sections.push(AudioSection {
            start_sample: section_start,
            end_sample: audio.len(),
            section_type: current_type,
        });
        
        Ok(sections)
    }
    
    fn classify(&self, features: &AudioFeatures) -> Result<SectionType, String> {
        // Simple heuristic approach (can be replaced with ML model)
        
        // Speech characteristics:
        // - High zero-crossing rate (consonants)
        // - Energy concentrated in 300Hz-4kHz
        // - Periodic structure (pitch)
        
        // Music characteristics:
        // - Lower zero-crossing rate
        // - Energy spread across full spectrum
        // - Sustained tones, harmonic structure
        
        let speech_score = 
            features.zcr * 0.3 +
            features.spectral_centroid_ratio * 0.3 +
            features.periodicity * 0.4;
        
        if speech_score > 0.6 {
            Ok(SectionType::Speech)
        } else if speech_score < 0.3 {
            Ok(SectionType::Music)
        } else {
            Ok(SectionType::Mixed)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SectionType {
    Speech,
    Music,
    Mixed,
    Silence,
}

#[derive(Debug, Clone)]
pub struct AudioSection {
    pub start_sample: usize,
    pub end_sample: usize,
    pub section_type: SectionType,
}

#[derive(Debug)]
struct AudioFeatures {
    zcr: f32,                   // Zero-crossing rate
    spectral_centroid_ratio: f32, // How much energy is in speech frequencies
    periodicity: f32,           // Pitch strength
    energy: f32,
}

fn extract_audio_features(audio: &[f32], sample_rate: u32) -> Result<AudioFeatures, String> {
    // 1. Zero-crossing rate
    let mut zero_crossings = 0;
    for i in 1..audio.len() {
        if (audio[i] >= 0.0) != (audio[i - 1] >= 0.0) {
            zero_crossings += 1;
        }
    }
    let zcr = zero_crossings as f32 / audio.len() as f32;
    
    // 2. Spectral centroid (via FFT)
    let spectrum = compute_magnitude_spectrum(audio);
    let centroid_hz = compute_spectral_centroid(&spectrum, sample_rate);
    let spectral_centroid_ratio = if centroid_hz > 300.0 && centroid_hz < 4000.0 {
        1.0
    } else {
        0.0
    };
    
    // 3. Periodicity (autocorrelation)
    let periodicity = estimate_periodicity(audio, sample_rate);
    
    // 4. RMS energy
    let energy = audio.iter().map(|&s| s * s).sum::<f32>().sqrt() / audio.len() as f32;
    
    Ok(AudioFeatures {
        zcr,
        spectral_centroid_ratio,
        periodicity,
        energy,
    })
}
```

**Integration with Enhancement Pipeline:**

```rust
// Modified enhance_audio function

pub async fn enhance_audio_with_music_awareness(
    input_path: &Path,
    output_path: &Path,
    preset: &EnhancementPreset,
    intensity: f32,
    progress_tx: Sender<f32>,
) -> Result<(), String> {
    let audio = load_audio_f32(input_path)?;
    let sample_rate = 48000;
    
    // 1. Detect speech/music sections
    let music_detector = MusicDetector::new()?;
    let sections = music_detector.detect_sections(&audio, sample_rate)?;
    
    // 2. Process with section-aware enhancement
    let model = load_deepfilternet_model()?;
    let mut output_audio = Vec::with_capacity(audio.len());
    
    for section in sections {
        let section_audio = &audio[section.start_sample..section.end_sample];
        
        // Adjust enhancement strength based on content type
        let adjusted_intensity = match section.section_type {
            SectionType::Speech => intensity,                    // Full enhancement
            SectionType::Music => intensity * 0.3,               // Gentle (30%)
            SectionType::Mixed => intensity * 0.6,               // Moderate (60%)
            SectionType::Silence => 0.0,                         // No processing
        };
        
        let adjusted_preset = EnhancementPreset {
            denoise_strength: preset.denoise_strength * (adjusted_intensity / intensity),
            dereverb_strength: if section.section_type == SectionType::Music {
                0.0 // Never dereverb music
            } else {
                preset.dereverb_strength
            },
            ..preset.clone()
        };
        
        // Process this section
        let enhanced_section = if adjusted_intensity > 0.01 {
            process_audio_chunk(section_audio, &model, &adjusted_preset, sample_rate)?
        } else {
            section_audio.to_vec() // Pass through silence
        };
        
        output_audio.extend_from_slice(&enhanced_section);
        
        // Update progress
        let progress = section.end_sample as f32 / audio.len() as f32;
        let _ = progress_tx.send(progress * 0.6).await;
    }
    
    save_audio_f32(output_path, &output_audio, sample_rate)?;
    Ok(())
}
```

**User Control:**

```typescript
// src/components/PresetSelector.tsx

<PresetOption 
  name="YouTube (Music-Aware)"
  description="Smart enhancement that preserves background music while cleaning speech"
  preset="youtube_music_aware"
/>

<ToggleSetting
  label="Music-Aware Processing"
  description="Automatically detect and preserve music sections (may increase processing time)"
  enabled={settings.musicAwareProcessing}
  onChange={(enabled) => updateSettings({ musicAwareProcessing: enabled })}
/>
```

**Performance Considerations:**
- Music detection adds ~5% to total processing time
- For MVP: Make it optional, default OFF
- For v1: Enable by default once optimized

---

#### **Stage 5: DSP Chain**

```rust
// src/dsp/processing.rs

pub fn apply_dsp_chain(
    input_path: &Path,
    output_path: &Path,
    preset: &EnhancementPreset,
) -> Result<(), String> {
    let mut audio = load_audio_f32(input_path)?;
    let sample_rate = 48000;
    
    // 1. EQ
    match preset.eq_profile {
        EQProfile::YouTube => {
            // High-pass: 80Hz
            audio = biquad_filter(&audio, FilterType::HighPass(80.0), sample_rate);
            // Presence boost: +3dB @ 4kHz (bell, Q=1.0)
            audio = biquad_filter(&audio, FilterType::PeakingEQ(4000.0, 1.0, 3.0), sample_rate);
            // Mud reduction: -2dB @ 300Hz (bell, Q=0.7)
            audio = biquad_filter(&audio, FilterType::PeakingEQ(300.0, 0.7, -2.0), sample_rate);
        }
        EQProfile::Podcast => {
            audio = biquad_filter(&audio, FilterType::HighPass(100.0), sample_rate);
            audio = biquad_filter(&audio, FilterType::PeakingEQ(3000.0, 1.5, 4.0), sample_rate);
        }
        // ... other profiles
        _ => {}
    }
    
    // 2. Gentle compression
    if preset.compression_ratio > 1.0 {
        audio = apply_compressor(&audio, CompressorParams {
            threshold_db: -20.0,
            ratio: preset.compression_ratio,
            attack_ms: 10.0,
            release_ms: 100.0,
            knee_db: 6.0,
        }, sample_rate);
    }
    
    // 3. De-esser
    audio = apply_deesser(&audio, DeEsserParams {
        freq_hz: 7000.0,
        threshold_db: -15.0,
        ratio: 3.0,
    }, sample_rate);
    
    save_audio_f32(output_path, &audio, sample_rate)?;
    Ok(())
}
```

#### **Stage 6: Loudness Normalization**

```rust
// src/dsp/loudness.rs

pub fn normalize_loudness(
    input_path: &Path,
    output_path: &Path,
    target_lufs: f32,
    true_peak_max_dbfs: f32,
) -> Result<(), String> {
    let audio = load_audio_f32(input_path)?;
    let sample_rate = 48000;
    
    // 1. Measure integrated loudness (ITU-R BS.1770-4)
    let current_lufs = measure_integrated_loudness(&audio, sample_rate)?;
    
    // 2. Calculate gain needed
    let gain_db = target_lufs - current_lufs;
    let gain_linear = db_to_linear(gain_db);
    
    // 3. Apply gain
    let mut gained_audio: Vec<f32> = audio.iter()
        .map(|&s| s * gain_linear)
        .collect();
    
    // 4. Measure true peak
    let true_peak_dbfs = measure_true_peak(&gained_audio, sample_rate)?;
    
    // 5. Apply limiter if needed
    if true_peak_dbfs > true_peak_max_dbfs {
        let limiter_threshold = db_to_linear(true_peak_max_dbfs);
        gained_audio = apply_limiter(&gained_audio, limiter_threshold, sample_rate);
    }
    
    save_audio_f32(output_path, &gained_audio, sample_rate)?;
    Ok(())
}

// ITU-R BS.1770-4 implementation (simplified)
fn measure_integrated_loudness(audio: &[f32], sample_rate: u32) -> Result<f32, String> {
    // K-weighting filter
    let k_weighted = apply_k_weighting(audio, sample_rate);
    
    // Gating (absolute: -70 LUFS, relative: -10 LU)
    let gated_blocks = compute_gated_blocks(&k_weighted, sample_rate);
    
    // Mean square of gated blocks
    let mean_square = gated_blocks.iter().sum::<f32>() / gated_blocks.len() as f32;
    
    // Convert to LUFS
    Ok(-0.691 + 10.0 * mean_square.log10())
}
```

**Platform-Specific Targets:**
| Platform | Target LUFS | True Peak Max |
|----------|-------------|---------------|
| YouTube | -14 to -16 LUFS | -1.0 dBFS |
| LinkedIn | -16 LUFS | -1.0 dBFS |
| Instagram | -14 LUFS | -2.0 dBFS |

#### **Stage 9: Remuxing**

```rust
// src/remux/video.rs

pub async fn remux_video(
    original_video: &Path,
    enhanced_audio: &Path,
    output_path: &Path,
    config: &ExportConfig,
) -> Result<(), String> {
    // Encode audio to target codec
    let encoded_audio = encode_audio(enhanced_audio, config).await?;
    
    // Remux: copy video, replace audio
    let status = Command::new("ffmpeg")
        .args(&[
            "-i", original_video.to_str().unwrap(),
            "-i", encoded_audio.to_str().unwrap(),
            "-map", "0:v:0",               // Copy first video stream
            "-map", "1:a:0",               // Use new audio stream
            "-c:v", "copy",                // No video re-encode
            "-shortest",                   // Match shortest stream duration
            "-movflags", "+faststart",     // Enable streaming (for YouTube)
            "-y",
            output_path.to_str().unwrap(),
        ])
        .status()
        .await
        .map_err(|e| format!("Remux failed: {}", e))?;
    
    if !status.success() {
        return Err("Video remux failed".into());
    }
    
    // Validate A/V sync
    validate_sync(output_path)?;
    
    Ok(())
}

fn validate_sync(video_path: &Path) -> Result<(), String> {
    // Use ffprobe to check stream timestamps
    let output = Command::new("ffprobe")
        .args(&[
            "-v", "error",
            "-select_streams", "a:0",
            "-show_entries", "stream=start_time",
            "-of", "default=noprint_wrappers=1:nokey=1",
            video_path.to_str().unwrap(),
        ])
        .output()
        .map_err(|e| format!("ffprobe failed: {}", e))?;
    
    // Parse and validate (implementation omitted for brevity)
    // If drift > 40ms, return error
    
    Ok(())
}
```

### 6.3 Cancellation & Error Recovery

```rust
// Cancellation token pattern
pub async fn process_job(
    job_id: String,
    job: ProcessingJob,
    cancel_token: CancellationToken,
) -> Result<(), String> {
    // Check cancellation at each stage
    if cancel_token.is_cancelled() {
        return Err("Processing cancelled".into());
    }
    
    extract_audio(&job.media_file, &temp_path).await?;
    
    if cancel_token.is_cancelled() {
        cleanup_temp_files(&job_id);
        return Err("Processing cancelled".into());
    }
    
    // ... continue with other stages
}

// Error recovery
pub async fn retry_failed_job(job_id: &str) -> Result<(), String> {
    // Load job state
    let mut job = get_job(job_id)?;
    
    // Determine retry strategy based on failure stage
    match job.status {
        JobStatus::Failed => {
            // Reset to last successful stage
            if let Some(last_checkpoint) = job.last_checkpoint {
                job.status = last_checkpoint;
                save_job(&job)?;
                start_processing(job_id).await?;
            }
        }
        _ => return Err("Job is not in failed state".into()),
    }
    
    Ok(())
}
```

---

## 7. PROJECT FILE STRUCTURE

```
audio-cleaner/
├── src-tauri/                      # Rust backend
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── build.rs
│   ├── src/
│   │   ├── main.rs                # Tauri app entry point
│   │   ├── commands/              # Tauri command handlers
│   │   │   ├── mod.rs
│   │   │   ├── media.rs           # probe_media_file, etc.
│   │   │   ├── processing.rs      # create_job, start_processing
│   │   │   ├── preview.rs         # generate_preview
│   │   │   ├── export.rs          # export_enhanced_video
│   │   │   ├── license.rs         # activate_license, check_feature
│   │   │   └── settings.rs        # get/update settings
│   │   ├── models/                # Data models
│   │   │   ├── mod.rs
│   │   │   ├── media.rs
│   │   │   ├── processing.rs
│   │   │   ├── export.rs
│   │   │   ├── license.rs
│   │   │   └── settings.rs
│   │   ├── processing/            # Core processing logic
│   │   │   ├── mod.rs
│   │   │   ├── extractor.rs       # Audio extraction (FFmpeg)
│   │   │   ├── pipeline.rs        # Enhancement pipeline orchestration
│   │   │   ├── pool.rs            # ProcessingPool (concurrency)
│   │   │   └── temp_manager.rs    # Temp file management
│   │   ├── enhancement/           # ML enhancement
│   │   │   ├── mod.rs
│   │   │   ├── deepfilternet.rs   # DeepFilterNet model runner
│   │   │   ├── onnx_runtime.rs    # ONNX Runtime wrapper
│   │   │   └── chunking.rs        # Audio chunking logic
│   │   ├── dsp/                   # DSP processing
│   │   │   ├── mod.rs
│   │   │   ├── filters.rs         # Biquad filters (EQ, HPF)
│   │   │   ├── compressor.rs      # Dynamics compression
│   │   │   ├── deesser.rs         # De-esser
│   │   │   └── loudness.rs        # Loudness normalization (BS.1770)
│   │   ├── analysis/              # Audio analysis
│   │   │   ├── mod.rs
│   │   │   ├── quality.rs         # Quality metrics
│   │   │   ├── vad.rs             # Voice Activity Detection (Silero)
│   │   │   └── waveform.rs        # Waveform generation
│   │   ├── remux/                 # Video remuxing
│   │   │   ├── mod.rs
│   │   │   ├── video.rs           # FFmpeg remuxing
│   │   │   └── validation.rs      # A/V sync validation
│   │   ├── ffmpeg/                # FFmpeg wrapper
│   │   │   ├── mod.rs
│   │   │   ├── probe.rs           # Media probing
│   │   │   ├── extract.rs         # Audio extraction
│   │   │   └── encode.rs          # Audio encoding
│   │   ├── persistence/           # Storage & caching
│   │   │   ├── mod.rs
│   │   │   ├── settings.rs        # Settings persistence
│   │   │   ├── license_cache.rs   # License caching
│   │   │   ├── model_cache.rs     # ML model caching
│   │   │   └── presets.rs         # Preset storage
│   │   ├── state/                 # Application state
│   │   │   ├── mod.rs
│   │   │   └── app_state.rs       # AppState struct
│   │   ├── license/               # License validation
│   │   │   ├── mod.rs
│   │   │   ├── validator.rs       # License validation logic
│   │   │   └── features.rs        # Feature gating
│   │   └── utils/                 # Utilities
│   │       ├── mod.rs
│   │       ├── audio_io.rs        # Audio file I/O helpers
│   │       ├── paths.rs           # Path utilities
│   │       └── progress.rs        # Progress tracking helpers
│   └── icons/                     # App icons (platform-specific)
│
├── src/                           # React frontend
│   ├── main.tsx                   # Entry point
│   ├── App.tsx                    # Root component
│   ├── styles/
│   │   ├── global.css
│   │   └── themes.css
│   ├── components/                # React components
│   │   ├── FileDropZone.tsx
│   │   ├── MediaPreview.tsx
│   │   ├── ProcessingQueue.tsx
│   │   ├── PresetSelector.tsx
│   │   ├── QualityMeter.tsx
│   │   ├── IntensitySlider.tsx
│   │   ├── ExportDialog.tsx
│   │   ├── SettingsPanel.tsx
│   │   ├── LicenseActivation.tsx
│   │   └── Waveform.tsx
│   ├── hooks/                     # Custom React hooks
│   │   ├── useAppContext.ts
│   │   ├── useProcessingJob.ts
│   │   ├── usePreview.ts
│   │   └── useLicense.ts
│   ├── context/                   # React Context providers
│   │   └── AppContext.tsx
│   ├── services/                  # Tauri API wrappers
│   │   ├── mediaService.ts        # Media commands
│   │   ├── processingService.ts   # Processing commands
│   │   ├── previewService.ts      # Preview commands
│   │   ├── exportService.ts       # Export commands
│   │   ├── licenseService.ts      # License commands
│   │   └── settingsService.ts     # Settings commands
│   ├── types/                     # TypeScript type definitions
│   │   ├── media.ts
│   │   ├── processing.ts
│   │   ├── preview.ts
│   │   ├── export.ts
│   │   ├── license.ts
│   │   └── settings.ts
│   └── utils/                     # Frontend utilities
│       ├── formatters.ts          # Time/size formatting
│       ├── validators.ts          # Input validation
│       └── constants.ts           # App constants
│
├── public/                        # Static assets
│   └── models/                    # Bundled ML models (copied to app)
│       ├── deepfilternet_3_quant.onnx
│       └── silero_vad_v4.onnx
│
├── package.json
├── tsconfig.json
├── vite.config.ts
├── tailwind.config.js             # Tailwind CSS config
├── .gitignore
└── README.md
```

**Key Files to Create First (MVP Priority):**

1. **Backend Core:**
   - `src-tauri/src/main.rs` - App initialization
   - `src-tauri/src/models/` - All data models
   - `src-tauri/src/commands/media.rs` - File probing
   - `src-tauri/src/ffmpeg/probe.rs` - FFmpeg wrapper
   - `src-tauri/src/processing/extractor.rs` - Audio extraction

2. **Frontend Core:**
   - `src/App.tsx` - Main UI shell
   - `src/components/FileDropZone.tsx` - File ingestion
   - `src/types/` - All TypeScript interfaces
   - `src/services/mediaService.ts` - Tauri API wrapper

3. **Processing Pipeline:**
   - `src-tauri/src/enhancement/deepfilternet.rs` - ML model integration
   - `src-tauri/src/dsp/loudness.rs` - Loudness normalization
   - `src-tauri/src/remux/video.rs` - Remuxing logic

---

## 8. KEY ALGORITHMS

### 8.1 Audio Chunking Strategy

**Challenge:** Process long audio files (10-60 minutes) with ML models designed for short segments.

**Solution: Overlapped Chunking with Crossfade**

```rust
// Pseudocode
fn chunk_audio(audio: &[f32], chunk_size: usize, overlap: usize) -> Vec<Chunk> {
    let mut chunks = Vec::new();
    let mut position = 0;
    
    while position < audio.len() {
        let end = (position + chunk_size).min(audio.len());
        let chunk_data = audio[position..end].to_vec();
        
        chunks.push(Chunk {
            data: chunk_data,
            start_sample: position,
            overlap_samples: if position > 0 { overlap } else { 0 },
        });
        
        position += chunk_size - overlap;
    }
    
    chunks
}

fn reconstruct_audio(chunks: Vec<ProcessedChunk>) -> Vec<f32> {
    let mut output = Vec::new();
    
    for (i, chunk) in chunks.iter().enumerate() {
        if i == 0 {
            // First chunk: no crossfade
            output.extend_from_slice(&chunk.data);
        } else {
            // Crossfade with previous chunk's tail
            let overlap_samples = chunk.overlap_samples;
            let tail_start = output.len() - overlap_samples;
            
            // Linear crossfade
            for j in 0..overlap_samples {
                let fade_out = 1.0 - (j as f32 / overlap_samples as f32);
                let fade_in = j as f32 / overlap_samples as f32;
                
                output[tail_start + j] = 
                    output[tail_start + j] * fade_out + chunk.data[j] * fade_in;
            }
            
            // Append rest of chunk
            output.extend_from_slice(&chunk.data[overlap_samples..]);
        }
    }
    
    output
}
```

**Parameters:**
- Chunk size: 5 seconds (240,000 samples @ 48kHz)
- Overlap: 0.5 seconds (24,000 samples)
- Crossfade: Linear (simple, artifact-free for most content)

**Why it works:**
- ML model sees enough context (5s) for speech patterns
- Overlap prevents edge artifacts at chunk boundaries
- Crossfade smooths transitions

### 8.2 Real-Time Preview Generation

**Challenge:** Generate fast previews for A/B comparison without processing entire file.

**Solution: Region-Based Processing with Caching**

```rust
pub async fn generate_preview(
    job_id: &str,
    media: &MediaFile,
    preset: &EnhancementPreset,
    region: TimeRange,  // e.g., 30s-35s
) -> Result<PreviewPaths, String> {
    let cache_key = format!("{}_{}_{}_{}", job_id, region.start_ms, region.end_ms, preset.name);
    
    // Check cache
    if let Some(cached) = PREVIEW_CACHE.get(&cache_key) {
        return Ok(cached);
    }
    
    // Extract region from original
    let original_clip = extract_audio_region(
        &media.path,
        region.start_ms,
        region.end_ms,
    ).await?;
    
    // Process only this region
    let enhanced_clip = process_audio_fast(
        &original_clip,
        preset,
        ProcessingQuality::Fast,  // Lower quality for speed
    ).await?;
    
    let paths = PreviewPaths {
        original: original_clip,
        enhanced: enhanced_clip,
    };
    
    // Cache result
    PREVIEW_CACHE.insert(cache_key, paths.clone());
    
    Ok(paths)
}

// Fast processing mode: skip some DSP stages
async fn process_audio_fast(
    audio_path: &Path,
    preset: &EnhancementPreset,
    quality: ProcessingQuality,
) -> Result<PathBuf, String> {
    match quality {
        ProcessingQuality::Fast => {
            // Only denoise + basic loudness adjustment
            // Skip: EQ, compression, de-esser
            let enhanced = run_deepfilternet(audio_path, preset.denoise_strength).await?;
            normalize_loudness_simple(&enhanced, preset.target_loudness_lufs).await?;
            Ok(enhanced)
        }
        ProcessingQuality::Standard => {
            // Full pipeline
            run_full_pipeline(audio_path, preset).await
        }
        ProcessingQuality::Maximum => {
            // Full pipeline + higher ONNX precision
            run_full_pipeline_high_precision(audio_path, preset).await
        }
    }
}
```

**Key Optimizations:**
- Process only 5-10 second regions (vs. entire file)
- Use "Fast" quality mode for preview (skip non-critical DSP)
- Cache previews (same region + preset = reuse result)
- Generate previews from middle of file by default (most representative)

### 8.3 Loudness Normalization Approach

**Standard: ITU-R BS.1770-4**

```rust
// K-weighting filter (two-stage biquad)
fn apply_k_weighting(audio: &[f32], sample_rate: u32) -> Vec<f32> {
    // Stage 1: High-pass shelf (HSF) at ~38Hz
    let stage1 = biquad_filter(audio, FilterType::HighShelf {
        freq: 38.0,
        gain_db: 4.0,
        q: 0.5,
    }, sample_rate);
    
    // Stage 2: High-frequency shelf (HF) at ~1.5kHz
    let stage2 = biquad_filter(&stage1, FilterType::HighShelf {
        freq: 1500.0,
        gain_db: -3.5,
        q: 0.5,
    }, sample_rate);
    
    stage2
}

// Gating algorithm
fn compute_gated_blocks(k_weighted: &[f32], sample_rate: u32) -> Vec<f32> {
    let block_duration_s = 0.4; // 400ms blocks
    let block_samples = (block_duration_s * sample_rate as f32) as usize;
    
    let mut blocks = Vec::new();
    
    // Compute mean square for each block
    for chunk in k_weighted.chunks(block_samples) {
        let mean_square = chunk.iter().map(|&s| s * s).sum::<f32>() / chunk.len() as f32;
        blocks.push(mean_square);
    }
    
    // Absolute gate: -70 LUFS
    let absolute_gate = db_to_linear(-70.0);
    let gated_blocks: Vec<f32> = blocks.iter()
        .filter(|&&b| b >= absolute_gate)
        .copied()
        .collect();
    
    // Relative gate: -10 LU below mean of absolute-gated
    let relative_threshold = gated_blocks.iter().sum::<f32>() / gated_blocks.len() as f32;
    let relative_gate = relative_threshold * db_to_linear(-10.0);
    
    gated_blocks.iter()
        .filter(|&&b| b >= relative_gate)
        .copied()
        .collect()
}

// True peak detection (oversampling)
fn measure_true_peak(audio: &[f32], sample_rate: u32) -> Result<f32, String> {
    // Upsample 4x to catch inter-sample peaks
    let oversampled = upsample(audio, 4);
    
    // Find peak
    let peak = oversampled.iter()
        .map(|&s| s.abs())
        .max()
        .ok_or("Empty audio")?;
    
    Ok(linear_to_db(peak))
}
```

**Platform Presets:**
```rust
pub fn get_platform_loudness_targets(platform: Platform) -> (f32, f32) {
    // Returns: (target_lufs, true_peak_max_dbfs)
    match platform {
        Platform::YouTube => (-14.0, -1.0),
        Platform::LinkedIn => (-16.0, -1.0),
        Platform::Instagram => (-14.0, -2.0),  // Instagram is more conservative
        Platform::Custom => (-16.0, -1.0),     // Safe default
    }
}
```

### 8.4 FFmpeg Integration Pattern

**Wrapper Design:**

```rust
pub struct FFmpegCommand {
    input: PathBuf,
    output: PathBuf,
    args: Vec<String>,
}

impl FFmpegCommand {
    pub fn new(input: PathBuf, output: PathBuf) -> Self {
        Self {
            input,
            output,
            args: Vec::new(),
        }
    }
    
    pub fn map_stream(mut self, stream_spec: &str) -> Self {
        self.args.push("-map".to_string());
        self.args.push(stream_spec.to_string());
        self
    }
    
    pub fn video_codec(mut self, codec: &str) -> Self {
        self.args.push("-c:v".to_string());
        self.args.push(codec.to_string());
        self
    }
    
    pub fn audio_codec(mut self, codec: &str) -> Self {
        self.args.push("-c:a".to_string());
        self.args.push(codec.to_string());
        self
    }
    
    pub fn audio_bitrate(mut self, bitrate: u32) -> Self {
        self.args.push("-b:a".to_string());
        self.args.push(format!("{}k", bitrate / 1000));
        self
    }
    
    pub async fn execute(self) -> Result<(), String> {
        let mut cmd = Command::new("ffmpeg");
        cmd.arg("-i").arg(&self.input);
        cmd.args(&self.args);
        cmd.arg("-y");  // Overwrite
        cmd.arg(&self.output);
        
        let status = cmd.status().await
            .map_err(|e| format!("FFmpeg execution failed: {}", e))?;
        
        if status.success() {
            Ok(())
        } else {
            Err("FFmpeg command failed".into())
        }
    }
    
    pub async fn execute_with_progress<F>(
        self,
        duration_ms: u64,
        mut progress_callback: F,
    ) -> Result<(), String>
    where
        F: FnMut(f32) + Send + 'static,
    {
        // Parse FFmpeg stderr for progress info
        // Example: "time=00:01:23.45" → extract current time → compute progress
        // Call progress_callback(progress) periodically
        
        // Implementation omitted for brevity
        self.execute().await
    }
}

// Usage example
FFmpegCommand::new(input_video, output_video)
    .map_stream("0:v:0")
    .video_codec("copy")
    .map_stream("1:a:0")
    .audio_codec("aac")
    .audio_bitrate(192000)
    .execute()
    .await?;
```

**Error Handling:**
- Always check exit status
- Parse stderr for specific errors (e.g., "No such file", "Invalid codec")
- Provide user-friendly error messages
- Include FFmpeg output in crash logs for debugging

---

## IMPLEMENTATION PRIORITIES

### Phase 1: Core Pipeline (Weeks 1-4)
1. Tauri app scaffold + FFmpeg integration
2. Media probing + audio extraction
3. DeepFilterNet integration (ONNX Runtime)
4. Basic loudness normalization
5. Video remuxing

### Phase 2: UI & Preview (Weeks 5-7)
1. React UI shell + file drop zone
2. Processing queue component
3. Preview generation + A/B player
4. Waveform visualization
5. Progress tracking

### Phase 3: Polish & Features (Weeks 8-10)
1. Preset system (YouTube, LinkedIn, Instagram)
2. Intensity control (dry/wet mix)
3. Quality meter + warnings
4. Batch queue
5. Settings panel

### Phase 4: Commercial & Distribution (Weeks 11-12)
1. License system + feature gating
2. Code signing + notarization
3. Installer creation
4. Auto-update system
5. Analytics + crash reporting

---

## TESTING STRATEGY

### Unit Tests
- FFmpeg wrapper: Mock file operations, test argument construction
- DSP algorithms: Test filters, compressor, loudness with known inputs
- Chunking: Verify overlap-add correctness

### Integration Tests
- Full pipeline: Test with sample 30s video files
- A/V sync: Validate remuxed output has <40ms drift
- Error cases: Invalid files, missing codecs, disk full

### Benchmark Corpus
- Collect 25+ representative creator clips:
  - USB mic + untreated room
  - Webcam audio
  - Screen recording with fan noise
  - Various formats (MP4, MKV, MOV)
- Subjective testing: A/B comparison with beta users
- Automated regression: Store reference outputs, compare on each build

---

## DEPLOYMENT CONSIDERATIONS

### Windows
- Installer: NSIS or WiX
- Code signing: EV certificate for SmartScreen reputation
- FFmpeg: Bundle in `resources/` directory
- ONNX Runtime: Static linking or bundled DLL

### macOS
- DMG + `.app` bundle
- Developer ID signing + notarization (required)
- FFmpeg: Bundle in `Contents/Resources/`
- ONNX Runtime: Static linking or framework bundle

### Auto-Updates
- Tauri's built-in updater
- Host releases on GitHub Releases or custom CDN
- Delta updates for smaller download sizes

---

## 9. COMMERCIAL & OPERATIONAL FEATURES

### 9.1 License Enforcement

#### **Free Tier Limits**

```rust
// src/license/limits.rs

pub const FREE_TIER_EXPORT_LIMIT_PER_MONTH: u32 = 3;
pub const FREE_TIER_PREVIEW_DURATION_SEC: u32 = 10; // Unlimited previews, but short clips

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    pub exports_this_month: u32,
    pub month_start: SystemTime,
    pub total_exports_all_time: u32,
}

impl UsageStats {
    pub fn can_export(&self, license: &LicenseInfo) -> Result<(), String> {
        match license.tier {
            LicenseTier::Free => {
                if self.exports_this_month >= FREE_TIER_EXPORT_LIMIT_PER_MONTH {
                    return Err(format!(
                        "Free tier limit reached ({} exports per month). Upgrade to Pro for unlimited exports.",
                        FREE_TIER_EXPORT_LIMIT_PER_MONTH
                    ));
                }
                Ok(())
            }
            _ => Ok(()), // Pro tiers have unlimited exports
        }
    }
    
    pub fn increment_export(&mut self) {
        // Check if we've rolled into a new month
        if self.month_start.elapsed().unwrap_or_default().as_secs() > 30 * 24 * 3600 {
            self.exports_this_month = 0;
            self.month_start = SystemTime::now();
        }
        
        self.exports_this_month += 1;
        self.total_exports_all_time += 1;
    }
}
```

#### **Enforcement Points**

```rust
// src/commands/export.rs

#[tauri::command]
async fn export_enhanced_video(
    state: State<'_, AppState>,
    job_id: String,
    config: ExportConfig,
) -> Result<String, String> {
    // 1. Check license and usage limits
    let license = state.license.lock().await;
    let mut usage = state.usage_stats.lock().await;
    
    usage.can_export(&license)?;
    
    // 2. Proceed with export
    let output_path = perform_export(&job_id, &config).await?;
    
    // 3. Increment usage counter
    usage.increment_export();
    save_usage_stats(&usage)?;
    
    // 4. Show upgrade prompt if approaching limit
    if license.tier == LicenseTier::Free && usage.exports_this_month >= 2 {
        emit_upgrade_prompt(state.app_handle.clone(), &usage)?;
    }
    
    Ok(output_path)
}
```

#### **Preview Limits for Free Tier**

```rust
// src/commands/preview.rs

#[tauri::command]
async fn generate_preview(
    state: State<'_, AppState>,
    job_id: String,
    start_ms: u64,
    duration_ms: u64,
) -> Result<PreviewPaths, String> {
    let license = state.license.lock().await;
    
    // Free tier: cap preview duration
    let max_duration_ms = match license.tier {
        LicenseTier::Free => (FREE_TIER_PREVIEW_DURATION_SEC * 1000) as u64,
        _ => duration_ms, // Pro: unlimited
    };
    
    let capped_duration = duration_ms.min(max_duration_ms);
    
    let paths = generate_preview_internal(&job_id, start_ms, capped_duration).await?;
    
    Ok(paths)
}
```

#### **Upgrade Prompt UX**

```typescript
// src/components/UpgradePrompt.tsx

interface UpgradePromptProps {
  exportsRemaining: number;
  exportsUsed: number;
  totalLimit: number;
}

export function UpgradePrompt({ exportsRemaining, exportsUsed, totalLimit }: UpgradePromptProps) {
  if (exportsRemaining > 1) return null;
  
  return (
    <div className="upgrade-banner">
      <div className="upgrade-content">
        <strong>⚠️ {exportsRemaining} export{exportsRemaining !== 1 ? 's' : ''} remaining this month</strong>
        <p>
          You've used {exportsUsed} of {totalLimit} free exports. 
          Upgrade to Pro for unlimited exports, batch processing, and advanced presets.
        </p>
      </div>
      <div className="upgrade-actions">
        <button onClick={handleUpgrade} className="btn-primary">
          Upgrade to Pro - $99/year
        </button>
        <button onClick={handleDismiss} className="btn-secondary">
          Remind me later
        </button>
      </div>
    </div>
  );
}
```

#### **Watermark Strategy**

**Decision: No watermarks on free tier exports**

**Rationale:**
- PRD specifies "watermark-free but capped usage"
- Export limits (3/month) are sufficient gating
- Watermarks would hurt word-of-mouth marketing
- Creators won't share watermarked content (kills virality)

**Alternative:** If piracy becomes an issue, consider:
- Audio watermark in free tier (subtle high-frequency tone)
- Requires additional DSP work (not MVP priority)

#### **License Validation Flow**

```rust
// src/license/validator.rs

pub struct LicenseValidator {
    cache_path: PathBuf,
    last_online_check: Option<SystemTime>,
}

impl LicenseValidator {
    pub async fn validate(&mut self, key: &str) -> Result<LicenseInfo, String> {
        // 1. Parse license key (format: PROD-XXXX-XXXX-XXXX-XXXX)
        let parsed = parse_license_key(key)?;
        
        // 2. Verify signature (offline check)
        verify_signature(&parsed)?;
        
        // 3. Check expiration
        if let Some(expires) = parsed.expires_at {
            if SystemTime::now() > expires {
                return Err("License expired".into());
            }
        }
        
        // 4. Online validation (if >24h since last check)
        if self.should_check_online() {
            self.validate_online(&parsed).await?;
            self.last_online_check = Some(SystemTime::now());
        }
        
        // 5. Cache valid license
        let license_info = LicenseInfo {
            tier: parsed.tier,
            key: Some(key.to_string()),
            expires_at: parsed.expires_at,
            features: get_features_for_tier(&parsed.tier),
        };
        
        self.save_to_cache(&license_info)?;
        
        Ok(license_info)
    }
    
    async fn validate_online(&self, parsed: &ParsedLicense) -> Result<(), String> {
        // Check with license server for revocation
        let response = reqwest::get(format!(
            "https://api.audiocleaner.app/v1/license/validate?key={}",
            parsed.key_hash
        ))
        .await
        .map_err(|e| format!("License validation failed: {}", e))?;
        
        if !response.status().is_success() {
            return Err("License revoked or invalid".into());
        }
        
        Ok(())
    }
}
```

---

### 9.2 Quality Warning System

#### **Warning Thresholds**

```rust
// src/analysis/warnings.rs

pub fn generate_warnings(metrics: &AudioQualityMetrics) -> Vec<QualityWarning> {
    let mut warnings = Vec::new();
    
    // 1. Noise floor warnings
    if metrics.noise_floor_db > -30.0 {
        warnings.push(QualityWarning {
            severity: WarningSeverity::Critical,
            title: "Very High Noise Floor".to_string(),
            message: "Background noise is extremely loud. Enhancement will help, but consider re-recording in a quieter environment for best results.".to_string(),
            recommendation: "Move to a quieter room, use directional microphone, or add acoustic treatment.".to_string(),
        });
    } else if metrics.noise_floor_db > -40.0 {
        warnings.push(QualityWarning {
            severity: WarningSeverity::Warning,
            title: "High Noise Floor".to_string(),
            message: "Noticeable background noise detected. Enhancement will improve clarity, but some noise may remain.".to_string(),
            recommendation: "Results will be good but not studio-quality. Consider improving recording environment for future videos.".to_string(),
        });
    }
    
    // 2. Clipping warnings
    if metrics.has_clipping {
        warnings.push(QualityWarning {
            severity: WarningSeverity::Critical,
            title: "Audio Clipping Detected".to_string(),
            message: "Parts of your audio are clipped (too loud), causing distortion. This cannot be fully repaired.".to_string(),
            recommendation: "Lower your microphone gain or move further from the mic to prevent clipping in future recordings.".to_string(),
        });
    }
    
    // 3. Dynamic range warnings
    if metrics.dynamic_range_db < 10.0 {
        warnings.push(QualityWarning {
            severity: WarningSeverity::Warning,
            title: "Very Low Dynamic Range".to_string(),
            message: "Audio is already heavily compressed or limited. Further processing may not improve quality.".to_string(),
            recommendation: "Check if your recording software has built-in compression enabled.".to_string(),
        });
    }
    
    // 4. Bandwidth warnings
    if metrics.bandwidth_hz < 8000 {
        warnings.push(QualityWarning {
            severity: WarningSeverity::Warning,
            title: "Limited Frequency Range".to_string(),
            message: "Audio sounds 'telephone-like' due to limited bandwidth. Enhancement cannot restore missing frequencies.".to_string(),
            recommendation: "Use a better microphone or check audio interface settings.".to_string(),
        });
    }
    
    // 5. Overall quality assessment
    match metrics.estimated_quality {
        QualityRating::Excellent => {
            // No warning
        }
        QualityRating::Good => {
            warnings.push(QualityWarning {
                severity: WarningSeverity::Info,
                title: "Good Source Quality".to_string(),
                message: "Your audio is already decent. Enhancement will add polish and consistency.".to_string(),
                recommendation: "".to_string(),
            });
        }
        QualityRating::Fair => {
            warnings.push(QualityWarning {
                severity: WarningSeverity::Info,
                title: "Fair Source Quality".to_string(),
                message: "Enhancement will make a noticeable improvement, but results won't be studio-quality.".to_string(),
                recommendation: "This is typical for home recordings. The app will make it upload-ready.".to_string(),
            });
        }
        QualityRating::Poor => {
            warnings.push(QualityWarning {
                severity: WarningSeverity::Warning,
                title: "Poor Source Quality".to_string(),
                message: "Multiple audio issues detected. Enhancement will help, but significant artifacts may remain.".to_string(),
                recommendation: "Consider re-recording if this is important content.".to_string(),
            });
        }
        QualityRating::Critical => {
            warnings.push(QualityWarning {
                severity: WarningSeverity::Critical,
                title: "Critical Source Quality".to_string(),
                message: "Audio quality is very poor. Enhancement may produce mixed results. Preview carefully before exporting.".to_string(),
                recommendation: "Strongly consider re-recording. If not possible, use 'Gentle' preset and reduce intensity to 50%.".to_string(),
            });
        }
    }
    
    warnings
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityWarning {
    pub severity: WarningSeverity,
    pub title: String,
    pub message: String,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WarningSeverity {
    Info,      // Blue - informational
    Warning,   // Yellow - proceed with caution
    Critical,  // Red - strongly consider alternative action
}
```

#### **UI Integration**

```typescript
// src/components/QualityMeter.tsx

export function QualityMeter({ metrics, warnings }: QualityMeterProps) {
  return (
    <div className="quality-meter">
      <div className="quality-score">
        <QualityGauge score={metrics.estimatedQuality} />
        <span className="quality-label">
          {getQualityLabel(metrics.estimatedQuality)}
        </span>
      </div>
      
      {warnings.length > 0 && (
        <div className="quality-warnings">
          <h4>Audio Quality Analysis</h4>
          {warnings.map((warning, idx) => (
            <div 
              key={idx} 
              className={`warning warning-${warning.severity.toLowerCase()}`}
            >
              <div className="warning-header">
                <WarningIcon severity={warning.severity} />
                <strong>{warning.title}</strong>
              </div>
              <p className="warning-message">{warning.message}</p>
              {warning.recommendation && (
                <p className="warning-recommendation">
                  <strong>💡 Tip:</strong> {warning.recommendation}
                </p>
              )}
            </div>
          ))}
        </div>
      )}
      
      <div className="quality-details">
        <DetailRow label="Noise Floor" value={`${metrics.noiseFloorDb.toFixed(1)} dB`} />
        <DetailRow label="Dynamic Range" value={`${metrics.dynamicRangeDb.toFixed(1)} dB`} />
        <DetailRow label="Bandwidth" value={`${(metrics.bandwidthHz / 1000).toFixed(1)} kHz`} />
        <DetailRow 
          label="Clipping" 
          value={metrics.hasClipping ? "⚠️ Detected" : "✓ None"} 
          alert={metrics.hasClipping}
        />
      </div>
    </div>
  );
}
```

---

### 9.3 Analytics & Telemetry

#### **Event Definitions**

```rust
// src/analytics/events.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type", rename_all = "snake_case")]
pub enum AnalyticsEvent {
    AppLaunched {
        version: String,
        platform: String,
        first_launch: bool,
    },
    FileImported {
        format: String,
        duration_sec: u64,
        file_size_mb: f64,
        has_audio: bool,
        has_video: bool,
    },
    QualityAnalyzed {
        noise_floor_db: f32,
        has_clipping: bool,
        quality_rating: String,
        warnings_count: u32,
    },
    PreviewGenerated {
        job_id: String,
        processing_time_ms: u64,
        preset_name: String,
    },
    ABComparisonToggled {
        job_id: String,
        from_version: String,
        to_version: String,
    },
    IntensityAdjusted {
        job_id: String,
        old_value: f32,
        new_value: f32,
    },
    ExportStarted {
        job_id: String,
        preset_name: String,
        intensity: f32,
        platform: String,
        audio_codec: String,
    },
    ExportCompleted {
        job_id: String,
        processing_time_sec: u64,
        input_size_mb: f64,
        output_size_mb: f64,
        success: bool,
    },
    ExportFailed {
        job_id: String,
        stage: String,
        error_type: String,
        error_message: String,
    },
    PresetChanged {
        job_id: String,
        from_preset: String,
        to_preset: String,
    },
    UpgradePromptShown {
        exports_remaining: u32,
        trigger: String,
    },
    UpgradeButtonClicked {
        source: String,
    },
    LicenseActivated {
        tier: String,
        is_trial: bool,
    },
}

impl AnalyticsEvent {
    pub fn timestamp(&self) -> SystemTime {
        SystemTime::now()
    }
    
    pub fn session_id(&self) -> String {
        // Generate session ID on app launch, persist for session duration
        get_current_session_id()
    }
}
```

#### **Analytics Backend**

```rust
// src/analytics/tracker.rs

pub struct AnalyticsTracker {
    enabled: Arc<Mutex<bool>>,
    event_queue: Arc<Mutex<Vec<AnalyticsEvent>>>,
    log_file: PathBuf,
}

impl AnalyticsTracker {
    pub fn new(settings: &UserSettings) -> Self {
        let log_file = dirs::data_dir()
            .unwrap()
            .join("audio-cleaner")
            .join("analytics")
            .join("events.jsonl");
        
        Self {
            enabled: Arc::new(Mutex::new(settings.analytics_enabled)),
            event_queue: Arc::new(Mutex::new(Vec::new())),
            log_file,
        }
    }
    
    pub async fn track(&self, event: AnalyticsEvent) {
        let enabled = self.enabled.lock().await;
        if !*enabled {
            return;
        }
        
        // 1. Add to in-memory queue
        let mut queue = self.event_queue.lock().await;
        queue.push(event.clone());
        
        // 2. Write to local log (JSONL format)
        self.write_to_log(&event).await;
        
        // 3. Batch send to server (if online and opted-in)
        // For MVP: Local-only logging
        // For v2: Optional telemetry server
    }
    
    async fn write_to_log(&self, event: &AnalyticsEvent) {
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file)
        {
            if let Ok(json) = serde_json::to_string(event) {
                let _ = writeln!(file, "{}", json);
            }
        }
    }
    
    pub async fn flush(&self) {
        // Flush queue to disk, for use on app shutdown
        let queue = self.event_queue.lock().await;
        for event in queue.iter() {
            self.write_to_log(event).await;
        }
    }
}
```

#### **Privacy Considerations**

**Data Collected:**
- ✅ App usage patterns (features used, export counts)
- ✅ Performance metrics (processing times, file sizes)
- ✅ Error events (for debugging)
- ✅ Quality metrics (aggregate data only)

**Data NOT Collected:**
- ❌ File names or paths
- ❌ Audio content
- ❌ User names or emails (unless license key)
- ❌ IP addresses (if telemetry server is added)

**User Control:**
```typescript
// src/components/SettingsPanel.tsx

<ToggleSetting
  label="Help improve Audio Cleaner"
  description="Share anonymous usage data to help us improve the app. No personal information or file content is collected."
  enabled={settings.analyticsEnabled}
  onChange={(enabled) => updateSettings({ analyticsEnabled: enabled })}
/>
```

#### **Command Integration**

```rust
// Every Tauri command should track relevant events

#[tauri::command]
async fn probe_media_file(
    state: State<'_, AppState>,
    path: String,
) -> Result<MediaFile, String> {
    let media = probe_media_internal(&path)?;
    
    // Track event
    state.analytics.track(AnalyticsEvent::FileImported {
        format: media.format.to_string(),
        duration_sec: media.duration_ms / 1000,
        file_size_mb: get_file_size_mb(&path),
        has_audio: !media.audio_streams.is_empty(),
        has_video: !media.video_streams.is_empty(),
    }).await;
    
    Ok(media)
}
```

---

### 9.4 Auto-Update System

#### **Update Check Strategy**

```rust
// src/updates/checker.rs

pub struct UpdateChecker {
    current_version: Version,
    last_check: Option<SystemTime>,
    check_interval_hours: u64,
}

impl UpdateChecker {
    pub fn new(current_version: &str) -> Self {
        Self {
            current_version: Version::parse(current_version).unwrap(),
            last_check: None,
            check_interval_hours: 24,
        }
    }
    
    pub async fn check_for_updates(&mut self) -> Result<Option<UpdateInfo>, String> {
        // Don't check too frequently
        if let Some(last) = self.last_check {
            let elapsed = last.elapsed().unwrap_or_default();
            if elapsed.as_secs() < self.check_interval_hours * 3600 {
                return Ok(None);
            }
        }
        
        // Query GitHub Releases API
        let latest = self.fetch_latest_release().await?;
        self.last_check = Some(SystemTime::now());
        
        // Compare versions
        if latest.version > self.current_version {
            Ok(Some(latest))
        } else {
            Ok(None)
        }
    }
    
    async fn fetch_latest_release(&self) -> Result<UpdateInfo, String> {
        let response = reqwest::get(
            "https://api.github.com/repos/youruser/audio-cleaner/releases/latest"
        )
        .await
        .map_err(|e| format!("Update check failed: {}", e))?;
        
        let release: GitHubRelease = response.json().await
            .map_err(|e| format!("Failed to parse release: {}", e))?;
        
        Ok(UpdateInfo {
            version: Version::parse(&release.tag_name.trim_start_matches('v'))?,
            download_url: self.get_platform_asset_url(&release)?,
            release_notes: release.body,
            published_at: release.published_at,
        })
    }
}

#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub version: Version,
    pub download_url: String,
    pub release_notes: String,
    pub published_at: String,
}
```

#### **Tauri Updater Integration**

```rust
// src/main.rs

use tauri::Manager;
use tauri::updater::UpdaterBuilder;

#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle();
            
            // Check for updates on startup
            tokio::spawn(async move {
                check_and_prompt_update(handle).await;
            });
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn check_and_prompt_update(app_handle: tauri::AppHandle) {
    // Wait 5 seconds after launch to avoid blocking startup
    tokio::time::sleep(Duration::from_secs(5)).await;
    
    match UpdaterBuilder::new().check().await {
        Ok(Some(update)) => {
            // Emit event to frontend
            app_handle.emit_all("update-available", UpdateAvailablePayload {
                version: update.version.to_string(),
                release_notes: update.body.clone(),
                download_size_mb: update.download_size_mb(),
            }).unwrap();
        }
        Ok(None) => {
            // No update available
        }
        Err(e) => {
            eprintln!("Update check failed: {}", e);
        }
    }
}
```

#### **User Notification UI**

```typescript
// src/components/UpdateNotification.tsx

export function UpdateNotification() {
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [isDownloading, setIsDownloading] = useState(false);
  const [downloadProgress, setDownloadProgress] = useState(0);
  
  useEffect(() => {
    const unlisten = listen<UpdateInfo>('update-available', (event) => {
      setUpdateInfo(event.payload);
    });
    
    return () => {
      unlisten.then(fn => fn());
    };
  }, []);
  
  const handleUpdate = async () => {
    setIsDownloading(true);
    
    try {
      // Tauri handles download and installation
      await invoke('install_update');
      
      // Prompt user to restart
      showRestartPrompt();
    } catch (error) {
      console.error('Update failed:', error);
      showError('Failed to download update. Please try again later.');
    } finally {
      setIsDownloading(false);
    }
  };
  
  if (!updateInfo) return null;
  
  return (
    <div className="update-banner">
      <div className="update-icon">🎉</div>
      <div className="update-content">
        <strong>Update Available: v{updateInfo.version}</strong>
        <p>{updateInfo.releaseNotes.split('\n')[0]}</p>
      </div>
      <div className="update-actions">
        {isDownloading ? (
          <ProgressBar progress={downloadProgress} label="Downloading..." />
        ) : (
          <>
            <button onClick={handleUpdate} className="btn-primary">
              Update Now
            </button>
            <button onClick={() => setUpdateInfo(null)} className="btn-link">
              Remind me later
            </button>
          </>
        )}
      </div>
    </div>
  );
}
```

#### **Tauri Configuration**

```json
// src-tauri/tauri.conf.json

{
  "tauri": {
    "updater": {
      "active": true,
      "endpoints": [
        "https://github.com/youruser/audio-cleaner/releases/latest/download/latest.json"
      ],
      "dialog": false,
      "pubkey": "YOUR_PUBLIC_KEY_HERE"
    }
  }
}
```

#### **Rollback Mechanism**

```rust
// For emergency rollback

impl UpdateChecker {
    pub async fn rollback_to_previous(&self) -> Result<(), String> {
        // Tauri keeps previous version for rollback
        // In case of critical bug:
        
        let backup_path = self.get_backup_path()?;
        if backup_path.exists() {
            // Restore previous binary
            std::fs::copy(&backup_path, &self.get_current_binary_path())?;
            Ok(())
        } else {
            Err("No backup version available".into())
        }
    }
}
```

---

### 9.5 Crash Reporting

#### **Panic Handler**

```rust
// src/crash_reporting/handler.rs

use std::panic;
use backtrace::Backtrace;

pub fn setup_crash_handler(app_handle: tauri::AppHandle) {
    panic::set_hook(Box::new(move |panic_info| {
        let backtrace = Backtrace::new();
        
        let crash_report = CrashReport {
            timestamp: SystemTime::now(),
            panic_message: panic_info.to_string(),
            backtrace: format!("{:?}", backtrace),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            os: std::env::consts::OS.to_string(),
            os_version: get_os_version(),
            last_operation: get_last_operation(),
        };
        
        // Write to crash log
        save_crash_report(&crash_report);
        
        // Show user-friendly dialog
        show_crash_dialog(&crash_report);
        
        // Resume panic (let app crash gracefully)
        std::process::exit(1);
    }));
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CrashReport {
    pub timestamp: SystemTime,
    pub panic_message: String,
    pub backtrace: String,
    pub app_version: String,
    pub os: String,
    pub os_version: String,
    pub last_operation: Option<String>,
}

fn save_crash_report(report: &CrashReport) {
    let crash_dir = dirs::data_dir()
        .unwrap()
        .join("audio-cleaner")
        .join("logs")
        .join("crashes");
    
    std::fs::create_dir_all(&crash_dir).ok();
    
    let filename = format!(
        "crash_{}.json",
        report.timestamp.duration_since(UNIX_EPOCH).unwrap().as_secs()
    );
    
    let path = crash_dir.join(filename);
    
    if let Ok(json) = serde_json::to_string_pretty(report) {
        std::fs::write(path, json).ok();
    }
}
```

#### **Operation Tracking**

```rust
// Track what the app was doing when it crashed

pub struct OperationTracker {
    current_operation: Arc<Mutex<Option<String>>>,
}

impl OperationTracker {
    pub fn set(&self, operation: String) {
        if let Ok(mut op) = self.current_operation.lock() {
            *op = Some(operation);
        }
    }
    
    pub fn clear(&self) {
        if let Ok(mut op) = self.current_operation.lock() {
            *op = None;
        }
    }
    
    pub fn get(&self) -> Option<String> {
        self.current_operation.lock().ok().and_then(|op| op.clone())
    }
}

// Usage in commands
#[tauri::command]
async fn start_processing(
    state: State<'_, AppState>,
    job_id: String,
) -> Result<(), String> {
    state.operations.set(format!("Processing job {}", job_id));
    
    let result = process_job_internal(&job_id).await;
    
    state.operations.clear();
    
    result
}
```

#### **User-Facing Crash Dialog**

```typescript
// src/components/CrashDialog.tsx

export function CrashDialog({ crashReport }: CrashDialogProps) {
  const [userDescription, setUserDescription] = useState('');
  const [includeSystemInfo, setIncludeSystemInfo] = useState(true);
  
  const handleSubmit = async () => {
    const report = {
      ...crashReport,
      userDescription,
      includeSystemInfo,
    };
    
    try {
      // Submit to crash reporting service (e.g., Sentry, custom backend)
      await invoke('submit_crash_report', { report });
      showSuccess('Thank you for helping us improve Audio Cleaner!');
    } catch (error) {
      showError('Failed to submit crash report');
    }
  };
  
  return (
    <Dialog title="Audio Cleaner Crashed" icon="error">
      <p>
        We're sorry, Audio Cleaner encountered an unexpected error and needs to close.
      </p>
      
      <details>
        <summary>Technical Details</summary>
        <pre className="crash-details">{crashReport.panicMessage}</pre>
      </details>
      
      <textarea
        placeholder="What were you doing when the crash happened? (optional)"
        value={userDescription}
        onChange={(e) => setUserDescription(e.target.value)}
        rows={4}
      />
      
      <label>
        <input
          type="checkbox"
          checked={includeSystemInfo}
          onChange={(e) => setIncludeSystemInfo(e.target.checked)}
        />
        Include system information (OS, app version) to help us fix this bug
      </label>
      
      <div className="dialog-actions">
        <button onClick={handleSubmit} className="btn-primary">
          Send Crash Report
        </button>
        <button onClick={handleClose} className="btn-secondary">
          Close
        </button>
      </div>
    </Dialog>
  );
}
```

#### **Privacy & Data Collection**

**Crash Report Contents:**
- ✅ Panic message and backtrace
- ✅ App version, OS, OS version
- ✅ Last operation (e.g., "Processing job abc123")
- ✅ User description (if provided)
- ❌ No file paths or names
- ❌ No audio content
- ❌ No user credentials

**Submission is Optional:**
- User must click "Send Crash Report"
- Can review technical details before sending
- Can opt out of system info inclusion

---

## RISKS & MITIGATIONS

| **Risk** | **Mitigation** |
|---------|---------------|
| FFmpeg licensing issues | Use LGPL-compliant build, document clearly |
| DeepFilterNet causes robotic voices | Conservative default preset, easy rollback with intensity slider |
| Poor dereverb on bad rooms | Quality meter warns users, consider commercial model later |
| Slow processing on old hardware | Offer "Fast" mode, progress indication, GPU acceleration (future) |
| A/V sync bugs | Extensive testing, validation checks, regression suite |
| License piracy | Machine fingerprinting, online validation, acceptable leakage |

---

### Critical Files for Implementation

1. `/Users/artinigam/practice code/weekend-project/src-tauri/src/main.rs`
2. `/Users/artinigam/practice code/weekend-project/src-tauri/src/processing/pipeline.rs`
3. `/Users/artinigam/practice code/weekend-project/src-tauri/src/enhancement/deepfilternet.rs`
4. `/Users/artinigam/practice code/weekend-project/src-tauri/src/remux/video.rs`
5. `/Users/artinigam/practice code/weekend-project/src/App.tsx`
