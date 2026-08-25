pub mod app;
pub mod projects;

pub use app::get_app_status;
pub use projects::{
    add_project, get_settings, list_projects, remove_project, restart_project, scan_projects,
    set_scan_roots, start_project, stop_project, update_project_commands,
};
