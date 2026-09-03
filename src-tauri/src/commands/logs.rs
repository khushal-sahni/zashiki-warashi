use std::sync::Arc;

use tauri::State;

use crate::domain::{LogChunk, LogSource};
use crate::error::AppError;
use crate::services::{CatalogService, LogService, StackService};

#[tauri::command]
pub fn get_project_logs(
    project_id: String,
    source: LogSource,
    tail: Option<u32>,
    logs: State<'_, Arc<LogService>>,
    catalog: State<'_, Arc<CatalogService>>,
    stack: State<'_, Arc<StackService>>,
) -> Result<LogChunk, AppError> {
    match source {
        LogSource::Process => logs.read_tail(&project_id, tail),
        LogSource::Compose => {
            let project = catalog.get_project(&project_id)?;
            let Some(command) = stack.compose_log_command(&project_id, tail.unwrap_or(2000))? else {
                return Ok(LogChunk {
                    project_id,
                    source: LogSource::Compose,
                    lines: Vec::new(),
                    truncated: false,
                });
            };
            logs.compose_logs(&project_id, &project.path, &command, tail)
        }
    }
}

#[tauri::command]
pub fn clear_project_logs(
    project_id: String,
    logs: State<'_, Arc<LogService>>,
) -> Result<(), AppError> {
    logs.clear(&project_id)
}

#[tauri::command]
pub fn project_has_compose(
    project_id: String,
    catalog: State<'_, Arc<CatalogService>>,
) -> Result<bool, AppError> {
    let project = catalog.get_project(&project_id)?;
    Ok(crate::services::infer::has_compose(std::path::Path::new(
        &project.path,
    )))
}
