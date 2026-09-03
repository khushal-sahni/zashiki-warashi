use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use rusqlite::Connection;
use tracing::info;

use crate::error::AppError;

const SCHEMA_VERSION: i64 = 3;

pub struct Database {
    connection: Mutex<Connection>,
    path: PathBuf,
}

impl Database {
    pub fn open(app_data_dir: &Path) -> Result<Self, AppError> {
        std::fs::create_dir_all(app_data_dir)?;
        let path = app_data_dir.join("zashiki.db");
        let connection = Connection::open(&path)?;
        connection.execute_batch("PRAGMA foreign_keys = ON;")?;
        let db = Self {
            connection: Mutex::new(connection),
            path,
        };
        db.migrate()?;
        info!(path = %db.path.display(), "opened sqlite database");
        Ok(db)
    }

    pub fn schema_version(&self) -> Result<i64, AppError> {
        let conn = self.lock()?;
        let version: i64 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        Ok(version)
    }

    pub fn ping(&self) -> Result<(), AppError> {
        let conn = self.lock()?;
        conn.query_row("SELECT 1", [], |_| Ok(()))?;
        Ok(())
    }

    pub fn with_conn<T, F>(&self, f: F) -> Result<T, AppError>
    where
        F: FnOnce(&Connection) -> Result<T, AppError>,
    {
        let conn = self.lock()?;
        f(&conn)
    }

    fn migrate(&self) -> Result<(), AppError> {
        let conn = self.lock()?;
        let version: i64 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;

        if version < 1 {
            conn.execute_batch(
                "
                CREATE TABLE IF NOT EXISTS meta (
                    key TEXT PRIMARY KEY NOT NULL,
                    value TEXT NOT NULL
                );

                CREATE TABLE IF NOT EXISTS projects (
                    id TEXT PRIMARY KEY NOT NULL,
                    name TEXT NOT NULL,
                    path TEXT NOT NULL UNIQUE,
                    start_command TEXT,
                    stop_command TEXT,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                );

                CREATE TABLE IF NOT EXISTS project_runs (
                    project_id TEXT PRIMARY KEY NOT NULL,
                    pid INTEGER,
                    started_at TEXT,
                    status TEXT NOT NULL,
                    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
                );

                PRAGMA user_version = 1;
                ",
            )?;
            info!(from = version, to = 1, "applied database migration");
        }

        let version: i64 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        if version < 2 {
            conn.execute_batch(
                "
                ALTER TABLE project_runs ADD COLUMN pgid INTEGER;
                ALTER TABLE project_runs ADD COLUMN last_error TEXT;
                ALTER TABLE project_runs ADD COLUMN started_at_unix INTEGER;
                PRAGMA user_version = 2;
                ",
            )?;
            info!(from = version, to = 2, "applied database migration");
        }

        let version: i64 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        if version < 3 {
            conn.execute_batch(
                "
                CREATE TABLE IF NOT EXISTS port_overrides (
                    project_id TEXT NOT NULL,
                    service TEXT NOT NULL,
                    host_port INTEGER NOT NULL,
                    container_port INTEGER NOT NULL,
                    PRIMARY KEY (project_id, service),
                    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
                );
                PRAGMA user_version = 3;
                ",
            )?;
            info!(from = version, to = SCHEMA_VERSION, "applied database migration");
        }

        Ok(())
    }

    fn lock(&self) -> Result<MutexGuard<'_, Connection>, AppError> {
        self.connection
            .lock()
            .map_err(|_| AppError::message("database lock poisoned"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir() -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("zashiki-test-{stamp}"));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn opens_and_migrates_fresh_database() {
        let dir = temp_dir();
        let db = Database::open(&dir).expect("open database");
        assert_eq!(db.schema_version().expect("schema"), 3);
        db.ping().expect("ping");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
