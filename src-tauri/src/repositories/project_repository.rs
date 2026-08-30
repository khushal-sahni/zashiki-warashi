use std::sync::Arc;

use rusqlite::{params, OptionalExtension};

use crate::domain::{AppSettings, ProjectRun, RunState};
use crate::error::AppError;
use crate::repositories::Database;

const SCAN_ROOTS_KEY: &str = "scan_roots";
const KEEP_AWAKE_ENABLED_KEY: &str = "keep_awake_enabled";
const KEEP_AWAKE_CAFFEINATE_PID_KEY: &str = "keep_awake_caffeinate_pid";

pub struct ProjectRepository {
    database: Arc<Database>,
}

#[derive(Debug, Clone)]
pub struct ProjectRow {
    pub id: String,
    pub name: String,
    pub path: String,
    pub start_command: Option<String>,
    pub stop_command: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl ProjectRepository {
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    pub fn insert_project(&self, project: &ProjectRow) -> Result<(), AppError> {
        self.database.with_conn(|conn| {
            let result = conn.execute(
                "
                INSERT INTO projects (id, name, path, start_command, stop_command, created_at, updated_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                ",
                params![
                    project.id,
                    project.name,
                    project.path,
                    project.start_command,
                    project.stop_command,
                    project.created_at,
                    project.updated_at,
                ],
            );

            match result {
                Ok(_) => {}
                Err(rusqlite::Error::SqliteFailure(info, _))
                    if info.code == rusqlite::ErrorCode::ConstraintViolation =>
                {
                    return Err(AppError::conflict(format!(
                        "project already registered at {}",
                        project.path
                    )));
                }
                Err(err) => return Err(AppError::from(err)),
            }

            conn.execute(
                "
                INSERT INTO project_runs (project_id, pid, pgid, started_at, started_at_unix, status, last_error)
                VALUES (?1, NULL, NULL, NULL, NULL, ?2, NULL)
                ",
                params![project.id, RunState::Stopped.as_str()],
            )?;

            Ok(())
        })
    }

    pub fn delete_project(&self, id: &str) -> Result<bool, AppError> {
        self.database.with_conn(|conn| {
            let removed = conn.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
            Ok(removed > 0)
        })
    }

    pub fn get_project(&self, id: &str) -> Result<Option<ProjectRow>, AppError> {
        self.database.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "
                SELECT id, name, path, start_command, stop_command, created_at, updated_at
                FROM projects
                WHERE id = ?1
                ",
            )?;
            let row = stmt
                .query_row(params![id], |row| {
                    Ok(ProjectRow {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        path: row.get(2)?,
                        start_command: row.get(3)?,
                        stop_command: row.get(4)?,
                        created_at: row.get(5)?,
                        updated_at: row.get(6)?,
                    })
                })
                .optional()?;
            Ok(row)
        })
    }

    pub fn find_by_path(&self, path: &str) -> Result<Option<ProjectRow>, AppError> {
        self.database.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "
                SELECT id, name, path, start_command, stop_command, created_at, updated_at
                FROM projects
                WHERE path = ?1
                ",
            )?;
            let row = stmt
                .query_row(params![path], |row| {
                    Ok(ProjectRow {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        path: row.get(2)?,
                        start_command: row.get(3)?,
                        stop_command: row.get(4)?,
                        created_at: row.get(5)?,
                        updated_at: row.get(6)?,
                    })
                })
                .optional()?;
            Ok(row)
        })
    }

    pub fn list_projects(&self) -> Result<Vec<ProjectRow>, AppError> {
        self.database.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "
                SELECT id, name, path, start_command, stop_command, created_at, updated_at
                FROM projects
                ORDER BY name COLLATE NOCASE ASC
                ",
            )?;
            let rows = stmt
                .query_map([], |row| {
                    Ok(ProjectRow {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        path: row.get(2)?,
                        start_command: row.get(3)?,
                        stop_command: row.get(4)?,
                        created_at: row.get(5)?,
                        updated_at: row.get(6)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
    }

    pub fn update_commands(
        &self,
        id: &str,
        start_command: Option<Option<String>>,
        stop_command: Option<Option<String>>,
        updated_at: &str,
    ) -> Result<bool, AppError> {
        self.database.with_conn(|conn| {
            let existing = conn
                .query_row(
                    "
                    SELECT start_command, stop_command FROM projects WHERE id = ?1
                    ",
                    params![id],
                    |row| {
                        Ok((
                            row.get::<_, Option<String>>(0)?,
                            row.get::<_, Option<String>>(1)?,
                        ))
                    },
                )
                .optional()?;

            let Some((current_start, current_stop)) = existing else {
                return Ok(false);
            };

            let next_start = match start_command {
                Some(value) => value,
                None => current_start,
            };
            let next_stop = match stop_command {
                Some(value) => value,
                None => current_stop,
            };

            conn.execute(
                "
                UPDATE projects
                SET start_command = ?1, stop_command = ?2, updated_at = ?3
                WHERE id = ?4
                ",
                params![next_start, next_stop, updated_at, id],
            )?;
            Ok(true)
        })
    }

    pub fn get_run(&self, project_id: &str) -> Result<Option<ProjectRun>, AppError> {
        self.database.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "
                SELECT project_id, pid, pgid, started_at, started_at_unix, status, last_error
                FROM project_runs
                WHERE project_id = ?1
                ",
            )?;
            let row = stmt
                .query_row(params![project_id], |row| {
                    let status: String = row.get(5)?;
                    Ok(ProjectRun {
                        project_id: row.get(0)?,
                        pid: row.get(1)?,
                        pgid: row.get(2)?,
                        started_at: row.get(3)?,
                        started_at_unix: row.get(4)?,
                        status: RunState::parse(&status),
                        last_error: row.get(6)?,
                    })
                })
                .optional()?;
            Ok(row)
        })
    }

    pub fn list_runs(&self) -> Result<Vec<ProjectRun>, AppError> {
        self.database.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "
                SELECT project_id, pid, pgid, started_at, started_at_unix, status, last_error
                FROM project_runs
                ",
            )?;
            let rows = stmt
                .query_map([], |row| {
                    let status: String = row.get(5)?;
                    Ok(ProjectRun {
                        project_id: row.get(0)?,
                        pid: row.get(1)?,
                        pgid: row.get(2)?,
                        started_at: row.get(3)?,
                        started_at_unix: row.get(4)?,
                        status: RunState::parse(&status),
                        last_error: row.get(6)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
    }

    pub fn upsert_run(&self, run: &ProjectRun) -> Result<(), AppError> {
        self.database.with_conn(|conn| {
            conn.execute(
                "
                INSERT INTO project_runs (
                    project_id, pid, pgid, started_at, started_at_unix, status, last_error
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                ON CONFLICT(project_id) DO UPDATE SET
                    pid = excluded.pid,
                    pgid = excluded.pgid,
                    started_at = excluded.started_at,
                    started_at_unix = excluded.started_at_unix,
                    status = excluded.status,
                    last_error = excluded.last_error
                ",
                params![
                    run.project_id,
                    run.pid,
                    run.pgid,
                    run.started_at,
                    run.started_at_unix,
                    run.status.as_str(),
                    run.last_error,
                ],
            )?;
            Ok(())
        })
    }

    pub fn get_settings(&self) -> Result<AppSettings, AppError> {
        self.database.with_conn(|conn| {
            let value: Option<String> = conn
                .query_row(
                    "SELECT value FROM meta WHERE key = ?1",
                    params![SCAN_ROOTS_KEY],
                    |row| row.get(0),
                )
                .optional()?;

            let scan_roots = match value {
                Some(raw) => serde_json::from_str(&raw)?,
                None => Vec::new(),
            };

            Ok(AppSettings { scan_roots })
        })
    }

    pub fn set_scan_roots(&self, roots: &[String]) -> Result<AppSettings, AppError> {
        let encoded = serde_json::to_string(roots)?;
        self.set_meta_value(SCAN_ROOTS_KEY, &encoded)?;
        Ok(AppSettings {
            scan_roots: roots.to_vec(),
        })
    }

    pub fn get_keep_awake_enabled(&self) -> Result<bool, AppError> {
        match self.get_meta_value(KEEP_AWAKE_ENABLED_KEY)? {
            Some(raw) => parse_bool_meta(&raw),
            None => Ok(false),
        }
    }

    pub fn set_keep_awake_enabled(&self, enabled: bool) -> Result<(), AppError> {
        self.set_meta_value(KEEP_AWAKE_ENABLED_KEY, if enabled { "true" } else { "false" })
    }

    pub fn get_keep_awake_caffeinate_pid(&self) -> Result<Option<i32>, AppError> {
        match self.get_meta_value(KEEP_AWAKE_CAFFEINATE_PID_KEY)? {
            Some(raw) => raw
                .parse::<i32>()
                .map(Some)
                .map_err(|err| AppError::Database(format!("invalid caffeinate pid: {err}"))),
            None => Ok(None),
        }
    }

    pub fn set_keep_awake_caffeinate_pid(&self, pid: Option<i32>) -> Result<(), AppError> {
        match pid {
            Some(value) => self.set_meta_value(KEEP_AWAKE_CAFFEINATE_PID_KEY, &value.to_string()),
            None => self.delete_meta_value(KEEP_AWAKE_CAFFEINATE_PID_KEY),
        }
    }

    fn get_meta_value(&self, key: &str) -> Result<Option<String>, AppError> {
        self.database.with_conn(|conn| {
            conn.query_row(
                "SELECT value FROM meta WHERE key = ?1",
                params![key],
                |row| row.get(0),
            )
            .optional()
            .map_err(AppError::from)
        })
    }

    fn set_meta_value(&self, key: &str, value: &str) -> Result<(), AppError> {
        self.database.with_conn(|conn| {
            conn.execute(
                "
                INSERT INTO meta (key, value) VALUES (?1, ?2)
                ON CONFLICT(key) DO UPDATE SET value = excluded.value
                ",
                params![key, value],
            )?;
            Ok(())
        })
    }

    fn delete_meta_value(&self, key: &str) -> Result<(), AppError> {
        self.database.with_conn(|conn| {
            conn.execute("DELETE FROM meta WHERE key = ?1", params![key])?;
            Ok(())
        })
    }
}

fn parse_bool_meta(raw: &str) -> Result<bool, AppError> {
    match raw {
        "true" | "1" => Ok(true),
        "false" | "0" => Ok(false),
        _ => Err(AppError::Database(format!("invalid boolean meta value: {raw}"))),
    }
}
