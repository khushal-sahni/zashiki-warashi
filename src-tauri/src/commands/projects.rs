use std::sync::Arc;

use tauri::State;

use crate::domain::{AppSettings, Project, ScanCandidate};
use crate::error::AppError;
use crate::services::{CatalogService, ProcessService};

#[tauri::command]
pub fn list_projects(catalog: State<'_, Arc<CatalogService>>) -> Result<Vec<Project>, AppError> {
    catalog.list_projects()
}

#[tauri::command]
pub fn add_project(
    path: String,
    catalog: State<'_, Arc<CatalogService>>,
) -> Result<Project, AppError> {
    catalog.add_project(&path)
}

#[tauri::command]
pub fn remove_project(
    id: String,
    catalog: State<'_, Arc<CatalogService>>,
) -> Result<(), AppError> {
    catalog.remove_project(&id)
}

#[tauri::command]
pub fn update_project_commands(
    id: String,
    start_command: Option<String>,
    stop_command: Option<String>,
    catalog: State<'_, Arc<CatalogService>>,
) -> Result<Project, AppError> {
    let start = start_command.and_then(|value| {
        let trimmed = value.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    });
    let stop = stop_command.and_then(|value| {
        let trimmed = value.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    });
    catalog.update_commands(&id, Some(start), Some(stop))
}

#[tauri::command]
pub fn scan_projects(
    roots: Option<Vec<String>>,
    catalog: State<'_, Arc<CatalogService>>,
) -> Result<Vec<ScanCandidate>, AppError> {
    let settings = catalog.get_settings()?;
    let scan_roots = roots.unwrap_or(settings.scan_roots);
    catalog.scan_projects(&scan_roots)
}

#[tauri::command]
pub fn get_settings(catalog: State<'_, Arc<CatalogService>>) -> Result<AppSettings, AppError> {
    catalog.get_settings()
}

#[tauri::command]
pub fn set_scan_roots(
    roots: Vec<String>,
    catalog: State<'_, Arc<CatalogService>>,
) -> Result<AppSettings, AppError> {
    catalog.set_scan_roots(roots)
}

#[tauri::command]
pub fn start_project(
    id: String,
    process: State<'_, Arc<ProcessService>>,
) -> Result<Project, AppError> {
    process.start_project(&id)
}

#[tauri::command]
pub fn stop_project(
    id: String,
    process: State<'_, Arc<ProcessService>>,
) -> Result<Project, AppError> {
    process.stop_project(&id)
}

#[tauri::command]
pub fn restart_project(
    id: String,
    process: State<'_, Arc<ProcessService>>,
) -> Result<Project, AppError> {
    process.restart_project(&id)
}
