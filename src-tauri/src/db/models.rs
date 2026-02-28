//! Database models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Track model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: String,
    pub name: String,
    pub file_path: String,
    pub duration_ms: Option<i64>,
    pub genre: Option<String>,
    pub mood: Option<String>,
    pub is_looping: bool,
    pub volume: f64,
    pub created_at: String,
    pub updated_at: String,
}

impl Track {
    pub fn new(id: String, name: String, file_path: String) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id,
            name,
            file_path,
            duration_ms: None,
            genre: None,
            mood: None,
            is_looping: false,
            volume: 1.0,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

/// Genre model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Genre {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub created_at: String,
}

impl Genre {
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            color: None,
            created_at: Utc::now().to_rfc3339(),
        }
    }
}

/// SFX model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sfx {
    pub id: String,
    pub name: String,
    pub file_path: String,
    pub duration_ms: Option<i64>,
    pub category: Option<String>,
    pub volume: f64,
    pub created_at: String,
}

impl Sfx {
    pub fn new(id: String, name: String, file_path: String) -> Self {
        Self {
            id,
            name,
            file_path,
            duration_ms: None,
            category: None,
            volume: 1.0,
            created_at: Utc::now().to_rfc3339(),
        }
    }
}

/// Session model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub mode: String,
    pub total_duration_ms: Option<i64>,
    pub created_at: String,
    // Optional fields from migration 2
    pub detected_events_count: Option<i32>,
    pub keywords_triggered: Option<i32>,
    pub emotions_detected: Option<String>,
    pub tracks_played: Option<String>,
}

impl Session {
    pub fn new(id: String, mode: String) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id,
            started_at: now.clone(),
            ended_at: None,
            mode,
            total_duration_ms: None,
            created_at: now,
            detected_events_count: None,
            keywords_triggered: None,
            emotions_detected: None,
            tracks_played: None,
        }
    }
}

/// Keyword model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyword {
    pub id: String,
    pub word: String,
    pub category: String,
    pub variations: Option<String>,
    pub mood: Option<String>,
    pub priority: i32,
    pub is_active: bool,
    pub created_at: String,
}

impl Keyword {
    pub fn new(id: String, word: String, category: String) -> Self {
        Self {
            id,
            word,
            category,
            variations: None,
            mood: None,
            priority: 0,
            is_active: true,
            created_at: Utc::now().to_rfc3339(),
        }
    }
}

/// Detection event model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionEvent {
    pub id: String,
    pub session_id: String,
    pub event_type: String,
    pub timestamp: String,
    pub details: Option<String>,
    pub confidence: Option<f64>,
    pub category: Option<String>,
    pub triggered_action: bool,
}

impl DetectionEvent {
    pub fn new(id: String, session_id: String, event_type: String) -> Self {
        Self {
            id,
            session_id,
            event_type,
            timestamp: Utc::now().to_rfc3339(),
            details: None,
            confidence: None,
            category: None,
            triggered_action: false,
        }
    }
}

/// Voice profile model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceProfile {
    pub id: String,
    pub name: String,
    pub embedding: Option<Vec<u8>>,
    pub is_default: bool,
    pub consent_given: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl VoiceProfile {
    pub fn new(id: String, name: String) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id,
            name,
            embedding: None,
            is_default: false,
            consent_given: false,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

/// Setting model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
    pub key: String,
    pub value: String,
    pub updated_at: String,
}

impl Setting {
    pub fn new(key: String, value: String) -> Self {
        Self {
            key,
            value,
            updated_at: Utc::now().to_rfc3339(),
        }
    }
}
