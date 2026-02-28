//! Speaker verification module

use crate::error::AppError;
use crate::state::constants::SPEAKER_SIMILARITY_THRESHOLD;

/// Speaker embedding vector (typically 256-512 dimensions)
#[derive(Debug, Clone)]
pub struct SpeakerEmbedding {
    pub data: Vec<f32>,
    pub dimension: usize,
}

impl SpeakerEmbedding {
    /// Create a new embedding from raw data
    pub fn new(data: Vec<f32>) -> Self {
        let dimension = data.len();
        Self { data, dimension }
    }

    /// Compute cosine similarity with another embedding
    pub fn cosine_similarity(&self, other: &SpeakerEmbedding) -> f32 {
        if self.data.is_empty() || other.data.is_empty() {
            return 0.0;
        }

        let dot_product: f32 = self
            .data
            .iter()
            .zip(other.data.iter())
            .map(|(a, b)| a * b)
            .sum();

        let norm_a: f32 = self.data.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = other.data.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a * norm_b)
    }
}

/// Speaker verification result
#[derive(Debug, Clone)]
pub struct SpeakerVerificationResult {
    pub is_verified: bool,
    pub similarity: f32,
    pub speaker_id: Option<String>,
}

/// GM (Game Master) voice profile
#[derive(Debug, Clone)]
pub struct VoiceProfile {
    pub id: String,
    pub name: String,
    pub embedding: SpeakerEmbedding,
    pub created_at: i64,
    pub is_default: bool,
}

impl VoiceProfile {
    /// Create a new voice profile
    pub fn new(id: String, name: String, embedding: SpeakerEmbedding) -> Self {
        Self {
            id,
            name,
            embedding,
            created_at: chrono::Utc::now().timestamp(),
            is_default: false,
        }
    }
}

/// Speaker verification system
/// Note: This is a placeholder. For production, use Resemblyzer ONNX model.
pub struct SpeakerVerifier {
    threshold: f32,
    enrolled_profiles: Vec<VoiceProfile>,
}

impl SpeakerVerifier {
    /// Create a new speaker verifier
    pub fn new() -> Self {
        Self {
            threshold: SPEAKER_SIMILARITY_THRESHOLD,
            enrolled_profiles: Vec::new(),
        }
    }

    /// Set the verification threshold
    pub fn set_threshold(&mut self, threshold: f32) {
        self.threshold = threshold.clamp(0.0, 1.0);
    }

    /// Enroll a new voice profile
    pub fn enroll(&mut self, profile: VoiceProfile) {
        tracing::info!("Enrolling voice profile: {}", profile.name);
        self.enrolled_profiles.push(profile);
    }

    /// Remove a voice profile
    pub fn remove_profile(&mut self, profile_id: &str) {
        self.enrolled_profiles
            .retain(|p| p.id != profile_id);
    }

    /// Get all enrolled profiles
    pub fn get_profiles(&self) -> &[VoiceProfile] {
        &self.enrolled_profiles
    }

    /// Verify speaker against enrolled profiles
    pub fn verify(&self, embedding: &SpeakerEmbedding) -> SpeakerVerificationResult {
        if self.enrolled_profiles.is_empty() {
            return SpeakerVerificationResult {
                is_verified: false,
                similarity: 0.0,
                speaker_id: None,
            };
        }

        let mut best_match: Option<(String, f32)> = None;

        for profile in &self.enrolled_profiles {
            let similarity = embedding.cosine_similarity(&profile.embedding);

            if best_match.is_none() || similarity > best_match.as_ref().unwrap().1 {
                best_match = Some((profile.id.clone(), similarity));
            }
        }

        if let Some((id, similarity)) = best_match {
            let is_verified = similarity >= self.threshold;
            SpeakerVerificationResult {
                is_verified,
                similarity,
                speaker_id: Some(id),
            }
        } else {
            SpeakerVerificationResult {
                is_verified: false,
                similarity: 0.0,
                speaker_id: None,
            }
        }
    }

    /// Extract embedding from audio (placeholder)
    pub fn extract_embedding(&self, _samples: &[f32], _sample_rate: u32) -> SpeakerEmbedding {
        // Placeholder: In production, run ONNX inference with Resemblyzer model
        // For now, return a random embedding
        let dimension = 256;
        let data: Vec<f32> = (0..dimension).map(|_| rand_simple()).collect();
        SpeakerEmbedding::new(data)
    }
}

impl Default for SpeakerVerifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple random number generator (placeholder)
fn rand_simple() -> f32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    ((nanos % 1000) as f32) / 1000.0
}

/// Resemblyzer ONNX-based speaker verification
pub struct ResemblyzerVerifier {
    session: Option<()>,
    threshold: f32,
}

impl ResemblyzerVerifier {
    /// Create a new Resemblyzer verifier
    pub fn new() -> Self {
        Self {
            session: None,
            threshold: SPEAKER_SIMILARITY_THRESHOLD,
        }
    }

    /// Initialize with ONNX model
    pub fn init(&mut self, model_path: &str) -> Result<(), AppError> {
        tracing::info!(
            "Initializing Resemblyzer with model: {}",
            model_path
        );
        // Placeholder: Load ONNX model here
        // self.session = Some(ort::Session::from_file(model_path)?);
        Ok(())
    }

    /// Verify speaker
    pub fn verify(&self, embedding: &SpeakerEmbedding) -> SpeakerVerificationResult {
        // Placeholder implementation
        SpeakerVerificationResult {
            is_verified: false,
            similarity: 0.0,
            speaker_id: None,
        }
    }
}

impl Default for ResemblyzerVerifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let emb1 = SpeakerEmbedding::new(vec![1.0, 0.0, 0.0]);
        let emb2 = SpeakerEmbedding::new(vec![1.0, 0.0, 0.0]);
        let emb3 = SpeakerEmbedding::new(vec![0.0, 1.0, 0.0]);

        assert!((emb1.cosine_similarity(&emb2) - 1.0).abs() < 0.001);
        assert!((emb1.cosine_similarity(&emb3) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_speaker_verification() {
        let mut verifier = SpeakerVerifier::new();

        // Create test profile
        let embedding = SpeakerEmbedding::new(vec![1.0, 0.0, 0.0]);
        let profile = VoiceProfile::new("test".to_string(), "Test GM".to_string(), embedding);
        verifier.enroll(profile);

        // Verify with same embedding
        let test_embedding = SpeakerEmbedding::new(vec![1.0, 0.0, 0.0]);
        let result = verifier.verify(&test_embedding);
        assert!(result.is_verified);
    }
}
