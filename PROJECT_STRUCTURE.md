# TTRPG Companion - Project Structure & Initialization Plan

## Context

The TTRPG Companion is a real-time audio companion app for tabletop RPG sessions. It needs to be built as a Tauri desktop application (Rust backend + web frontend). This plan outlines the project structure and initialization steps.

## Project Structure

```
ttrpg_companion/
├── src/                      # Rust source code (Tauri backend)
│   ├── main.rs              # Entry point
│   ├── audio/               # Audio I/O module
│   │   ├── mod.rs
│   │   ├── capture.rs       # Microphone input (cpal/rodio)
│   │   └── playback.rs      # Audio output
│   ├── dsp/                 # Digital Signal Processing
│   │   ├── mod.rs
│   │   └── processing.rs    # FFT, normalization
│   ├── inference/           # ML inference engine
│   │   ├── mod.rs
│   │   ├── whisper.rs       # Speech-to-text
│   │   └── emotion.rs       # Vocal mood analysis
│   ├── orchestrator/        # State machine management
│   │   ├── mod.rs
│   │   └── state.rs         # IDLE -> RECORDING -> THINKING
│   └── commands/            # Tauri commands
│       ├── mod.rs
│       └── session.rs       # Session control
├── src-tauri/               # Tauri configuration
│   ├── Cargo.toml           # Rust dependencies
│   ├── tauri.conf.json      # Tauri app config
│   ├── icons/
│   │   └── icons.app.png   # App icons
│   └── build.rs
├── src-web/                 # Frontend (web UI)
│   ├── index.html
│   ├── main.js
│   ├── styles.css
│   └── components/          # UI components
├── assets/                  # Bundled audio content
│   ├── music/
│   └── sfx/
├── SPEC.md                  # Project specification
├── README.md
└── Cargo.lock
```

## Implementation Steps

### 1. Initialize Tauri Project
- Create `src-tauri/Cargo.toml` with all required dependencies
- Create `tauri.conf.json` with app metadata, window config, system tray
- Set up logging with `tracing`

### 2. Create Rust Backend Modules
- **audio/capture.rs**: Microphone input using `cpal`
- **audio/playback.rs**: Audio output using `rodio`
- **dsp/processing.rs**: Audio preprocessing with `symphonia`/`hound`
- **inference/whisper.rs**: Whisper.cpp bindings for STT
- **inference/emotion.rs**: Emotion2vec integration
- **orchestrator/state.rs**: Tokio-based state machine
- **commands/**: Tauri IPC commands

### 3. Set Up Frontend
- Basic HTML/CSS/JS setup for pre-session configuration
- System tray integration for session mode

### 4. Dependencies to Add (Cargo.toml)
```toml
cpal = "0.15"           # Audio I/O
rodio = "0.19"          # Simple audio playback
symphonia = "0.5"       # Audio decoding
hound = "3.5"           # WAV handling
tokio = { version = "1", features = ["full"] }  # Async runtime
tracing = "0.1"         # Logging
tracing-subscriber = "0.3"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

## Verification

- Build the empty shell project: `cd src-tauri && cargo build`
- Verify Tauri dev server starts: `npm run tauri dev`
- Confirm system tray icon appears
