//! Audio playback using rodio

use rodio::{OutputStream, OutputStreamHandle, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use thiserror::Error;
use tracing::{debug, error, info};

#[derive(Error, Debug)]
pub enum PlaybackError {
    #[error("No output device available")]
    NoOutputDevice,
    #[error("Failed to open file: {0}")]
    FileOpenError(String),
    #[error("Failed to decode audio: {0}")]
    DecodeError(String),
    #[error("Playback error: {0}")]
    PlaybackError(String),
}

/// Audio playback state
pub struct AudioPlayback {
    _stream: Option<OutputStream>,
    _stream_handle: Option<OutputStreamHandle>,
    sink: Option<Sink>,
    is_playing: bool,
}

impl AudioPlayback {
    /// Create a new AudioPlayback instance
    pub fn new() -> Result<Self, PlaybackError> {
        let (stream, stream_handle) =
            rodio::OutputStream::try_default().map_err(|_| PlaybackError::NoOutputDevice)?;

        Ok(Self {
            _stream: Some(stream),
            _stream_handle: Some(stream_handle),
            sink: None,
            is_playing: false,
        })
    }

    /// List all available output devices
    pub fn list_devices() -> Result<Vec<String>, PlaybackError> {
        // rodio doesn't expose device listing directly
        // Return default device name
        Ok(vec!["Default Output".to_string()])
    }

    /// Play audio from a file
    pub fn play_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), PlaybackError> {
        let file = File::open(path.as_ref())
            .map_err(|e| PlaybackError::FileOpenError(e.to_string()))?;

        let reader = BufReader::new(file);

        let source = rodio::Decoder::new(reader)
            .map_err(|e| PlaybackError::DecodeError(e.to_string()))?;

        let sink = Sink::try_new(self._stream_handle.as_ref().unwrap())
            .map_err(|e| PlaybackError::PlaybackError(e.to_string()))?;

        sink.append(source);
        self.sink = Some(sink);
        self.is_playing = true;

        info!("Playing audio: {:?}", path.as_ref());

        Ok(())
    }

    /// Play raw audio samples
    pub fn play_samples(&mut self, samples: &[f32], sample_rate: u32) -> Result<(), PlaybackError> {
        let source = rodio::buffer::SamplesBuffer::new(1, sample_rate, samples);
        let sink = Sink::try_new(self._stream_handle.as_ref().unwrap())
            .map_err(|e| PlaybackError::PlaybackError(e.to_string()))?;

        sink.append(source);
        self.sink = Some(sink);
        self.is_playing = true;

        debug!("Playing {} samples at {} Hz", samples.len(), sample_rate);

        Ok(())
    }

    /// Stop playback
    pub fn stop(&mut self) {
        if let Some(sink) = self.sink.take() {
            sink.stop();
        }
        self.is_playing = false;
        info!("Playback stopped");
    }

    /// Pause playback
    pub fn pause(&mut self) {
        if let Some(ref sink) = self.sink {
            sink.pause();
            self.is_playing = false;
            debug!("Playback paused");
        }
    }

    /// Resume playback
    pub fn resume(&mut self) {
        if let Some(ref sink) = self.sink {
            sink.play();
            self.is_playing = true;
            debug!("Playback resumed");
        }
    }

    /// Check if currently playing
    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    /// Set volume (0.0 to 1.0)
    pub fn set_volume(&self, volume: f32) {
        if let Some(ref sink) = self.sink {
            sink.set_volume(volume.clamp(0.0, 1.0));
        }
    }
}

impl Default for AudioPlayback {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            _stream: None,
            _stream_handle: None,
            sink: None,
            is_playing: false,
        })
    }
}
