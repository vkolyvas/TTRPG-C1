//! Session control commands

use crate::audio::capture::AudioCapture;
use crate::orchestrator::state::{SessionState, SessionResult};
use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{error, info};

/// Response for session commands
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionResponse {
    pub success: bool,
    pub message: String,
    pub state: String,
}

/// Session status response
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionStatus {
    pub state: String,
    pub is_recording: bool,
}

/// Audio device info
#[derive(Debug, Serialize, Deserialize)]
pub struct AudioDevice {
    pub name: String,
    pub is_input: bool,
}

/// Get available audio devices
#[tauri::command]
pub fn get_available_devices() -> Result<Vec<AudioDevice>, String> {
    info!("Getting available audio devices");

    let mut devices = Vec::new();

    // Get input devices
    match AudioCapture::list_devices() {
        Ok(input_devices) => {
            for name in input_devices {
                devices.push(AudioDevice {
                    name,
                    is_input: true,
                });
            }
        }
        Err(e) => {
            error!("Failed to list input devices: {}", e);
        }
    }

    Ok(devices)
}

/// Start a recording session
#[tauri::command]
pub fn start_session(state: State<'_, AppState>) -> Result<SessionResponse, String> {
    info!("Starting session command");

    let mut orchestrator = state
        .orchestrator
        .lock()
        .map_err(|e| format!("Failed to lock orchestrator: {}", e))?;

    // Initialize if needed
    if orchestrator.state() == SessionState::Idle {
        if let Err(e) = orchestrator.init() {
            error!("Failed to initialize orchestrator: {}", e);
            return Err(format!("Initialization error: {}", e));
        }
    }

    match orchestrator.start_session() {
        Ok(()) => {
            info!("Session started successfully");
            Ok(SessionResponse {
                success: true,
                message: "Recording started".to_string(),
                state: orchestrator.state().to_string(),
            })
        }
        Err(e) => {
            error!("Failed to start session: {}", e);
            Err(format!("Failed to start session: {}", e))
        }
    }
}

/// Stop a recording session
#[tauri::command]
pub fn stop_session(state: State<'_, AppState>) -> Result<SessionResponse, String> {
    info!("Stopping session command");

    let mut orchestrator = state
        .orchestrator
        .lock()
        .map_err(|e| format!("Failed to lock orchestrator: {}", e))?;

    match orchestrator.stop_session() {
        Ok(result) => {
            info!("Session stopped successfully");

            let mut message = String::from("Session completed");
            if let Some(t) = &result.transcription {
                message.push_str(&format!("\nTranscription: {}", t.text));
            }
            if let Some(e) = &result.emotion {
                message.push_str(&format!("\nEmotion: {}", e.primary));
            }

            Ok(SessionResponse {
                success: true,
                message,
                state: orchestrator.state().to_string(),
            })
        }
        Err(e) => {
            error!("Failed to stop session: {}", e);
            Err(format!("Failed to stop session: {}", e))
        }
    }
}

/// Get current session status
#[tauri::command]
pub fn get_session_status(state: State<'_, AppState>) -> Result<SessionStatus, String> {
    let orchestrator = state
        .orchestrator
        .lock()
        .map_err(|e| format!("Failed to lock orchestrator: {}", e))?;

    Ok(SessionStatus {
        state: orchestrator.state().to_string(),
        is_recording: orchestrator.state() == SessionState::Recording,
    })
}
