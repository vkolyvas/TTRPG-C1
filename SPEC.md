# TTRPG Companion - Project Specification

## Overview
**Project Name:** TTRPG Companion
**Type:** Desktop Application (Tauri - Rust + Web Frontend)
**Core Feature:** Real-time audio companion for tabletop RPG sessions
**Target Users:** Tabletop RPG Game Masters and players

## Technical Architecture

### Backend (Rust/Tauri)
- **Audio Input:** cpal for cross-platform microphone capture
- **Audio Output:** rodio for sound playback
- **DSP:** Custom processing (normalization, filtering, resampling)
- **Inference:** whisper.cpp for speech-to-text, emotion2vec for vocal mood analysis
- **State Management:** Tokio-based async state machine

### Frontend (Web)
- Vanilla HTML/CSS/JavaScript
- System tray integration
- Real-time status updates via Tauri IPC

## Core Features

### 1. Audio Capture
- Microphone input with device selection
- Real-time audio buffering
- Configurable sample rate (default: 16kHz)
- Noise gate and normalization

### 2. Speech-to-Text (Whisper)
- Local inference (no cloud dependency)
- Multi-language support
- Transcription with confidence scores

### 3. Emotion Analysis
- Vocal mood detection (neutral, happy, sad, angry, fearful, surprised, disgusted)
- Real-time processing
- Confidence scores per emotion

### 4. Session Management
- State machine: IDLE -> RECORDING -> PROCESSING -> IDLE
- System tray controls
- Session logging

## UI/UX Specification

### Layout
- **Header:** App title and subtitle
- **Config Section:** Device selection, feature toggles
- **Session Section:** Recording controls with status indicator
- **Results Section:** Transcription and emotion output
- **Log Section:** Session activity log

### Visual Design
- **Primary Color:** Indigo (#6366f1)
- **Status Colors:**
  - Idle: Gray (#64748b)
  - Recording: Red (#ef4444) with pulse animation
  - Processing: Amber (#f59e0b) with pulse animation
  - Success: Green (#22c55e)
- **Typography:** System fonts (San Francisco, Segoe UI, Roboto)
- **Spacing:** 8px base unit, 16-24px padding

## System Integration

### Window Management
- Standard window frame with native controls
- Minimize to system tray
- Remember window position

### System Tray
- Tray icon with context menu
- Start/Stop session from tray
- Quit application

## Data Flow

```
Microphone -> AudioCapture -> DSP Processing -> Inference Engine -> Results
                                        |
                                    AudioBuffer
                                        |
                                    Whisper/STT -> Transcription
                                        |
                                    Emotion2Vec -> Emotion Analysis
```

## Configuration

### Default Settings
- Sample Rate: 16000 Hz
- Buffer Size: 100ms
- Silence Threshold: 0.01
- Transcription: Enabled
- Emotion Analysis: Enabled

## Dependencies

### Rust Crates
- tauri: 1.6
- cpal: 0.15
- rodio: 0.19
- symphonia: 0.5
- tokio: 1.x
- tracing: 0.1

## File Structure

```
ttrpg_companion/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── audio/
│   │   │   ├── capture.rs
│   │   │   └── playback.rs
│   │   ├── dsp/
│   │   │   └── processing.rs
│   │   ├── inference/
│   │   │   ├── whisper.rs
│   │   │   └── emotion.rs
│   │   ├── orchestrator/
│   │   │   └── state.rs
│   │   └── commands/
│   │       └── session.rs
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── build.rs
├── src-web/
│   ├── index.html
│   ├── main.js
│   └── styles.css
├── SPEC.md
└── README.md
```

## Future Enhancements

1. **Audio Playback:** Background music and sound effects
2. **Multiple Languages:** Expanded language support for transcription
3. **Custom Prompts:** Context-aware prompts for game-specific scenarios
4. **Plugin System:** Extensible AI backend
5. **Recording:** Save sessions for later review

## Acceptance Criteria

- [x] Project structure created
- [x] Tauri backend compiles without errors
- [x] Basic audio capture functional
- [x] Frontend loads and displays UI
- [x] System tray integration working
- [x] Session state machine operational
- [x] Transcription placeholder implemented
- [x] Emotion analysis placeholder implemented
