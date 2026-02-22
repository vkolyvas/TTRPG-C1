//! TTRPG Companion Library
//!
//! Core functionality for the TTRPG Companion desktop application.

pub mod audio;
pub mod commands;
pub mod dsp;
pub mod inference;
pub mod orchestrator;

use std::sync::Mutex;
use tauri::{CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu};
use tracing::{error, info};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Application state shared across Tauri commands
pub struct AppState {
    pub orchestrator: Mutex<orchestrator::SessionOrchestrator>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            orchestrator: Mutex::new(orchestrator::SessionOrchestrator::new()),
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

/// Set up system tray with menu items
fn setup_system_tray() -> SystemTray {
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let start_session = CustomMenuItem::new("start_session".to_string(), "Start Session");
    let stop_session = CustomMenuItem::new("stop_session".to_string(), "Stop Session");

    let tray_menu = SystemTrayMenu::new()
        .add_item(start_session)
        .add_item(stop_session)
        .add_native_item(tauri::SystemTrayMenuItem::Separator)
        .add_item(quit);

    SystemTray::new().with_menu(tray_menu)
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
            info!("Application setup complete");

            // Handle system tray events
            let _tray = app.system_tray().unwrap();
            let handle = app.handle();
            _tray.on_system_tray_event(move |_tray, event| {
                match event {
                    SystemTrayEvent::LeftClick { .. } => {
                        if let Some(window) = handle.get_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                }
            });

            Ok(())
        })
        .on_system_tray_event(|app, event| {
            match event {
                SystemTrayEvent::MenuItemClick { id, .. } => {
                    match id.as_str() {
                        "quit" => {
                            info!("Quit requested from system tray");
                            app.exit(0);
                        }
                        "start_session" => {
                            info!("Start session requested from system tray");
                            // Session start logic handled via commands
                        }
                        "stop_session" => {
                            info!("Stop session requested from system tray");
                            // Session stop logic handled via commands
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        })
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::session::start_session,
            commands::session::stop_session,
            commands::session::get_session_status,
            commands::session::get_available_devices,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
