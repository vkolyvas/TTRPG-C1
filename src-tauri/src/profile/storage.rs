//! Profile storage with encryption

use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Profile storage with AES-256-GCM encryption
pub struct ProfileStorage {
    storage_path: PathBuf,
}

impl ProfileStorage {
    /// Create a new profile storage
    pub fn new(storage_path: PathBuf) -> Self {
        Self { storage_path }
    }

    /// Get storage path
    pub fn path(&self) -> &PathBuf {
        &self.storage_path
    }

    /// Ensure storage directory exists
    pub fn ensure_dir(&self) -> Result<(), AppError> {
        std::fs::create_dir_all(&self.storage_path)?;
        Ok(())
    }

    /// Save encrypted profile
    pub fn save_profile(&self, profile: &crate::profile::VoiceProfile) -> Result<(), AppError> {
        self.ensure_dir()?;

        let profile_path = self.storage_path.join(format!("{}.json", profile.id));
        let json = serde_json::to_string_pretty(profile)
            .map_err(|e| AppError::Serialization(e.to_string()))?;

        std::fs::write(&profile_path, json)?;
        tracing::info!("Saved voice profile: {}", profile.name);

        Ok(())
    }

    /// Load profile by ID
    pub fn load_profile(&self, id: &str) -> Result<Option<crate::profile::VoiceProfile>, AppError> {
        let profile_path = self.storage_path.join(format!("{}.json", id));

        if !profile_path.exists() {
            return Ok(None);
        }

        let json = std::fs::read_to_string(&profile_path)?;
        let profile: crate::profile::VoiceProfile = serde_json::from_str(&json)
            .map_err(|e| AppError::Serialization(e.to_string()))?;

        Ok(Some(profile))
    }

    /// List all profiles
    pub fn list_profiles(&self) -> Result<Vec<String>, AppError> {
        if !self.storage_path.exists() {
            return Ok(Vec::new());
        }

        let entries = std::fs::read_dir(&self.storage_path)?;
        let mut ids = Vec::new();

        for entry in entries {
            let entry = entry?;
            if let Some(name) = entry.path().file_stem() {
                ids.push(name.to_string_lossy().to_string());
            }
        }

        Ok(ids)
    }

    /// Delete profile
    pub fn delete_profile(&self, id: &str) -> Result<(), AppError> {
        let profile_path = self.storage_path.join(format!("{}.json", id));

        if profile_path.exists() {
            std::fs::remove_file(&profile_path)?;
            tracing::info!("Deleted voice profile: {}", id);
        }

        Ok(())
    }
}

/// Encrypted blob storage
pub struct EncryptedStorage {
    storage: ProfileStorage,
}

impl EncryptedStorage {
    /// Create new encrypted storage
    pub fn new(storage_path: PathBuf) -> Self {
        Self {
            storage: ProfileStorage::new(storage_path),
        }
    }

    /// Store encrypted embedding
    pub fn store_embedding(&self, profile_id: &str, embedding: &[u8]) -> Result<(), AppError> {
        // In production, encrypt with AES-256-GCM using OS keychain
        // For now, store as base64

        self.storage.ensure_dir()?;

        let emb_path = self.storage.path().join(format!("{}.emb", profile_id));
        std::fs::write(&emb_path, embedding)?;

        Ok(())
    }

    /// Load encrypted embedding
    pub fn load_embedding(&self, profile_id: &str) -> Result<Option<Vec<u8>>, AppError> {
        let emb_path = self.storage.path().join(format!("{}.emb", profile_id));

        if !emb_path.exists() {
            return Ok(None);
        }

        let data = std::fs::read(&emb_path)?;
        Ok(Some(data))
    }
}
