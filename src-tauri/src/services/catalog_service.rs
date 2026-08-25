use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use uuid::Uuid;

use crate::domain::{AppSettings, Project, ProjectRun, RunState, ScanCandidate};
use crate::error::AppError;
use crate::repositories::{ProjectRepository, ProjectRow};
use crate::services::infer::{infer_start_command, is_project_candidate};

pub struct CatalogService {
    repository: Arc<ProjectRepository>,
}

impl CatalogService {
    pub fn new(repository: Arc<ProjectRepository>) -> Self {
        Self { repository }
    }

    pub fn list_projects(&self) -> Result<Vec<Project>, AppError> {
        let rows = self.repository.list_projects()?;
        rows.into_iter()
            .map(|row| self.to_project(row))
            .collect()
    }

    pub fn get_project(&self, id: &str) -> Result<Project, AppError> {
        let row = self
            .repository
            .get_project(id)?
            .ok_or_else(|| AppError::not_found(format!("project {id} not found")))?;
        self.to_project(row)
    }

    pub fn add_project(&self, path: &str) -> Result<Project, AppError> {
        let canonical = canonicalize_existing_dir(path)?;
        let path_str = canonical.to_string_lossy().to_string();

        if self.repository.find_by_path(&path_str)?.is_some() {
            return Err(AppError::conflict(format!(
                "project already registered at {path_str}"
            )));
        }

        let name = canonical
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("untitled")
            .to_string();

        let now = now_iso();
        let row = ProjectRow {
            id: Uuid::new_v4().to_string(),
            name,
            path: path_str,
            start_command: None,
            stop_command: None,
            created_at: now.clone(),
            updated_at: now,
        };

        self.repository.insert_project(&row)?;
        self.to_project(row)
    }

    pub fn remove_project(&self, id: &str) -> Result<(), AppError> {
        let removed = self.repository.delete_project(id)?;
        if !removed {
            return Err(AppError::not_found(format!("project {id} not found")));
        }
        Ok(())
    }

    pub fn update_commands(
        &self,
        id: &str,
        start_command: Option<Option<String>>,
        stop_command: Option<Option<String>>,
    ) -> Result<Project, AppError> {
        let updated = self.repository.update_commands(
            id,
            start_command,
            stop_command,
            &now_iso(),
        )?;
        if !updated {
            return Err(AppError::not_found(format!("project {id} not found")));
        }
        self.get_project(id)
    }

    pub fn scan_projects(&self, roots: &[String]) -> Result<Vec<ScanCandidate>, AppError> {
        let registered = self.repository.list_projects()?;
        let registered_paths: Vec<String> = registered.into_iter().map(|row| row.path).collect();

        let mut candidates = Vec::new();
        for root in roots {
            let root_path = expand_home(root);
            if !root_path.is_dir() {
                continue;
            }
            let entries = match fs::read_dir(&root_path) {
                Ok(entries) => entries,
                Err(_) => continue,
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if !is_project_candidate(&path) {
                    continue;
                }
                let path_str = match path.canonicalize() {
                    Ok(canonical) => canonical.to_string_lossy().to_string(),
                    Err(_) => path.to_string_lossy().to_string(),
                };
                let name = path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or("untitled")
                    .to_string();
                let already_registered = registered_paths.iter().any(|existing| existing == &path_str);
                candidates.push(ScanCandidate {
                    name,
                    path: path_str.clone(),
                    inferred_start_command: infer_start_command(Path::new(&path_str)),
                    already_registered,
                });
            }
        }

        candidates.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        Ok(candidates)
    }

    pub fn get_settings(&self) -> Result<AppSettings, AppError> {
        let mut settings = self.repository.get_settings()?;
        if settings.scan_roots.is_empty() {
            if let Some(default_root) = default_scan_root() {
                settings.scan_roots.push(default_root);
            }
        }
        Ok(settings)
    }

    pub fn set_scan_roots(&self, roots: Vec<String>) -> Result<AppSettings, AppError> {
        let normalized: Vec<String> = roots
            .into_iter()
            .map(|root| expand_home(&root).to_string_lossy().to_string())
            .filter(|root| !root.trim().is_empty())
            .collect();
        self.repository.set_scan_roots(&normalized)
    }

    fn to_project(&self, row: ProjectRow) -> Result<Project, AppError> {
        let inferred = infer_start_command(Path::new(&row.path));
        let run = self
            .repository
            .get_run(&row.id)?
            .unwrap_or_else(|| ProjectRun {
                project_id: row.id.clone(),
                pid: None,
                pgid: None,
                started_at: None,
                started_at_unix: None,
                status: RunState::Stopped,
                last_error: None,
            });

        Ok(Project {
            id: row.id,
            name: row.name,
            path: row.path,
            start_command: row.start_command,
            stop_command: row.stop_command,
            created_at: row.created_at,
            updated_at: row.updated_at,
            inferred_start_command: inferred,
            run,
        })
    }
}

fn canonicalize_existing_dir(path: &str) -> Result<PathBuf, AppError> {
    let expanded = expand_home(path);
    if !expanded.is_dir() {
        return Err(AppError::invalid(format!(
            "path is not a directory: {}",
            expanded.display()
        )));
    }
    expanded
        .canonicalize()
        .map_err(|err| AppError::io(err.to_string()))
}

fn expand_home(path: &str) -> PathBuf {
    if path == "~" {
        return dirs_home().unwrap_or_else(|| PathBuf::from(path));
    }
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs_home() {
            return home.join(rest);
        }
    }
    PathBuf::from(path)
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

fn default_scan_root() -> Option<String> {
    let home = dirs_home()?;
    let projects = home.join("Documents").join("Projects");
    if projects.is_dir() {
        Some(projects.to_string_lossy().to_string())
    } else {
        None
    }
}

fn now_iso() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", now.as_secs())
}

trait IoErrorExt {
    fn io(msg: String) -> AppError;
}

impl IoErrorExt for AppError {
    fn io(msg: String) -> AppError {
        AppError::Io(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::Database;

    fn setup() -> (CatalogService, PathBuf) {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("zashiki-catalog-{stamp}"));
        fs::create_dir_all(&dir).unwrap();
        let db = Arc::new(Database::open(&dir).unwrap());
        let repo = Arc::new(ProjectRepository::new(db));
        (CatalogService::new(repo), dir)
    }

    #[test]
    fn add_list_remove_and_reject_duplicate() {
        let (service, db_dir) = setup();
        let project_dir = db_dir.join("sample-app");
        fs::create_dir_all(&project_dir).unwrap();
        fs::write(
            project_dir.join("package.json"),
            r#"{"scripts":{"dev":"vite"}}"#,
        )
        .unwrap();

        let added = service
            .add_project(project_dir.to_str().unwrap())
            .expect("add");
        assert_eq!(added.name, "sample-app");
        assert_eq!(added.inferred_start_command.as_deref(), Some("npm run dev"));

        let listed = service.list_projects().unwrap();
        assert_eq!(listed.len(), 1);

        let duplicate = service.add_project(project_dir.to_str().unwrap());
        assert!(matches!(duplicate, Err(AppError::Conflict(_))));

        service.remove_project(&added.id).unwrap();
        assert!(service.list_projects().unwrap().is_empty());

        let _ = fs::remove_dir_all(db_dir);
    }

    #[test]
    fn scan_returns_candidates_without_inserting() {
        let (service, db_dir) = setup();
        let root = db_dir.join("roots");
        let child = root.join("demo");
        fs::create_dir_all(&child).unwrap();
        fs::write(child.join("Cargo.toml"), "[package]\nname=\"demo\"\n").unwrap();

        let candidates = service
            .scan_projects(&[root.to_string_lossy().to_string()])
            .unwrap();
        assert_eq!(candidates.len(), 1);
        assert!(!candidates[0].already_registered);
        assert!(service.list_projects().unwrap().is_empty());

        let _ = fs::remove_dir_all(db_dir);
    }
}
