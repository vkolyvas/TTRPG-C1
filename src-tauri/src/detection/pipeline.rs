//! Detection pipeline - orchestrates all detection components

use crate::detection::fsm::{DetectionEvent, DetectionFsm, DetectionMode, DetectionState};
use crate::detection::keyword::{default_ttrpg_vocabulary, KeywordDetector};
use crate::detection::speaker::{SpeakerVerifier, SpeakerEmbedding};
use crate::detection::vad::VoiceActivityDetector;
use crate::error::AppError;
use crate::inference::emotion::EmotionAnalyzer;
use crate::inference::whisper::WhisperEngine;
use flume::{Receiver, Sender};
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;

/// Detection pipeline configuration
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub enable_vad: bool,
    pub enable_speaker_verification: bool,
    pub enable_transcription: bool,
    pub enable_emotion: bool,
    pub vad_threshold: f32,
    pub transcription_segment_ms: u32,
    pub detection_timeout_ms: u64,
    pub cooldown_ms: u64,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            enable_vad: true,
            enable_speaker_verification: false,
            enable_transcription: true,
            enable_emotion: true,
            vad_threshold: 0.5,
            transcription_segment_ms: 8000,
            detection_timeout_ms: 10000,
            cooldown_ms: 3000,
        }
    }
}

/// Detection pipeline event
#[derive(Debug, Clone)]
pub enum PipelineEvent {
    /// Voice activity started
    VoiceStart(u64),
    /// Voice activity ended
    VoiceEnd { start_ms: u64, end_ms: u64 },
    /// Transcription ready
    Transcription(String),
    /// Keyword detected
    Keyword(String),
    /// Emotion detected
    Emotion(String, f32),
    /// Dual signal confirmed
    DualSignal { keyword: String, emotion: String },
    /// Speaker verified
    SpeakerVerified(bool),
    /// Pipeline error
    Error(String),
}

/// Detection pipeline
pub struct DetectionPipeline {
    config: PipelineConfig,
    vad: VoiceActivityDetector,
    speaker_verifier: SpeakerVerifier,
    keyword_detector: KeywordDetector,
    whisper: WhisperEngine,
    emotion_analyzer: EmotionAnalyzer,
    fsm: DetectionFsm,
    audio_buffer: Arc<RwLock<Vec<f32>>>,
    segment_buffer: Vec<f32>,
    event_tx: Option<Sender<PipelineEvent>>,
    sample_rate: u32,
    last_voice_time: Option<Instant>,
    is_running: bool,
}

impl DetectionPipeline {
    /// Create a new detection pipeline
    pub fn new(config: PipelineConfig) -> Self {
        let mut vad = VoiceActivityDetector::new();
        vad.set_threshold(config.vad_threshold);

        let mut keyword_detector = KeywordDetector::new();
        keyword_detector.set_vocabulary(default_ttrpg_vocabulary());

        Self {
            config,
            vad,
            speaker_verifier: SpeakerVerifier::new(),
            keyword_detector,
            whisper: WhisperEngine::new(),
            emotion_analyzer: EmotionAnalyzer::new(),
            fsm: DetectionFsm::new(),
            audio_buffer: Arc::new(RwLock::new(Vec::new())),
            segment_buffer: Vec::new(),
            event_tx: None,
            sample_rate: 16000,
            last_voice_time: None,
            is_running: false,
        }
    }

    /// Initialize the pipeline
    pub fn init(&mut self) -> Result<(), AppError> {
        tracing::info!("Initializing detection pipeline");

        // Initialize whisper
        if let Err(e) = self.whisper.init("models/whisper-tiny.bin") {
            tracing::warn!("Whisper init warning: {}", e);
        }

        // Initialize emotion analyzer
        if let Err(e) = self.emotion_analyzer.init() {
            tracing::warn!("Emotion analyzer init warning: {}", e);
        }

        tracing::info!("Detection pipeline initialized");
        Ok(())
    }

    /// Set the event sender
    pub fn set_event_sender(&mut self, tx: Sender<PipelineEvent>) {
        self.event_tx = Some(tx);
    }

    /// Set the audio buffer
    pub fn set_audio_buffer(&mut self, buffer: Arc<RwLock<Vec<f32>>>) {
        self.audio_buffer = buffer;
    }

    /// Set sample rate
    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate;
        self.vad.set_sample_rate(sample_rate);
    }

    /// Set detection mode
    pub fn set_mode(&mut self, mode: DetectionMode) {
        self.fsm.set_mode(mode);
    }

    /// Process incoming audio data
    pub fn process_audio(&mut self, samples: &[f32], timestamp_ms: u64) {
        if !self.is_running {
            return;
        }

        // Add to buffers
        {
            let mut buffer = self.audio_buffer.write();
            buffer.extend_from_slice(samples);
        }
        self.segment_buffer.extend_from_slice(samples);

        // Run VAD
        if self.config.enable_vad {
            let vad_result = self.vad.process_frame(samples, timestamp_ms);

            if vad_result.is_speech {
                self.last_voice_time = Some(Instant::now());

                // Notify FSM
                self.fsm.process_event(&DetectionEvent::VoiceDetected);

                // Emit event
                self.emit(PipelineEvent::VoiceStart(timestamp_ms));
            } else if vad_result.start_ms.is_some() {
                // Voice ended
                if let Some(start) = vad_result.start_ms {
                    self.emit(PipelineEvent::VoiceEnd {
                        start_ms: start,
                        end_ms: timestamp_ms,
                    });
                }
            }
        }

        // Check if we should process a segment
        let segment_samples = (self.sample_rate as u32 * self.config.transcription_segment_ms) / 1000;
        if self.segment_buffer.len() >= segment_samples as usize {
            self.process_segment();
        }
    }

    /// Process accumulated audio segment
    fn process_segment(&mut self) {
        if self.segment_buffer.is_empty() {
            return;
        }

        let segment = std::mem::take(&mut self.segment_buffer);
        self.segment_buffer = Vec::new();

        // Run transcription
        if self.config.enable_transcription {
            match self.whisper.transcribe(&segment, self.sample_rate) {
                Ok(result) => {
                    if !result.text.is_empty() {
                        tracing::debug!("Transcription: {}", result.text);
                        self.emit(PipelineEvent::Transcription(result.text.clone()));

                        // Check keywords
                        let matches = self.keyword_detector.detect(&result.text);
                        for m in matches {
                            tracing::info!("Keyword detected: {} ({})", m.keyword, m.category);
                            self.fsm.process_event(&DetectionEvent::KeywordMatched(m.keyword.clone()));
                            self.emit(PipelineEvent::Keyword(m.keyword));
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Transcription error: {}", e);
                }
            }
        }

        // Run emotion analysis
        if self.config.enable_emotion {
            match self.emotion_analyzer.analyze(&segment, self.sample_rate) {
                Ok(result) => {
                    let emotion_str = result.primary.to_string();
                    tracing::debug!("Emotion: {} ({:.2})", emotion_str, result.confidence);
                    self.fsm.process_event(&DetectionEvent::EmotionDetected(
                        emotion_str.clone(),
                        result.confidence,
                    ));
                    self.emit(PipelineEvent::Emotion(emotion_str, result.confidence));
                }
                Err(e) => {
                    tracing::warn!("Emotion analysis error: {}", e);
                }
            }
        }

        // Check FSM state
        let state = self.fsm.state();
        if self.fsm.is_dual_signal_confirmed() {
            if let (Some(keyword), Some(emotion)) = (
                self.fsm.get_last_keyword().cloned(),
                self.fsm.get_last_emotion().cloned(),
            ) {
                self.emit(PipelineEvent::DualSignal {
                    keyword,
                    emotion,
                });
            }
        }
    }

    /// Start the pipeline
    pub fn start(&mut self) {
        self.is_running = true;
        self.fsm.process_event(&DetectionEvent::Reset);
        tracing::info!("Detection pipeline started");
    }

    /// Stop the pipeline
    pub fn stop(&mut self) {
        self.is_running = false;
        tracing::info!("Detection pipeline stopped");
    }

    /// Check if running
    pub fn is_running(&self) -> bool {
        self.is_running
    }

    /// Get current detection state
    pub fn state(&self) -> DetectionState {
        self.fsm.state()
    }

    /// Emit an event
    fn emit(&self, event: PipelineEvent) {
        if let Some(tx) = &self.event_tx {
            let _ = tx.send(event);
        }
    }
}

impl Default for DetectionPipeline {
    fn default() -> Self {
        Self::new(PipelineConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_creation() {
        let pipeline = DetectionPipeline::new(PipelineConfig::default());
        assert!(!pipeline.is_running());
    }
}
