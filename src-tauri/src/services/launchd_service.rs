use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use plist::{Dictionary, Value};
use tracing::{info, warn};

use crate::domain::{ForeignAgent, Job};
use crate::error::AppError;
use crate::services::job_paths::{job_label, JobPaths, APP_IDENTIFIER, JOB_LABEL_PREFIX, SUPERVISOR_LABEL};
use crate::services::job_schedule::{calendar_slots, CalendarSlot};

const LAUNCHCTL: &str = "/bin/launchctl";

/// launchctl calls, faked in tests.
pub trait LaunchctlRunner: Send + Sync {
    fn bootstrap(&self, plist: &Path) -> Result<(), AppError>;
    fn bootout(&self, label: &str);
    fn list(&self) -> Result<String, AppError>;
}

pub struct SystemLaunchctl;

impl LaunchctlRunner for SystemLaunchctl {
    fn bootstrap(&self, plist: &Path) -> Result<(), AppError> {
        let plist = plist.display().to_string();
        let domain = gui_domain();
        if launchctl(&["bootstrap", &domain, &plist]).is_ok() {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(500));
        launchctl(&["bootstrap", &domain, &plist])
    }

    fn bootout(&self, label: &str) {
        let target = format!("{}/{label}", gui_domain());
        let _ = launchctl(&["bootout", &target]);
    }

    fn list(&self) -> Result<String, AppError> {
        let output = Command::new(LAUNCHCTL).arg("list").stdin(Stdio::null()).output()?;
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }
}

fn gui_domain() -> String {
    format!("gui/{}", unsafe { libc::getuid() })
}

fn launchctl(args: &[&str]) -> Result<(), AppError> {
    let output = Command::new(LAUNCHCTL).args(args).stdin(Stdio::null()).output()?;
    if output.status.success() {
        return Ok(());
    }
    let detail = String::from_utf8_lossy(&output.stderr);
    Err(AppError::message(format!("launchctl {} failed: {}", args[0], detail.trim())))
}

/// Owns Zashiki's user LaunchAgents: one per enabled job, plus the supervisor.
pub struct LaunchdService {
    agents_dir: PathBuf,
    paths: JobPaths,
    runner: Box<dyn LaunchctlRunner>,
}

impl LaunchdService {
    pub fn new(agents_dir: PathBuf, paths: JobPaths, runner: Box<dyn LaunchctlRunner>) -> Self {
        Self { agents_dir, paths, runner }
    }

    fn plist_path(&self, label: &str) -> PathBuf {
        self.agents_dir.join(format!("{label}.plist"))
    }

    /// Write `contents` and reload only when it changed, so running jobs are not killed.
    fn ensure_agent(&self, label: &str, contents: &str) -> Result<(), AppError> {
        let path = self.plist_path(label);
        if fs::read_to_string(&path).is_ok_and(|current| current == contents) {
            return Ok(());
        }
        fs::create_dir_all(&self.agents_dir)?;
        fs::write(&path, contents)?;
        self.runner.bootout(label);
        self.runner.bootstrap(&path)?;
        info!(label, "loaded launch agent");
        Ok(())
    }

    fn remove_agent(&self, label: &str) -> Result<(), AppError> {
        let path = self.plist_path(label);
        if !path.exists() {
            return Ok(());
        }
        self.runner.bootout(label);
        fs::remove_file(&path)?;
        info!(label, "removed launch agent");
        Ok(())
    }

    pub fn install_supervisor(&self, exe: &Path) -> Result<(), AppError> {
        let log = self.paths.data_dir().join("supervisor.log");
        let contents = render_supervisor_plist(exe, &log)?;
        self.ensure_agent(SUPERVISOR_LABEL, &contents)
    }

    pub fn supervisor_installed(&self) -> bool {
        self.plist_path(SUPERVISOR_LABEL).exists()
    }

    pub fn sync_job(&self, job: &Job, exe: &Path) -> Result<(), AppError> {
        let label = job_label(&job.id);
        if !job.enabled {
            return self.remove_agent(&label);
        }
        let log = self.paths.job_dir(&job.id).join("launchd.log");
        fs::create_dir_all(self.paths.job_dir(&job.id))?;
        self.ensure_agent(&label, &render_job_plist(job, exe, &log)?)
    }

    pub fn remove_job(&self, job_id: &str) -> Result<(), AppError> {
        self.remove_agent(&job_label(job_id))
    }

    /// Make the agents directory match the catalog: add/refresh enabled, drop the rest.
    pub fn sync_all(&self, jobs: &[Job], exe: &Path) -> Result<(), AppError> {
        for job in jobs {
            if let Err(err) = self.sync_job(job, exe) {
                warn!(job_id = %job.id, error = %err, "could not sync launch agent");
            }
        }
        let wanted: HashSet<String> = jobs.iter().filter(|j| j.enabled).map(|j| job_label(&j.id)).collect();
        for label in self.managed_labels()? {
            if !wanted.contains(&label) {
                self.remove_agent(&label)?;
            }
        }
        Ok(())
    }

    fn managed_labels(&self) -> Result<Vec<String>, AppError> {
        let Ok(entries) = fs::read_dir(&self.agents_dir) else {
            return Ok(Vec::new());
        };
        Ok(entries
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| entry.file_name().to_str()?.strip_suffix(".plist").map(str::to_string))
            .filter(|label| label.starts_with(JOB_LABEL_PREFIX))
            .collect())
    }

    pub fn list_foreign_agents(&self) -> Result<Vec<ForeignAgent>, AppError> {
        let loaded = parse_launchctl_list(&self.runner.list().unwrap_or_default());
        let Ok(entries) = fs::read_dir(&self.agents_dir) else {
            return Ok(Vec::new());
        };
        let mut agents: Vec<ForeignAgent> = entries
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "plist"))
            .filter_map(|path| read_foreign_agent(&path, &loaded))
            .filter(|agent| !agent.label.starts_with(APP_IDENTIFIER))
            .collect();
        agents.sort_by(|a, b| a.label.cmp(&b.label));
        Ok(agents)
    }
}

pub fn render_job_plist(job: &Job, exe: &Path, log: &Path) -> Result<String, AppError> {
    let mut dict = Dictionary::new();
    dict.insert("Label".into(), Value::String(job_label(&job.id)));
    dict.insert("ProgramArguments".into(), program_args(exe, &["job", "run", &job.id]));
    let slots = calendar_slots(&job.schedule).iter().map(slot_value).collect();
    dict.insert("StartCalendarInterval".into(), Value::Array(slots));
    dict.insert("StandardOutPath".into(), Value::String(log.display().to_string()));
    dict.insert("StandardErrorPath".into(), Value::String(log.display().to_string()));
    write_xml(dict)
}

pub fn render_supervisor_plist(exe: &Path, log: &Path) -> Result<String, AppError> {
    let mut dict = Dictionary::new();
    dict.insert("Label".into(), Value::String(SUPERVISOR_LABEL.into()));
    dict.insert("ProgramArguments".into(), program_args(exe, &["job", "supervise"]));
    dict.insert("RunAtLoad".into(), Value::Boolean(true));
    dict.insert("KeepAlive".into(), Value::Boolean(true));
    dict.insert("ProcessType".into(), Value::String("Background".into()));
    dict.insert("StandardOutPath".into(), Value::String(log.display().to_string()));
    dict.insert("StandardErrorPath".into(), Value::String(log.display().to_string()));
    write_xml(dict)
}

pub fn program_args(exe: &Path, args: &[&str]) -> Value {
    let mut values = vec![Value::String(exe.display().to_string())];
    values.extend(args.iter().map(|arg| Value::String((*arg).to_string())));
    Value::Array(values)
}

fn slot_value(slot: &CalendarSlot) -> Value {
    let mut dict = Dictionary::new();
    dict.insert("Minute".into(), Value::Integer(i64::from(slot.minute).into()));
    if let Some(hour) = slot.hour {
        dict.insert("Hour".into(), Value::Integer(i64::from(hour).into()));
    }
    if let Some(weekday) = slot.weekday {
        dict.insert("Weekday".into(), Value::Integer(i64::from(weekday).into()));
    }
    Value::Dictionary(dict)
}

pub fn write_xml(dict: Dictionary) -> Result<String, AppError> {
    let mut buffer = Vec::new();
    Value::Dictionary(dict)
        .to_writer_xml(&mut buffer)
        .map_err(|err| AppError::message(format!("could not render plist: {err}")))?;
    String::from_utf8(buffer).map_err(|err| AppError::message(format!("plist was not utf-8: {err}")))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoadedAgent {
    pub pid: Option<i32>,
    pub last_exit: Option<i32>,
}

/// Parse `launchctl list` (`PID\tStatus\tLabel`).
pub fn parse_launchctl_list(output: &str) -> HashMap<String, LoadedAgent> {
    output
        .lines()
        .skip(1)
        .filter_map(|line| {
            let mut parts = line.split('\t');
            let pid = parts.next()?.trim().parse().ok();
            let last_exit = parts.next()?.trim().parse().ok();
            let label = parts.next()?.trim().to_string();
            Some((label, LoadedAgent { pid, last_exit }))
        })
        .collect()
}

fn read_foreign_agent(path: &Path, loaded: &HashMap<String, LoadedAgent>) -> Option<ForeignAgent> {
    let dict = Value::from_file(path).ok()?.into_dictionary()?;
    let label = dict.get("Label")?.as_string()?.to_string();
    let program = dict
        .get("Program")
        .and_then(Value::as_string)
        .map(str::to_string)
        .or_else(|| program_from_arguments(&dict));
    let state = loaded.get(&label);
    Some(ForeignAgent {
        schedule_hint: schedule_hint(&dict),
        loaded: state.is_some(),
        pid: state.and_then(|s| s.pid),
        last_exit: state.and_then(|s| s.last_exit),
        plist_path: path.display().to_string(),
        program,
        label,
    })
}

fn program_from_arguments(dict: &Dictionary) -> Option<String> {
    let args = dict.get("ProgramArguments")?.as_array()?;
    let parts: Vec<&str> = args.iter().filter_map(Value::as_string).collect();
    (!parts.is_empty()).then(|| parts.join(" "))
}

fn schedule_hint(dict: &Dictionary) -> Option<String> {
    if let Some(seconds) = dict.get("StartInterval").and_then(Value::as_unsigned_integer) {
        return Some(format!("every {}", humanize_seconds(seconds)));
    }
    let calendar = dict.get("StartCalendarInterval")?;
    let first = match calendar {
        Value::Array(items) => items.first()?.as_dictionary()?,
        Value::Dictionary(inner) => inner,
        _ => return None,
    };
    let hour = first.get("Hour").and_then(Value::as_unsigned_integer);
    let minute = first.get("Minute").and_then(Value::as_unsigned_integer).unwrap_or(0);
    Some(match hour {
        Some(hour) => format!("calendar {hour:02}:{minute:02}"),
        None => format!("calendar :{minute:02} hourly"),
    })
}

fn humanize_seconds(seconds: u64) -> String {
    match seconds {
        s if s % 3600 == 0 => format!("{}h", s / 3600),
        s if s % 60 == 0 => format!("{}m", s / 60),
        s => format!("{s}s"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{JobPolicy, JobSchedule, NetworkPolicy};
    use std::sync::{Arc, Mutex};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[derive(Default)]
    struct FakeLaunchctl {
        calls: Arc<Mutex<Vec<String>>>,
    }

    impl LaunchctlRunner for FakeLaunchctl {
        fn bootstrap(&self, plist: &Path) -> Result<(), AppError> {
            self.calls.lock().expect("lock").push(format!("bootstrap {}", plist.display()));
            Ok(())
        }
        fn bootout(&self, label: &str) {
            self.calls.lock().expect("lock").push(format!("bootout {label}"));
        }
        fn list(&self) -> Result<String, AppError> {
            Ok("PID\tStatus\tLabel\n-\t0\tdev.jobradar.ingest\n".into())
        }
    }

    fn job(id: &str, enabled: bool) -> Job {
        Job {
            id: id.into(),
            name: id.into(),
            command: "true".into(),
            working_dir: None,
            enabled,
            schedule: JobSchedule::Weekly { weekdays: vec![1, 3], hour: 7, minute: 15 },
            policy: JobPolicy::Optimistic,
            network: NetworkPolicy::BestEffort,
            network_grace_seconds: 90,
            max_runtime_seconds: None,
            created_at: 0,
            updated_at: 0,
        }
    }

    fn service() -> (LaunchdService, Arc<Mutex<Vec<String>>>, PathBuf) {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).expect("time").as_nanos();
        let dir = std::env::temp_dir().join(format!("zashiki-launchd-{stamp}"));
        let fake = FakeLaunchctl::default();
        let calls = fake.calls.clone();
        let svc = LaunchdService::new(dir.join("agents"), JobPaths::new(dir.join("data")), Box::new(fake));
        (svc, calls, dir)
    }

    #[test]
    fn job_plist_has_weekday_calendar_entries() {
        let xml = render_job_plist(&job("abc", true), Path::new("/Apps/Z.app/z"), Path::new("/tmp/l.log")).expect("xml");
        assert!(xml.contains("<string>com.zashiki.warashi.job.abc</string>"));
        assert!(xml.contains("<key>Weekday</key>"));
        assert_eq!(xml.matches("<key>Hour</key>").count(), 2);
        assert!(xml.contains("<string>run</string>"));
    }

    #[test]
    fn sync_reloads_only_on_change_and_prunes() {
        let (svc, calls, dir) = service();
        let exe = Path::new("/bin/z");
        svc.sync_all(&[job("a", true), job("b", true)], exe).expect("sync");
        svc.sync_all(&[job("a", true), job("b", true)], exe).expect("sync again");
        assert_eq!(calls.lock().expect("lock").iter().filter(|c| c.starts_with("bootstrap")).count(), 2);

        svc.sync_all(&[job("a", true), job("b", false)], exe).expect("disable b");
        assert!(!dir.join("agents/com.zashiki.warashi.job.b.plist").exists());
        assert!(dir.join("agents/com.zashiki.warashi.job.a.plist").exists());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn lists_foreign_agents_with_launchctl_state() {
        let (svc, _calls, dir) = service();
        fs::create_dir_all(dir.join("agents")).expect("dir");
        let mut dict = Dictionary::new();
        dict.insert("Label".into(), Value::String("dev.jobradar.ingest".into()));
        dict.insert("ProgramArguments".into(), program_args(Path::new("/usr/bin/node"), &["ingest.js"]));
        dict.insert("StartInterval".into(), Value::Integer(3600.into()));
        fs::write(dir.join("agents/dev.jobradar.ingest.plist"), write_xml(dict).expect("xml")).expect("write");
        svc.install_supervisor(Path::new("/bin/z")).expect("supervisor");

        let agents = svc.list_foreign_agents().expect("list");
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].program.as_deref(), Some("/usr/bin/node ingest.js"));
        assert_eq!(agents[0].schedule_hint.as_deref(), Some("every 1h"));
        assert!(agents[0].loaded);
        assert_eq!(agents[0].last_exit, Some(0));
        let _ = fs::remove_dir_all(dir);
    }
}
