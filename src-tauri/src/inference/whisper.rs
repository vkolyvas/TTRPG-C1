//! Whisper-based speech-to-text inference
//!
//! This module provides integration with whisper.cpp for local speech recognition.
//! When the "whisper" feature is enabled, it uses whisper-rs for actual inference.
//! Without the feature, it provides a placeholder implementation.

use thiserror::Error;
use tracing::{debug, info, warn};

#[cfg(feature = "whisper")]
use std::path::Path;

#[derive(Error, Debug)]
pub enum WhisperError {
    #[error("Whisper not initialized")]
    NotInitialized,
    #[error("Model load error: {0}")]
    ModelLoadError(String),
    #[error("Inference error: {0}")]
    InferenceError(String),
    #[error("Audio processing error: {0}")]
    AudioError(String),
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    #[error("Feature not enabled: whisper feature required")]
    FeatureNotEnabled,
}

/// Transcription result
#[derive(Debug, Clone)]
pub struct Transcription {
    pub text: String,
    pub language: Option<String>,
    pub confidence: f32,
}

/// Whisper inference engine
#[cfg(feature = "whisper")]
pub struct WhisperEngine {
    model_path: Option<String>,
    context: Option<whisper_rs::WhisperContext>,
    params: whisper_rs::WhisperParams,
}

/// Placeholder Whisper engine when feature is disabled
#[cfg(not(feature = "whisper"))]
pub struct WhisperEngine {
    model_path: Option<String>,
    initialized: bool,
}

#[cfg(feature = "whisper")]
impl WhisperEngine {
    /// Create a new WhisperEngine instance
    pub fn new() -> Self {
        let params = whisper_rs::WhisperParams::new()
            .with_n_threads(4)
            .with_language(Some("en"));

        Self {
            model_path: None,
            context: None,
            params,
        }
    }

    /// Initialize with a model file
    pub fn init(&mut self, model_path: &str) -> Result<(), WhisperError> {
        info!("Initializing Whisper engine with model: {}", model_path);

        if !Path::new(model_path).exists() {
            return Err(WhisperError::ModelNotFound(model_path.to_string()));
        }

        let context = whisper_rs::WhisperContext::new(model_path)
            .map_err(|e| WhisperError::ModelLoadError(e.to_string()))?;

        self.model_path = Some(model_path.to_string());
        self.context = Some(context);

        info!("Whisper engine initialized successfully");
        Ok(())
    }

    /// Transcribe audio samples
    pub fn transcribe(&mut self, samples: &[f32], sample_rate: u32) -> Result<Transcription, WhisperError> {
        let context = self.context.as_ref().ok_or(WhisperError::NotInitialized)?;

        // Convert f32 to i16 (whisper expects 16-bit PCM)
        let pcm_samples: Vec<i16> = samples
            .iter()
            .map(|&s| (s * i16::MAX as f32) as i16)
            .collect();

        let duration_secs = samples.len() as f32 / sample_rate as f32;
        debug!("Transcribing audio: {:.2}s, {} Hz", duration_secs, sample_rate);

        let mut state = context
            .create_state()
            .map_err(|e| WhisperError::InferenceError(e.to_string()))?;

        state
            .process(&self.params, &pcm_samples)
            .map_err(|e| WhisperError::InferenceError(e.to_string()))?;

        let segments = state
            .get_segments()
            .map_err(|e| WhisperError::InferenceError(e.to_string()))?;

        let mut full_text = String::new();
        for segment in segments {
            full_text.push_str(&segment.text);
        }

        let confidence = if full_text.trim().is_empty() {
            0.0
        } else {
            0.85
        };

        let language = state
            .get_language()
            .map(|l| l.to_string())
            .ok();

        debug!("Transcription result: {} chars", full_text.len());

        Ok(Transcription {
            text: full_text.trim().to_string(),
            language,
            confidence,
        })
    }

    /// Check if engine is initialized
    pub fn is_initialized(&self) -> bool {
        self.context.is_some()
    }

    /// Get model path
    pub fn model_path(&self) -> Option<&str> {
        self.model_path.as_deref()
    }
}

#[cfg(not(feature = "whisper"))]
impl WhisperEngine {
    /// Create a new WhisperEngine instance (placeholder)
    pub fn new() -> Self {
        Self {
            model_path: None,
            initialized: false,
        }
    }

    /// Initialize with a model file (placeholder - always succeeds)
    pub fn init(&mut self, model_path: &str) -> Result<(), WhisperError> {
        info!("Initializing Whisper engine (placeholder mode) with model: {}", model_path);

        if !std::path::Path::new(model_path).exists() {
            warn!("Model file not found: {} - running in placeholder mode", model_path);
        }

        self.model_path = Some(model_path.to_string());
        self.initialized = true;

        debug!("Whisper engine initialized (placeholder)");
        Ok(())
    }

    /// Transcribe audio samples (placeholder implementation)
    pub fn transcribe(&self, samples: &[f32], sample_rate: u32) -> Result<Transcription, WhisperError> {
        if !self.initialized {
            return Err(WhisperError::NotInitialized);
        }

        let duration_secs = samples.len() as f32 / sample_rate as f32;
        debug!("Transcribing audio (placeholder): {:.2}s, {} Hz", duration_secs, sample_rate);

        // Return placeholder text
        Ok(Transcription {
            text: "[Transcription placeholder - enable whisper feature]".to_string(),
            language: Some("en".to_string()),
            confidence: 0.0,
        })
    }

    /// Check if engine is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Get model path
    pub fn model_path(&self) -> Option<&str> {
        self.model_path.as_deref()
    }
}

impl Default for WhisperEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the default model path
/// Checks in order:
/// 1. ./assets/models/whisper/tiny.bin (bundled)
/// 2. ./models/whisper-tiny.bin (local)
/// 3. ~/.local/share/ttrpg_companion/models/whisper/tiny.bin (user data)
pub fn get_model_path() -> std::path::PathBuf {
    // Check bundled path
    let bundled = std::path::PathBuf::from("assets")
        .join("models")
        .join("whisper")
        .join("tiny.bin");

    if bundled.exists() {
        return bundled;
    }

    // Check local models directory
    let local = std::path::PathBuf::from("models").join("whisper-tiny.bin");
    if local.exists() {
        return local;
    }

    // Check user data directory
    if let Some(data_dir) = dirs::data_local_dir() {
        let user_path = data_dir
            .join("ttrpg_companion")
            .join("models")
            .join("whisper")
            .join("tiny.bin");
        if user_path.exists() {
            return user_path;
        }
    }

    // Return default path even if it doesn't exist
    std::path::PathBuf::from("models/whisper-tiny.bin")
}

/// Check if whisper model is available
pub fn is_model_available() -> bool {
    get_model_path().exists()
}

/// Check if whisper feature is enabled
#[cfg(feature = "whisper")]
pub fn is_whisper_enabled() -> bool {
    true
}

#[cfg(not(feature = "whisper"))]
pub fn is_whisper_enabled() -> bool {
    false
}

/// Download URL for tiny.en model
pub fn get_model_download_url() -> &'static str {
    "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en.bin"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_model_path() {
        let path = get_model_path();
        assert!(path.file_name().is_some());
    }

    #[test]
    fn test_engine_creation() {
        let engine = WhisperEngine::new();
        assert!(!engine.is_initialized());
    }

    #[test]
    #[cfg(not(feature = "whisper"))]
    fn test_placeholder_transcribe() {
        let mut engine = WhisperEngine::new();
        engine.init("dummy.bin").unwrap();

        let samples = vec![0.0f32; 16000];
        let result = engine.transcribe(&samples, 16000).unwrap();

        assert!(result.text.contains("placeholder"));
    }
}
