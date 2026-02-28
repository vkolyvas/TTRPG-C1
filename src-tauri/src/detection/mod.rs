//! Detection pipeline module
//!
//! This module handles the real-time audio analysis pipeline:
//! - Voice Activity Detection (VAD)
//! - Speaker verification
//! - Speech-to-text transcription
//! - Keyword matching
//! - Detection state machine

pub mod fsm;
pub mod keyword;
pub mod logger;
pub mod pipeline;
pub mod speaker;
pub mod vad;

pub use fsm::*;
pub use keyword::*;
pub use logger::*;
pub use pipeline::*;
pub use speaker::*;
pub use vad::*;
