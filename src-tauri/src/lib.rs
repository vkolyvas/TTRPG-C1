//! TTRPG Companion Library
//!
//! Core functionality for the TTRPG Companion desktop application.

pub mod audio;
pub mod commands;
pub mod dsp;
pub mod inference;
pub mod orchestrator;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};
use tracing::{error, info};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Application state shared across Tauri commands
pub struct AppState {
    // Placeholder for now - can be extended later
    pub initialized: std::sync::Mutex<bool>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            initialized: std::sync::Mutex::new(false),
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

            // Create system tray menu
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let start_session = MenuItem::with_id(app, "start_session", "Start Session", true, None::<&str>)?;
            let stop_session = MenuItem::with_id(app, "stop_session", "Stop Session", true, None::<&str>)?;

            let menu = Menu::with_items(app, &[&start_session, &stop_session, &quit])?;

            // Build system tray
            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("TTRPG Companion")
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "quit" => {
                            info!("Quit requested from system tray");
                            app.exit(0);
                        }
                        "start_session" => {
                            info!("Start session requested from system tray");
                        }
                        "stop_session" => {
                            info!("Stop session requested from system tray");
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

            Ok(())
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
