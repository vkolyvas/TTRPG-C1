//! Voice Activity Detection (VAD) module

use crate::error::AppError;
use crate::state::constants::VAD_THRESHOLD;
use std::sync::Arc;
use parking_lot::RwLock;

/// Voice Activity Detection result
#[derive(Debug, Clone)]
pub struct VadResult {
    /// Whether voice is present
    pub is_speech: bool,
    /// Confidence score (0-1)
    pub confidence: f32,
    /// Start timestamp of speech segment (if any)
    pub start_ms: Option<u64>,
    /// End timestamp of speech segment (if any)
    pub end_ms: Option<u64>,
}

/// Voice Activity Detector using energy-based detection
/// Note: This is a placeholder. For production, use Silero VAD via ONNX.
pub struct VoiceActivityDetector {
    threshold: f32,
    min_speech_duration_ms: u32,
    min_silence_duration_ms: u32,
    frame_size_ms: u32,
    is_speaking: bool,
    speech_start_ms: Option<u64>,
    sample_rate: u32,
}

impl VoiceActivityDetector {
    /// Create a new VAD instance
    pub fn new() -> Self {
        Self {
            threshold: VAD_THRESHOLD,
            min_speech_duration_ms: 100,
            min_silence_duration_ms: 300,
            frame_size_ms: 30,
            is_speaking: false,
            speech_start_ms: None,
            sample_rate: 16000,
        }
    }

    /// Set the detection threshold
    pub fn set_threshold(&mut self, threshold: f32) {
        self.threshold = threshold.clamp(0.0, 1.0);
    }

    /// Set sample rate
    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate;
    }

    /// Process audio frame and detect voice activity
    pub fn process_frame(&mut self, samples: &[f32], timestamp_ms: u64) -> VadResult {
        let energy = self.compute_energy(samples);
        let is_speech = energy > self.threshold;

        let result = if is_speech && !self.is_speaking {
            // Speech started
            self.is_speaking = true;
            self.speech_start_ms = Some(timestamp_ms);
            VadResult {
                is_speech: true,
                confidence: energy.min(1.0),
                start_ms: Some(timestamp_ms),
                end_ms: None,
            }
        } else if !is_speech && self.is_speaking {
            // Speech ended
            self.is_speaking = false;
            let start = self.speech_start_ms.take();
            VadResult {
                is_speech: false,
                confidence: 1.0 - energy.min(1.0),
                start_ms: start,
                end_ms: Some(timestamp_ms),
            }
        } else {
            VadResult {
                is_speech,
                confidence: energy.min(1.0),
                start_ms: None,
                end_ms: None,
            }
        };

        result
    }

    /// Compute RMS energy of audio frame
    fn compute_energy(&self, samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }

        let sum: f32 = samples.iter().map(|&s| s * s).sum();
        let rms = (sum / samples.len() as f32).sqrt();

        // Normalize to 0-1 range (assuming max amplitude of 1.0)
        rms
    }

    /// Check if currently detecting speech
    pub fn is_speaking(&self) -> bool {
        self.is_speaking
    }

    /// Reset the VAD state
    pub fn reset(&mut self) {
        self.is_speaking = false;
        self.speech_start_ms = None;
    }
}

impl Default for VoiceActivityDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// ONNX-based VAD using Silero
/// Requires the ort crate and Silero VAD model
pub struct SileroVad {
    /// Placeholder for ONNX session
    session: Option<()>,
    threshold: f32,
    sample_rate: u32,
}

impl SileroVad {
    /// Create a new Silero VAD instance
    pub fn new() -> Self {
        Self {
            session: None,
            threshold: VAD_THRESHOLD,
            sample_rate: 16000,
        }
    }

    /// Initialize with ONNX model
    pub fn init(&mut self, model_path: &str) -> Result<(), AppError> {
        tracing::info!("Initializing Silero VAD with model: {}", model_path);
        // Placeholder: In production, load ONNX model here
        // self.session = Some(ort::Session::from_file(model_path)?);
        Ok(())
    }

    /// Process audio frame
    pub fn process(&mut self, samples: &[f32], timestamp_ms: u64) -> VadResult {
        // Placeholder: In production, run ONNX inference
        // For now, fall back to energy-based detection
        let energy = if samples.is_empty() {
            0.0
        } else {
            let sum: f32 = samples.iter().map(|&s| s * s).sum();
            (sum / samples.len() as f32).sqrt()
        };

        VadResult {
            is_speech: energy > self.threshold,
            confidence: energy.min(1.0),
            start_ms: None,
            end_ms: None,
        }
    }

    /// Set detection threshold
    pub fn set_threshold(&mut self, threshold: f32) {
        self.threshold = threshold;
    }
}

impl Default for SileroVad {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_energy_vad() {
        let mut vad = VoiceActivityDetector::new();
        vad.set_threshold(0.1);

        // Silent frame
        let silent = vec![0.0; 160];
        let result = vad.process_frame(&silent, 0);
        assert!(!result.is_speech);

        // Speech frame (louder)
        let speech: Vec<f32> = (0..160).map(|i| (i as f32 * 0.01).sin()).collect();
        let result = vad.process_frame(&speech, 30);
        assert!(result.is_speech);
    }
}
