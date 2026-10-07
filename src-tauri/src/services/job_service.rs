use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::thread;

use chrono::Local;
use tracing::warn;

use crate::domain::{ForeignAgent, Job, JobInput, JobLogTail, JobRun, JobSummary, JobSystemStatus};
use crate::error::AppError;
use crate::repositories::JobRepository;
use crate::services::job_paths::JobPaths;
use crate::services::job_runner::reap_orphaned_runs;
use crate::services::job_schedule::{next_fire, validate};
use crate::services::launchd_service::LaunchdService;
use crate::services::system_probe::SystemProbe;
use crate::services::wake_helper::{helper_installed, install_helper, uninstall_helper};
use crate::services::wake_requests::{desired_wakes, write_requests};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

const MAX_GRACE_SECONDS: u32 = 600;
const DEFAULT_LOG_LINES: usize = 400;

/// Job catalog + lifecycle. Shared by Tauri commands, the socket API, and MCP.
pub struct JobService {
    repository: Arc<JobRepository>,
    paths: JobPaths,
    launchd: LaunchdService,
    exe: PathBuf,
}

impl JobService {
    pub fn new(repository: Arc<JobRepository>, paths: JobPaths, launchd: LaunchdService, exe: PathBuf) -> Self {
        Self { repository, paths, launchd, exe }
    }

    pub fn list(&self) -> Result<Vec<JobSummary>, AppError> {
        self.repository.list_jobs()?.into_iter().map(|job| self.summarize(job)).collect()
    }

    pub fn get(&self, id: &str) -> Result<JobSummary, AppError> {
        self.summarize(self.require(id)?)
    }

    pub fn create(&self, input: JobInput) -> Result<JobSummary, AppError> {
        let now = Local::now().timestamp();
        let job = build_job(uuid::Uuid::new_v4().to_string(), input, now, now)?;
        self.save(job)
    }

    pub fn update(&self, id: &str, input: JobInput) -> Result<JobSummary, AppError> {
        let existing = self.require(id)?;
        let job = build_job(existing.id, input, existing.created_at, Local::now().timestamp())?;
        self.save(job)
    }

    pub fn set_enabled(&self, id: &str, enabled: bool) -> Result<JobSummary, AppError> {
        let mut job = self.require(id)?;
        job.enabled = enabled;
        job.updated_at = Local::now().timestamp();
        self.save(job)
    }

    pub fn delete(&self, id: &str) -> Result<(), AppError> {
        self.require(id)?;
        self.launchd.remove_job(id)?;
        self.repository.delete_job(id)?;
        let dir = self.paths.job_dir(id);
        if dir.exists() {
            fs::remove_dir_all(dir)?;
        }
        self.refresh_wakes()
    }

    /// Start `job run <id> --manual` detached; it records its own run row.
    pub fn run_now(&self, id: &str) -> Result<(), AppError> {
        self.require(id)?;
        let mut command = Command::new(&self.exe);
        command
            .args(["job", "run", id, "--manual"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        #[cfg(unix)]
        unsafe {
            command.pre_exec(|| {
                libc::setsid();
                Ok(())
            });
        }
        let mut child = command
            .spawn()
            .map_err(|err| AppError::Io(format!("failed to spawn job runner: {err}")))?;
        thread::spawn(move || child.wait());
        Ok(())
    }

    pub fn list_runs(&self, id: &str, limit: Option<u32>) -> Result<Vec<JobRun>, AppError> {
        self.repository.list_runs(id, limit.unwrap_or(50).min(500))
    }

    pub fn log_tail(&self, run_id: &str, lines: Option<usize>) -> Result<JobLogTail, AppError> {
        let run = self
            .repository
            .get_run(run_id)?
            .ok_or_else(|| AppError::not_found(format!("run {run_id} not found")))?;
        let lines = read_tail(Path::new(&run.log_path), lines.unwrap_or(DEFAULT_LOG_LINES))?;
        Ok(JobLogTail { run_id: run.id, lines })
    }

    pub fn foreign_agents(&self) -> Result<Vec<ForeignAgent>, AppError> {
        self.launchd.list_foreign_agents()
    }

    /// Spawns pmset / route; call off the UI thread.
    pub fn system_status(&self, probe: &dyn SystemProbe) -> JobSystemStatus {
        JobSystemStatus {
            supervisor_installed: self.launchd.supervisor_installed(),
            wake_helper_installed: helper_installed(),
            power_source: probe.power_source(),
            online: probe.is_online(),
            socket_path: self.paths.socket().display().to_string(),
            mcp_command: format!("\"{}\" job mcp", self.exe.display()),
        }
    }

    pub fn install_wake_helper(&self) -> Result<(), AppError> {
        install_helper(&self.exe, &self.paths.wake_requests())?;
        self.refresh_wakes()
    }

    pub fn uninstall_wake_helper(&self) -> Result<(), AppError> {
        uninstall_helper()
    }

    /// Install the supervisor, rewrite job agents for this binary, reap, re-arm.
    pub fn reconcile_system(&self) -> Result<(), AppError> {
        self.launchd.install_supervisor(&self.exe)?;
        self.launchd.sync_all(&self.repository.list_jobs()?, &self.exe)?;
        self.housekeeping()
    }

    /// Cheap periodic work: mark dead runs, refresh wake requests.
    pub fn housekeeping(&self) -> Result<(), AppError> {
        reap_orphaned_runs(&self.repository)?;
        self.refresh_wakes()
    }

    pub fn refresh_wakes(&self) -> Result<(), AppError> {
        let wakes = desired_wakes(&self.repository.list_jobs()?, Local::now().timestamp());
        fs::create_dir_all(self.paths.data_dir())?;
        write_requests(&self.paths.wake_requests(), &wakes)
    }

    fn save(&self, job: Job) -> Result<JobSummary, AppError> {
        self.repository.upsert_job(&job)?;
        if let Err(err) = self.launchd.sync_job(&job, &self.exe) {
            warn!(job_id = %job.id, error = %err, "launch agent sync failed");
            return Err(err);
        }
        self.refresh_wakes()?;
        self.summarize(job)
    }

    fn require(&self, id: &str) -> Result<Job, AppError> {
        self.repository
            .get_job(id)?
            .ok_or_else(|| AppError::not_found(format!("job {id} not found")))
    }

    fn summarize(&self, job: Job) -> Result<JobSummary, AppError> {
        let next_fire_unix = job
            .enabled
            .then(|| next_fire(&job.schedule, &Local::now()).map(|at| at.timestamp()))
            .flatten();
        let last_run = self.repository.list_runs(&job.id, 1)?.into_iter().next();
        Ok(JobSummary { job, next_fire_unix, last_run })
    }
}

fn build_job(id: String, input: JobInput, created_at: i64, updated_at: i64) -> Result<Job, AppError> {
    let input = normalize_input(input)?;
    Ok(Job {
        id,
        name: input.name,
        command: input.command,
        working_dir: input.working_dir,
        enabled: input.enabled,
        schedule: input.schedule,
        policy: input.policy,
        network: input.network,
        network_grace_seconds: input.network_grace_seconds,
        max_runtime_seconds: input.max_runtime_seconds,
        created_at,
        updated_at,
    })
}

pub fn normalize_input(mut input: JobInput) -> Result<JobInput, AppError> {
    input.name = input.name.trim().to_string();
    input.command = input.command.trim().to_string();
    input.working_dir = input.working_dir.map(|dir| dir.trim().to_string()).filter(|dir| !dir.is_empty());
    if input.name.is_empty() {
        return Err(AppError::invalid("job name is required"));
    }
    if input.command.is_empty() {
        return Err(AppError::invalid("job command is required"));
    }
    if let Some(dir) = &input.working_dir {
        if !Path::new(dir).is_dir() {
            return Err(AppError::invalid(format!("working directory does not exist: {dir}")));
        }
    }
    if input.network_grace_seconds > MAX_GRACE_SECONDS {
        return Err(AppError::invalid("network wait must be 10 minutes or less"));
    }
    input.max_runtime_seconds = input.max_runtime_seconds.filter(|max| *max > 0);
    validate(&input.schedule)?;
    Ok(input)
}

fn read_tail(path: &Path, lines: usize) -> Result<Vec<String>, AppError> {
    let Ok(file) = fs::File::open(path) else {
        return Ok(Vec::new());
    };
    let all: Vec<String> = BufReader::new(file).lines().map_while(Result::ok).collect();
    let skip = all.len().saturating_sub(lines);
    Ok(all.into_iter().skip(skip).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{JobPolicy, JobSchedule, NetworkPolicy};
    use crate::repositories::Database;
    use crate::services::launchd_service::LaunchctlRunner;
    use crate::services::wake_requests::read_requests;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct NoopLaunchctl;

    impl LaunchctlRunner for NoopLaunchctl {
        fn bootstrap(&self, _plist: &Path) -> Result<(), AppError> {
            Ok(())
        }
        fn bootout(&self, _label: &str) {}
        fn list(&self) -> Result<String, AppError> {
            Ok(String::new())
        }
    }

    fn service() -> (JobService, PathBuf) {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).expect("time").as_nanos();
        let dir = std::env::temp_dir().join(format!("zashiki-jobsvc-{stamp}"));
        let paths = JobPaths::new(dir.join("data"));
        let repo = Arc::new(JobRepository::new(Arc::new(Database::open(paths.data_dir()).expect("db"))));
        let launchd = LaunchdService::new(dir.join("agents"), paths.clone(), Box::new(NoopLaunchctl));
        (JobService::new(repo, paths, launchd, PathBuf::from("/bin/z")), dir)
    }

    fn input(policy: JobPolicy) -> JobInput {
        JobInput {
            name: "  Ingest ".into(),
            command: "echo hi".into(),
            working_dir: Some("".into()),
            enabled: true,
            schedule: JobSchedule::Interval { minutes: 30 },
            policy,
            network: NetworkPolicy::Required,
            network_grace_seconds: 90,
            max_runtime_seconds: Some(0),
        }
    }

    #[test]
    fn create_writes_agent_and_wake_request() {
        let (svc, dir) = service();
        let created = svc.create(input(JobPolicy::Exact)).expect("create");
        assert_eq!(created.job.name, "Ingest");
        assert_eq!(created.job.working_dir, None);
        assert_eq!(created.job.max_runtime_seconds, None);
        assert!(created.next_fire_unix.is_some());
        let plist = dir.join(format!("agents/com.zashiki.warashi.job.{}.plist", created.job.id));
        assert!(plist.exists());
        let wakes = read_requests(&dir.join("data/wake-requests.json"), Local::now().timestamp());
        assert_eq!(wakes.len(), 1);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn disabling_removes_agent_and_wake() {
        let (svc, dir) = service();
        let created = svc.create(input(JobPolicy::Exact)).expect("create");
        let disabled = svc.set_enabled(&created.job.id, false).expect("disable");
        assert!(disabled.next_fire_unix.is_none());
        let plist = dir.join(format!("agents/com.zashiki.warashi.job.{}.plist", created.job.id));
        assert!(!plist.exists());
        assert!(read_requests(&dir.join("data/wake-requests.json"), 0).is_empty());
        svc.delete(&created.job.id).expect("delete");
        assert!(svc.list().expect("list").is_empty());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn rejects_invalid_input() {
        let (svc, dir) = service();
        let mut bad = input(JobPolicy::Optimistic);
        bad.command = "  ".into();
        assert!(svc.create(bad).is_err());
        let mut bad_schedule = input(JobPolicy::Optimistic);
        bad_schedule.schedule = JobSchedule::Interval { minutes: 7 };
        assert!(svc.create(bad_schedule).is_err());
        let _ = fs::remove_dir_all(dir);
    }
}
