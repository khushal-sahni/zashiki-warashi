mod commands;
mod domain;
mod error;
mod repositories;
mod services;
mod tray;

use std::sync::Arc;

use tauri::Manager;
use tracing::info;

use commands::{
    add_project, clear_project_logs, get_app_status, get_keep_awake_status, get_project_logs,
    get_project_stack, get_settings, list_projects, open_project_in_cursor, open_project_in_finder,
    peek_project_stack, remove_project, resolve_port_conflict, restart_project, scan_projects,
    set_keep_awake_enabled, set_scan_roots, start_project, start_project_stack, stop_project,
    stop_project_stack, update_project_commands,
};
use repositories::{Database, ProjectRepository};
use services::{
    docker_bin, AppService, CatalogService, ComposeService, KeepAwakeService, LogService,
    OpenService, PortOccupancyService, ProcessService, StackService,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
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
            let project_repository = Arc::new(ProjectRepository::new(database.clone()));
            let catalog_service = Arc::new(CatalogService::new(project_repository.clone()));
            let log_service = Arc::new(LogService::new(app_data_dir.join("logs")));
            log_service.start_tailer(app.handle().clone());
            let occupancy_service = Arc::new(PortOccupancyService::new(project_repository.clone()));
            let compose_service = Arc::new(ComposeService::new(app_data_dir.join("compose-overrides")));
            docker_bin::warm_docker_bin();
            let stack_service = Arc::new(StackService::new(
                catalog_service.clone(),
                project_repository.clone(),
                occupancy_service,
                compose_service,
            ));
            let process_service = Arc::new(ProcessService::new(
                project_repository.clone(),
                catalog_service.clone(),
                log_service.clone(),
                stack_service.clone(),
            ));
            process_service.rehydrate_all()?;

            let keep_awake_service = Arc::new(KeepAwakeService::new(project_repository.clone()));
            keep_awake_service.rehydrate()?;

            let open_service = Arc::new(OpenService::new(catalog_service.clone()));

            let app_service = AppService::new(database, app_data_dir.display().to_string());
            app.manage(app_service);
            app.manage(catalog_service);
            app.manage(process_service);
            app.manage(keep_awake_service);
            app.manage(log_service);
            app.manage(stack_service);
            app.manage(open_service);

            tray::install_tray(app).map_err(|err| AppSetupError(err.to_string()))?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_app_status,
            get_keep_awake_status,
            set_keep_awake_enabled,
            list_projects,
            add_project,
            remove_project,
            update_project_commands,
            scan_projects,
            get_settings,
            set_scan_roots,
            open_project_in_finder,
            open_project_in_cursor,
            start_project,
            stop_project,
            restart_project,
            get_project_logs,
            clear_project_logs,
            peek_project_stack,
            get_project_stack,
            start_project_stack,
            stop_project_stack,
            resolve_port_conflict,
        ])
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
