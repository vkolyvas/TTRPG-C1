//! Two-phase startup module
//!
//! Implements a two-phase startup process:
//! 1. UI Ready (≤3s) - Fast window display
//! 2. Detection Ready (≤15s) - ML models loaded

use crate::error::AppError;
use crate::state::constants::{DETECTION_READY_TIMEOUT_MS, UI_READY_TIMEOUT_MS};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;

/// Startup phase
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub enum StartupPhase {
    /// Initial state
    Initial,
    /// UI is ready (window displayed)
    UiReady,
    /// Detection pipeline is ready
    DetectionReady,
    /// Fully initialized
    Complete,
}

impl Default for StartupPhase {
    fn default() -> Self {
        StartupPhase::Initial
    }
}

impl std::fmt::Display for StartupPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StartupPhase::Initial => write!(f, "initial"),
            StartupPhase::UiReady => write!(f, "ui_ready"),
            StartupPhase::DetectionReady => write!(f, "detection_ready"),
            StartupPhase::Complete => write!(f, "complete"),
        }
    }
}

/// Startup state
pub struct StartupState {
    phase: RwLock<StartupPhase>,
    start_time: RwLock<Option<Instant>>,
    ui_ready_time: RwLock<Option<Duration>>,
    detection_ready_time: RwLock<Option<Duration>>,
    error: RwLock<Option<String>>,
}

impl StartupState {
    /// Create a new startup state
    pub fn new() -> Self {
        Self {
            phase: RwLock::new(StartupPhase::Initial),
            start_time: RwLock::new(None),
            ui_ready_time: RwLock::new(None),
            detection_ready_time: RwLock::new(None),
            error: RwLock::new(None),
        }
    }

    /// Mark UI as ready
    pub fn mark_ui_ready(&self) {
        let mut phase = self.phase.write();
        *phase = StartupPhase::UiReady;

        let mut start = self.start_time.write();
        if let Some(start_time) = *start {
            let elapsed = start_time.elapsed();
            *self.ui_ready_time.write() = Some(elapsed);
            tracing::info!("UI ready in {}ms", elapsed.as_millis());
        }
    }

    /// Mark detection as ready
    pub fn mark_detection_ready(&self) {
        let mut phase = self.phase.write();
        *phase = StartupPhase::DetectionReady;

        let mut start = self.start_time.write();
        if let Some(start_time) = *start {
            let elapsed = start_time.elapsed();
            *self.detection_ready_time.write() = Some(elapsed);
            tracing::info!("Detection ready in {}ms", elapsed.as_millis());
        }
    }

    /// Mark startup complete
    pub fn mark_complete(&self) {
        let mut phase = self.phase.write();
        *phase = StartupPhase::Complete;
        tracing::info!("Startup complete");
    }

    /// Mark error
    pub fn mark_error(&self, error: String) {
        let error_clone = error.clone();
        *self.error.write() = Some(error);
        tracing::error!("Startup error: {}", error_clone);
    }

    /// Get current phase
    pub fn phase(&self) -> StartupPhase {
        *self.phase.read()
    }

    /// Check if UI is ready
    pub fn is_ui_ready(&self) -> bool {
        *self.phase.read() >= StartupPhase::UiReady
    }

    /// Check if detection is ready
    pub fn is_detection_ready(&self) -> bool {
        *self.phase.read() >= StartupPhase::DetectionReady
    }

    /// Check if complete
    pub fn is_complete(&self) -> bool {
        *self.phase.read() == StartupPhase::Complete
    }

    /// Get UI ready time
    pub fn ui_ready_time(&self) -> Option<Duration> {
        *self.ui_ready_time.read()
    }

    /// Get detection ready time
    pub fn detection_ready_time(&self) -> Option<Duration> {
        *self.detection_ready_time.read()
    }

    /// Get error if any
    pub fn error(&self) -> Option<String> {
        self.error.read().clone()
    }
}

impl Default for StartupState {
    fn default() -> Self {
        Self::new()
    }
}

/// Startup manager
pub struct StartupManager {
    state: Arc<StartupState>,
    timeout_ui_ms: u64,
    timeout_detection_ms: u64,
}

impl StartupManager {
    /// Create a new startup manager
    pub fn new() -> Self {
        Self {
            state: Arc::new(StartupState::new()),
            timeout_ui_ms: UI_READY_TIMEOUT_MS,
            timeout_detection_ms: DETECTION_READY_TIMEOUT_MS,
        }
    }

    /// Create with custom timeouts
    pub fn with_timeouts(timeout_ui_ms: u64, timeout_detection_ms: u64) -> Self {
        Self {
            state: Arc::new(StartupState::new()),
            timeout_ui_ms,
            timeout_detection_ms,
        }
    }

    /// Get the startup state
    pub fn state(&self) -> &Arc<StartupState> {
        &self.state
    }

    /// Start the startup timer
    pub fn start(&self) {
        *self.state.start_time.write() = Some(Instant::now());
        tracing::info!("Startup started");
    }

    /// Check UI timeout
    pub fn check_ui_timeout(&self) -> bool {
        if let Some(start) = *self.state.start_time.read() {
            start.elapsed().as_millis() as u64 > self.timeout_ui_ms
        } else {
            false
        }
    }

    /// Check detection timeout
    pub fn check_detection_timeout(&self) -> bool {
        if let Some(start) = *self.state.start_time.read() {
            start.elapsed().as_millis() as u64 > self.timeout_detection_ms
        } else {
            false
        }
    }
}

impl Default for StartupManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_startup_phases() {
        let state = StartupState::new();
        assert_eq!(state.phase(), StartupPhase::Initial);

        state.mark_ui_ready();
        assert!(state.is_ui_ready());
        assert!(!state.is_detection_ready());

        state.mark_detection_ready();
        assert!(state.is_detection_ready());

        state.mark_complete();
        assert!(state.is_complete());
    }
}
