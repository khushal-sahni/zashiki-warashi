use std::ffi::CStr;
use std::process::{Command, Stdio};

use tracing::warn;

#[cfg(unix)]
use std::os::unix::process::CommandExt;

/// The user's login shell. launchd agents often run without `$SHELL`.
pub fn user_shell() -> String {
    if let Ok(shell) = std::env::var("SHELL") {
        if !shell.is_empty() {
            return shell;
        }
    }
    passwd_shell().unwrap_or_else(|| "/bin/zsh".to_string())
}

#[cfg(unix)]
fn passwd_shell() -> Option<String> {
    // getpwuid returns a pointer into static storage; we copy out immediately.
    let entry = unsafe { libc::getpwuid(libc::getuid()) };
    if entry.is_null() {
        return None;
    }
    let shell = unsafe { (*entry).pw_shell };
    if shell.is_null() {
        return None;
    }
    let value = unsafe { CStr::from_ptr(shell) }.to_string_lossy().into_owned();
    (!value.is_empty()).then_some(value)
}

#[cfg(not(unix))]
fn passwd_shell() -> Option<String> {
    None
}

/// `$SHELL -lc <command>` in `cwd`, leading a new process group (pgid == pid).
pub fn group_command(cwd: &str, command: &str) -> Command {
    let mut child = Command::new(user_shell());
    child.args(["-lc", command]).current_dir(cwd).stdin(Stdio::null());
    #[cfg(unix)]
    unsafe {
        child.pre_exec(|| {
            if libc::setpgid(0, 0) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    child
}

#[cfg(unix)]
pub fn signal_group(pgid: i32, signal: i32) {
    if unsafe { libc::kill(-pgid, signal) } != 0 {
        let err = std::io::Error::last_os_error();
        warn!(pgid, signal, error = %err, "signal process group failed");
    }
}

#[cfg(not(unix))]
pub fn signal_group(_pgid: i32, _signal: i32) {}

#[cfg(unix)]
pub fn is_pid_alive(pid: i32) -> bool {
    unsafe { libc::kill(pid, 0) == 0 }
}

#[cfg(not(unix))]
pub fn is_pid_alive(_pid: i32) -> bool {
    false
}
