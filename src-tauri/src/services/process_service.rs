use std::fs::OpenOptions;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tracing::{info, warn};

use crate::domain::{Project, ProjectRun, RunState};
use crate::error::AppError;
use crate::repositories::ProjectRepository;
use crate::services::catalog_service::CatalogService;
use crate::services::infer::infer_start_command;
use crate::services::log_service::LogService;
use crate::services::stack_service::StackService;

#[cfg(unix)]
use std::os::unix::process::CommandExt;

const STOP_GRACE_MS: u64 = 2_000;

pub struct ProcessService {
    repository: Arc<ProjectRepository>,
    catalog: Arc<CatalogService>,
    logs: Arc<LogService>,
    stack: Arc<StackService>,
}

impl ProcessService {
    pub fn new(
        repository: Arc<ProjectRepository>,
        catalog: Arc<CatalogService>,
        logs: Arc<LogService>,
        stack: Arc<StackService>,
    ) -> Self {
        Self {
            repository,
            catalog,
            logs,
            stack,
        }
    }

    pub fn rehydrate_all(&self) -> Result<(), AppError> {
        let runs = self.repository.list_runs()?;
        for run in runs {
            if matches!(run.status, RunState::Running | RunState::Starting) {
                let alive = match (run.pid, run.pgid) {
                    (Some(pid), pgid) => is_process_alive(pid, pgid),
                    _ => false,
                };
                if alive {
                    let mut next = run.clone();
                    next.status = RunState::Running;
                    self.repository.upsert_run(&next)?;
                } else {
                    let mut next = run;
                    next.pid = None;
                    next.pgid = None;
                    next.started_at = None;
                    next.started_at_unix = None;
                    next.status = RunState::Stopped;
                    next.last_error = None;
                    self.repository.upsert_run(&next)?;
                }
            }
        }
        Ok(())
    }

    pub fn start_project(&self, id: &str) -> Result<Project, AppError> {
        let project = self.catalog.get_project(id)?;
        if matches!(project.run.status, RunState::Running | RunState::Starting) {
            if let (Some(pid), pgid) = (project.run.pid, project.run.pgid) {
                if is_process_alive(pid, pgid) {
                    return Ok(project);
                }
            }
        }

        let command = resolve_start_command(&project)?;
        self.stack.ensure_databases(id)?;
        let export_prefix = self.stack.spawn_export_prefix(id)?;
        let command = if export_prefix.is_empty() {
            command
        } else {
            format!("{export_prefix}; {command}")
        };
        let log_path = self.logs.prepare_session(id)?;
        self.repository.upsert_run(&ProjectRun {
            project_id: id.to_string(),
            pid: None,
            pgid: None,
            started_at: Some(now_iso()),
            started_at_unix: Some(now_unix()),
            status: RunState::Starting,
            last_error: None,
        })?;

        match spawn_login_shell(&project.path, &command, Some(&log_path)) {
            Ok((pid, pgid)) => {
                info!(project_id = id, pid, pgid, command = %command, "started project");
                self.repository.upsert_run(&ProjectRun {
                    project_id: id.to_string(),
                    pid: Some(pid),
                    pgid: Some(pgid),
                    started_at: Some(now_iso()),
                    started_at_unix: Some(now_unix()),
                    status: RunState::Running,
                    last_error: None,
                })?;
            }
            Err(err) => {
                warn!(project_id = id, error = %err, "failed to start project");
                self.repository.upsert_run(&ProjectRun {
                    project_id: id.to_string(),
                    pid: None,
                    pgid: None,
                    started_at: None,
                    started_at_unix: None,
                    status: RunState::Failed,
                    last_error: Some(err.to_string()),
                })?;
                return Err(err);
            }
        }

        self.catalog.get_project(id)
    }

    pub fn stop_project(&self, id: &str) -> Result<Project, AppError> {
        let project = self.catalog.get_project(id)?;

        if let Some(stop_command) = project
            .stop_command
            .as_ref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
        {
            if let Err(err) = run_login_shell_wait(&project.path, stop_command) {
                warn!(project_id = id, error = %err, "custom stop command failed; signaling group");
            }
        }

        if let Some(pid) = project.run.pid {
            let pgid = project.run.pgid.unwrap_or(pid);
            signal_process_group(pgid, libc::SIGTERM);
            thread::sleep(Duration::from_millis(STOP_GRACE_MS));
            if is_process_alive(pid, Some(pgid)) {
                signal_process_group(pgid, libc::SIGKILL);
                thread::sleep(Duration::from_millis(200));
            }
        }

        self.repository.upsert_run(&ProjectRun {
            project_id: id.to_string(),
            pid: None,
            pgid: None,
            started_at: None,
            started_at_unix: None,
            status: RunState::Stopped,
            last_error: None,
        })?;

        self.catalog.get_project(id)
    }

    pub fn restart_project(&self, id: &str) -> Result<Project, AppError> {
        let _ = self.stop_project(id)?;
        self.start_project(id)
    }
}

fn resolve_start_command(project: &Project) -> Result<String, AppError> {
    if let Some(override_cmd) = project
        .start_command
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        return Ok(override_cmd.to_string());
    }
    if let Some(inferred) = project
        .inferred_start_command
        .clone()
        .or_else(|| infer_start_command(std::path::Path::new(&project.path)))
    {
        return Ok(inferred);
    }
    Err(AppError::invalid(
        "no start command configured or inferred for this project",
    ))
}

fn spawn_login_shell(
    cwd: &str,
    command: &str,
    log_path: Option<&Path>,
) -> Result<(i32, i32), AppError> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
    let mut child = Command::new(&shell);
    child.args(["-lc", command]).current_dir(cwd).stdin(Stdio::null());
    apply_log_stdio(&mut child, log_path)?;

    #[cfg(unix)]
    unsafe {
        child.pre_exec(|| {
            // Become leader of a new process group (pgid == pid).
            if libc::setpgid(0, 0) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }

    let child = child.spawn().map_err(|err| {
        AppError::Io(format!("failed to spawn `{command}` via {shell}: {err}"))
    })?;

    let pid = child.id() as i32;
    let pgid = pid;
    // Detach so Drop does not wait — process must outlive the app.
    std::mem::forget(child);
    Ok((pid, pgid))
}

fn apply_log_stdio(child: &mut Command, log_path: Option<&Path>) -> Result<(), AppError> {
    let Some(path) = log_path else {
        child.stdout(Stdio::null()).stderr(Stdio::null());
        return Ok(());
    };
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(path)
        .map_err(|err| AppError::Io(format!("failed to open log file: {err}")))?;
    let err_file = file
        .try_clone()
        .map_err(|err| AppError::Io(format!("failed to clone log file: {err}")))?;
    child.stdout(Stdio::from(file)).stderr(Stdio::from(err_file));
    Ok(())
}

fn run_login_shell_wait(cwd: &str, command: &str) -> Result<(), AppError> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
    let status = Command::new(&shell)
        .args(["-lc", command])
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|err| AppError::Io(format!("failed to run stop command: {err}")))?;

    if status.success() {
        Ok(())
    } else {
        Err(AppError::message(format!(
            "stop command exited with {status}"
        )))
    }
}

#[cfg(unix)]
fn signal_process_group(pgid: i32, signal: i32) {
    // Negative pid means process group.
    let result = unsafe { libc::kill(-pgid, signal) };
    if result != 0 {
        let err = std::io::Error::last_os_error();
        warn!(pgid, signal, error = %err, "signal process group failed");
    }
}

#[cfg(not(unix))]
fn signal_process_group(_pgid: i32, _signal: i32) {}

#[cfg(unix)]
fn is_process_alive(pid: i32, expected_pgid: Option<i32>) -> bool {
    let alive = unsafe { libc::kill(pid, 0) == 0 };
    if !alive {
        return false;
    }
    if let Some(expected) = expected_pgid {
        let actual = unsafe { libc::getpgid(pid) };
        if actual < 0 || actual != expected {
            return false;
        }
    }
    true
}

#[cfg(not(unix))]
fn is_process_alive(_pid: i32, _expected_pgid: Option<i32>) -> bool {
    false
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn now_iso() -> String {
    now_unix().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::Database;
    use crate::services::{CatalogService, LogService};
    use std::fs;

    fn wait_for_log_line(
        logs: &LogService,
        project_id: &str,
        needle: &str,
    ) -> crate::domain::LogChunk {
        for _ in 0..20 {
            thread::sleep(Duration::from_millis(100));
            let chunk = logs.read_tail(project_id, Some(50)).expect("log tail");
            if chunk.lines.iter().any(|line| line.contains(needle)) {
                return chunk;
            }
        }
        logs.read_tail(project_id, Some(50)).expect("log tail")
    }

    #[test]
    fn start_stop_and_rehydrate_sleep_process() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let db_dir = std::env::temp_dir().join(format!("zashiki-proc-{stamp}"));
        fs::create_dir_all(&db_dir).unwrap();
        let project_dir = db_dir.join("app");
        fs::create_dir_all(&project_dir).unwrap();
        fs::write(
            project_dir.join("package.json"),
            r#"{"scripts":{"dev":"sleep 30"}}"#,
        )
        .unwrap();

        let db = Arc::new(Database::open(&db_dir).unwrap());
        let repo = Arc::new(ProjectRepository::new(db));
        let catalog = Arc::new(CatalogService::new(repo.clone()));
        let logs = Arc::new(LogService::new(db_dir.join("logs")));
        let occupancy = Arc::new(crate::services::PortOccupancyService::new(repo.clone()));
        let compose = Arc::new(crate::services::ComposeService::new(
            db_dir.join("overrides"),
        ));
        let stack = Arc::new(crate::services::StackService::new(
            catalog.clone(),
            repo.clone(),
            occupancy,
            compose,
        ));
        let process = ProcessService::new(repo, catalog.clone(), logs.clone(), stack);

        let project = catalog
            .add_project(project_dir.to_str().unwrap())
            .expect("add");
        let project = catalog
            .update_commands(
                &project.id,
                Some(Some("/bin/echo hello-log; sleep 30".to_string())),
                None,
            )
            .expect("override start");
        let started = process.start_project(&project.id).expect("start");
        assert_eq!(started.run.status, RunState::Running);
        assert!(started.run.pid.is_some());
        let chunk = wait_for_log_line(&logs, &project.id, "hello-log");
        assert!(
            chunk.lines.iter().any(|line| line.contains("hello-log")),
            "expected captured stdout, got {:?}",
            chunk.lines
        );

        process.rehydrate_all().expect("rehydrate");
        let after = catalog.get_project(&project.id).unwrap();
        assert_eq!(after.run.status, RunState::Running);

        let stopped = process.stop_project(&project.id).expect("stop");
        assert_eq!(stopped.run.status, RunState::Stopped);
        assert!(stopped.run.pid.is_none());

        let _ = fs::remove_dir_all(db_dir);
    }
}
