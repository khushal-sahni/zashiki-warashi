use std::sync::Arc;

use tauri::State;

use crate::domain::{PortConflict, ProjectStack, ReconcileAction};
use crate::error::AppError;
use crate::services::StackService;

#[tauri::command]
pub async fn peek_project_stack(
    project_id: String,
    stack: State<'_, Arc<StackService>>,
) -> Result<ProjectStack, AppError> {
    let stack = Arc::clone(&stack);
    tauri::async_runtime::spawn_blocking(move || stack.peek_stack(&project_id))
        .await
        .map_err(|err| AppError::Io(format!("peek stack join failed: {err}")))?
}

#[tauri::command]
pub async fn get_project_stack(
    project_id: String,
    stack: State<'_, Arc<StackService>>,
) -> Result<ProjectStack, AppError> {
    let stack = Arc::clone(&stack);
    tauri::async_runtime::spawn_blocking(move || stack.get_stack(&project_id))
        .await
        .map_err(|err| AppError::Io(format!("get stack join failed: {err}")))?
}

#[tauri::command]
pub async fn start_project_stack(
    project_id: String,
    stack: State<'_, Arc<StackService>>,
) -> Result<ProjectStack, AppError> {
    let stack = Arc::clone(&stack);
    tauri::async_runtime::spawn_blocking(move || stack.start_stack(&project_id))
        .await
        .map_err(|err| AppError::Io(format!("start stack join failed: {err}")))?
}

#[tauri::command]
pub async fn stop_project_stack(
    project_id: String,
    stack: State<'_, Arc<StackService>>,
) -> Result<ProjectStack, AppError> {
    let stack = Arc::clone(&stack);
    tauri::async_runtime::spawn_blocking(move || stack.stop_stack(&project_id))
        .await
        .map_err(|err| AppError::Io(format!("stop stack join failed: {err}")))?
}

#[tauri::command]
pub async fn resolve_port_conflict(
    project_id: String,
    action: ReconcileAction,
    write_to_repo: bool,
    confirm_native: bool,
    conflict: Option<PortConflict>,
    stack: State<'_, Arc<StackService>>,
) -> Result<ProjectStack, AppError> {
    let stack = Arc::clone(&stack);
    tauri::async_runtime::spawn_blocking(move || {
        stack.resolve_from_scan(
            &project_id,
            action,
            write_to_repo,
            confirm_native,
            conflict,
        )
    })
    .await
    .map_err(|err| AppError::Io(format!("resolve conflict join failed: {err}")))?
}
