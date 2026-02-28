//! Voice profile module

use serde::{Deserialize, Serialize};

/// Voice profile for a GM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceProfile {
    /// Unique ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Speaker embedding (encrypted blob)
    pub embedding: Vec<u8>,
    /// Baseline emotions
    pub emotion_baseline: EmotionBaseline,
    /// Is default profile
    pub is_default: bool,
    /// Consent given
    pub consent_given: bool,
    /// Created at
    pub created_at: i64,
    /// Updated at
    pub updated_at: i64,
}

/// Baseline emotion values
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EmotionBaseline {
    /// Neutral baseline
    pub neutral: f32,
    /// Happy baseline
    pub happy: f32,
    /// Sad baseline
    pub sad: f32,
    /// Angry baseline
    pub angry: f32,
    /// Fearful baseline
    pub fearful: f32,
    /// Surprised baseline
    pub surprised: f32,
    /// Disgusted baseline
    pub disgusted: f32,
}

impl VoiceProfile {
    /// Create a new voice profile
    pub fn new(id: String, name: String) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id,
            name,
            embedding: Vec::new(),
            emotion_baseline: EmotionBaseline::default(),
            is_default: false,
            consent_given: false,
            created_at: now,
            updated_at: now,
        }
    }

    /// Set embedding
    pub fn set_embedding(&mut self, embedding: Vec<u8>) {
        self.embedding = embedding;
        self.updated_at = chrono::Utc::now().timestamp();
    }

    /// Set emotion baseline
    pub fn set_emotion_baseline(&mut self, baseline: EmotionBaseline) {
        self.emotion_baseline = baseline;
        self.updated_at = chrono::Utc::now().timestamp();
    }
}

/// Training passage for voice enrollment
#[derive(Debug, Clone)]
pub struct TrainingPassage {
    /// Passage text
    pub text: String,
    /// Target emotion
    pub emotion: String,
    /// Duration estimate (seconds)
    pub duration_secs: u32,
}

/// Default training passages
pub fn default_training_passages() -> Vec<TrainingPassage> {
    vec![
        TrainingPassage {
            text: "Welcome, adventurers, to the beginning of your journey. The world before you is vast and full of mystery.".to_string(),
            emotion: "neutral".to_string(),
            duration_secs: 10,
        },
        TrainingPassage {
            text: "Huzzah! Victory is yours! The treasure is yours to claim!".to_string(),
            emotion: "happy".to_string(),
            duration_secs: 8,
        },
        TrainingPassage {
            text: "Alas, your companion has fallen. The weight of loss settles upon you all.".to_string(),
            emotion: "sad".to_string(),
            duration_secs: 10,
        },
        TrainingPassage {
            text: "You dare challenge me? I shall crush you like the insect you are!".to_string(),
            emotion: "angry".to_string(),
            duration_secs: 8,
        },
        TrainingPassage {
            text: "Did you hear that? Something moves in the darkness. We are not alone...".to_string(),
            emotion: "fearful".to_string(),
            duration_secs: 10,
        },
        TrainingPassage {
            text: "Behold! The ancient dragon awakens from its slumber!".to_string(),
            emotion: "surprised".to_string(),
            duration_secs: 8,
        },
        TrainingPassage {
            text: "The smell of decay fills your nostrils. Something terrible has happened here.".to_string(),
            emotion: "disgusted".to_string(),
            duration_secs: 9,
        },
    ]
}

/// Voice training session
pub struct VoiceTraining {
    passages: Vec<TrainingPassage>,
    current_passage: usize,
    recordings: Vec<Vec<f32>>,
}

impl VoiceTraining {
    /// Create a new training session
    pub fn new() -> Self {
        Self {
            passages: default_training_passages(),
            current_passage: 0,
            recordings: Vec::new(),
        }
    }

    /// Get current passage
    pub fn current_passage(&self) -> Option<&TrainingPassage> {
        self.passages.get(self.current_passage)
    }

    /// Add recording for current passage
    pub fn add_recording(&mut self, audio: Vec<f32>) {
        self.recordings.push(audio);
    }

    /// Move to next passage
    pub fn next_passage(&mut self) -> bool {
        if self.current_passage < self.passages.len() - 1 {
            self.current_passage += 1;
            true
        } else {
            false
        }
    }

    /// Check if training is complete
    pub fn is_complete(&self) -> bool {
        self.recordings.len() >= self.passages.len()
    }

    /// Get progress
    pub fn progress(&self) -> (usize, usize) {
        (self.recordings.len(), self.passages.len())
    }
}

impl Default for VoiceTraining {
    fn default() -> Self {
        Self::new()
    }
}
