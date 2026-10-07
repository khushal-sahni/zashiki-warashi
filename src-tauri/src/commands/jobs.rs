use std::sync::Arc;

use tauri::State;

use crate::domain::{ForeignAgent, JobInput, JobLogTail, JobRun, JobSummary, JobSystemStatus};
use crate::error::AppError;
use crate::services::system_probe::MacSystemProbe;
use crate::services::JobService;

async fn blocking<T, F>(jobs: &State<'_, Arc<JobService>>, label: &str, f: F) -> Result<T, AppError>
where
    T: Send + 'static,
    F: FnOnce(&JobService) -> Result<T, AppError> + Send + 'static,
{
    let service = Arc::clone(jobs);
    tauri::async_runtime::spawn_blocking(move || f(&service))
        .await
        .map_err(|err| AppError::Io(format!("{label} join failed: {err}")))?
}

#[tauri::command]
pub fn list_jobs(jobs: State<'_, Arc<JobService>>) -> Result<Vec<JobSummary>, AppError> {
    jobs.list()
}

#[tauri::command]
pub fn list_job_runs(
    id: String,
    limit: Option<u32>,
    jobs: State<'_, Arc<JobService>>,
) -> Result<Vec<JobRun>, AppError> {
    jobs.list_runs(&id, limit)
}

#[tauri::command]
pub fn get_job_run_log(
    run_id: String,
    lines: Option<usize>,
    jobs: State<'_, Arc<JobService>>,
) -> Result<JobLogTail, AppError> {
    jobs.log_tail(&run_id, lines)
}

#[tauri::command]
pub async fn create_job(input: JobInput, jobs: State<'_, Arc<JobService>>) -> Result<JobSummary, AppError> {
    blocking(&jobs, "create job", move |svc| svc.create(input)).await
}

#[tauri::command]
pub async fn update_job(
    id: String,
    input: JobInput,
    jobs: State<'_, Arc<JobService>>,
) -> Result<JobSummary, AppError> {
    blocking(&jobs, "update job", move |svc| svc.update(&id, input)).await
}

#[tauri::command]
pub async fn set_job_enabled(
    id: String,
    enabled: bool,
    jobs: State<'_, Arc<JobService>>,
) -> Result<JobSummary, AppError> {
    blocking(&jobs, "toggle job", move |svc| svc.set_enabled(&id, enabled)).await
}

#[tauri::command]
pub async fn delete_job(id: String, jobs: State<'_, Arc<JobService>>) -> Result<(), AppError> {
    blocking(&jobs, "delete job", move |svc| svc.delete(&id)).await
}

#[tauri::command]
pub async fn run_job_now(id: String, jobs: State<'_, Arc<JobService>>) -> Result<(), AppError> {
    blocking(&jobs, "run job", move |svc| svc.run_now(&id)).await
}

#[tauri::command]
pub async fn get_job_system_status(jobs: State<'_, Arc<JobService>>) -> Result<JobSystemStatus, AppError> {
    blocking(&jobs, "job status", |svc| Ok(svc.system_status(&MacSystemProbe))).await
}

#[tauri::command]
pub async fn list_foreign_agents(jobs: State<'_, Arc<JobService>>) -> Result<Vec<ForeignAgent>, AppError> {
    blocking(&jobs, "list agents", |svc| svc.foreign_agents()).await
}

#[tauri::command]
pub async fn install_wake_helper(jobs: State<'_, Arc<JobService>>) -> Result<(), AppError> {
    blocking(&jobs, "install wake helper", |svc| svc.install_wake_helper()).await
}

#[tauri::command]
pub async fn uninstall_wake_helper(jobs: State<'_, Arc<JobService>>) -> Result<(), AppError> {
    blocking(&jobs, "remove wake helper", |svc| svc.uninstall_wake_helper()).await
}
