//! `zashiki-warashi job …` subcommands. Started by launchd or by MCP clients;
//! never touches Tauri.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use log::{LevelFilter, Log, Metadata, Record};
use tracing::{error, warn};

use crate::error::AppError;
use crate::repositories::{Database, JobRepository};
use crate::services::job_paths::{launch_agents_dir, JobPaths};
use crate::services::job_runner::{JobRunner, RunMode};
use crate::services::job_service::JobService;
use crate::services::launchd_service::{LaunchdService, SystemLaunchctl};
use crate::services::system_probe::MacSystemProbe;
use crate::services::{job_mcp, job_socket, wake_helper};

const HOUSEKEEPING_EVERY: Duration = Duration::from_secs(60);

/// Handle `job …` argv. Returns `None` when the app should start the GUI.
pub fn run_cli(args: &[String]) -> Option<i32> {
    if args.get(1).map(String::as_str) != Some("job") {
        return None;
    }
    install_stderr_logger();
    let rest: Vec<&str> = args.iter().skip(2).map(String::as_str).collect();
    let result = match rest.as_slice() {
        ["run", id, flags @ ..] => run_job(id, flags.contains(&"--manual")),
        ["supervise"] => supervise(),
        ["wake-daemon", "--requests", path] => wake_helper::run_daemon(Path::new(path)),
        ["wake-daemon", "--cancel-all"] => wake_helper::cancel_all(&wake_helper::SystemPmset),
        ["mcp"] => mcp(),
        _ => Err(AppError::invalid(
            "usage: job run <id> [--manual] | job supervise | job mcp | job wake-daemon --requests <path>",
        )),
    };
    match result {
        Ok(()) => Some(0),
        Err(err) => {
            error!(error = %err, "job command failed");
            Some(1)
        }
    }
}

fn open_repository(paths: &JobPaths) -> Result<Arc<JobRepository>, AppError> {
    let database = Arc::new(Database::open(paths.data_dir())?);
    Ok(Arc::new(JobRepository::new(database)))
}

fn current_exe() -> Result<PathBuf, AppError> {
    Ok(std::env::current_exe()?)
}

fn build_service(paths: &JobPaths) -> Result<JobService, AppError> {
    let launchd = LaunchdService::new(launch_agents_dir()?, paths.clone(), Box::new(SystemLaunchctl));
    Ok(JobService::new(open_repository(paths)?, paths.clone(), launchd, current_exe()?))
}

fn run_job(id: &str, manual: bool) -> Result<(), AppError> {
    let paths = JobPaths::from_home()?;
    let repository = open_repository(&paths)?;
    let runner = JobRunner::new(repository, paths.clone(), Arc::new(MacSystemProbe));
    let mode = if manual { RunMode::Manual } else { RunMode::Scheduled };
    let outcome = runner.run(id, mode);
    if let Err(err) = build_service(&paths).and_then(|service| service.refresh_wakes()) {
        warn!(error = %err, "could not re-arm wakes after run");
    }
    outcome.map(|_| ())
}

fn supervise() -> Result<(), AppError> {
    let paths = JobPaths::from_home()?;
    let service = Arc::new(build_service(&paths)?);
    job_socket::serve(service.clone(), &paths.socket())?;
    loop {
        if let Err(err) = service.housekeeping() {
            warn!(error = %err, "job housekeeping failed");
        }
        thread::sleep(HOUSEKEEPING_EVERY);
    }
}

fn mcp() -> Result<(), AppError> {
    job_mcp::serve_stdio(&JobPaths::from_home()?.socket())
}

/// launchd captures stderr into the job's log file.
struct StderrLogger;

impl Log for StderrLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let stamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
            eprintln!("{stamp} {} {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

fn install_stderr_logger() {
    if log::set_boxed_logger(Box::new(StderrLogger)).is_ok() {
        log::set_max_level(LevelFilter::Info);
    }
}
