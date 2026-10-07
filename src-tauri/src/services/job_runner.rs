use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use chrono::Local;
use tracing::{info, warn};

use crate::domain::{
    Job, JobPolicy, JobRun, JobRunStatus, NetworkOutcome, NetworkPolicy, RunTrigger,
};
use crate::error::AppError;
use crate::repositories::JobRepository;
use crate::services::job_paths::JobPaths;
use crate::services::job_schedule::classify_scheduled;
use crate::services::login_shell::{group_command, is_pid_alive, signal_group};
use crate::services::system_probe::{wait_for_network, SystemProbe};

const WAIT_POLL: Duration = Duration::from_millis(500);
const STOP_GRACE: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunMode {
    Scheduled,
    Manual,
}

/// Executes one invocation of a job. Lives in the `job run` subprocess.
pub struct JobRunner {
    repository: Arc<JobRepository>,
    paths: JobPaths,
    probe: Arc<dyn SystemProbe>,
}

struct Execution {
    status: JobRunStatus,
    exit_code: Option<i32>,
    message: Option<String>,
}

impl JobRunner {
    pub fn new(repository: Arc<JobRepository>, paths: JobPaths, probe: Arc<dyn SystemProbe>) -> Self {
        Self { repository, paths, probe }
    }

    pub fn run(&self, job_id: &str, mode: RunMode) -> Result<Option<JobRun>, AppError> {
        let job = self
            .repository
            .get_job(job_id)?
            .ok_or_else(|| AppError::not_found(format!("job {job_id} not found")))?;
        if mode == RunMode::Scheduled && !job.enabled {
            info!(job_id, "skipping disabled job");
            return Ok(None);
        }
        let mut run = self.begin_run(&job, mode)?;
        let mut log = open_log(&run.log_path)?;
        let _hold = self.probe.hold_awake(job.policy == JobPolicy::Exact);

        let execution = self.execute(&job, &mut run, &mut log);
        finish_run(&mut run, execution, &mut log);
        self.repository.update_run(&run)?;
        Ok(Some(run))
    }

    fn begin_run(&self, job: &Job, mode: RunMode) -> Result<JobRun, AppError> {
        let (trigger, scheduled_for) = match mode {
            RunMode::Manual => (RunTrigger::Manual, None),
            RunMode::Scheduled => classify_scheduled(&job.schedule, &Local::now()),
        };
        let run_id = uuid::Uuid::new_v4().to_string();
        let log_path = self.paths.run_log(&job.id, &run_id);
        if let Some(parent) = log_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let run = JobRun {
            id: run_id,
            job_id: job.id.clone(),
            trigger,
            status: JobRunStatus::Running,
            scheduled_for_unix: scheduled_for,
            started_at_unix: Local::now().timestamp(),
            finished_at_unix: None,
            exit_code: None,
            network: None,
            power_source: Some(self.probe.power_source()),
            runner_pid: Some(std::process::id() as i32),
            log_path: log_path.display().to_string(),
            message: None,
        };
        self.repository.insert_run(&run)?;
        Ok(run)
    }

    fn execute(&self, job: &Job, run: &mut JobRun, log: &mut File) -> Execution {
        let power = run.power_source.map_or("unknown", |source| source.as_str());
        log_line(log, &format!("trigger {} · power {power}", run.trigger.as_str()));
        let outcome = self.network_gate(job, log);
        run.network = Some(outcome);
        self.repository.update_run(run).unwrap_or_else(|err| warn!(error = %err, "run update failed"));
        if gate_skips(job.network, outcome) {
            return Execution {
                status: JobRunStatus::SkippedOffline,
                exit_code: None,
                message: Some("No network within the grace period; skipped.".into()),
            };
        }
        match spawn_job(job, run, outcome, log) {
            Ok(child) => wait_with_deadline(child, job.max_runtime_seconds, log),
            Err(err) => Execution {
                status: JobRunStatus::Failed,
                exit_code: None,
                message: Some(err.user_message()),
            },
        }
    }

    fn network_gate(&self, job: &Job, log: &mut File) -> NetworkOutcome {
        if job.network == NetworkPolicy::None {
            return NetworkOutcome::NotChecked;
        }
        log_line(log, &format!("waiting up to {}s for network", job.network_grace_seconds));
        let grace = Duration::from_secs(job.network_grace_seconds as u64);
        let outcome = wait_for_network(self.probe.as_ref(), grace);
        log_line(log, &format!("network {}", outcome.as_str()));
        outcome
    }
}

/// Mark `running` rows whose runner process is gone as interrupted.
pub fn reap_orphaned_runs(repository: &JobRepository) -> Result<usize, AppError> {
    let mut reaped = 0;
    for mut run in repository.list_running_runs()? {
        if run.runner_pid.is_some_and(is_pid_alive) {
            continue;
        }
        run.status = JobRunStatus::Interrupted;
        run.finished_at_unix = Some(Local::now().timestamp());
        run.message = Some("The runner stopped before the job finished (sleep, logout, or crash).".into());
        repository.update_run(&run)?;
        reaped += 1;
    }
    Ok(reaped)
}

pub fn gate_skips(policy: NetworkPolicy, outcome: NetworkOutcome) -> bool {
    policy == NetworkPolicy::Required && outcome == NetworkOutcome::Offline
}

pub fn deadline_reached(started: Instant, max_runtime_seconds: Option<u32>) -> bool {
    max_runtime_seconds.is_some_and(|max| started.elapsed() >= Duration::from_secs(max as u64))
}

pub fn status_for_exit(status: ExitStatus) -> (JobRunStatus, Option<i32>) {
    match status.code() {
        Some(0) => (JobRunStatus::Succeeded, Some(0)),
        code => (JobRunStatus::Failed, code),
    }
}

fn spawn_job(job: &Job, run: &JobRun, network: NetworkOutcome, log: &File) -> Result<Child, AppError> {
    let cwd = job
        .working_dir
        .clone()
        .or_else(|| std::env::var("HOME").ok())
        .unwrap_or_else(|| "/".into());
    let mut command: Command = group_command(&cwd, &job.command);
    command
        .env("ZASHIKI_JOB_ID", &job.id)
        .env("ZASHIKI_RUN_ID", &run.id)
        .env("ZASHIKI_TRIGGER", run.trigger.as_str())
        .env("ZASHIKI_NETWORK", network.as_str())
        .stdout(Stdio::from(log.try_clone()?))
        .stderr(Stdio::from(log.try_clone()?));
    command
        .spawn()
        .map_err(|err| AppError::Io(format!("failed to spawn `{}`: {err}", job.command)))
}

fn wait_with_deadline(mut child: Child, max_runtime: Option<u32>, log: &mut File) -> Execution {
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let (status, exit_code) = status_for_exit(status);
                return Execution { status, exit_code, message: None };
            }
            Ok(None) if deadline_reached(started, max_runtime) => {
                log_line(log, "max runtime reached; stopping");
                stop_child(&mut child);
                return Execution {
                    status: JobRunStatus::TimedOut,
                    exit_code: None,
                    message: Some("Stopped after reaching the max runtime.".into()),
                };
            }
            Ok(None) => thread::sleep(WAIT_POLL),
            Err(err) => {
                return Execution {
                    status: JobRunStatus::Failed,
                    exit_code: None,
                    message: Some(format!("Lost track of the job process: {err}")),
                }
            }
        }
    }
}

fn stop_child(child: &mut Child) {
    let pgid = child.id() as i32;
    signal_group(pgid, libc::SIGTERM);
    let grace_end = Instant::now() + STOP_GRACE;
    while Instant::now() < grace_end {
        if matches!(child.try_wait(), Ok(Some(_))) {
            return;
        }
        thread::sleep(Duration::from_millis(100));
    }
    signal_group(pgid, libc::SIGKILL);
    let _ = child.wait();
}

fn finish_run(run: &mut JobRun, execution: Execution, log: &mut File) {
    run.status = execution.status;
    run.exit_code = execution.exit_code;
    run.message = execution.message;
    run.finished_at_unix = Some(Local::now().timestamp());
    let exit = run.exit_code.map_or("none".to_string(), |code| code.to_string());
    log_line(log, &format!("finished {} (exit {exit})", run.status.as_str()));
}

fn open_log(path: &str) -> Result<File, AppError> {
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|err| AppError::Io(format!("failed to open run log: {err}")))
}

fn log_line(log: &mut File, text: &str) {
    let stamp = Local::now().format("%H:%M:%S");
    let _ = writeln!(log, "[zashiki {stamp}] {text}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{JobSchedule, PowerSource};
    use crate::repositories::Database;
    use crate::services::system_probe::AwakeHold;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    struct FakeProbe {
        online: AtomicBool,
    }

    impl SystemProbe for FakeProbe {
        fn power_source(&self) -> PowerSource {
            PowerSource::Ac
        }
        fn is_online(&self) -> bool {
            self.online.load(Ordering::SeqCst)
        }
        fn ensure_wifi_on(&self) {}
        fn hold_awake(&self, _prevent_system_sleep: bool) -> AwakeHold {
            AwakeHold::none()
        }
    }

    fn setup(online: bool) -> (JobRunner, Arc<JobRepository>, std::path::PathBuf) {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).expect("time").as_nanos();
        let dir = std::env::temp_dir().join(format!("zashiki-runner-{stamp}"));
        let repo = Arc::new(JobRepository::new(Arc::new(Database::open(&dir).expect("db"))));
        let probe = Arc::new(FakeProbe { online: AtomicBool::new(online) });
        (JobRunner::new(repo.clone(), JobPaths::new(&dir), probe), repo, dir)
    }

    fn job(command: &str, network: NetworkPolicy, max_runtime: Option<u32>) -> Job {
        Job {
            id: "j".into(),
            name: "test".into(),
            command: command.into(),
            working_dir: None,
            enabled: true,
            schedule: JobSchedule::Daily { hour: 3, minute: 0 },
            policy: JobPolicy::Optimistic,
            network,
            network_grace_seconds: 0,
            max_runtime_seconds: max_runtime,
            created_at: 0,
            updated_at: 0,
        }
    }

    #[test]
    fn gate_only_skips_required_when_offline() {
        assert!(gate_skips(NetworkPolicy::Required, NetworkOutcome::Offline));
        assert!(!gate_skips(NetworkPolicy::BestEffort, NetworkOutcome::Offline));
        assert!(!gate_skips(NetworkPolicy::Required, NetworkOutcome::Online));
    }

    #[test]
    fn deadline_needs_a_budget() {
        let long_ago = Instant::now() - Duration::from_secs(10);
        assert!(deadline_reached(long_ago, Some(5)));
        assert!(!deadline_reached(long_ago, Some(60)));
        assert!(!deadline_reached(long_ago, None));
    }

    #[test]
    fn manual_run_captures_output_and_succeeds() {
        let (runner, repo, dir) = setup(true);
        repo.upsert_job(&job("echo hello-job; echo $ZASHIKI_TRIGGER", NetworkPolicy::None, None)).expect("job");
        let run = runner.run("j", RunMode::Manual).expect("run").expect("ran");
        assert_eq!(run.status, JobRunStatus::Succeeded);
        let log = fs::read_to_string(&run.log_path).expect("log");
        assert!(log.contains("hello-job"));
        assert!(log.contains("manual"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn required_network_skips_when_offline() {
        let (runner, repo, dir) = setup(false);
        repo.upsert_job(&job("echo nope", NetworkPolicy::Required, None)).expect("job");
        let run = runner.run("j", RunMode::Manual).expect("run").expect("ran");
        assert_eq!(run.status, JobRunStatus::SkippedOffline);
        assert_eq!(run.network, Some(NetworkOutcome::Offline));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn max_runtime_stops_the_process_group() {
        let (runner, repo, dir) = setup(true);
        repo.upsert_job(&job("sleep 30", NetworkPolicy::None, Some(1))).expect("job");
        let run = runner.run("j", RunMode::Manual).expect("run").expect("ran");
        assert_eq!(run.status, JobRunStatus::TimedOut);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn reaps_runs_whose_runner_died() {
        let (runner, repo, dir) = setup(true);
        repo.upsert_job(&job("true", NetworkPolicy::None, None)).expect("job");
        let mut run = runner.run("j", RunMode::Manual).expect("run").expect("ran");
        run.status = JobRunStatus::Running;
        run.runner_pid = Some(i32::MAX - 1);
        repo.update_run(&run).expect("update");
        assert_eq!(reap_orphaned_runs(&repo).expect("reap"), 1);
        let _ = fs::remove_dir_all(dir);
    }
}
