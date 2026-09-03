use std::sync::Arc;

use tauri::State;

use crate::domain::{PortConflict, ProjectStack, ReconcileAction};
use crate::error::AppError;
use crate::services::StackService;

#[tauri::command]
pub fn get_project_stack(
    project_id: String,
    stack: State<'_, Arc<StackService>>,
) -> Result<ProjectStack, AppError> {
    stack.get_stack(&project_id)
}

#[tauri::command]
pub fn start_project_stack(
    project_id: String,
    stack: State<'_, Arc<StackService>>,
) -> Result<ProjectStack, AppError> {
    stack.start_stack(&project_id)
}

#[tauri::command]
pub fn stop_project_stack(
    project_id: String,
    stack: State<'_, Arc<StackService>>,
) -> Result<ProjectStack, AppError> {
    stack.stop_stack(&project_id)
}

#[tauri::command]
pub fn resolve_port_conflict(
    project_id: String,
    action: ReconcileAction,
    write_to_repo: bool,
    confirm_native: bool,
    conflict: Option<PortConflict>,
    stack: State<'_, Arc<StackService>>,
) -> Result<ProjectStack, AppError> {
    stack.resolve_from_scan(
        &project_id,
        action,
        write_to_repo,
        confirm_native,
        conflict,
    )
}
