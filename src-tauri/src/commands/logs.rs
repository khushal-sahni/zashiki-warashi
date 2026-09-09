use std::sync::Arc;

use tauri::State;

use crate::domain::{LogChunk, LogSource};
use crate::error::AppError;
use crate::services::{LogService, StackService};

#[tauri::command]
pub async fn get_project_logs(
    project_id: String,
    source: LogSource,
    tail: Option<u32>,
    logs: State<'_, Arc<LogService>>,
    stack: State<'_, Arc<StackService>>,
) -> Result<LogChunk, AppError> {
    let logs = Arc::clone(&logs);
    let stack = Arc::clone(&stack);
    tauri::async_runtime::spawn_blocking(move || match source {
        LogSource::Process => logs.read_tail(&project_id, tail),
        LogSource::Compose => {
            let limit = tail.unwrap_or(2000);
            match stack.compose_logs(&project_id, limit)? {
                None => Ok(LogChunk {
                    project_id,
                    source: LogSource::Compose,
                    lines: Vec::new(),
                    truncated: false,
                }),
                Some(raw) => logs.compose_logs_from_output(&project_id, &raw, Some(limit)),
            }
        }
    })
    .await
    .map_err(|err| AppError::Io(format!("get logs join failed: {err}")))?
}

#[tauri::command]
pub fn clear_project_logs(
    project_id: String,
    logs: State<'_, Arc<LogService>>,
) -> Result<(), AppError> {
    logs.clear(&project_id)
}
