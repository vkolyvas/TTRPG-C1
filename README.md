# TTRPG Companion

Real-time audio companion app for tabletop RPG sessions. Runs in the background on the GM's laptop and automatically matches ambient music and sound effects to the current moment through keyword detection and vocal mood analysis.

## Architecture

| Module | Purpose | Recommended Open-Source Tool |
|--------|---------|------------------------------|
| Audio I/O | Capturing mic input and playing back processed sound. | cpal (Low-level) or Rodio (Simpler) |
| DSP (Digital Signal Processing) | Cleaning audio, normalization, or FFT (Visualizers). | symphonia (Decoding) / hound (WAV) |
| Inference Engine | Running the ML models (STT, TTS, or LLM) locally. | Candle (by Hugging Face) |
| Orchestrator | Managing the state machine (e.g., "IDLE" -> "RECORDING" -> "THINKING"). | Tokio (Async runtime) |

## Features

- Mood-aware music playback that responds to what's happening at the table
- Keyword detection for automatic track/mood shifting via on-device Whisper
- Vocal mood analysis for enhanced scene detection
- Smart crossfading between tracks
- Sound effects triggered by keywords
- Voice training for speaker profile creation
- Bring-your-own-music (BYOM) support with automatic mood tagging

## Tech Stack

- **Framework:** Tauri (Rust-based, lightweight, native binaries)
- **Frontend:** Web-based setup UI
- **Speech-to-text:** whisper.cpp
- **Audio Engine:** FMOD or platform-native (Core Audio / WASAPI)
- **Vocal Analysis:** emotion2vec+

## License

See individual component licenses for bundled content.

## Building

### Prerequisites
- Rust (1.70+)
- Node.js 18+
- Tauri CLI

### Build Commands

```bash
# Install dependencies
cd src-tauri && cargo build

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

### Project Structure

```
ttrpg_companion/
├── src-tauri/           # Rust backend (Tauri)
├── src-web/             # Frontend (HTML/CSS/JS)
├── assets/              # Bundled audio (music, sfx)
├── SPEC.md              # Project specification
└── README.md            # This file
```
