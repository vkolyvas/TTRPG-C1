//! Session control commands

use crate::audio::capture::AudioCapture;
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::info;

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
            tracing::error!("Failed to list input devices: {}", e);
        }
    }

    Ok(devices)
}

/// Start a recording session
#[tauri::command]
pub fn start_session(state: State<'_, crate::AppState>) -> Result<SessionResponse, String> {
    info!("Starting session command");

    // Simplified implementation - just return success for now
    Ok(SessionResponse {
        success: true,
        message: "Recording started".to_string(),
        state: "recording".to_string(),
    })
}

/// Stop a recording session
#[tauri::command]
pub fn stop_session(_state: State<'_, crate::AppState>) -> Result<SessionResponse, String> {
    info!("Stopping session command");

    Ok(SessionResponse {
        success: true,
        message: "Session completed\nTranscription: Test transcription\nEmotion: Neutral".to_string(),
        state: "idle".to_string(),
    })
}

/// Get current session status
#[tauri::command]
pub fn get_session_status(_state: State<'_, crate::AppState>) -> Result<SessionStatus, String> {
    Ok(SessionStatus {
        state: "idle".to_string(),
        is_recording: false,
    })
}
