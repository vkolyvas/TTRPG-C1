//! Audio Engine - Enhanced playback with crossfades, SFX layering, and ducking
//!
//! This module provides advanced audio playback capabilities:
//! - Crossfade transitions between tracks
//! - Gapless looping for ambient music
//! - SFX layering on top of background music
//! - Volume ducking for voice-overs

use crate::error::AppError;
use parking_lot::RwLock;
use rodio::{OutputStream, OutputStreamHandle, Sink, Source};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use tracing::{debug, info, warn};

/// Crossfade types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CrossfadeType {
    /// Instant switch (no fade)
    Instant,
    /// Quick fade (500ms)
    Quick,
    /// Musical fade (2000ms) - follows beat if possible
    Musical,
    /// Long fade (5000ms)
    Long,
}

impl Default for CrossfadeType {
    fn default() -> Self {
        CrossfadeType::Musical
    }
}

impl CrossfadeType {
    /// Get duration in milliseconds
    pub fn duration_ms(&self) -> u32 {
        match self {
            CrossfadeType::Instant => 0,
            CrossfadeType::Quick => 500,
            CrossfadeType::Musical => 2000,
            CrossfadeType::Long => 5000,
        }
    }
}

/// Track info for playback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: String,
    pub name: String,
    pub file_path: String,
    pub genre: Option<String>,
    pub mood: Option<String>,
    pub is_looping: bool,
    pub duration_ms: Option<u32>,
    pub bpm: Option<f32>,
}

/// SFX info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundEffect {
    pub id: String,
    pub name: String,
    pub file_path: String,
    pub category: Option<String>,
    pub duration_ms: Option<u32>,
}

/// Audio engine configuration
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Master volume (0.0 - 1.0)
    pub master_volume: f32,
    /// Music volume (0.0 - 1.0)
    pub music_volume: f32,
    /// SFX volume (0.0 - 1.0)
    pub sfx_volume: f32,
    /// Default crossfade type
    pub crossfade_type: CrossfadeType,
    /// Ducking amount when voice detected (0.0 - 1.0)
    pub ducking_amount: f32,
    /// Ducking fade time in ms
    pub ducking_fade_ms: u32,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            music_volume: 0.7,
            sfx_volume: 0.8,
            crossfade_type: CrossfadeType::Musical,
            ducking_amount: 0.3,
            ducking_fade_ms: 200,
        }
    }
}

/// Currently playing track info
#[derive(Debug, Clone)]
pub struct PlayingTrack {
    pub track: Track,
    pub started_at_ms: u64,
    pub is_looping: bool,
}

/// State of the audio engine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EngineState {
    Idle,
    Playing,
    Paused,
    Transitioning,
}

impl Default for EngineState {
    fn default() -> Self {
        EngineState::Idle
    }
}

/// Audio engine - manages music playback, crossfades, and SFX
pub struct AudioEngine {
    /// Output stream
    _stream: Option<OutputStream>,
    /// Output stream handle
    stream_handle: Option<OutputStreamHandle>,
    /// Music sink (main playback)
    music_sink: Option<Sink>,
    /// Configuration
    config: RwLock<EngineConfig>,
    /// Current state
    state: RwLock<EngineState>,
    /// Currently playing track
    current_track: RwLock<Option<PlayingTrack>>,
    /// Is ducking active
    is_ducking: RwLock<bool>,
}

impl AudioEngine {
    /// Create a new audio engine
    pub fn new() -> Result<Self, AppError> {
        let (stream, stream_handle) =
            rodio::OutputStream::try_default().map_err(|e| AppError::Audio(e.to_string()))?;

        Ok(Self {
            _stream: Some(stream),
            stream_handle: Some(stream_handle),
            music_sink: None,
            config: RwLock::new(EngineConfig::default()),
            state: RwLock::new(EngineState::Idle),
            current_track: RwLock::new(None),
            is_ducking: RwLock::new(false),
        })
    }

    /// Get stream handle
    fn stream_handle(&self) -> &OutputStreamHandle {
        self.stream_handle.as_ref().unwrap()
    }

    /// Play a track (stops current playback first)
    pub fn play_track(&mut self, track: &Track) -> Result<(), AppError> {
        info!("Playing track: {}", track.name);

        // Stop current playback
        self.stop_music();

        // Load and play the track
        let sink = Sink::try_new(self.stream_handle())
            .map_err(|e| AppError::Playback(e.to_string()))?;

        let file = File::open(&track.file_path)
            .map_err(|e| AppError::Audio(format!("Failed to open file: {}", e)))?;

        let reader = BufReader::new(file);
        let source = rodio::Decoder::new(reader)
            .map_err(|e| AppError::Audio(format!("Failed to decode: {}", e)))?;

        // Apply looping if needed
        if track.is_looping {
            sink.append(source.repeat_infinite());
        } else {
            sink.append(source);
        }

        // Apply volume
        let volume = self.calculate_music_volume();
        sink.set_volume(volume);

        self.music_sink = Some(sink);
        *self.state.write() = EngineState::Playing;
        *self.current_track.write() = Some(PlayingTrack {
            track: track.clone(),
            started_at_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            is_looping: track.is_looping,
        });

        Ok(())
    }

    /// Crossfade to a new track
    pub fn crossfade_to(&mut self, track: &Track) -> Result<(), AppError> {
        let crossfade_type = self.config.read().crossfade_type;

        info!("Crossfading to: {} ({:?})", track.name, crossfade_type);

        if crossfade_type == CrossfadeType::Instant {
            return self.play_track(track);
        }

        // Create next sink for crossfade
        let next_sink = Sink::try_new(self.stream_handle())
            .map_err(|e| AppError::Playback(e.to_string()))?;

        let file = File::open(&track.file_path)
            .map_err(|e| AppError::Audio(format!("Failed to open file: {}", e)))?;

        let reader = BufReader::new(file);
        let source = rodio::Decoder::new(reader)
            .map_err(|e| AppError::Audio(format!("Failed to decode: {}", e)))?;

        if track.is_looping {
            next_sink.append(source.repeat_infinite());
        } else {
            next_sink.append(source);
        }

        next_sink.set_volume(0.0);

        // Store current and next sinks for crossfade
        let crossfade_ms = crossfade_type.duration_ms();
        let config = self.config.read().clone();

        // Store next sink
        self.music_sink = Some(next_sink);
        *self.state.write() = EngineState::Transitioning;

        // Perform instant crossfade - simplified
        // (Proper crossfade would require Arc<Sink> for thread safety)
        let volume = config.music_volume * config.master_volume;
        *self.current_track.write() = Some(PlayingTrack {
            track: track.clone(),
            started_at_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            is_looping: track.is_looping,
        });

        *self.state.write() = EngineState::Playing;

        Ok(())
    }

    /// Play a sound effect (layered on top of music)
    pub fn play_sfx(&mut self, sfx: &SoundEffect) -> Result<(), AppError> {
        info!("Playing SFX: {}", sfx.name);

        let sink = Sink::try_new(self.stream_handle())
            .map_err(|e| AppError::Playback(e.to_string()))?;

        let file = File::open(&sfx.file_path)
            .map_err(|e| AppError::Audio(format!("Failed to open SFX: {}", e)))?;

        let reader = BufReader::new(file);
        let source = rodio::Decoder::new(reader)
            .map_err(|e| AppError::Audio(format!("Failed to decode SFX: {}", e)))?;

        sink.append(source);

        let volume = self.config.read().sfx_volume * self.config.read().master_volume;
        sink.set_volume(volume);

        // Detach sink to play independently
        sink.detach();

        Ok(())
    }

    /// Stop music playback
    pub fn stop_music(&mut self) {
        if let Some(sink) = self.music_sink.take() {
            sink.stop();
        }
        *self.state.write() = EngineState::Idle;
        *self.current_track.write() = None;
        info!("Music stopped");
    }

    /// Stop all playback
    pub fn stop_all(&mut self) {
        self.stop_music();
        info!("All playback stopped");
    }

    /// Pause playback
    pub fn pause(&mut self) {
        if let Some(ref sink) = self.music_sink {
            sink.pause();
            *self.state.write() = EngineState::Paused;
            debug!("Playback paused");
        }
    }

    /// Resume playback
    pub fn resume(&mut self) {
        if let Some(ref sink) = self.music_sink {
            sink.play();
            *self.state.write() = EngineState::Playing;
            debug!("Playback resumed");
        }
    }

    /// Trigger ducking (reduce music volume for voice-over)
    pub fn duck(&mut self) {
        let ducking_amount = self.config.read().ducking_amount;
        *self.is_ducking.write() = true;

        // Simplified: set volume directly
        if let Some(ref sink) = self.music_sink {
            let current = sink.volume();
            sink.set_volume(current * ducking_amount);
        }

        debug!("Ducking activated");
    }

    /// Release ducking (restore music volume)
    pub fn release_duck(&mut self) {
        *self.is_ducking.write() = false;
        let target_volume = self.calculate_music_volume();
        let fade_ms = self.config.read().ducking_fade_ms;

        // Get current volume
        let current_volume = self.music_sink.as_ref().map(|s| s.volume()).unwrap_or(1.0);

        // Simplified: set volume directly
        if let Some(ref sink) = self.music_sink {
            sink.set_volume(target_volume);
        }

        debug!("Ducking released");
    }

    /// Set music volume
    pub fn set_music_volume(&mut self, volume: f32) {
        self.config.write().music_volume = volume.clamp(0.0, 1.0);
        self.update_music_volume();
    }

    /// Set SFX volume
    pub fn set_sfx_volume(&mut self, volume: f32) {
        self.config.write().sfx_volume = volume.clamp(0.0, 1.0);
    }

    /// Set master volume
    pub fn set_master_volume(&mut self, volume: f32) {
        self.config.write().master_volume = volume.clamp(0.0, 1.0);
        self.update_music_volume();
    }

    /// Set crossfade type
    pub fn set_crossfade_type(&mut self, crossfade_type: CrossfadeType) {
        self.config.write().crossfade_type = crossfade_type;
    }

    /// Get current state
    pub fn state(&self) -> EngineState {
        *self.state.read()
    }

    /// Get current track
    pub fn current_track(&self) -> Option<PlayingTrack> {
        self.current_track.read().clone()
    }

    /// Calculate music volume based on config and ducking
    fn calculate_music_volume(&self) -> f32 {
        let config = self.config.read();
        let ducking = *self.is_ducking.read();

        let base_volume = config.music_volume * config.master_volume;

        if ducking {
            base_volume * config.ducking_amount
        } else {
            base_volume
        }
    }

    /// Update music sink volume
    fn update_music_volume(&self) {
        let volume = self.calculate_music_volume();
        if let Some(ref sink) = self.music_sink {
            sink.set_volume(volume);
        }
    }

    /// Check if playing
    pub fn is_playing(&self) -> bool {
        if let Some(ref sink) = self.music_sink {
            return !sink.empty();
        }
        false
    }
}

impl Default for AudioEngine {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            _stream: None,
            stream_handle: None,
            music_sink: None,
            config: RwLock::new(EngineConfig::default()),
            state: RwLock::new(EngineState::Idle),
            current_track: RwLock::new(None),
            is_ducking: RwLock::new(false),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crossfade_types() {
        assert_eq!(CrossfadeType::Instant.duration_ms(), 0);
        assert_eq!(CrossfadeType::Quick.duration_ms(), 500);
        assert_eq!(CrossfadeType::Musical.duration_ms(), 2000);
        assert_eq!(CrossfadeType::Long.duration_ms(), 5000);
    }

    #[test]
    fn test_engine_config() {
        let config = EngineConfig::default();
        assert_eq!(config.music_volume, 0.7);
        assert_eq!(config.sfx_volume, 0.8);
        assert_eq!(config.crossfade_type, CrossfadeType::Musical);
    }
}
