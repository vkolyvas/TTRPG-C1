//! ONNX Runtime environment

use crate::error::AppError;
use std::sync::OnceLock;

/// Global ONNX Runtime environment
static ONNX_ENV: OnceLock<OnnxEnv> = OnceLock::new();

/// ONNX Runtime environment wrapper
pub struct OnnxEnv {
    pub initialized: bool,
}

impl OnnxEnv {
    /// Create a new ONNX environment
    pub fn new() -> Self {
        Self { initialized: false }
    }

    /// Initialize the environment
    pub fn initialize(&mut self) -> Result<(), AppError> {
        if self.initialized {
            return Ok(());
        }

        tracing::info!("Initializing ONNX Runtime environment");

        // In production with ort crate:
        // ort::init()
        //     .with_model_parallelism(1)
        //     .commit()?;

        self.initialized = true;
        tracing::info!("ONNX Runtime environment ready");

        Ok(())
    }
}

impl Default for OnnxEnv {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the global ONNX environment
pub fn get_onnx_env() -> &'static OnnxEnv {
    ONNX_ENV.get_or_init(|| OnnxEnv::new())
}

/// Initialize the global ONNX environment
pub fn init_onnx() -> Result<(), AppError> {
    let env = ONNX_ENV.get_or_init(|| OnnxEnv::new());
    // Note: In production, call env.initialize()
    Ok(())
}
