//! Detection state machine

use serde::{Deserialize, Serialize};
use std::fmt;

/// Detection modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DetectionMode {
    /// Autonomous mode - auto-triggers music/SFX based on detection
    Autonomous,
    /// Collaborative mode - suggests but requires GM confirmation
    Collaborative,
}

impl Default for DetectionMode {
    fn default() -> Self {
        DetectionMode::Autonomous
    }
}

impl fmt::Display for DetectionMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DetectionMode::Autonomous => write!(f, "autonomous"),
            DetectionMode::Collaborative => write!(f, "collaborative"),
        }
    }
}

/// Detection states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DetectionState {
    /// Listening for voice activity
    Listening,
    /// Voice detected, processing
    Detecting,
    /// Signal locked, executing response
    Locked,
    /// Cooldown between detections
    Cooldown,
}

impl Default for DetectionState {
    fn default() -> Self {
        DetectionState::Listening
    }
}

impl fmt::Display for DetectionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DetectionState::Listening => write!(f, "listening"),
            DetectionState::Detecting => write!(f, "detecting"),
            DetectionState::Locked => write!(f, "locked"),
            DetectionState::Cooldown => write!(f, "cooldown"),
        }
    }
}

/// Detection events that drive the state machine
#[derive(Debug, Clone)]
pub enum DetectionEvent {
    /// Voice activity detected
    VoiceDetected,
    /// No voice activity
    VoiceEnded,
    /// Transcription result available
    TranscriptionReady(String),
    /// Keyword matched
    KeywordMatched(String),
    /// Emotion detected
    EmotionDetected(String, f32),
    /// Speaker verified
    SpeakerVerified(bool),
    /// Signal 1 (keyword) trigger
    Signal1Triggered(String),
    /// Signal 2 (emotion) trigger
    Signal2Triggered(String, f32),
    /// Both signals confirmed
    DualSignalConfirmed { keyword: String, emotion: String },
    /// Detection timeout
    Timeout,
    /// Cooldown complete
    CooldownComplete,
    /// Reset to listening
    Reset,
}

impl fmt::Display for DetectionEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DetectionEvent::VoiceDetected => write!(f, "voice_detected"),
            DetectionEvent::VoiceEnded => write!(f, "voice_ended"),
            DetectionEvent::TranscriptionReady(text) => write!(f, "transcription: {}", text),
            DetectionEvent::KeywordMatched(kw) => write!(f, "keyword: {}", kw),
            DetectionEvent::EmotionDetected(emotion, conf) => {
                write!(f, "emotion: {} ({:.2})", emotion, conf)
            }
            DetectionEvent::SpeakerVerified(verified) => {
                write!(f, "speaker_verified: {}", verified)
            }
            DetectionEvent::Signal1Triggered(kw) => write!(f, "signal1: {}", kw),
            DetectionEvent::Signal2Triggered(emotion, conf) => {
                write!(f, "signal2: {} ({:.2})", emotion, conf)
            }
            DetectionEvent::DualSignalConfirmed { keyword, emotion } => {
                write!(f, "dual: {} + {}", keyword, emotion)
            }
            DetectionEvent::Timeout => write!(f, "timeout"),
            DetectionEvent::CooldownComplete => write!(f, "cooldown_complete"),
            DetectionEvent::Reset => write!(f, "reset"),
        }
    }
}

/// Detection state machine
pub struct DetectionFsm {
    state: DetectionState,
    mode: DetectionMode,
    signal1_confirmed: bool,
    signal2_confirmed: bool,
    last_keyword: Option<String>,
    last_emotion: Option<String>,
    cooldown_frames: u32,
    max_cooldown_frames: u32,
}

impl DetectionFsm {
    /// Create a new detection FSM
    pub fn new() -> Self {
        Self {
            state: DetectionState::Listening,
            mode: DetectionMode::Autonomous,
            signal1_confirmed: false,
            signal2_confirmed: false,
            last_keyword: None,
            last_emotion: None,
            cooldown_frames: 0,
            max_cooldown_frames: 300, // ~5 seconds at 60fps
        }
    }

    /// Set the detection mode
    pub fn set_mode(&mut self, mode: DetectionMode) {
        self.mode = mode;
    }

    /// Get current state
    pub fn state(&self) -> DetectionState {
        self.state
    }

    /// Process an event and return the new state
    pub fn process_event(&mut self, event: &DetectionEvent) -> DetectionState {
        use DetectionState::*;

        match (self.state.clone(), event) {
            // Listening state transitions
            (Listening, DetectionEvent::VoiceDetected) => {
                self.state = Detecting;
                self.signal1_confirmed = false;
                self.signal2_confirmed = false;
                tracing::debug!("Detection FSM: Listening -> Detecting");
            }

            // Detecting state transitions
            (Detecting, DetectionEvent::KeywordMatched(kw)) => {
                self.signal1_confirmed = true;
                self.last_keyword = Some(kw.clone());
                self.check_and_transition();
            }
            (Detecting, DetectionEvent::EmotionDetected(emotion, conf)) => {
                if *conf > 0.6 {
                    self.signal2_confirmed = true;
                    self.last_emotion = Some(emotion.clone());
                    self.check_and_transition();
                }
            }
            (Detecting, DetectionEvent::VoiceEnded) => {
                if !self.signal1_confirmed && !self.signal2_confirmed {
                    self.state = Listening;
                    tracing::debug!("Detection FSM: Detecting -> Listening (no signal)");
                }
            }
            (Detecting, DetectionEvent::Timeout) => {
                self.state = Listening;
                tracing::debug!("Detection FSM: Detecting -> Listening (timeout)");
            }

            // Locked state transitions
            (Locked, DetectionEvent::CooldownComplete) => {
                self.state = Listening;
                self.signal1_confirmed = false;
                self.signal2_confirmed = false;
                self.last_keyword = None;
                self.last_emotion = None;
                tracing::debug!("Detection FSM: Locked -> Listening");
            }

            // Any state can be reset
            (_, DetectionEvent::Reset) => {
                self.state = Listening;
                self.signal1_confirmed = false;
                self.signal2_confirmed = false;
                self.last_keyword = None;
                self.last_emotion = None;
                tracing::debug!("Detection FSM: Reset to Listening");
            }

            // Handle dual signal confirmation in any state
            _ => {}
        }

        self.state
    }

    /// Check if both signals are confirmed and transition to locked
    fn check_and_transition(&mut self) {
        if self.signal1_confirmed && self.signal2_confirmed {
            self.state = DetectionState::Locked;
            tracing::info!(
                "Detection FSM: Dual signal confirmed - keyword: {:?}, emotion: {:?}",
                self.last_keyword,
                self.last_emotion
            );
        }
    }

    /// Get the last triggered keyword
    pub fn get_last_keyword(&self) -> Option<&String> {
        self.last_keyword.as_ref()
    }

    /// Get the last detected emotion
    pub fn get_last_emotion(&self) -> Option<&String> {
        self.last_emotion.as_ref()
    }

    /// Check if dual signal is confirmed
    pub fn is_dual_signal_confirmed(&self) -> bool {
        self.signal1_confirmed && self.signal2_confirmed
    }
}

impl Default for DetectionFsm {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detection_fsm() {
        let mut fsm = DetectionFsm::new();

        // Initial state should be listening
        assert_eq!(fsm.state(), DetectionState::Listening);

        // Voice detected -> detecting
        fsm.process_event(&DetectionEvent::VoiceDetected);
        assert_eq!(fsm.state(), DetectionState::Detecting);

        // Keyword matched -> still detecting
        fsm.process_event(&DetectionEvent::KeywordMatched("battle".to_string()));
        assert_eq!(fsm.state(), DetectionState::Detecting);

        // Emotion detected -> locked
        fsm.process_event(&DetectionEvent::EmotionDetected("angry".to_string(), 0.8));
        assert_eq!(fsm.state(), DetectionState::Locked);
        assert!(fsm.is_dual_signal_confirmed());
    }
}
