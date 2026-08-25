use tauri::State;

use crate::domain::AppStatus;
use crate::error::AppError;
use crate::services::AppService;

#[tauri::command]
pub fn get_app_status(app_service: State<'_, AppService>) -> Result<AppStatus, AppError> {
    app_service.get_status()
}
