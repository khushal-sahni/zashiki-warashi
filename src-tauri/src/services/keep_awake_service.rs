use std::process::{Command, Stdio};
use std::sync::{Arc, OnceLock};

use tracing::{info, warn};

use crate::domain::KeepAwakeStatus;
use crate::error::AppError;
use crate::repositories::ProjectRepository;

const CAFFEINATE_PATH: &str = "/usr/bin/caffeinate";
const OSASCRIPT_PATH: &str = "/usr/bin/osascript";
const PMSET_CANDIDATES: &[&str] = &["/usr/bin/pmset", "/usr/sbin/pmset"];

pub trait KeepAwakeRunner: Send + Sync {
    fn spawn_caffeinate(&self) -> Result<i32, AppError>;
    fn kill_process(&self, pid: i32) -> Result<(), AppError>;
    fn is_process_alive(&self, pid: i32) -> bool;
    fn set_disable_sleep(&self, disabled: bool) -> Result<(), AppError>;
    fn is_sleep_disabled(&self) -> Result<bool, AppError>;
}

pub struct SystemKeepAwakeRunner;

impl KeepAwakeRunner for SystemKeepAwakeRunner {
    fn spawn_caffeinate(&self) -> Result<i32, AppError> {
        spawn_caffeinate_process()
    }

    fn kill_process(&self, pid: i32) -> Result<(), AppError> {
        kill_process(pid)
    }

    fn is_process_alive(&self, pid: i32) -> bool {
        is_process_alive(pid)
    }

    fn set_disable_sleep(&self, disabled: bool) -> Result<(), AppError> {
        set_disable_sleep(disabled)
    }

    fn is_sleep_disabled(&self) -> Result<bool, AppError> {
        is_sleep_disabled()
    }
}

pub struct KeepAwakeService {
    repository: Arc<ProjectRepository>,
    runner: Arc<dyn KeepAwakeRunner>,
}

impl KeepAwakeService {
    pub fn new(repository: Arc<ProjectRepository>) -> Self {
        Self {
            repository,
            runner: Arc::new(SystemKeepAwakeRunner),
        }
    }

    #[cfg(test)]
    pub fn with_runner(
        repository: Arc<ProjectRepository>,
        runner: Arc<dyn KeepAwakeRunner>,
    ) -> Self {
        Self { repository, runner }
    }

    pub fn status(&self) -> Result<KeepAwakeStatus, AppError> {
        let enabled = self.repository.get_keep_awake_enabled()?;
        if !enabled {
            return Ok(KeepAwakeStatus::off());
        }

        let pid = self.resolve_caffeinate_pid()?;
        let lid_closed_armed = self.runner.is_sleep_disabled()?;

        Ok(KeepAwakeStatus {
            enabled: true,
            lid_closed_armed,
            caffeinate_pid: pid,
        })
    }

    pub fn set_enabled(&self, enabled: bool) -> Result<KeepAwakeStatus, AppError> {
        if enabled {
            self.enable()
        } else {
            self.disable()?;
            Ok(KeepAwakeStatus::off())
        }
    }

    pub fn rehydrate(&self) -> Result<(), AppError> {
        let enabled = self.repository.get_keep_awake_enabled()?;
        if !enabled {
            return Ok(());
        }

        let pid = self.ensure_caffeinate_running()?;
        info!(pid = ?pid, "rehydrated keep-awake caffeinate");

        if !self.runner.is_sleep_disabled()? {
            warn!("keep-awake enabled but lid-close sleep is not disabled");
        }

        Ok(())
    }

    fn enable(&self) -> Result<KeepAwakeStatus, AppError> {
        let pid = self.ensure_caffeinate_running()?;
        self.repository.set_keep_awake_enabled(true)?;

        match self.runner.set_disable_sleep(true) {
            Ok(()) => Ok(KeepAwakeStatus {
                enabled: true,
                lid_closed_armed: true,
                caffeinate_pid: pid,
            }),
            Err(err) => Err(AppError::message(format!(
                "{err} Idle sleep is blocked, but closing the lid will still sleep the Mac until lid-close sleep is enabled."
            ))),
        }
    }

    fn disable(&self) -> Result<(), AppError> {
        if let Some(pid) = self.repository.get_keep_awake_caffeinate_pid()? {
            if self.runner.is_process_alive(pid) {
                if let Err(err) = self.runner.kill_process(pid) {
                    warn!(pid, error = %err, "failed to stop caffeinate");
                }
            }
        }

        if self.runner.is_sleep_disabled()? {
            self.runner.set_disable_sleep(false)?;
        }

        self.repository.set_keep_awake_caffeinate_pid(None)?;
        self.repository.set_keep_awake_enabled(false)?;
        Ok(())
    }

    fn ensure_caffeinate_running(&self) -> Result<Option<i32>, AppError> {
        if let Some(pid) = self.resolve_caffeinate_pid()? {
            return Ok(Some(pid));
        }

        let pid = self.runner.spawn_caffeinate()?;
        self.repository.set_keep_awake_caffeinate_pid(Some(pid))?;
        info!(pid, "started keep-awake caffeinate");
        Ok(Some(pid))
    }

    fn resolve_caffeinate_pid(&self) -> Result<Option<i32>, AppError> {
        let Some(pid) = self.repository.get_keep_awake_caffeinate_pid()? else {
            return Ok(None);
        };

        if self.runner.is_process_alive(pid) {
            return Ok(Some(pid));
        }

        self.repository.set_keep_awake_caffeinate_pid(None)?;
        Ok(None)
    }
}

fn spawn_caffeinate_process() -> Result<i32, AppError> {
    let child = Command::new(CAFFEINATE_PATH)
        .args(["-ims"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|err| AppError::Io(format!("failed to spawn caffeinate: {err}")))?;

    let pid = child.id() as i32;
    std::mem::forget(child);
    Ok(pid)
}

fn pmset_path() -> Result<&'static str, AppError> {
    static PATH: OnceLock<Result<&'static str, String>> = OnceLock::new();
    match PATH.get_or_init(resolve_pmset_path) {
        Ok(path) => Ok(*path),
        Err(err) => Err(AppError::Io(err.clone())),
    }
}

fn resolve_pmset_path() -> Result<&'static str, String> {
    for candidate in PMSET_CANDIDATES {
        if std::path::Path::new(candidate).is_file() {
            return Ok(*candidate);
        }
    }
    Err("pmset not found at /usr/bin/pmset or /usr/sbin/pmset".to_string())
}

fn set_disable_sleep(disabled: bool) -> Result<(), AppError> {
    let pmset = pmset_path()?;
    let value = if disabled { "1" } else { "0" };
    let shell_cmd = format!("cd / && {pmset} -a disablesleep {value}");
    let script = format!(
        "do shell script \"{shell_cmd}\" with administrator privileges"
    );

    let output = Command::new(OSASCRIPT_PATH)
        .current_dir("/")
        .args(["-e", &script])
        .output()
        .map_err(|err| AppError::Io(format!("failed to run osascript: {err}")))?;

    if output.status.success() || sleep_disabled_matches(disabled)? {
        return Ok(());
    }

    Err(AppError::message(format_pmset_auth_error(
        disabled,
        &String::from_utf8_lossy(&output.stderr),
    )))
}

fn sleep_disabled_matches(expected_disabled: bool) -> Result<bool, AppError> {
    Ok(is_sleep_disabled()? == expected_disabled)
}

fn format_pmset_auth_error(disabled: bool, detail: &str) -> String {
    let trimmed = detail.trim();
    if is_user_cancelled(trimmed) {
        return if disabled {
            "Administrator approval was cancelled. Idle sleep is still blocked, but closing the lid will sleep the Mac until you approve the prompt.".to_string()
        } else {
            "Administrator approval was cancelled. Coffee idle sleep is off, but lid-close sleep may still be disabled until you approve turning it back on.".to_string()
        };
    }

    if trimmed.is_empty() {
        return "Administrator approval is required to change lid-close sleep. Use Touch ID or your password at the macOS prompt.".to_string();
    }

    format!(
        "Could not change lid-close sleep settings: {trimmed}. If macOS offered Touch ID, you can use that instead of typing your password (System Settings → Touch ID & Password)."
    )
}

fn is_user_cancelled(detail: &str) -> bool {
    detail.contains("User canceled")
        || detail.contains("User cancelled")
        || detail.contains("(-128)")
}

fn is_sleep_disabled() -> Result<bool, AppError> {
    let pmset = pmset_path()?;
    let output = Command::new(pmset)
        .current_dir("/")
        .args(["-g"])
        .output()
        .map_err(|err| AppError::Io(format!("failed to run pmset: {err}")))?;

    if !output.status.success() {
        return Err(AppError::Io(format!(
            "pmset exited with {}",
            output.status
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_sleep_disabled(&stdout))
}

fn parse_sleep_disabled(pmset_output: &str) -> bool {
    pmset_output.lines().any(|line| {
        let trimmed = line.trim();
        trimmed.starts_with("SleepDisabled") && trimmed.split_whitespace().any(|part| part == "1")
    })
}

#[cfg(unix)]
fn kill_process(pid: i32) -> Result<(), AppError> {
    let term = unsafe { libc::kill(pid, libc::SIGTERM) };
    if term != 0 {
        let err = std::io::Error::last_os_error();
        return Err(AppError::Io(format!("failed to SIGTERM pid {pid}: {err}")));
    }
    Ok(())
}

#[cfg(not(unix))]
fn kill_process(_pid: i32) -> Result<(), AppError> {
    Err(AppError::message("keep-awake is only supported on macOS"))
}

#[cfg(unix)]
fn is_process_alive(pid: i32) -> bool {
    unsafe { libc::kill(pid, 0) == 0 }
}

#[cfg(not(unix))]
fn is_process_alive(_pid: i32) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::Database;
    use std::fs;
    use std::sync::Mutex;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct FakeRunner {
        state: Mutex<FakeRunnerState>,
    }

    struct FakeRunnerState {
        caffeinate_pid: i32,
        alive_pids: Vec<i32>,
        sleep_disabled: bool,
        spawn_count: u32,
        kill_count: u32,
    }

    impl FakeRunner {
        fn new() -> Self {
            Self {
                state: Mutex::new(FakeRunnerState {
                    caffeinate_pid: 42_001,
                    alive_pids: Vec::new(),
                    sleep_disabled: false,
                    spawn_count: 0,
                    kill_count: 0,
                }),
            }
        }
    }

    impl KeepAwakeRunner for FakeRunner {
        fn spawn_caffeinate(&self) -> Result<i32, AppError> {
            let mut state = self.state.lock().expect("lock");
            state.spawn_count += 1;
            state.caffeinate_pid += 1;
            let pid = state.caffeinate_pid;
            state.alive_pids.push(pid);
            Ok(pid)
        }

        fn kill_process(&self, pid: i32) -> Result<(), AppError> {
            let mut state = self.state.lock().expect("lock");
            state.kill_count += 1;
            state.alive_pids.retain(|value| *value != pid);
            Ok(())
        }

        fn is_process_alive(&self, pid: i32) -> bool {
            let state = self.state.lock().expect("lock");
            state.alive_pids.contains(&pid)
        }

        fn set_disable_sleep(&self, disabled: bool) -> Result<(), AppError> {
            let mut state = self.state.lock().expect("lock");
            state.sleep_disabled = disabled;
            Ok(())
        }

        fn is_sleep_disabled(&self) -> Result<bool, AppError> {
            let state = self.state.lock().expect("lock");
            Ok(state.sleep_disabled)
        }
    }

    fn temp_repo() -> (Arc<ProjectRepository>, std::path::PathBuf) {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let db_dir = std::env::temp_dir().join(format!("zashiki-coffee-{stamp}"));
        fs::create_dir_all(&db_dir).unwrap();
        let db = Arc::new(Database::open(&db_dir).unwrap());
        (Arc::new(ProjectRepository::new(db)), db_dir)
    }

    #[test]
    fn enable_spawns_caffeinate_and_persists() {
        let (repo, db_dir) = temp_repo();
        let runner = Arc::new(FakeRunner::new());
        let service = KeepAwakeService::with_runner(repo.clone(), runner.clone());

        let status = service.set_enabled(true).expect("enable");
        assert!(status.enabled);
        assert!(status.lid_closed_armed);
        assert_eq!(status.caffeinate_pid, Some(42_002));
        assert!(repo.get_keep_awake_enabled().unwrap());
        assert_eq!(repo.get_keep_awake_caffeinate_pid().unwrap(), Some(42_002));

        let _ = fs::remove_dir_all(db_dir);
    }

    #[test]
    fn disable_kills_and_clears_preference() {
        let (repo, db_dir) = temp_repo();
        let runner = Arc::new(FakeRunner::new());
        let service = KeepAwakeService::with_runner(repo.clone(), runner.clone());

        service.set_enabled(true).expect("enable");
        service.set_enabled(false).expect("disable");

        assert!(!repo.get_keep_awake_enabled().unwrap());
        assert_eq!(repo.get_keep_awake_caffeinate_pid().unwrap(), None);

        let state = runner.state.lock().expect("lock");
        assert_eq!(state.kill_count, 1);
        assert!(!state.sleep_disabled);

        let _ = fs::remove_dir_all(db_dir);
    }

    #[test]
    fn rehydrate_respawns_when_enabled_and_pid_dead() {
        let (repo, db_dir) = temp_repo();
        let runner = Arc::new(FakeRunner::new());
        let service = KeepAwakeService::with_runner(repo.clone(), runner.clone());

        service.set_enabled(true).expect("enable");
        {
            let mut state = runner.state.lock().expect("lock");
            state.alive_pids.clear();
        }

        service.rehydrate().expect("rehydrate");

        let status = service.status().expect("status");
        assert!(status.enabled);
        assert_eq!(status.caffeinate_pid, Some(42_003));

        let state = runner.state.lock().expect("lock");
        assert_eq!(state.spawn_count, 2);

        let _ = fs::remove_dir_all(db_dir);
    }

    #[test]
    fn enable_without_admin_keeps_caffeinate_and_reports_partial() {
        struct DenyPmsetRunner(FakeRunner);

        impl KeepAwakeRunner for DenyPmsetRunner {
            fn spawn_caffeinate(&self) -> Result<i32, AppError> {
                self.0.spawn_caffeinate()
            }

            fn kill_process(&self, pid: i32) -> Result<(), AppError> {
                self.0.kill_process(pid)
            }

            fn is_process_alive(&self, pid: i32) -> bool {
                self.0.is_process_alive(pid)
            }

            fn set_disable_sleep(&self, _disabled: bool) -> Result<(), AppError> {
                Err(AppError::message("user cancelled"))
            }

            fn is_sleep_disabled(&self) -> Result<bool, AppError> {
                self.0.is_sleep_disabled()
            }
        }

        let (repo, db_dir) = temp_repo();
        let runner = Arc::new(DenyPmsetRunner(FakeRunner::new()));
        let service = KeepAwakeService::with_runner(repo.clone(), runner);

        let err = service.set_enabled(true).expect_err("partial enable");
        assert!(err.to_string().contains("lid will still sleep"));

        let status = service.status().expect("status");
        assert!(status.enabled);
        assert!(!status.lid_closed_armed);
        assert!(status.caffeinate_pid.is_some());

        let _ = fs::remove_dir_all(db_dir);
    }

    #[test]
    fn parse_sleep_disabled_reads_pmset_output() {
        let sample = "System-wide power settings:\nSleepDisabled\t1\n";
        assert!(parse_sleep_disabled(sample));
        assert!(!parse_sleep_disabled("SleepDisabled\t0\n"));
    }

    #[test]
    fn resolve_pmset_path_prefers_usr_bin() {
        let path = resolve_pmset_path().expect("pmset should exist on dev machine");
        assert!(
            path == "/usr/bin/pmset" || path == "/usr/sbin/pmset",
            "unexpected pmset path: {path}"
        );
    }

    #[test]
    fn format_pmset_auth_error_detects_user_cancel() {
        let msg = format_pmset_auth_error(true, "User canceled. (-128)");
        assert!(msg.contains("cancelled"));
        assert!(!msg.contains("getcwd"));
    }
}
