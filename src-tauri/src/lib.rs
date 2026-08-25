mod commands;
mod domain;
mod error;
mod repositories;
mod services;

use std::sync::Arc;

use tauri::Manager;
use tracing::info;

use commands::get_app_status;
use repositories::Database;
use services::AppService;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|err| AppSetupError(err.to_string()))?;

            std::fs::create_dir_all(&app_data_dir)?;
            info!(path = %app_data_dir.display(), "app data directory ready");

            let database = Arc::new(Database::open(&app_data_dir)?);
            let app_service = AppService::new(database, app_data_dir.display().to_string());
            app.manage(app_service);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_app_status])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[derive(Debug)]
struct AppSetupError(String);

impl std::fmt::Display for AppSetupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for AppSetupError {}
