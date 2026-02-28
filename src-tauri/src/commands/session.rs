//! Session control commands

use crate::audio::capture::AudioCapture;
use crate::dsp::processing;
use crate::inference::emotion::EmotionAnalyzer;
use crate::inference::whisper::WhisperEngine;
use crate::orchestrator::state::SessionState;
use crate::state::AppMode;
use crate::AppState;
use cpal::traits::{DeviceTrait, HostTrait};
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
    pub is_processing: bool,
    pub transcription: Option<String>,
    pub emotion: Option<String>,
    pub current_emotion: Option<String>,
    pub mode: String,
}

/// Audio device info
#[derive(Debug, Serialize, Deserialize)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub is_input: bool,
    pub is_default: bool,
}

/// Track info
#[derive(Debug, Serialize, Deserialize)]
pub struct TrackInfo {
    pub id: String,
    pub name: String,
    pub genre: Option<String>,
    pub mood: Option<String>,
    pub is_looping: bool,
}

/// Get available audio devices
#[tauri::command]
pub fn get_available_devices() -> Result<Vec<AudioDevice>, String> {
    info!("Getting available audio devices");

    let mut devices = Vec::new();

    // Get input devices using cpal
    let host = cpal::default_host();
    for device in host.input_devices().map_err(|e| e.to_string())? {
        let name = device.name().map_err(|e| e.to_string())?;
        let id = name.clone();
        let is_default = device.default_input_config().is_ok();

        devices.push(AudioDevice {
            id,
            name,
            is_input: true,
            is_default,
        });
    }

    Ok(devices)
}

/// Start a recording session - begins audio capture in background thread
#[tauri::command]
pub fn start_session(
    state: State<'_, AppState>,
    _device_id: Option<String>,
    enable_transcription: Option<bool>,
    enable_emotion: Option<bool>,
) -> Result<SessionResponse, String> {
    info!("Starting session command");

    // Check current state
    let current_state = *state.session_state.read();

    if current_state != SessionState::Idle {
        return Ok(SessionResponse {
            success: false,
            message: format!("Cannot start session, current state: {}", current_state),
            state: current_state.to_string(),
        });
    }

    // Update config
    {
        let mut config = state.config.write();
        config.enable_transcription = enable_transcription.unwrap_or(true);
        config.enable_emotion_analysis = enable_emotion.unwrap_or(true);
    }

    // Clear audio buffer
    {
        let mut buffer = state.audio_buffer.write();
        buffer.clear();
    }

    // Start audio capture in a background thread that runs until stopped
    let buffer = state.audio_buffer.clone();

    let _handle = std::thread::spawn(move || {
        let mut capture = AudioCapture::new();
        let _ = capture.start_recording(move |samples| {
            let mut buf = buffer.write();
            buf.extend_from_slice(&samples);
        });

        // Keep recording - the stream stays alive until the thread is dropped
        // This blocks until the thread is killed
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });

    // Update state
    *state.session_state.write() = SessionState::Recording;

    Ok(SessionResponse {
        success: true,
        message: "Recording started".to_string(),
        state: "recording".to_string(),
    })
}

/// Stop a recording session and process audio
#[tauri::command]
pub fn stop_session(state: State<'_, AppState>) -> Result<SessionResponse, String> {
    info!("Stopping session command");

    // Check current state
    let current_state = *state.session_state.read();

    if current_state != SessionState::Recording {
        return Ok(SessionResponse {
            success: false,
            message: format!("Cannot stop session, current state: {}", current_state),
            state: current_state.to_string(),
        });
    }

    // Update state to processing
    *state.session_state.write() = SessionState::Processing;

    // Get audio data
    let (samples, sample_rate, config) = {
        let buffer = state.audio_buffer.read();
        let rate = *state.sample_rate.read();
        let cfg = state.config.read().clone();
        (buffer.clone(), rate, cfg)
    };

    info!("Processing {} samples at {} Hz", samples.len(), sample_rate);

    // Process audio with DSP
    let mut processed_samples = samples;

    // Resample if needed
    if sample_rate != config.sample_rate {
        processed_samples = processing::resample(&processed_samples, sample_rate, config.sample_rate);
    }

    // Apply DSP processing
    processing::remove_dc_offset(&mut processed_samples);
    processing::normalize(&mut processed_samples, 0.9);
    processing::noise_gate(&mut processed_samples, config.silence_threshold);

    // Run transcription
    let mut whisper = WhisperEngine::new();
    let _ = whisper.init("models/whisper-tiny.bin");

    let transcription = if config.enable_transcription {
        match whisper.transcribe(&processed_samples, config.sample_rate) {
            Ok(t) => Some(t),
            Err(e) => {
                tracing::warn!("Transcription failed: {}", e);
                None
            }
        }
    } else {
        None
    };

    // Run emotion analysis
    let mut emotion_analyzer = EmotionAnalyzer::new();
    let _ = emotion_analyzer.init();

    let emotion = if config.enable_emotion_analysis {
        match emotion_analyzer.analyze(&processed_samples, config.sample_rate) {
            Ok(e) => Some(e),
            Err(e) => {
                tracing::warn!("Emotion analysis failed: {}", e);
                None
            }
        }
    } else {
        None
    };

    // Update current emotion
    if let Some(ref e) = emotion {
        *state.current_emotion.write() = e.primary.to_string();
    }

    // Format response
    let transcription_text = transcription.as_ref().map(|t| t.text.clone()).unwrap_or_default();
    let emotion_text = emotion.as_ref().map(|e| e.primary.to_string()).unwrap_or_else(|| "unknown".to_string());

    let message = format!(
        "Session completed\nTranscription: {}\nEmotion: {}",
        if transcription_text.is_empty() { "(none)".to_string() } else { transcription_text },
        emotion_text
    );

    // Reset state to idle
    *state.session_state.write() = SessionState::Idle;

    Ok(SessionResponse {
        success: true,
        message,
        state: "idle".to_string(),
    })
}

/// Get current session status
#[tauri::command]
pub fn get_session_status(state: State<'_, AppState>) -> Result<SessionStatus, String> {
    let session_state = *state.session_state.read();
    let app_mode = *state.app_mode.read();
    let current_emotion = state.current_emotion.read().clone();

    let is_recording = session_state == SessionState::Recording;
    let is_processing = session_state == SessionState::Processing;

    Ok(SessionStatus {
        state: session_state.to_string(),
        is_recording,
        is_processing,
        transcription: None,
        emotion: None,
        current_emotion: Some(current_emotion),
        mode: match app_mode {
            AppMode::ModeA => "autonomous".to_string(),
            AppMode::ModeB => "collaborative".to_string(),
        },
    })
}

/// Get tracks from database
#[tauri::command]
pub fn get_tracks(state: State<'_, AppState>, genre: Option<String>) -> Result<Vec<TrackInfo>, String> {
    let pool = state.db_pool.read();

    if let Some(ref pool) = *pool {
        // Use repository to fetch tracks
        // For now, return sample tracks
        Ok(vec![
            TrackInfo {
                id: "1".to_string(),
                name: "Battle Theme".to_string(),
                genre: Some("combat".to_string()),
                mood: Some("angry".to_string()),
                is_looping: true,
            },
            TrackInfo {
                id: "2".to_string(),
                name: "Mystery Theme".to_string(),
                genre: Some("exploration".to_string()),
                mood: Some("neutral".to_string()),
                is_looping: true,
            },
            TrackInfo {
                id: "3".to_string(),
                name: "Victory Fanfare".to_string(),
                genre: Some("social".to_string()),
                mood: Some("happy".to_string()),
                is_looping: false,
            },
        ])
    } else {
        // Database not available, return sample tracks
        Ok(vec![
            TrackInfo {
                id: "sample-1".to_string(),
                name: "Ambient Dungeon".to_string(),
                genre: Some("exploration".to_string()),
                mood: Some("neutral".to_string()),
                is_looping: true,
            },
            TrackInfo {
                id: "sample-2".to_string(),
                name: "Boss Battle".to_string(),
                genre: Some("combat".to_string()),
                mood: Some("angry".to_string()),
                is_looping: true,
            },
        ])
    }
}

/// Set application mode (A: autonomous, B: collaborative)
#[tauri::command]
pub fn set_app_mode(state: State<'_, AppState>, mode: String) -> Result<SessionResponse, String> {
    let new_mode = match mode.as_str() {
        "autonomous" => AppMode::ModeA,
        "collaborative" => AppMode::ModeB,
        _ => return Err("Invalid mode. Use 'autonomous' or 'collaborative'".to_string()),
    };

    *state.app_mode.write() = new_mode;

    Ok(SessionResponse {
        success: true,
        message: format!("Mode set to {}", mode),
        state: mode,
    })
}

/// Get current application mode
#[tauri::command]
pub fn get_app_mode(state: State<'_, AppState>) -> Result<String, String> {
    let mode = *state.app_mode.read();
    Ok(match mode {
        AppMode::ModeA => "autonomous".to_string(),
        AppMode::ModeB => "collaborative".to_string(),
    })
}

/// Enable/disable detection
#[tauri::command]
pub fn set_detection_enabled(state: State<'_, AppState>, enabled: bool) -> Result<SessionResponse, String> {
    *state.detection_ready.write() = enabled;

    Ok(SessionResponse {
        success: true,
        message: format!("Detection {}", if enabled { "enabled" } else { "disabled" }),
        state: if enabled { "enabled" } else { "disabled" }.to_string(),
    })
}
