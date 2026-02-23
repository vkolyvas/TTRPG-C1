# TTRPG Companion - Claude Code Instructions

*Read this entire file before writing any code.*

This is the authoritative source for every architectural decision, constraint,
schema, API contract, and convention in this codebase. Decisions marked
*[LOCKED]* are not open for reconsideration without a documented blocking
technical reason. Decisions marked *[OPEN]* are tracked but unresolved —
raise an issue before acting on them.

When product decisions change: update this file first.
When implementation diverges from this file: update this file or revert the
implementation. Never silently let them drift apart.

---

## Project Overview

**Project Name:** TTRPG Companion
**Type:** Desktop Application (Tauri 2.x + Rust + Vanilla Web Frontend)
**Core Purpose:** Real-time audio companion for tabletop RPG sessions - provides voice transcription and emotion analysis for NPC interactions

---

## Technology Stack *[LOCKED]*

| Layer | Technology | Version |
|-------|------------|---------|
| Framework | Tauri | 2.x |
| Backend | Rust | 2021 edition |
| Frontend | Vanilla HTML/CSS/JS | ES6+ |
| Audio Input | cpal | 0.15 |
| Audio Output | rodio | 0.19 |
| Audio Processing | symphonia, hound | 0.5, 3.5 |
| Async Runtime | tokio | full features |
| Logging | tracing | latest |
| STT (optional) | whisper-rs | 0.15 (feature-gated) |

---

## Architecture

### Directory Structure *[LOCKED]*

```
ttrpg_companion/
├── src-tauri/           # Rust backend
│   ├── src/
│   │   ├── main.rs      # Entry point
│   │   ├── lib.rs       # Application setup, logging, system tray
│   │   ├── audio/       # Audio I/O module
│   │   │   ├── mod.rs
│   │   │   ├── capture.rs    # Microphone input (cpal)
│   │   │   └── playback.rs   # Audio output (rodio)
│   │   ├── dsp/         # Digital Signal Processing
│   │   │   ├── mod.rs
│   │   │   └── processing.rs # FFT, normalization, filters
│   │   ├── inference/   # ML inference
│   │   │   ├── mod.rs
│   │   │   ├── whisper.rs    # Speech-to-text
│   │   │   └── emotion.rs    # Vocal mood analysis
│   │   ├── orchestrator/ # State machine
│   │   │   ├── mod.rs
│   │   │   └── state.rs
│   │   └── commands/    # Tauri IPC commands
│   │       ├── mod.rs
│   │       └── session.rs
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── build.rs
├── src-web/             # Frontend
│   ├── index.html
│   ├── main.js
│   └── styles.css
├── assets/              # Bundled audio (music/, sfx/)
├── SPEC.md
└── README.md
```

### Module Responsibilities

| Module | Responsibility |
|--------|----------------|
| `audio/capture` | Microphone input using cpal |
| `audio/playback` | Sound playback using rodio |
| `dsp/processing` | Audio preprocessing (normalize, filter, resample) |
| `inference/whisper` | Speech-to-text transcription |
| `inference/emotion` | Vocal mood detection (7 emotions) |
| `orchestrator/state` | Session state machine (Idle -> Recording -> Processing) |
| `commands/session` | Tauri IPC command handlers |
| `src-web/*` | UI logic and Tauri communication |

---

## State Machine *[LOCKED]*

The application follows a strict state machine:

```
Idle -> Recording -> Processing -> Idle
```

- **Idle:** No active session, awaiting user start
- **Recording:** Capturing audio from microphone
- **Processing:** Running inference on captured audio

---

## API Contracts

### Tauri IPC Commands *[LOCKED]*

All commands use `window.__TAURI__.invoke()`:

| Command | Parameters | Returns |
|---------|------------|---------|
| `start_session` | `{ device_id: string, enable_transcription: bool, enable_emotion: bool }` | `{ success: bool, error?: string }` |
| `stop_session` | none | `{ success: bool, error?: string }` |
| `get_session_status` | none | `{ status: "idle" \| "recording" \| "processing", ... }` |
| `get_available_devices` | none | `[{ id: string, name: string, is_default: bool }]` |

### Event Emissions (Frontend Listeners)

| Event | Payload |
|-------|---------|
| `session-status-changed` | `{ status: string }` |
| `transcription-result` | `{ text: string, timestamp: number }` |
| `emotion-result` | `{ emotion: string, confidence: number }` |
| `log-message` | `{ level: string, message: string }` |

---

## Code Conventions

### Rust

- Module organization: One file per module at `src-tauri/src/`, submodules in directories
- Error handling: Use `Result<T, E>` with meaningful error types
- Logging: Use `tracing` crate with `info!`, `warn!`, `error!` macros
- Async: Use `tokio` for all async operations

### Frontend (JavaScript)

- Use ES6+ features (const/let, arrow functions, async/await)
- Use `window.__TAURI__` for all Tauri communication
- State management: Simple state object in main.js
- DOM manipulation: Vanilla JS (no frameworks)

### CSS

- Primary color: Indigo (#6366f1)
- Follow existing style patterns in `styles.css`
- Use CSS custom properties for theming

---

## Current Implementation Status

### Implemented *[LOCKED]*

- [x] Audio device listing and selection
- [x] Audio capture infrastructure (cpal)
- [x] Audio playback infrastructure (rodio)
- [x] DSP preprocessing functions with unit tests
- [x] Session state machine
- [x] System tray with menu
- [x] Frontend UI with session controls
- [x] Logging system (file + console)
- [x] Whisper STT module (feature-gated, requires cmake to enable)

### Placeholder/Stubs *[LOCKED]*

- [ ] (none - all core modules implemented)

---

## Open Decisions *[OPEN]*

| ID | Decision | Status | Notes |
|----|----------|--------|-------|
| O-003 | Audio processing pipeline buffering | Open | Chunk size, overlap strategy |
| O-004 | Frontend framework migration | Open | Consider React/Vue if complexity grows |

---

## Build & Run Commands

```bash
# Install dependencies
npm install
cargo build

# Development
npm run tauri dev

# Production build
npm run tauri build
```

---

## Configuration Files

| File | Purpose |
|------|---------|
| `src-tauri/tauri.conf.json` | Tauri app configuration (window, tray, devtools) |
| `src-tauri/Cargo.toml` | Rust dependencies |
| `package.json` | npm dependencies and scripts |

---

## Important Notes

1. **System Tray:** App minimizes to system tray; tray icon provides quick actions
2. **Audio Format:** Internal processing uses 16kHz mono for inference compatibility
3. **Event-Driven:** Frontend responds to events emitted from Rust backend
4. **No Database:** Currently stateless (no persistent storage)
5. **Whisper STT Feature:** Disabled by default. Enable with `cargo build --features whisper`. Requires cmake and C++ compiler to build whisper-rs. When disabled, returns placeholder text.
6. **Model Download:** Whisper models (tiny.bin ~75MB) can be downloaded from: https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en.bin
7. **Emotion Analysis:** Feature-based heuristic analysis using acoustic features (RMS, ZCR, pitch, energy variance). Supports 7 emotions: neutral, happy, sad, angry, fearful, surprised, disgusted.

---

*Last updated: 2026-02-23*
