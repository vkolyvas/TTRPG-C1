//! Session state machine

use crate::audio::capture::AudioCapture;
use crate::dsp::processing;
use crate::inference::emotion::{EmotionAnalyzer, EmotionResult};
use crate::inference::whisper::{Transcription, WhisperEngine};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use thiserror::Error;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

#[derive(Error, Debug)]
pub enum OrchestratorError {
    #[error("Not in correct state for operation: {0}")]
    InvalidState(String),
    #[error("Audio capture error: {0}")]
    AudioError(String),
    #[error("Inference error: {0}")]
    InferenceError(String),
}

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

/// Audio buffer for processing
#[derive(Clone, Debug)]
pub struct AudioBuffer {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
}

/// Session event
#[derive(Debug)]
pub enum SessionEvent {
    StartRecording,
    StopRecording,
    AudioData(AudioBuffer),
    TranscriptionReady(Transcription),
    EmotionAnalysisReady(EmotionResult),
    Error(String),
}

/// Session configuration
#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub sample_rate: u32,
    pub buffer_size_ms: u32,
    pub silence_threshold: f32,
    pub enable_transcription: bool,
    pub enable_emotion_analysis: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            sample_rate: 16000,
            buffer_size_ms: 100,
            silence_threshold: 0.01,
            enable_transcription: true,
            enable_emotion_analysis: true,
        }
    }
}

/// Session orchestrator - manages the audio processing pipeline
pub struct SessionOrchestrator {
    state: SessionState,
    config: SessionConfig,
    capture: AudioCapture,
    whisper: WhisperEngine,
    emotion: EmotionAnalyzer,
    audio_buffer: Arc<Mutex<Vec<f32>>>,  // Thread-safe buffer
    event_tx: Option<mpsc::Sender<SessionEvent>>,
}

impl SessionOrchestrator {
    /// Create a new SessionOrchestrator
    pub fn new() -> Self {
        Self {
            state: SessionState::Idle,
            config: SessionConfig::default(),
            capture: AudioCapture::new(),
            whisper: WhisperEngine::new(),
            emotion: EmotionAnalyzer::new(),
            audio_buffer: Arc::new(Mutex::new(Vec::new())),
            event_tx: None,
        }
    }

    /// Initialize the orchestrator
    pub fn init(&mut self) -> Result<(), OrchestratorError> {
        info!("Initializing session orchestrator");

        // Initialize whisper (placeholder model path)
        if let Err(e) = self.whisper.init("models/whisper-tiny.bin") {
            warn!("Whisper init warning: {}", e);
        }

        // Initialize emotion analyzer
        if let Err(e) = self.emotion.init() {
            warn!("Emotion analyzer init warning: {}", e);
        }

        info!("Session orchestrator initialized");
        Ok(())
    }

    /// Start a recording session
    pub fn start_session(&mut self) -> Result<(), OrchestratorError> {
        if self.state != SessionState::Idle {
            return Err(OrchestratorError::InvalidState(format!(
                "Cannot start session in state: {}",
                self.state
            )));
        }

        info!("Starting recording session");

        // Clear and get buffer reference
        {
            let mut buffer = self.audio_buffer.lock().unwrap();
            buffer.clear();
        }

        // Clone the Arc for the callback
        let buffer = self.audio_buffer.clone();

        // Start audio capture with callback that stores samples
        self.capture
            .start_recording(move |samples| {
                if let Ok(mut buffer) = buffer.lock() {
                    buffer.extend_from_slice(&samples);
                }
            })
            .map_err(|e| OrchestratorError::AudioError(e.to_string()))?;

        self.state = SessionState::Recording;
        info!("Session started, state: {}", self.state);

        Ok(())
    }

    /// Stop the recording session
    pub fn stop_session(&mut self) -> Result<SessionResult, OrchestratorError> {
        if self.state != SessionState::Recording {
            return Err(OrchestratorError::InvalidState(format!(
                "Cannot stop session in state: {}",
                self.state
            )));
        }

        info!("Stopping recording session");

        self.capture
            .stop_recording()
            .map_err(|e| OrchestratorError::AudioError(e.to_string()))?;

        self.state = SessionState::Processing;

        // Process the captured audio
        let result = self.process_audio()?;

        self.state = SessionState::Idle;

        info!("Session stopped, state: {}", self.state);

        Ok(result)
    }

    /// Get current session state
    pub fn state(&self) -> SessionState {
        self.state
    }

    /// Get session configuration
    pub fn config(&self) -> &SessionConfig {
        &self.config
    }

    /// Update session configuration
    pub fn set_config(&mut self, config: SessionConfig) {
        self.config = config;
        debug!("Session config updated");
    }

    /// Process captured audio
    fn process_audio(&self) -> Result<SessionResult, OrchestratorError> {
        // Get samples from the thread-safe buffer
        let samples = {
            let buffer = self.audio_buffer.lock().unwrap();
            buffer.clone()
        };

        info!("Processing audio buffer ({} samples)", samples.len());

        let mut samples = samples;

        // Resample if needed
        let capture_rate = self.capture.sample_rate();
        if capture_rate != self.config.sample_rate {
            samples = processing::resample(&samples, capture_rate, self.config.sample_rate);
        }

        // Convert to mono if needed
        let channels = self.capture.channels();
        if channels > 1 {
            samples = processing::stereo_to_mono(&samples, channels);
        }

        // Apply DSP processing
        processing::remove_dc_offset(&mut samples);
        processing::normalize(&mut samples, 0.9);
        processing::noise_gate(&mut samples, self.config.silence_threshold);

        let mut transcription = None;
        let mut emotion_result = None;

        // Run transcription
        if self.config.enable_transcription {
            match self.whisper.transcribe(&samples, self.config.sample_rate) {
                Ok(t) => {
                    info!("Transcription: {}", t.text);
                    transcription = Some(t);
                }
                Err(e) => {
                    error!("Transcription error: {}", e);
                }
            }
        }

        // Run emotion analysis
        if self.config.enable_emotion_analysis {
            match self.emotion.analyze(&samples, self.config.sample_rate) {
                Ok(e) => {
                    info!("Emotion: {} ({:.2})", e.primary, e.confidence);
                    emotion_result = Some(e);
                }
                Err(e) => {
                    error!("Emotion analysis error: {}", e);
                }
            }
        }

        Ok(SessionResult {
            transcription,
            emotion: emotion_result,
        })
    }
}

impl Default for SessionOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a recording session
#[derive(Debug)]
pub struct SessionResult {
    pub transcription: Option<Transcription>,
    pub emotion: Option<EmotionResult>,
}
