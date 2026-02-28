//! Global hotkeys module
//!
//! Provides global hotkey support for session control:
//! - Next: Skip to next track/mood
//! - Shift: Switch between autonomous/collaborative mode
//! - Hold/Lock: Hold current music or lock to current mood

use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use parking_lot::RwLock;

/// Hotkey action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HotkeyAction {
    /// Play next track
    Next,
    /// Shift mode (A/B)
    Shift,
    /// Hold current music
    Hold,
    /// Lock to current mood
    Lock,
    /// Toggle recording
    ToggleRecording,
    /// Emergency stop
    Stop,
}

/// Hotkey configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    /// Modifier keys (ctrl, alt, shift, super)
    pub modifiers: Vec<String>,
    /// Key code
    pub key: String,
    /// Action to perform
    pub action: HotkeyAction,
}

impl HotkeyConfig {
    /// Create a new hotkey config
    pub fn new(key: String, action: HotkeyAction) -> Self {
        Self {
            modifiers: vec![],
            key,
            action,
        }
    }

    /// With modifiers
    pub fn with_modifiers(mut self, modifiers: Vec<String>) -> Self {
        self.modifiers = modifiers;
        self
    }
}

/// Hotkey event
#[derive(Debug, Clone)]
pub struct HotkeyEvent {
    pub action: HotkeyAction,
    pub timestamp: std::time::SystemTime,
}

/// Hotkey manager
pub struct HotkeyManager {
    /// Registered hotkeys
    hotkeys: RwLock<HashMap<HotkeyAction, HotkeyConfig>>,
    /// Event sender
    event_tx: RwLock<Option<flume::Sender<HotkeyEvent>>>,
    /// Is enabled
    enabled: RwLock<bool>,
}

impl HotkeyManager {
    /// Create a new hotkey manager
    pub fn new() -> Self {
        Self {
            hotkeys: RwLock::new(HashMap::new()),
            event_tx: RwLock::new(None),
            enabled: RwLock::new(true),
        }
    }

    /// Register a hotkey
    pub fn register(&self, config: HotkeyConfig) -> Result<(), AppError> {
        tracing::info!("Registering hotkey: {:?} + {:?}", config.modifiers, config.key);

        let mut hotkeys = self.hotkeys.write();
        hotkeys.insert(config.action, config);

        Ok(())
    }

    /// Unregister a hotkey
    pub fn unregister(&self, action: HotkeyAction) {
        tracing::info!("Unregistering hotkey: {:?}", action);
        let mut hotkeys = self.hotkeys.write();
        hotkeys.remove(&action);
    }

    /// Set event sender
    pub fn set_event_sender(&self, tx: flume::Sender<HotkeyEvent>) {
        *self.event_tx.write() = Some(tx);
    }

    /// Enable hotkeys
    pub fn enable(&self) {
        *self.enabled.write() = true;
        tracing::info!("Hotkeys enabled");
    }

    /// Disable hotkeys
    pub fn disable(&self) {
        *self.enabled.write() = false;
        tracing::info!("Hotkeys disabled");
    }

    /// Check if enabled
    pub fn is_enabled(&self) -> bool {
        *self.enabled.read()
    }

    /// Get hotkey config
    pub fn get_hotkey(&self, action: HotkeyAction) -> Option<HotkeyConfig> {
        self.hotkeys.read().get(&action).cloned()
    }

    /// Get all hotkeys
    pub fn get_all_hotkeys(&self) -> Vec<HotkeyConfig> {
        self.hotkeys.read().values().cloned().collect()
    }

    /// Handle hotkey event
    pub fn handle_event(&self, action: HotkeyAction) {
        if !self.is_enabled() {
            return;
        }

        tracing::debug!("Hotkey triggered: {:?}", action);

        if let Some(tx) = self.event_tx.read().as_ref() {
            let event = HotkeyEvent {
                action,
                timestamp: std::time::SystemTime::now(),
            };
            let _ = tx.send(event);
        }
    }
}

impl Default for HotkeyManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Default hotkey bindings
pub fn default_hotkeys() -> Vec<HotkeyConfig> {
    vec![
        HotkeyConfig::new("n".to_string(), HotkeyAction::Next)
            .with_modifiers(vec!["ctrl".to_string()]),
        HotkeyConfig::new("m".to_string(), HotkeyAction::Shift)
            .with_modifiers(vec!["ctrl".to_string()]),
        HotkeyConfig::new("h".to_string(), HotkeyAction::Hold)
            .with_modifiers(vec!["ctrl".to_string()]),
        HotkeyConfig::new("l".to_string(), HotkeyAction::Lock)
            .with_modifiers(vec!["ctrl".to_string()]),
        HotkeyConfig::new("r".to_string(), HotkeyAction::ToggleRecording)
            .with_modifiers(vec!["ctrl".to_string()]),
        HotkeyConfig::new("escape".to_string(), HotkeyAction::Stop)
            .with_modifiers(vec!["ctrl".to_string()]),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hotkey_manager() {
        let manager = HotkeyManager::new();

        let config = HotkeyConfig::new("n".to_string(), HotkeyAction::Next);
        manager.register(config.clone()).unwrap();

        assert!(manager.get_hotkey(HotkeyAction::Next).is_some());
        assert!(manager.get_hotkey(HotkeyAction::Shift).is_none());

        manager.unregister(HotkeyAction::Next);
        assert!(manager.get_hotkey(HotkeyAction::Next).is_none());
    }
}
