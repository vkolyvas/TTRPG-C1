//! Microphone input capture using cpal

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat, Stream, StreamConfig};
use thiserror::Error;
use tracing::{debug, error, info};

#[derive(Error, Debug)]
pub enum CaptureError {
    #[error("No input device available")]
    NoInputDevice,
    #[error("Failed to get device config: {0}")]
    ConfigError(String),
    #[error("Failed to build stream: {0}")]
    StreamBuildError(String),
    #[error("Stream play error: {0}")]
    StreamPlayError(String),
}

/// Audio capture state
pub struct AudioCapture {
    stream: Option<Stream>,
    is_recording: bool,
    sample_rate: u32,
    channels: u16,
}

impl AudioCapture {
    /// Create a new AudioCapture instance
    pub fn new() -> Self {
        Self {
            stream: None,
            is_recording: false,
            sample_rate: 16000,
            channels: 1,
        }
    }

    /// Get the default input device
    fn get_default_input_device() -> Result<Device, CaptureError> {
        let host = cpal::default_host();
        host.default_input_device()
            .ok_or(CaptureError::NoInputDevice)
    }

    /// List all available input devices
    pub fn list_devices() -> Result<Vec<String>, CaptureError> {
        let host = cpal::default_host();
        let mut devices = Vec::new();

        for device in host.input_devices().map_err(|e| CaptureError::ConfigError(e.to_string()))? {
            if let Ok(name) = device.name() {
                devices.push(name);
            }
        }

        Ok(devices)
    }

    /// Start recording audio
    pub fn start_recording<F>(&mut self, mut callback: F) -> Result<(), CaptureError>
    where
        F: FnMut(Vec<f32>) + Send + 'static,
    {
        let device = Self::get_default_input_device()?;
        info!("Using input device: {:?}", device.name());

        let config = device
            .default_input_config()
            .map_err(|e| CaptureError::ConfigError(e.to_string()))?;

        debug!("Input config: {:?}", config);

        let sample_rate = config.sample_rate().0;
        let channels = config.channels();

        let err_fn = |err| error!("Audio stream error: {}", err);

        let stream = match config.sample_format() {
            SampleFormat::F32 => device.build_input_stream(
                &config.into(),
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    callback(data.to_vec());
                },
                err_fn,
                None,
            ),
            SampleFormat::I16 => device.build_input_stream(
                &config.into(),
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    let float_data: Vec<f32> =
                        data.iter().map(|&s| s as f32 / i16::MAX as f32).collect();
                    callback(float_data);
                },
                err_fn,
                None,
            ),
            SampleFormat::U16 => device.build_input_stream(
                &config.into(),
                move |data: &[u16], _: &cpal::InputCallbackInfo| {
                    let float_data: Vec<f32> = data
                        .iter()
                        .map(|&s| (s as f32 / u16::MAX as f32) - 0.5)
                        .collect();
                    callback(float_data);
                },
                err_fn,
                None,
            ),
            _ => {
                return Err(CaptureError::StreamBuildError(
                    "Unsupported sample format".to_string(),
                ))
            }
        }
        .map_err(|e| CaptureError::StreamBuildError(e.to_string()))?;

        stream
            .play()
            .map_err(|e| CaptureError::StreamPlayError(e.to_string()))?;

        self.stream = Some(stream);
        self.is_recording = true;
        self.sample_rate = sample_rate;
        self.channels = channels;

        info!(
            "Recording started: {} Hz, {} channels",
            sample_rate, channels
        );

        Ok(())
    }

    /// Stop recording audio
    pub fn stop_recording(&mut self) -> Result<(), CaptureError> {
        if let Some(stream) = self.stream.take() {
            drop(stream);
        }
        self.is_recording = false;
        info!("Recording stopped");
        Ok(())
    }

    /// Check if currently recording
    pub fn is_recording(&self) -> bool {
        self.is_recording
    }

    /// Get current sample rate
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Get number of channels
    pub fn channels(&self) -> u16 {
        self.channels
    }
}

impl Default for AudioCapture {
    fn default() -> Self {
        Self::new()
    }
}
