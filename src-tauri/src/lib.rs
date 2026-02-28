//! TTRPG Companion Library
//!
//! Core functionality for the TTRPG Companion desktop application.

pub mod audio;
pub mod commands;
pub mod db;
pub mod detection;
pub mod dsp;
pub mod error;
pub mod hotkeys;
pub mod inference;
pub mod ml;
pub mod orchestrator;
pub mod profile;
pub mod startup;
pub mod state;

use db::Database;
use error::AppError;
use state::{AppMode, SessionConfig, SessionState};
use std::sync::Arc;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};
use tracing::{error, info, warn};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Application state shared across Tauri commands
pub struct AppState {
    /// Current session state
    pub session_state: parking_lot::RwLock<SessionState>,
    /// Application mode (A or B)
    pub app_mode: parking_lot::RwLock<AppMode>,
    /// Session configuration
    pub config: parking_lot::RwLock<SessionConfig>,
    /// Audio buffer for processing (thread-safe)
    pub audio_buffer: Arc<parking_lot::RwLock<Vec<f32>>>,
    /// Current sample rate
    pub sample_rate: parking_lot::RwLock<u32>,
    /// Database connection pool
    pub db_pool: parking_lot::RwLock<Option<db::DbPool>>,
    /// Current detected emotion
    pub current_emotion: parking_lot::RwLock<String>,
    /// Keyword vocabulary version
    pub keyword_version: parking_lot::RwLock<u64>,
    /// Is detection pipeline ready
    pub detection_ready: parking_lot::RwLock<bool>,
    /// Startup complete flag
    pub startup_complete: parking_lot::RwLock<bool>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            session_state: parking_lot::RwLock::new(SessionState::Idle),
            app_mode: parking_lot::RwLock::new(AppMode::default()),
            config: parking_lot::RwLock::new(SessionConfig::default()),
            audio_buffer: Arc::new(parking_lot::RwLock::new(Vec::new())),
            sample_rate: parking_lot::RwLock::new(16000),
            db_pool: parking_lot::RwLock::new(None),
            current_emotion: parking_lot::RwLock::new("neutral".to_string()),
            keyword_version: parking_lot::RwLock::new(0),
            detection_ready: parking_lot::RwLock::new(false),
            startup_complete: parking_lot::RwLock::new(false),
        }
    }
}

/// Initialize logging system with file output
fn init_logging() {
    let log_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("ttrpg_companion")
        .join("logs");

    std::fs::create_dir_all(&log_dir).ok();

    let file_appender = RollingFileAppender::new(Rotation::DAILY, log_dir, "ttrpg_companion.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // Keep the guard alive for the lifetime of the application
    std::mem::forget(_guard);

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_writer(non_blocking))
        .with(fmt::layer().with_writer(std::io::stdout))
        .init();
}

/// Initialize database
fn init_database(app: &tauri::App) -> Result<db::DbPool, AppError> {
    let app_dir = app.path().app_data_dir().map_err(|e| AppError::Config(e.to_string()))?;
    let db_path = app_dir.join("ttrpg_companion.db");

    info!("Initializing database at: {:?}", db_path);

    let db = Database::new(db_path.to_str().unwrap_or("ttrpg_companion.db"))?;
    Ok(db.pool().clone())
}

/// Main entry point for the Tauri application
pub fn run() {
    // Initialize logging first
    init_logging();
    info!("Starting TTRPG Companion v{}", env!("CARGO_PKG_VERSION"));

    // Set up panic hook for logging
    std::panic::set_hook(Box::new(|panic_info| {
        error!("Application panic: {:?}", panic_info);
    }));

    tauri::Builder::default()
        .setup(|app| {
            info!("Application setup starting");

            // Initialize database
            match init_database(app) {
                Ok(pool) => {
                    info!("Database initialized successfully");
                    app.state::<AppState>().db_pool.write().replace(pool);
                }
                Err(e) => {
                    warn!("Database initialization failed: {}", e);
                }
            }

            // Create system tray menu with mood indicator
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let start_session = MenuItem::with_id(app, "start_session", "Start Session", true, None::<&str>)?;
            let stop_session = MenuItem::with_id(app, "stop_session", "Stop Session", true, None::<&str>)?;
            let separator = MenuItem::with_id(app, "separator", "─────────", false, None::<&str>)?;
            let toggle_mode = MenuItem::with_id(app, "toggle_mode", "Toggle Mode (A/B)", true, None::<&str>)?;

            let menu = Menu::with_items(app, &[
                &start_session,
                &stop_session,
                &separator,
                &toggle_mode,
                &quit,
            ])?;

            // Build system tray
            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("TTRPG Companion - Ready")
                .on_menu_event(|app, event| {
                    let state = app.state::<AppState>();

                    match event.id.as_ref() {
                        "quit" => {
                            info!("Quit requested from system tray");
                            app.exit(0);
                        }
                        "start_session" => {
                            info!("Start session requested from system tray");
                            // Trigger start session
                            *state.session_state.write() = SessionState::Recording;
                        }
                        "stop_session" => {
                            info!("Stop session requested from system tray");
                            *state.session_state.write() = SessionState::Idle;
                        }
                        "toggle_mode" => {
                            let current_mode = *state.app_mode.read();
                            let new_mode = match current_mode {
                                AppMode::ModeA => AppMode::ModeB,
                                AppMode::ModeB => AppMode::ModeA,
                            };
                            *state.app_mode.write() = new_mode;
                            info!("Mode toggled to: {:?}", new_mode);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            // Mark startup as complete
            *app.state::<AppState>().startup_complete.write() = true;

            info!("Application setup complete");
            Ok(())
        })
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::session::start_session,
            commands::session::stop_session,
            commands::session::get_session_status,
            commands::session::get_available_devices,
            commands::session::get_tracks,
            commands::session::set_app_mode,
            commands::session::get_app_mode,
            commands::session::set_detection_enabled,
            commands::training::get_training_passages,
            commands::training::get_training_status,
            commands::training::save_voice_profile,
            commands::training::delete_voice_profile,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
