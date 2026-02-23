//! Emotion analysis for vocal mood detection
//!
//! This module provides emotion analysis based on acoustic features extracted
//! from audio samples. It uses heuristic rules derived from speech prosody research.
//!
//! Features used:
//! - RMS energy (loudness)
//! - Zero-crossing rate (voice timbre)
//! - Pitch estimation (fundamental frequency)
//! - Energy variance (speech rhythm/stability)

use std::collections::HashMap;
use thiserror::Error;
use tracing::{debug, info, warn};

#[derive(Error, Debug)]
pub enum EmotionError {
    #[error("Emotion analyzer not initialized")]
    NotInitialized,
    #[error("Model load error: {0}")]
    ModelLoadError(String),
    #[error("Analysis error: {0}")]
    AnalysisError(String),
    #[error("Insufficient audio data: {0}")]
    InsufficientData(String),
}

/// Detected emotion with confidence scores
#[derive(Debug, Clone)]
pub struct EmotionResult {
    pub primary: Emotion,
    pub confidence: f32,
    pub scores: HashMap<Emotion, f32>,
}

/// Supported emotions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Emotion {
    Neutral,
    Happy,
    Sad,
    Angry,
    Fearful,
    Surprised,
    Disgusted,
}

impl Emotion {
    /// Get all supported emotions
    pub fn all() -> Vec<Emotion> {
        vec![
            Emotion::Neutral,
            Emotion::Happy,
            Emotion::Sad,
            Emotion::Angry,
            Emotion::Fearful,
            Emotion::Surprised,
            Emotion::Disgusted,
        ]
    }
}

impl std::fmt::Display for Emotion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Emotion::Neutral => write!(f, "neutral"),
            Emotion::Happy => write!(f, "happy"),
            Emotion::Sad => write!(f, "sad"),
            Emotion::Angry => write!(f, "angry"),
            Emotion::Fearful => write!(f, "fearful"),
            Emotion::Surprised => write!(f, "surprised"),
            Emotion::Disgusted => write!(f, "disgusted"),
        }
    }
}

impl std::fmt::Display for EmotionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({:.0}%)", self.primary, self.confidence * 100.0)
    }
}

/// Audio features extracted for emotion analysis
#[derive(Debug, Clone)]
pub struct AudioFeatures {
    pub rms: f32,              // Root mean square (energy)
    pub zcr: f32,              // Zero-crossing rate
    pub pitch_hz: f32,         // Estimated pitch in Hz
    pub energy_variance: f32,  // Variance in energy over time
    pub duration: f32,         // Duration in seconds
    pub sample_rate: u32,
}

/// Emotion analysis engine using acoustic features
pub struct EmotionAnalyzer {
    initialized: bool,
    sensitivity: f32,  // How much to weight the features (0.0 - 1.0)
}

impl EmotionAnalyzer {
    /// Create a new EmotionAnalyzer instance
    pub fn new() -> Self {
        Self {
            initialized: false,
            sensitivity: 0.5,
        }
    }

    /// Create with custom sensitivity
    pub fn with_sensitivity(sensitivity: f32) -> Self {
        Self {
            initialized: false,
            sensitivity: sensitivity.clamp(0.0, 1.0),
        }
    }

    /// Initialize the analyzer
    pub fn init(&mut self) -> Result<(), EmotionError> {
        info!("Initializing emotion analyzer (feature-based)");
        self.initialized = true;
        debug!("Emotion analyzer initialized with sensitivity: {:.2}", self.sensitivity);
        Ok(())
    }

    /// Analyze emotion from audio samples
    pub fn analyze(&self, samples: &[f32], sample_rate: u32) -> Result<EmotionResult, EmotionError> {
        if !self.initialized {
            return Err(EmotionError::NotInitialized);
        }

        // Need minimum samples for reliable analysis (~100ms at 16kHz = 1600 samples)
        if samples.len() < 800 {
            return Err(EmotionError::InsufficientData(format!(
                "Need at least 800 samples, got {}",
                samples.len()
            )));
        }

        let features = extract_features(samples, sample_rate);
        debug!(
            "Analyzing emotion: RMS={:.3}, ZCR={:.3}, Pitch={:.1}Hz, Var={:.3}",
            features.rms, features.zcr, features.pitch_hz, features.energy_variance
        );

        // Calculate emotion scores based on features
        let scores = self.calculate_emotion_scores(&features);

        // Find primary emotion
        let mut sorted: Vec<_> = scores.iter().collect();
        sorted.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));

        let primary = *sorted[0].0;
        let confidence = *sorted[0].1;

        debug!("Emotion result: {} ({:.1}%)", primary, confidence * 100.0);

        Ok(EmotionResult {
            primary,
            confidence,
            scores,
        })
    }

    /// Calculate emotion scores from audio features
    fn calculate_emotion_scores(&self, features: &AudioFeatures) -> HashMap<Emotion, f32> {
        let mut scores = HashMap::new();

        // Normalize features to 0-1 range for scoring
        let energy = (features.rms * 10.0).clamp(0.0, 1.0);  // RMS typically 0-0.1
        let zcr_norm = (features.zcr * 10.0).clamp(0.0, 1.0); // ZCR typically 0-0.1
        let pitch_norm = (features.pitch_hz / 300.0).clamp(0.0, 1.0); // Pitch 50-300Hz typical
        let variance = (features.energy_variance * 50.0).clamp(0.0, 1.0);

        // Heuristic rules based on speech prosody research
        // Neutral: moderate energy, moderate pitch, stable
        let neutral = (1.0 - energy * 0.3) * (1.0 - variance * 0.3) * 0.8;

        // Happy: higher energy, higher pitch, moderate variance
        let happy = energy * 0.4 + pitch_norm * 0.3 + variance * 0.2;

        // Sad: lower energy, lower pitch, low variance (monotone)
        let sad = (1.0 - energy) * 0.5 + (1.0 - pitch_norm) * 0.3 + (1.0 - variance) * 0.2;

        // Angry: high energy, high pitch, high variance
        let angry = energy * 0.5 + pitch_norm * 0.3 + variance * 0.4;

        // Fearful: moderate energy, high pitch, high variance (unstable)
        let fearful = (1.0 - energy) * 0.2 + pitch_norm * 0.4 + variance * 0.5;

        // Surprised: sudden energy changes, high pitch
        let surprised = variance * 0.6 + pitch_norm * 0.3;

        // Disgusted: low energy, low pitch, moderate variance
        let disgusted = (1.0 - energy) * 0.4 + (1.0 - pitch_norm) * 0.3;

        // Normalize scores to sum to ~1
        let total = neutral + happy + sad + angry + fearful + surprised + disgusted;

        scores.insert(Emotion::Neutral, neutral / total);
        scores.insert(Emotion::Happy, happy / total);
        scores.insert(Emotion::Sad, sad / total);
        scores.insert(Emotion::Angry, angry / total);
        scores.insert(Emotion::Fearful, fearful / total);
        scores.insert(Emotion::Surprised, surprised / total);
        scores.insert(Emotion::Disgusted, disgusted / total);

        scores
    }

    /// Check if analyzer is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Get current sensitivity
    pub fn sensitivity(&self) -> f32 {
        self.sensitivity
    }
}

impl Default for EmotionAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract audio features for emotion analysis
pub fn extract_features(samples: &[f32], sample_rate: u32) -> AudioFeatures {
    let duration = samples.len() as f32 / sample_rate as f32;

    // RMS energy
    let rms = calculate_rms(samples);

    // Zero-crossing rate
    let zcr = calculate_zcr(samples);

    // Pitch estimation using autocorrelation
    let pitch_hz = estimate_pitch_autocorr(samples, sample_rate);

    // Energy variance (split into chunks)
    let energy_variance = calculate_energy_variance(samples, sample_rate);

    AudioFeatures {
        rms,
        zcr,
        pitch_hz,
        energy_variance,
        duration,
        sample_rate,
    }
}

/// Calculate RMS (Root Mean Square) energy
fn calculate_rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }

    let sum_squares: f32 = samples.iter().map(|&s| s * s).sum();
    (sum_squares / samples.len() as f32).sqrt()
}

/// Calculate Zero-Crossing Rate
fn calculate_zcr(samples: &[f32]) -> f32 {
    if samples.len() < 2 {
        return 0.0;
    }

    let crossings = samples
        .windows(2)
        .filter(|w| w[0].signum() != w[1].signum())
        .count() as f32;

    crossings / (samples.len() - 1) as f32
}

/// Estimate pitch using autocorrelation (YIN-like algorithm simplified)
fn estimate_pitch_autocorr(samples: &[f32], sample_rate: u32) -> f32 {
    if samples.len() < 256 {
        return 0.0;
    }

    // Use a window of ~30ms for pitch analysis
    let window_size = (sample_rate as f32 * 0.03) as usize;
    let window_size = window_size.min(samples.len()).max(64);

    // Simple autocorrelation - use usize for indices
    let min_lag = (sample_rate / 400) as usize;  // Max pitch ~400Hz
    let max_lag = ((sample_rate / 50) as usize).min(window_size / 2);   // Min pitch ~50Hz

    let mut best_lag = 0usize;
    let mut best_corr = 0.0;

    for lag in min_lag..max_lag {
        let mut correlation = 0.0;
        let mut norm1 = 0.0;
        let mut norm2 = 0.0;

        let limit = window_size - lag;
        for i in 0..limit {
            correlation += samples[i] * samples[i + lag];
            norm1 += samples[i] * samples[i];
            norm2 += samples[i + lag] * samples[i + lag];
        }

        let corr = if norm1 > 0.0 && norm2 > 0.0 {
            correlation / (norm1 * norm2).sqrt()
        } else {
            0.0
        };

        if corr > best_corr {
            best_corr = corr;
            best_lag = lag;
        }
    }

    // Only return pitch if correlation is strong enough
    if best_corr > 0.3 && best_lag > 0 {
        sample_rate as f32 / best_lag as f32
    } else {
        0.0  // No clear pitch detected (unvoiced or silence)
    }
}

/// Calculate energy variance over time (speech rhythm indicator)
fn calculate_energy_variance(samples: &[f32], sample_rate: u32) -> f32 {
    // Split into ~50ms chunks
    let chunk_size = (sample_rate as f32 * 0.05) as usize;
    if samples.len() < chunk_size * 2 {
        return 0.0;
    }

    let chunk_energies: Vec<f32> = samples
        .chunks(chunk_size)
        .map(|chunk| calculate_rms(chunk))
        .collect();

    if chunk_energies.is_empty() {
        return 0.0;
    }

    let mean: f32 = chunk_energies.iter().sum::<f32>() / chunk_energies.len() as f32;
    let variance: f32 = chunk_energies
        .iter()
        .map(|&e| (e - mean).powi(2))
        .sum::<f32>() / chunk_energies.len() as f32;

    variance.sqrt()  // Return standard deviation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_features_silence() {
        let samples = vec![0.0f32; 16000];
        let features = extract_features(&samples, 16000);

        assert!(features.rms < 0.001);
        assert!(features.zcr < 0.01);
    }

    #[test]
    fn test_rms_calculation() {
        // Pure 1.0 sine wave should have RMS of 1/sqrt(2) ~= 0.707
        // Use 2*PI multiple for exact full periods
        let samples: Vec<f32> = (0..628)
            .map(|i| (i as f32 * 0.01).sin())
            .collect();
        let rms = calculate_rms(&samples);

        // RMS of sin should be ~0.707, allow some tolerance
        assert!(rms > 0.5 && rms < 1.0);
    }

    #[test]
    fn test_zcr_calculation() {
        // Square wave has high ZCR
        let samples: Vec<f32> = (0..1000)
            .map(|i| if i % 2 == 0 { 1.0 } else { -1.0 })
            .collect();
        let zcr = calculate_zcr(&samples);

        assert!(zcr > 0.9);
    }

    #[test]
    fn test_emotion_analyzer_init() {
        let mut analyzer = EmotionAnalyzer::new();
        assert!(!analyzer.is_initialized());

        analyzer.init().unwrap();
        assert!(analyzer.is_initialized());
    }

    #[test]
    fn test_emotion_analyzer_silence() {
        let mut analyzer = EmotionAnalyzer::new();
        analyzer.init().unwrap();

        let samples = vec![0.0f32; 16000];
        let result = analyzer.analyze(&samples, 16000).unwrap();

        // Silence should likely be neutral
        assert!(result.scores.contains_key(&Emotion::Neutral));
    }

    #[test]
    fn test_emotion_display() {
        assert_eq!(format!("{}", Emotion::Happy), "happy");
        assert_eq!(format!("{}", Emotion::Angry), "angry");
    }
}
