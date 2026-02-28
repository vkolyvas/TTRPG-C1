//! Database repository - CRUD operations

use crate::db::models::*;
use crate::db::DbPool;
use crate::error::AppError;
use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use std::sync::Arc;

/// Database repository
pub struct Repository {
    pool: DbPool,
}

impl Repository {
    /// Create a new repository
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Get a connection from the pool
    fn get_conn(&self) -> Result<PooledConnection<SqliteConnectionManager>, AppError> {
        self.pool
            .get()
            .map_err(|e| AppError::Database(e.to_string()))
    }

    // ========== Tracks ==========

    /// Get all tracks
    pub fn get_all_tracks(&self) -> Result<Vec<Track>, AppError> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, file_path, duration_ms, genre, mood, is_looping, volume, created_at, updated_at FROM tracks ORDER BY name"
        )?;

        let tracks = stmt
            .query_map([], |row| {
                Ok(Track {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    file_path: row.get(2)?,
                    duration_ms: row.get(3)?,
                    genre: row.get(4)?,
                    mood: row.get(5)?,
                    is_looping: row.get::<_, i32>(6)? != 0,
                    volume: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(tracks)
    }

    /// Get tracks by genre
    pub fn get_tracks_by_genre(&self, genre: &str) -> Result<Vec<Track>, AppError> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, file_path, duration_ms, genre, mood, is_looping, volume, created_at, updated_at FROM tracks WHERE genre = ?1 ORDER BY name"
        )?;

        let tracks = stmt
            .query_map([genre], |row| {
                Ok(Track {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    file_path: row.get(2)?,
                    duration_ms: row.get(3)?,
                    genre: row.get(4)?,
                    mood: row.get(5)?,
                    is_looping: row.get::<_, i32>(6)? != 0,
                    volume: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(tracks)
    }

    /// Insert a track
    pub fn insert_track(&self, track: &Track) -> Result<(), AppError> {
        let conn = self.get_conn()?;
        conn.execute(
            "INSERT INTO tracks (id, name, file_path, duration_ms, genre, mood, is_looping, volume, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            [
                &track.id,
                &track.name,
                &track.file_path,
                &track.duration_ms.map(|v| v.to_string()).unwrap_or_default(),
                &track.genre.clone().unwrap_or_default(),
                &track.mood.clone().unwrap_or_default(),
                &(if track.is_looping { 1 } else { 0 }).to_string(),
                &track.volume.to_string(),
                &track.created_at,
                &track.updated_at,
            ],
        )?;
        Ok(())
    }

    // ========== Sessions ==========

    /// Start a new session
    pub fn start_session(&self, session: &Session) -> Result<(), AppError> {
        let conn = self.get_conn()?;
        conn.execute(
            "INSERT INTO sessions (id, started_at, mode, created_at) VALUES (?1, ?2, ?3, ?4)",
            [&session.id, &session.started_at, &session.mode, &session.created_at],
        )?;
        Ok(())
    }

    /// End a session
    pub fn end_session(&self, session_id: &str) -> Result<(), AppError> {
        let conn = self.get_conn()?;
        let ended_at = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE sessions SET ended_at = ?1 WHERE id = ?2",
            [&ended_at, session_id],
        )?;
        Ok(())
    }

    /// Get session by ID
    pub fn get_session(&self, session_id: &str) -> Result<Option<Session>, AppError> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, started_at, ended_at, mode, total_duration_ms, created_at, detected_events_count, keywords_triggered, emotions_detected, tracks_played FROM sessions WHERE id = ?1"
        )?;

        let session = stmt
            .query_row([session_id], |row| {
                Ok(Session {
                    id: row.get(0)?,
                    started_at: row.get(1)?,
                    ended_at: row.get(2)?,
                    mode: row.get(3)?,
                    total_duration_ms: row.get(4)?,
                    created_at: row.get(5)?,
                    detected_events_count: row.get(6)?,
                    keywords_triggered: row.get(7)?,
                    emotions_detected: row.get(8)?,
                    tracks_played: row.get(9)?,
                })
            })
            .ok();

        Ok(session)
    }

    // ========== Detection Events ==========

    /// Insert detection event
    pub fn insert_detection_event(&self, event: &DetectionEvent) -> Result<(), AppError> {
        let conn = self.get_conn()?;
        conn.execute(
            "INSERT INTO detection_events (id, session_id, event_type, timestamp, details, confidence, category, triggered_action) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            [
                &event.id,
                &event.session_id,
                &event.event_type,
                &event.timestamp,
                &event.details.clone().unwrap_or_default(),
                &event.confidence.map(|v| v.to_string()).unwrap_or_default(),
                &event.category.clone().unwrap_or_default(),
                &(if event.triggered_action { 1 } else { 0 }).to_string(),
            ],
        )?;
        Ok(())
    }

    /// Get detection events for session
    pub fn get_session_events(&self, session_id: &str) -> Result<Vec<DetectionEvent>, AppError> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, session_id, event_type, timestamp, details, confidence, category, triggered_action FROM detection_events WHERE session_id = ?1 ORDER BY timestamp"
        )?;

        let events = stmt
            .query_map([session_id], |row| {
                Ok(DetectionEvent {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    event_type: row.get(2)?,
                    timestamp: row.get(3)?,
                    details: row.get(4)?,
                    confidence: row.get(5)?,
                    category: row.get(6)?,
                    triggered_action: row.get::<_, i32>(7)? != 0,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(events)
    }

    // ========== Keywords ==========

    /// Get all active keywords
    pub fn get_active_keywords(&self) -> Result<Vec<Keyword>, AppError> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, word, category, variations, mood, priority, is_active, created_at FROM keywords WHERE is_active = 1 ORDER BY priority DESC"
        )?;

        let keywords = stmt
            .query_map([], |row| {
                Ok(Keyword {
                    id: row.get(0)?,
                    word: row.get(1)?,
                    category: row.get(2)?,
                    variations: row.get(3)?,
                    mood: row.get(4)?,
                    priority: row.get(5)?,
                    is_active: row.get::<_, i32>(6)? != 0,
                    created_at: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(keywords)
    }

    /// Insert keyword
    pub fn insert_keyword(&self, keyword: &Keyword) -> Result<(), AppError> {
        let conn = self.get_conn()?;
        conn.execute(
            "INSERT INTO keywords (id, word, category, variations, mood, priority, is_active, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            [
                &keyword.id,
                &keyword.word,
                &keyword.category,
                &keyword.variations.clone().unwrap_or_default(),
                &keyword.mood.clone().unwrap_or_default(),
                &keyword.priority.to_string(),
                &(if keyword.is_active { 1 } else { 0 }).to_string(),
                &keyword.created_at,
            ],
        )?;
        Ok(())
    }

    // ========== Settings ==========

    /// Get a setting
    pub fn get_setting(&self, key: &str) -> Result<Option<String>, AppError> {
        let conn = self.get_conn()?;
        let value = conn
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                [key],
                |row| row.get(0),
            )
            .ok();
        Ok(value)
    }

    /// Set a setting
    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), AppError> {
        let conn = self.get_conn()?;
        let updated_at = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)",
            [key, value, &updated_at],
        )?;
        Ok(())
    }
}
