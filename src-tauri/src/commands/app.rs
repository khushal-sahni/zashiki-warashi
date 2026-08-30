use tauri::State;

use crate::domain::{AppStatus, KeepAwakeStatus};
use crate::error::AppError;
use crate::services::{AppService, KeepAwakeService};
use std::sync::Arc;

#[tauri::command]
pub fn get_app_status(app_service: State<'_, AppService>) -> Result<AppStatus, AppError> {
    app_service.get_status()
}

#[tauri::command]
pub fn get_keep_awake_status(
    keep_awake: State<'_, Arc<KeepAwakeService>>,
) -> Result<KeepAwakeStatus, AppError> {
    keep_awake.status()
}

#[tauri::command]
pub fn set_keep_awake_enabled(
    enabled: bool,
    keep_awake: State<'_, Arc<KeepAwakeService>>,
) -> Result<KeepAwakeStatus, AppError> {
    keep_awake.set_enabled(enabled)
}
