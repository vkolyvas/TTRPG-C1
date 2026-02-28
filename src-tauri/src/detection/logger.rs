//! Detection event logging

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Detection event log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionLogEntry {
    /// Unique ID for this detection event
    pub id: String,
    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,
    /// Session ID
    pub session_id: String,
    /// Event type
    pub event_type: String,
    /// Event details (JSON)
    pub details: String,
    /// Confidence score (if applicable)
    pub confidence: Option<f32>,
    /// Category (if applicable)
    pub category: Option<String>,
    /// Whether this triggered an action
    pub triggered_action: bool,
}

impl DetectionLogEntry {
    /// Create a new log entry
    pub fn new(session_id: String, event_type: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            session_id,
            event_type: event_type.to_string(),
            details: String::new(),
            confidence: None,
            category: None,
            triggered_action: false,
        }
    }

    /// Set details
    pub fn with_details(mut self, details: &str) -> Self {
        self.details = details.to_string();
        self
    }

    /// Set confidence
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = Some(confidence);
        self
    }

    /// Set category
    pub fn with_category(mut self, category: &str) -> Self {
        self.category = Some(category.to_string());
        self
    }

    /// Mark as triggered action
    pub fn with_triggered_action(mut self) -> Self {
        self.triggered_action = true;
        self
    }
}

/// Detection event logger
pub struct DetectionLogger {
    session_id: String,
    entries: Vec<DetectionLogEntry>,
    max_entries: usize,
}

impl DetectionLogger {
    /// Create a new logger
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            entries: Vec::new(),
            max_entries: 10000,
        }
    }

    /// Set session ID
    pub fn set_session_id(&mut self, session_id: String) {
        self.session_id = session_id;
    }

    /// Log an event
    pub fn log(&mut self, event_type: &str) -> &DetectionLogEntry {
        let entry = DetectionLogEntry::new(self.session_id.clone(), event_type);
        self.entries.push(entry);
        self.trim();
        self.entries.last().unwrap()
    }

    /// Log keyword detection
    pub fn log_keyword(&mut self, keyword: &str, category: &str, confidence: f32) {
        let entry = DetectionLogEntry::new(self.session_id.clone(), "keyword")
            .with_details(keyword)
            .with_category(category)
            .with_confidence(confidence);
        self.entries.push(entry);
        self.trim();
    }

    /// Log emotion detection
    pub fn log_emotion(&mut self, emotion: &str, confidence: f32) {
        let entry = DetectionLogEntry::new(self.session_id.clone(), "emotion")
            .with_details(emotion)
            .with_confidence(confidence);
        self.entries.push(entry);
        self.trim();
    }

    /// Log dual signal detection
    pub fn log_dual_signal(&mut self, keyword: &str, emotion: &str) {
        let details = format!("keyword: {}, emotion: {}", keyword, emotion);
        let entry = DetectionLogEntry::new(self.session_id.clone(), "dual_signal")
            .with_details(&details)
            .with_triggered_action();
        self.entries.push(entry);
        self.trim();
    }

    /// Log voice activity
    pub fn log_voice_activity(&mut self, is_starting: bool, duration_ms: Option<u64>) {
        let event_type = if is_starting { "voice_start" } else { "voice_end" };
        let details = duration_ms
            .map(|d| format!("duration: {}ms", d))
            .unwrap_or_default();
        let entry = DetectionLogEntry::new(self.session_id.clone(), event_type)
            .with_details(&details);
        self.entries.push(entry);
        self.trim();
    }

    /// Log speaker verification
    pub fn log_speaker_verification(&mut self, verified: bool, similarity: f32) {
        let details = if verified { "verified" } else { "not_verified" };
        let entry = DetectionLogEntry::new(self.session_id.clone(), "speaker_verification")
            .with_details(details)
            .with_confidence(similarity);
        self.entries.push(entry);
        self.trim();
    }

    /// Get all entries
    pub fn entries(&self) -> &[DetectionLogEntry] {
        &self.entries
    }

    /// Get entries by type
    pub fn entries_by_type(&self, event_type: &str) -> Vec<&DetectionLogEntry> {
        self.entries
            .iter()
            .filter(|e| e.event_type == event_type)
            .collect()
    }

    /// Get triggered actions
    pub fn triggered_actions(&self) -> Vec<&DetectionLogEntry> {
        self.entries
            .iter()
            .filter(|e| e.triggered_action)
            .collect()
    }

    /// Export to JSON
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(&self.entries).unwrap_or_default()
    }

    /// Trim entries if over limit
    fn trim(&mut self) {
        if self.entries.len() > self.max_entries {
            self.entries.drain(0..self.entries.len() - self.max_entries);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detection_logger() {
        let mut logger = DetectionLogger::new("test-session".to_string());

        logger.log_keyword("battle", "combat", 1.0);
        logger.log_emotion("angry", 0.85);
        logger.log_dual_signal("battle", "angry");

        assert_eq!(logger.entries().len(), 3);
        assert_eq!(logger.triggered_actions().len(), 1);
    }
}
