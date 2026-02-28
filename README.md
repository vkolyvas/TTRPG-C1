# TTRPG Companion

Real-time audio companion app for tabletop RPG sessions. Runs in the background on the GM's laptop and automatically matches ambient music and sound effects to the current moment through keyword detection and vocal mood analysis.

## Installation

### Prerequisites

Before installing, ensure you have:

- **Rust** (1.70 or later)
- **Node.js** (18 or later)
- **Git**

### Quick Install (All Platforms)

```bash
# 1. Clone the repository
git clone https://github.com/vkolyvas/TTRPG-C1.git
cd TTRPG-C1

# 2. Install dependencies
npm install

# 3. Run the app
npm run tauri dev
```

---

### Platform-Specific Setup

#### macOS

**Install Rust:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

**Install Node.js:**
```bash
# Using Homebrew (recommended)
brew install node

# Or using nvm
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 20
nvm use 20
```

**Install dependencies:**
```bash
npm install
```

**Run the app:**
```bash
npm run tauri dev
```

**Build for production:**
```bash
npm run tauri build
```

The built app will be at: `src-tauri/target/release/bundle/dmg/`

---

#### Linux (Ubuntu/Debian)

**Install Rust:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

**Install Node.js:**
```bash
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt-get install -y nodejs
```

**Install system dependencies:**
```bash
# Ubuntu/Debian
sudo apt-get install -y \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev

# Fedora
sudo dnf install -y \
    gcc-c++ \
    curl \
    wget \
    file \
    openssl-devel \
    gtk3-devel \
    libappindicator-gtk3-devel
```

**Install dependencies:**
```bash
npm install
```

**Run the app:**
```bash
npm run tauri dev
```

**Build for production:**
```bash
npm run tauri build
```

The built packages will be at:
- Debian: `src-tauri/target/release/bundle/deb/`
- AppImage: `src-tauri/target/release/bundle/appimage/`
- RPM: `src-tauri/target/release/bundle/rpm/`

---

#### Windows

**Install Rust:**
1. Download and run: https://rustup.rs
2. Select "Default installation"
3. Restart your terminal after installation

**Install Node.js:**
1. Download from: https://nodejs.org (LTS version 20)
2. Or use Winget: `winget install OpenJS.NodeJS.LTS`

**Install Git (if not installed):**
```powershell
winget install Git.Git
```

**Install dependencies:**
```powershell
# Open PowerShell or Command Prompt
cd path\to\TTRPG-C1
npm install
```

**Run the app:**
```powershell
npm run tauri dev
```

**Build for production:**
```powershell
npm run tauri build
```

The built installer will be at: `src-tauri/target/release/bundle/msi/`

---

### Pre-built Packages

If you don't want to build from source, download the latest release from GitHub:

**Download:** https://github.com/vkolyvas/TTRPG-C1/releases/latest

| Platform | Package | Installation |
|----------|---------|--------------|
| Linux (Debian/Ubuntu) | `.deb` | `sudo dpkg -i TTRPG-Companion_0.1.0_amd64.deb` |
| Linux (Fedora/RHEL) | `.rpm` | `sudo rpm -i TTRPG-Companion-0.1.0-1.x86_64.rpm` |
| macOS | `.dmg` | Build from source (see above) |
| Windows | `.exe` or `.msi` | Build from source (see above) |

**Quick install on Linux:**
```bash
# Debian/Ubuntu
wget https://github.com/vkolyvas/TTRPG-C1/releases/download/v0.1.0/TTRPG-Companion_0.1.0_amd64.deb
sudo dpkg -i TTRPG-Companion_0.1.0_amd64.deb

# Fedora/RHEL
wget https://github.com/vkolyvas/TTRPG-C1/releases/download/v0.1.0/TTRPG-Companion-0.1.0-1.x86_64.rpm
sudo rpm -i TTRPG-Companion-0.1.0-1.x86_64.rpm
```

**Note:** macOS and Windows builds require building on those platforms. Clone the repo and run `npm run tauri build` on your Mac/Windows machine.

---

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
- Node.js (18+)
- Platform-specific build tools (see platform setup above)

### Build Commands

```bash
# Install dependencies
npm install

# Run in development mode (with hot-reload)
npm run tauri dev

# Build for production
npm run tauri build
```

### Project Structure

```
ttrpg_companion/
├── src-tauri/           # Rust backend (Tauri)
├── src-svelte/          # Frontend (Svelte 5 + Tailwind)
├── assets/              # Bundled audio (music, sfx)
├── SPEC.md              # Project specification
└── README.md            # This file
```
