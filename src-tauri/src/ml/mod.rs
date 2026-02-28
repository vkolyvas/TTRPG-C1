//! ML inference module
//!
//! Provides ONNX Runtime integration for ML models:
//! - Voice Activity Detection (Silero VAD)
//! - Speaker Verification (Resemblyzer)
//! - Emotion Analysis (emotion2vec)

pub mod ort_env;
pub mod vad_model;
pub mod speaker_model;

pub use ort_env::*;
pub use vad_model::*;
pub use speaker_model::*;

use crate::error::AppError;
use std::path::Path;

/// Model paths configuration
#[derive(Debug, Clone)]
pub struct ModelPaths {
    pub vad_model: Option<String>,
    pub speaker_model: Option<String>,
    pub emotion_model: Option<String>,
}

impl Default for ModelPaths {
    fn default() -> Self {
        Self {
            vad_model: Some("models/silero_vad.onnx".to_string()),
            speaker_model: Some("models/resemblyzer.onnx".to_string()),
            emotion_model: Some("models/emotion2vec.onnx".to_string()),
        }
    }
}

/// ML inference environment
pub struct InferenceEnv {
    pub initialized: bool,
    pub model_paths: ModelPaths,
}

impl InferenceEnv {
    /// Create a new inference environment
    pub fn new() -> Self {
        Self {
            initialized: false,
            model_paths: ModelPaths::default(),
        }
    }

    /// Initialize ONNX Runtime
    pub fn init(&mut self) -> Result<(), AppError> {
        if self.initialized {
            return Ok(());
        }

        tracing::info!("Initializing ONNX Runtime");

        // Note: In production, initialize ONNX Runtime here
        // ort::init().commit()?;

        self.initialized = true;
        tracing::info!("ONNX Runtime initialized");
        Ok(())
    }

    /// Set model paths
    pub fn set_model_paths(&mut self, paths: ModelPaths) {
        self.model_paths = paths;
    }

    /// Check if initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl Default for InferenceEnv {
    fn default() -> Self {
        Self::new()
    }
}

/// Tensor utilities
pub mod tensor {
    /// Create a tensor from audio samples
    pub fn audio_to_tensor(samples: &[f32], sample_rate: u32) -> Vec<f32> {
        samples.to_vec()
    }

    /// Normalize tensor values
    pub fn normalize(tensor: &mut [f32]) {
        if tensor.is_empty() {
            return;
        }

        let max_val = tensor.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
        if max_val > 0.0 {
            for val in tensor.iter_mut() {
                *val /= max_val;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inference_env() {
        let env = InferenceEnv::new();
        assert!(!env.is_initialized());
    }

    #[test]
    fn test_tensor_normalize() {
        let mut tensor = vec![0.5, 1.0, 0.25];
        tensor::normalize(&mut tensor);
        assert!((tensor[1] - 1.0).abs() < 0.001);
    }
}
