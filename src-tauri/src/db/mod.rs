//! Database module
//!
//! Provides SQLite database with connection pooling via r2d2.

pub mod migrations;
pub mod models;
pub mod repository;

pub use migrations::*;
pub use models::*;
pub use repository::*;

use crate::error::AppError;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::path::Path;

/// Database pool type alias
pub type DbPool = Pool<SqliteConnectionManager>;

/// Database manager
pub struct Database {
    pool: DbPool,
    db_path: String,
}

impl Database {
    /// Create a new database connection
    pub fn new(db_path: &str) -> Result<Self, AppError> {
        tracing::info!("Initializing database at: {}", db_path);

        // Ensure directory exists
        if let Some(parent) = Path::new(db_path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let manager = SqliteConnectionManager::file(db_path);
        let pool = Pool::builder()
            .max_size(10)
            .build(manager)
            .map_err(|e| AppError::Database(e.to_string()))?;

        let db = Self {
            pool,
            db_path: db_path.to_string(),
        };

        // Run migrations
        db.run_migrations()?;

        tracing::info!("Database initialized successfully");
        Ok(db)
    }

    /// Get the connection pool
    pub fn pool(&self) -> &DbPool {
        &self.pool
    }

    /// Get the database path
    pub fn path(&self) -> &str {
        &self.db_path
    }

    /// Run database migrations
    pub fn run_migrations(&self) -> Result<(), AppError> {
        let conn = self.pool.get()?;

        // Create migrations table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL
            )",
            [],
        )?;

        // Get current version
        let current_version: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        // Apply migrations
        for migration in get_migrations() {
            if migration.version > current_version {
                tracing::info!("Applying migration v{}", migration.version);

                conn.execute_batch(&migration.sql)?;

                conn.execute(
                    "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
                    [
                        migration.version.to_string(),
                        chrono::Utc::now().to_rfc3339(),
                    ],
                )?;
            }
        }

        Ok(())
    }
}

/// Migration definition
pub struct Migration {
    pub version: i64,
    pub name: &'static str,
    pub sql: &'static str,
}

/// Get all migrations
pub fn get_migrations() -> Vec<Migration> {
    vec![
        // Migration 1: Initial schema
        Migration {
            version: 1,
            name: "initial_schema",
            sql: r#"
                -- Tracks table
                CREATE TABLE IF NOT EXISTS tracks (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    file_path TEXT NOT NULL,
                    duration_ms INTEGER,
                    genre TEXT,
                    mood TEXT,
                    is_looping INTEGER DEFAULT 0,
                    volume REAL DEFAULT 1.0,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                );

                -- Track genres table
                CREATE TABLE IF NOT EXISTS track_genres (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL UNIQUE,
                    color TEXT,
                    created_at TEXT NOT NULL
                );

                -- SFX table
                CREATE TABLE IF NOT EXISTS sfx (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    file_path TEXT NOT NULL,
                    duration_ms INTEGER,
                    category TEXT,
                    volume REAL DEFAULT 1.0,
                    created_at TEXT NOT NULL
                );

                -- Sessions table
                CREATE TABLE IF NOT EXISTS sessions (
                    id TEXT PRIMARY KEY,
                    started_at TEXT NOT NULL,
                    ended_at TEXT,
                    mode TEXT NOT NULL,
                    total_duration_ms INTEGER,
                    created_at TEXT NOT NULL
                );

                -- Keywords table
                CREATE TABLE IF NOT EXISTS keywords (
                    id TEXT PRIMARY KEY,
                    word TEXT NOT NULL,
                    category TEXT NOT NULL,
                    variations TEXT,
                    mood TEXT,
                    priority INTEGER DEFAULT 0,
                    is_active INTEGER DEFAULT 1,
                    created_at TEXT NOT NULL
                );

                -- Detection events table
                CREATE TABLE IF NOT EXISTS detection_events (
                    id TEXT PRIMARY KEY,
                    session_id TEXT NOT NULL,
                    event_type TEXT NOT NULL,
                    timestamp TEXT NOT NULL,
                    details TEXT,
                    confidence REAL,
                    category TEXT,
                    triggered_action INTEGER DEFAULT 0,
                    FOREIGN KEY (session_id) REFERENCES sessions(id)
                );

                -- Voice profiles table
                CREATE TABLE IF NOT EXISTS voice_profiles (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    embedding BLOB,
                    is_default INTEGER DEFAULT 0,
                    consent_given INTEGER DEFAULT 0,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                );

                -- Settings table
                CREATE TABLE IF NOT EXISTS settings (
                    key TEXT PRIMARY KEY,
                    value TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                );

                -- Create indexes
                CREATE INDEX IF NOT EXISTS idx_tracks_genre ON tracks(genre);
                CREATE INDEX IF NOT EXISTS idx_tracks_mood ON tracks(mood);
                CREATE INDEX IF NOT EXISTS idx_detection_events_session ON detection_events(session_id);
                CREATE INDEX IF NOT EXISTS idx_detection_events_type ON detection_events(event_type);
            "#,
        },
        // Migration 2: Add more session details
        Migration {
            version: 2,
            name: "session_details",
            sql: r#"
                -- Add columns to sessions
                ALTER TABLE sessions ADD COLUMN detected_events_count INTEGER DEFAULT 0;
                ALTER TABLE sessions ADD COLUMN keywords_triggered INTEGER DEFAULT 0;
                ALTER TABLE sessions ADD COLUMN emotions_detected TEXT;
                ALTER TABLE sessions ADD COLUMN tracks_played TEXT;
            "#,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrations_defined() {
        let migrations = get_migrations();
        assert!(!migrations.is_empty());
        assert_eq!(migrations[0].version, 1);
    }
}
