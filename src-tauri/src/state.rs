//! Application state management

use crate::detection::fsm::DetectionMode;
use crate::db::DbPool;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Session states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionState {
    Idle,
    Recording,
    Processing,
    Error,
}

impl std::fmt::Display for SessionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionState::Idle => write!(f, "idle"),
            SessionState::Recording => write!(f, "recording"),
            SessionState::Processing => write!(f, "processing"),
            SessionState::Error => write!(f, "error"),
        }
    }
}

/// Application mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AppMode {
    /// Autonomous mode - auto-triggers based on detection
    ModeA,
    /// Collaborative mode - requires GM confirmation
    ModeB,
}

impl Default for AppMode {
    fn default() -> Self {
        AppMode::ModeA
    }
}

impl std::fmt::Display for AppMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppMode::ModeA => write!(f, "autonomous"),
            AppMode::ModeB => write!(f, "collaborative"),
        }
    }
}

/// Session configuration
#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub sample_rate: u32,
    pub buffer_size_ms: u32,
    pub silence_threshold: f32,
    pub enable_transcription: bool,
    pub enable_emotion_analysis: bool,
    pub enable_vad: bool,
    pub enable_speaker_verification: bool,
    pub detection_mode: DetectionMode,
    pub crossfade_duration_ms: u32,
    pub sfx_volume: f32,
    pub music_volume: f32,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            sample_rate: 16000,
            buffer_size_ms: 100,
            silence_threshold: 0.01,
            enable_transcription: true,
            enable_emotion_analysis: true,
            enable_vad: true,
            enable_speaker_verification: false,
            detection_mode: DetectionMode::Autonomous,
            crossfade_duration_ms: 2000,
            sfx_volume: 0.8,
            music_volume: 0.6,
        }
    }
}

/// Currently playing track info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayingTrack {
    pub id: String,
    pub name: String,
    pub genre: String,
    pub is_looping: bool,
}

/// Application state shared across Tauri commands
pub struct AppState {
    /// Current session state
    pub session_state: RwLock<SessionState>,
    /// Application mode (A or B)
    pub app_mode: RwLock<AppMode>,
    /// Session configuration
    pub config: RwLock<SessionConfig>,
    /// Audio buffer for processing (thread-safe)
    pub audio_buffer: Arc<RwLock<Vec<f32>>>,
    /// Current sample rate
    pub sample_rate: RwLock<u32>,
    /// Database connection pool
    pub db_pool: RwLock<Option<DbPool>>,
    /// Current detected emotion
    pub current_emotion: RwLock<String>,
    /// Keyword vocabulary version
    pub keyword_version: RwLock<u64>,
    /// Is detection pipeline ready
    pub detection_ready: RwLock<bool>,
    /// Startup complete flag
    pub startup_complete: RwLock<bool>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            session_state: RwLock::new(SessionState::Idle),
            app_mode: RwLock::new(AppMode::default()),
            config: RwLock::new(SessionConfig::default()),
            audio_buffer: Arc::new(RwLock::new(Vec::new())),
            sample_rate: RwLock::new(16000),
            db_pool: RwLock::new(None),
            current_emotion: RwLock::new("neutral".to_string()),
            keyword_version: RwLock::new(0),
            detection_ready: RwLock::new(false),
            startup_complete: RwLock::new(false),
        }
    }
}

/// Channel capacities for internal communication
pub mod channels {
    /// Audio buffer capacity (number of frames)
    pub const AUDIO_BUFFER_CAPACITY: usize = 16000 * 60; // 1 minute at 16kHz

    /// Detection event queue capacity
    pub const DETECTION_QUEUE_CAPACITY: usize = 100;

    /// Max transcription text length
    pub const MAX_TRANSCRIPTION_LENGTH: usize = 4096;
}

/// Constants for the application
pub mod constants {
    /// Minimum audio segment duration for VAD (ms)
    pub const VAD_MIN_SEGMENT_MS: u32 = 100;

    /// Maximum audio segment duration for transcription (ms)
    pub const TRANSCRIPTION_MAX_SEGMENT_MS: u32 = 12000;

    /// Default audio segment duration for transcription (ms)
    pub const TRANSCRIPTION_DEFAULT_SEGMENT_MS: u32 = 8000;

    /// Voice activity detection threshold
    pub const VAD_THRESHOLD: f32 = 0.5;

    /// Speaker verification similarity threshold
    pub const SPEAKER_SIMILARITY_THRESHOLD: f32 = 0.75;

    /// Emotion confidence threshold
    pub const EMOTION_CONFIDENCE_THRESHOLD: f32 = 0.6;

    /// Crossfade types
    pub const CROSSFADE_INSTANT_MS: u32 = 0;
    pub const CROSSFADE_QUICK_MS: u32 = 500;
    pub const CROSSFADE_MUSICAL_MS: u32 = 2000;
    pub const CROSSFADE_LONG_MS: u32 = 5000;

    /// Two-phase startup timeouts (ms)
    pub const UI_READY_TIMEOUT_MS: u64 = 3000;
    pub const DETECTION_READY_TIMEOUT_MS: u64 = 15000;
}
