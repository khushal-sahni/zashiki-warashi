use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Arc;

use crate::error::AppError;
use crate::services::CatalogService;

pub struct OpenService {
    catalog: Arc<CatalogService>,
}

impl OpenService {
    pub fn new(catalog: Arc<CatalogService>) -> Self {
        Self { catalog }
    }

    pub fn open_in_finder(&self, project_id: &str) -> Result<(), AppError> {
        let project = self.catalog.get_project(project_id)?;
        ensure_path_exists(&project.path)?;
        open_path(&project.path)
    }

    pub fn open_in_cursor(&self, project_id: &str) -> Result<(), AppError> {
        let project = self.catalog.get_project(project_id)?;
        ensure_path_exists(&project.path)?;
        open_in_app("Cursor", &project.path)
    }
}

fn ensure_path_exists(path: &str) -> Result<(), AppError> {
    if Path::new(path).exists() {
        Ok(())
    } else {
        Err(AppError::not_found(format!(
            "project folder is missing: {path}"
        )))
    }
}

fn open_path(path: &str) -> Result<(), AppError> {
    let status = Command::new("open")
        .arg(path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|err| AppError::Io(format!("failed to open Finder: {err}")))?;

    if status.success() {
        Ok(())
    } else {
        Err(AppError::message(format!(
            "could not open folder in Finder (exit {status})"
        )))
    }
}

fn open_in_app(app_name: &str, path: &str) -> Result<(), AppError> {
    let output = Command::new("open")
        .args(["-a", app_name, path])
        .stdin(Stdio::null())
        .output()
        .map_err(|err| AppError::Io(format!("failed to open {app_name}: {err}")))?;

    if output.status.success() {
        return Ok(());
    }

    let detail = String::from_utf8_lossy(&output.stderr).to_lowercase();
    if detail.contains("unable to find application")
        || detail.contains("application isn't running")
        || detail.contains("doesn't exist")
        || detail.contains("lsopenurlspecifies")
    {
        return Err(AppError::message(
            "Cursor is not installed. Install Cursor, or open the folder in Finder.",
        ));
    }

    let trimmed = String::from_utf8_lossy(&output.stderr);
    let trimmed = trimmed.trim();
    if trimmed.is_empty() {
        Err(AppError::message(
            "Cursor is not installed. Install Cursor, or open the folder in Finder.",
        ))
    } else {
        Err(AppError::message(format!(
            "could not open in Cursor: {trimmed}"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_path_exists_errors_when_missing() {
        let err = ensure_path_exists("/tmp/zashiki-warashi-missing-path-xyz").unwrap_err();
        let message = err.to_string();
        assert!(message.contains("missing"));
    }

    #[test]
    fn open_in_app_maps_missing_cursor() {
        // Use a nonsense app name so `open -a` fails predictably on macOS/CI.
        let err = open_in_app("ZashikiWarashiFakeAppThatDoesNotExist", "/tmp").unwrap_err();
        let message = err.to_string();
        assert!(
            message.contains("Cursor is not installed") || message.contains("could not open"),
            "unexpected message: {message}"
        );
    }
}
