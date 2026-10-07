use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use chrono::{Local, NaiveDateTime, TimeZone};
use plist::{Dictionary, Value};
use tracing::{info, warn};

use crate::error::AppError;
use crate::services::job_paths::{wake_helper_plist, APP_IDENTIFIER, WAKE_HELPER_LABEL};
use crate::services::launchd_service::{program_args, write_xml};
use crate::services::wake_requests::read_requests;

const PMSET: &str = "/usr/bin/pmset";
const OSASCRIPT: &str = "/usr/bin/osascript";
const HELPER_BIN: &str = "/Library/PrivilegedHelperTools/com.zashiki.warashi.wake";
const HELPER_LOG: &str = "/Library/Logs/com.zashiki.warashi.wake.log";
const RECONCILE_EVERY: Duration = Duration::from_secs(20);
const ARM_FORMAT: &str = "%m/%d/%y %H:%M:%S";
const SCHED_FORMAT: &str = "%m/%d/%Y %H:%M:%S";

/// pmset calls made by the root helper. Faked in tests.
pub trait PmsetRunner {
    fn sched(&self) -> Result<String, AppError>;
    fn schedule_wake(&self, when: &str) -> Result<(), AppError>;
    fn cancel_wake(&self, when: &str) -> Result<(), AppError>;
}

pub struct SystemPmset;

impl PmsetRunner for SystemPmset {
    fn sched(&self) -> Result<String, AppError> {
        let output = Command::new(PMSET).args(["-g", "sched"]).output()?;
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    fn schedule_wake(&self, when: &str) -> Result<(), AppError> {
        pmset(&["schedule", "wake", when, APP_IDENTIFIER])
    }

    fn cancel_wake(&self, when: &str) -> Result<(), AppError> {
        pmset(&["schedule", "cancel", "wake", when, APP_IDENTIFIER])
    }
}

fn pmset(args: &[&str]) -> Result<(), AppError> {
    let output = Command::new(PMSET).args(args).stdin(Stdio::null()).output()?;
    if output.status.success() {
        return Ok(());
    }
    let detail = String::from_utf8_lossy(&output.stderr);
    Err(AppError::message(format!("pmset {} failed: {}", args.join(" "), detail.trim())))
}

/// Wakes in `pmset -g sched` that Zashiki owns. Everyone else's are ignored.
pub fn parse_owned_wakes(sched: &str) -> Vec<i64> {
    let owner = format!("by '{APP_IDENTIFIER}'");
    sched
        .lines()
        .filter(|line| line.trim_end().ends_with(&owner))
        .filter_map(|line| {
            let rest = line.split("wake at ").nth(1)?;
            let stamp = rest.split(" by ").next()?.trim();
            let naive = NaiveDateTime::parse_from_str(stamp, SCHED_FORMAT).ok()?;
            Local.from_local_datetime(&naive).earliest().map(|at| at.timestamp())
        })
        .collect()
}

/// What to arm and cancel so our pmset entries equal `desired`.
pub fn plan_reconcile(desired: &[i64], armed: &[i64]) -> (Vec<i64>, Vec<i64>) {
    let desired_set: HashSet<i64> = desired.iter().copied().collect();
    let armed_set: HashSet<i64> = armed.iter().copied().collect();
    let to_arm = desired.iter().copied().filter(|w| !armed_set.contains(w)).collect();
    let to_cancel = armed.iter().copied().filter(|w| !desired_set.contains(w)).collect();
    (to_arm, to_cancel)
}

fn format_arm(unix: i64) -> Option<String> {
    Local.timestamp_opt(unix, 0).single().map(|at| at.format(ARM_FORMAT).to_string())
}

pub fn reconcile_once(requests: &Path, pmset: &dyn PmsetRunner) -> Result<(), AppError> {
    let desired = read_requests(requests, Local::now().timestamp());
    let armed = parse_owned_wakes(&pmset.sched()?);
    let (to_arm, to_cancel) = plan_reconcile(&desired, &armed);
    for wake in to_cancel.iter().filter_map(|w| format_arm(*w)) {
        pmset.cancel_wake(&wake)?;
        info!(%wake, "cancelled wake");
    }
    for wake in to_arm.iter().filter_map(|w| format_arm(*w)) {
        pmset.schedule_wake(&wake)?;
        info!(%wake, "armed wake");
    }
    Ok(())
}

pub fn cancel_all(pmset: &dyn PmsetRunner) -> Result<(), AppError> {
    for wake in parse_owned_wakes(&pmset.sched()?).iter().filter_map(|w| format_arm(*w)) {
        pmset.cancel_wake(&wake)?;
    }
    Ok(())
}

/// Root LaunchDaemon loop. Only ever touches wakes owned by Zashiki.
pub fn run_daemon(requests: &Path) -> ! {
    info!(requests = %requests.display(), "wake helper started");
    loop {
        if let Err(err) = reconcile_once(requests, &SystemPmset) {
            warn!(error = %err, "wake reconcile failed");
        }
        thread::sleep(RECONCILE_EVERY);
    }
}

pub fn helper_installed() -> bool {
    wake_helper_plist().exists()
}

fn render_helper_plist(requests: &Path) -> Result<String, AppError> {
    let requests = requests.display().to_string();
    let mut dict = Dictionary::new();
    dict.insert("Label".into(), Value::String(WAKE_HELPER_LABEL.into()));
    dict.insert(
        "ProgramArguments".into(),
        program_args(Path::new(HELPER_BIN), &["job", "wake-daemon", "--requests", &requests]),
    );
    dict.insert("RunAtLoad".into(), Value::Boolean(true));
    dict.insert("KeepAlive".into(), Value::Boolean(true));
    dict.insert("StandardErrorPath".into(), Value::String(HELPER_LOG.into()));
    dict.insert("StandardOutPath".into(), Value::String(HELPER_LOG.into()));
    write_xml(dict)
}

/// Copy the binary to a root-owned path so a user-writable app bundle cannot run as root.
pub fn install_helper(exe: &Path, requests: &Path) -> Result<(), AppError> {
    let staging = std::env::temp_dir().join(format!("zashiki-wake-{}", std::process::id()));
    fs::create_dir_all(&staging)?;
    let plist_path = staging.join("helper.plist");
    fs::write(&plist_path, render_helper_plist(requests)?)?;
    let target = wake_helper_plist().display().to_string();
    let script = format!(
        "set -e\n\
         install -d -m 755 -o root -g wheel /Library/PrivilegedHelperTools\n\
         install -m 755 -o root -g wheel {exe} {HELPER_BIN}\n\
         install -m 644 -o root -g wheel {plist} {target}\n\
         launchctl bootout system/{WAKE_HELPER_LABEL} 2>/dev/null || true\n\
         launchctl bootstrap system {target}\n",
        exe = shell_quote(&exe.display().to_string()),
        plist = shell_quote(&plist_path.display().to_string()),
        target = shell_quote(&target),
    );
    let result = run_admin_script(&staging, &script);
    let _ = fs::remove_dir_all(&staging);
    result
}

pub fn uninstall_helper() -> Result<(), AppError> {
    let staging = std::env::temp_dir().join(format!("zashiki-wake-rm-{}", std::process::id()));
    fs::create_dir_all(&staging)?;
    let target = shell_quote(&wake_helper_plist().display().to_string());
    let script = format!(
        "launchctl bootout system/{WAKE_HELPER_LABEL} 2>/dev/null || true\n\
         [ -x {HELPER_BIN} ] && {HELPER_BIN} job wake-daemon --cancel-all || true\n\
         rm -f {target} {HELPER_BIN}\n"
    );
    let result = run_admin_script(&staging, &script);
    let _ = fs::remove_dir_all(&staging);
    result
}

fn run_admin_script(staging: &Path, script: &str) -> Result<(), AppError> {
    let script_path = staging.join("run.sh");
    fs::write(&script_path, script)?;
    let apple = format!(
        "do shell script \"/bin/sh \" & quoted form of \"{}\" with administrator privileges",
        script_path.display()
    );
    let output = Command::new(OSASCRIPT).current_dir("/").args(["-e", &apple]).output()?;
    if output.status.success() {
        return Ok(());
    }
    let detail = String::from_utf8_lossy(&output.stderr);
    if detail.contains("(-128)") {
        return Err(AppError::message(
            "Administrator approval was cancelled. Exact jobs will run on the next wake instead.",
        ));
    }
    Err(AppError::message(format!("Could not install the wake helper: {}", detail.trim())))
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct FakePmset {
        sched: String,
        calls: RefCell<Vec<String>>,
    }

    impl PmsetRunner for FakePmset {
        fn sched(&self) -> Result<String, AppError> {
            Ok(self.sched.clone())
        }
        fn schedule_wake(&self, when: &str) -> Result<(), AppError> {
            self.calls.borrow_mut().push(format!("arm {when}"));
            Ok(())
        }
        fn cancel_wake(&self, when: &str) -> Result<(), AppError> {
            self.calls.borrow_mut().push(format!("cancel {when}"));
            Ok(())
        }
    }

    fn local(text: &str) -> i64 {
        let naive = NaiveDateTime::parse_from_str(text, SCHED_FORMAT).expect("parse");
        Local.from_local_datetime(&naive).earliest().expect("local").timestamp()
    }

    #[test]
    fn parses_only_our_wakes() {
        let sched = "Scheduled power events:\n \
            [0]  wake at 10/07/2026 11:58:29 by 'com.apple.alarm.user-invisible-com.apple.calaccessd'\n \
            [1]  wake at 10/08/2026 02:58:30 by 'com.zashiki.warashi'\n \
            [2]  wake at 10/08/2026 05:00:00 by 'com.zashiki.warashi.evil'\n";
        assert_eq!(parse_owned_wakes(sched), vec![local("10/08/2026 02:58:30")]);
    }

    #[test]
    fn reconcile_plan_arms_new_and_cancels_stale() {
        let (arm, cancel) = plan_reconcile(&[1, 2, 3], &[2, 9]);
        assert_eq!(arm, vec![1, 3]);
        assert_eq!(cancel, vec![9]);
    }

    #[test]
    fn cancel_all_never_touches_other_owners() {
        let fake = FakePmset {
            sched: " [0]  wake at 10/08/2026 05:21:27 by 'com.apple.osanalytics'\n \
                    [1]  wake at 10/08/2026 02:58:30 by 'com.zashiki.warashi'\n"
                .into(),
            calls: RefCell::new(Vec::new()),
        };
        cancel_all(&fake).expect("cancel");
        assert_eq!(*fake.calls.borrow(), vec!["cancel 10/08/26 02:58:30".to_string()]);
    }

    #[test]
    fn shell_quote_escapes_single_quotes() {
        assert_eq!(shell_quote("/Apps/Zashiki Warashi.app"), "'/Apps/Zashiki Warashi.app'");
        assert_eq!(shell_quote("a'b"), "'a'\\''b'");
    }
}
