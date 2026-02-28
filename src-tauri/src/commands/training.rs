//! Voice training commands

use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::info;

/// Training passage
#[derive(Debug, Serialize, Deserialize)]
pub struct TrainingPassage {
    pub text: String,
    pub emotion: String,
    pub duration_secs: u32,
}

/// Voice profile
#[derive(Debug, Serialize, Deserialize)]
pub struct VoiceProfile {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    pub consent_given: bool,
    pub created_at: String,
}

/// Training status
#[derive(Debug, Serialize, Deserialize)]
pub struct TrainingStatus {
    pub is_enrolled: bool,
    pub profile: Option<VoiceProfile>,
    pub passages_completed: u32,
    pub total_passages: u32,
}

/// Get training passages
#[tauri::command]
pub fn get_training_passages() -> Vec<TrainingPassage> {
    vec![
        TrainingPassage {
            text: "Welcome, adventurers, to the beginning of your journey. The world before you is vast and full of mystery.".to_string(),
            emotion: "neutral".to_string(),
            duration_secs: 10,
        },
        TrainingPassage {
            text: "Huzzah! Victory is yours! The treasure is yours to claim!".to_string(),
            emotion: "happy".to_string(),
            duration_secs: 8,
        },
        TrainingPassage {
            text: "Alas, your companion has fallen. The weight of loss settles upon you all.".to_string(),
            emotion: "sad".to_string(),
            duration_secs: 10,
        },
        TrainingPassage {
            text: "You dare challenge me? I shall crush you like the insect you are!".to_string(),
            emotion: "angry".to_string(),
            duration_secs: 8,
        },
        TrainingPassage {
            text: "Did you hear that? Something moves in the darkness. We are not alone...".to_string(),
            emotion: "fearful".to_string(),
            duration_secs: 10,
        },
        TrainingPassage {
            text: "Behold! The ancient dragon awakens from its slumber!".to_string(),
            emotion: "surprised".to_string(),
            duration_secs: 8,
        },
        TrainingPassage {
            text: "The smell of decay fills your nostrils. Something terrible has happened here.".to_string(),
            emotion: "disgusted".to_string(),
            duration_secs: 9,
        },
    ]
}

/// Get training status
#[tauri::command]
pub fn get_training_status(state: State<'_, AppState>) -> Result<TrainingStatus, String> {
    // For now, return mock status
    Ok(TrainingStatus {
        is_enrolled: false,
        profile: None,
        passages_completed: 0,
        total_passages: 7,
    })
}

/// Save voice profile
#[tauri::command]
pub fn save_voice_profile(
    state: State<'_, AppState>,
    name: String,
    consent_given: bool,
) -> Result<VoiceProfile, String> {
    info!("Saving voice profile: {}", name);

    // Create profile
    let profile = VoiceProfile {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        is_default: true,
        consent_given,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    Ok(profile)
}

/// Delete voice profile
#[tauri::command]
pub fn delete_voice_profile(
    state: State<'_, AppState>,
    profile_id: String,
) -> Result<(), String> {
    info!("Deleting voice profile: {}", profile_id);
    Ok(())
}
