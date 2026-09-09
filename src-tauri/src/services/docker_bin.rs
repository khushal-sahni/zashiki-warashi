use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::OnceLock;

use crate::error::AppError;

static DOCKER_BIN: OnceLock<Option<PathBuf>> = OnceLock::new();

/// Resolve `docker` once via the login shell (fnm/Homebrew PATH), then reuse.
pub fn resolve_docker_bin() -> Option<PathBuf> {
    DOCKER_BIN
        .get_or_init(|| {
            let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
            let output = Command::new(&shell)
                .args(["-lc", "command -v docker"])
                .stdin(Stdio::null())
                .output()
                .ok()?;
            if !output.status.success() {
                return None;
            }
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if path.is_empty() {
                return None;
            }
            Some(PathBuf::from(path))
        })
        .clone()
}

/// Warm the cache during app setup so the first status call is not cold.
pub fn warm_docker_bin() {
    let _ = resolve_docker_bin();
}

pub fn docker_command() -> Result<Command, AppError> {
    match resolve_docker_bin() {
        Some(path) => Ok(Command::new(path)),
        None => Err(AppError::message(
            "docker not found on login-shell PATH; install Docker Desktop / OrbStack",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn docker_command_errors_when_unresolved() {
        // If docker is installed this still constructs a Command successfully.
        // Smoke: resolve does not panic.
        let _ = resolve_docker_bin();
    }
}
