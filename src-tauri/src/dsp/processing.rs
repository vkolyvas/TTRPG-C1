//! Audio preprocessing and DSP operations

use thiserror::Error;
use tracing::debug;

#[derive(Error, Debug)]
pub enum DspError {
    #[error("FFT error: {0}")]
    FftError(String),
    #[error("Processing error: {0}")]
    ProcessingError(String),
}

/// Normalize audio samples to a target peak amplitude
pub fn normalize(samples: &mut [f32], target_peak: f32) {
    if samples.is_empty() {
        return;
    }

    let max_sample = samples
        .iter()
        .map(|s| s.abs())
        .fold(0.0_f32, |a, b| a.max(b));

    if max_sample > 0.0 {
        let scale = target_peak / max_sample;
        for sample in samples.iter_mut() {
            *sample *= scale;
        }
    }

    debug!("Normalized audio, peak: {:.3}", max_sample);
}

/// Apply a simple low-pass filter
pub fn low_pass_filter(samples: &mut [f32], cutoff_ratio: f32) {
    if samples.len() < 2 {
        return;
    }

    let alpha = cutoff_ratio.clamp(0.0, 1.0);

    for i in 1..samples.len() {
        samples[i] = alpha * samples[i] + (1.0 - alpha) * samples[i - 1];
    }
}

/// Apply a simple high-pass filter
pub fn high_pass_filter(samples: &mut [f32], cutoff_ratio: f32) {
    if samples.len() < 2 {
        return;
    }

    let alpha = cutoff_ratio.clamp(0.0, 1.0);

    for i in 1..samples.len() {
        samples[i] = alpha * (samples[i] - samples[i - 1]);
    }
}

/// Remove DC offset from audio samples
pub fn remove_dc_offset(samples: &mut [f32]) {
    if samples.is_empty() {
        return;
    }

    let mean: f32 = samples.iter().sum::<f32>() / samples.len() as f32;
    for sample in samples.iter_mut() {
        *sample -= mean;
    }

    debug!("Removed DC offset, mean was: {:.6}", mean);
}

/// Apply simple noise gate
pub fn noise_gate(samples: &mut [f32], threshold: f32) {
    for sample in samples.iter_mut() {
        if sample.abs() < threshold {
            *sample = 0.0;
        }
    }
}

/// Calculate root mean square (RMS) of samples
pub fn calculate_rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }

    let sum_squares: f32 = samples.iter().map(|s| s * s).sum();
    (sum_squares / samples.len() as f32).sqrt()
}

/// Calculate volume level in decibels
pub fn calculate_db(samples: &[f32]) -> f32 {
    let rms = calculate_rms(samples);
    if rms > 0.0 {
        20.0 * rms.log10()
    } else {
        -96.0 // Minimum dB level (silence)
    }
}

/// Resample audio to target sample rate using linear interpolation
pub fn resample(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if from_rate == to_rate {
        return samples.to_vec();
    }

    if samples.is_empty() || to_rate == 0 {
        return Vec::new();
    }

    let ratio = to_rate as f32 / from_rate as f32;
    let new_length = ((samples.len() as f32) * ratio).ceil() as usize;
    let mut result = Vec::with_capacity(new_length);

    for i in 0..new_length {
        let src_index = i as f32 / ratio;
        let src_index_floor = src_index.floor() as usize;

        if src_index_floor >= samples.len() - 1 {
            break;
        }

        let frac = src_index - src_index_floor as f32;
        let sample = samples[src_index_floor] * (1.0 - frac) + samples[src_index_floor + 1] * frac;
        result.push(sample);
    }

    debug!(
        "Resampled {} samples from {} Hz to {} Hz",
        samples.len(),
        from_rate,
        to_rate
    );

    result
}

/// Convert stereo to mono by averaging channels
pub fn stereo_to_mono(samples: &[f32], channels: u16) -> Vec<f32> {
    if channels < 2 || samples.len() < 2 {
        return samples.to_vec();
    }

    let frame_count = samples.len() / channels as usize;
    let mut mono = Vec::with_capacity(frame_count);

    for i in 0..frame_count {
        let sum: f32 = (0..channels as usize)
            .map(|ch| samples[i * channels as usize + ch])
            .sum();
        mono.push(sum / channels as f32);
    }

    mono
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize() {
        let mut samples = vec![0.1, 0.5, 1.0, -0.5, -1.0];
        normalize(&mut samples, 0.5);
        assert!((samples[4] - (-0.5)).abs() < 0.001);
    }

    #[test]
    fn test_calculate_rms() {
        let samples = vec![1.0, -1.0, 1.0, -1.0];
        let rms = calculate_rms(&samples);
        assert!((rms - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_resample() {
        let samples = vec![1.0, 2.0, 3.0, 4.0];
        let resampled = resample(&samples, 4000, 8000);
        // Doubling rate with 4 samples gives 6 output samples (last index breaks early)
        assert_eq!(resampled.len(), 6);
    }
}
