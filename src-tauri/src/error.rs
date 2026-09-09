use serde::Serialize;
use thiserror::Error;

use crate::domain::PortConflict;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    Message(String),

    #[error("database error: {0}")]
    Database(String),

    #[error("io error: {0}")]
    Io(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("port {} is already in use", .0.host_port)]
    PortConflict(PortConflict),
}

impl AppError {
    pub fn message(msg: impl Into<String>) -> Self {
        Self::Message(msg.into())
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    pub fn conflict(msg: impl Into<String>) -> Self {
        Self::Conflict(msg.into())
    }

    pub fn invalid(msg: impl Into<String>) -> Self {
        Self::InvalidArgument(msg.into())
    }

    pub fn port_conflict(conflict: PortConflict) -> Self {
        Self::PortConflict(conflict)
    }

    /// Short, actionable text for the UI and `last_error`.
    pub fn user_message(&self) -> String {
        match self {
            Self::Message(v) => map_raw_message(v),
            Self::Database(v) => format!("Could not read the local catalog: {v}"),
            Self::Io(v) => map_raw_message(v),
            Self::NotFound(v) => map_raw_message(v),
            Self::Conflict(v) => map_raw_message(v),
            Self::InvalidArgument(v) => map_raw_message(v),
            Self::PortConflict(conflict) => format!(
                "Port {} is already in use by {}",
                conflict.host_port,
                conflict.occupant.label()
            ),
        }
    }
}

fn map_raw_message(raw: &str) -> String {
    let lower = raw.to_lowercase();

    if lower.contains("docker not found") || lower.contains("command not found: docker") {
        return "Docker was not found on your login-shell PATH. Install Docker Desktop or OrbStack, then restart Zashiki.".to_string();
    }

    if lower.contains("cannot connect to the docker daemon")
        || lower.contains("is the docker daemon running")
        || lower.contains("error during connect")
        || lower.contains("docker desktop is not running")
        || (lower.contains("connection refused") && lower.contains("docker"))
    {
        return "Docker is installed but the daemon is not running. Start Docker Desktop or OrbStack, then try again.".to_string();
    }

    if lower.starts_with("compose failed:") {
        let detail = raw
            .trim_start_matches("compose failed:")
            .trim_start_matches("Compose failed:")
            .trim();
        let detail_lower = detail.to_lowercase();
        if detail_lower.contains("cannot connect to the docker daemon")
            || detail_lower.contains("is the docker daemon running")
            || detail_lower.contains("error during connect")
        {
            return "Docker is installed but the daemon is not running. Start Docker Desktop or OrbStack, then try again.".to_string();
        }
        return format!("Docker Compose failed: {}", first_line(detail));
    }

    if lower.contains("no start command") {
        return "No start command is configured. Set a start override, or add a package.json / Makefile / compose file so one can be inferred.".to_string();
    }

    if lower.contains("failed to spawn") {
        let detail = strip_known_prefix(raw, "failed to spawn");
        return format!(
            "Could not start the project. Check that the command exists on your PATH (fnm/nvm/Homebrew): {detail}"
        );
    }

    if lower.contains("cursor is not installed") {
        return raw.to_string();
    }

    if lower.starts_with("io error:") {
        return map_raw_message(raw.trim_start_matches("io error:").trim());
    }
    if lower.starts_with("not found:") {
        return map_raw_message(raw.trim_start_matches("not found:").trim());
    }
    if lower.starts_with("conflict:") {
        return map_raw_message(raw.trim_start_matches("conflict:").trim());
    }
    if lower.starts_with("invalid argument:") {
        return map_raw_message(raw.trim_start_matches("invalid argument:").trim());
    }
    if lower.starts_with("database error:") {
        return format!(
            "Could not read the local catalog: {}",
            raw.trim_start_matches("database error:").trim()
        );
    }

    first_line(raw).to_string()
}

fn strip_known_prefix<'a>(raw: &'a str, needle: &str) -> &'a str {
    if let Some(idx) = raw.to_lowercase().find(needle) {
        raw[idx + needle.len()..].trim_start_matches([' ', ':', '`']).trim()
    } else {
        raw
    }
}

fn first_line(raw: &str) -> &str {
    raw.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or(raw)
}

impl From<rusqlite::Error> for AppError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Database(value.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(value: serde_json::Error) -> Self {
        Self::Message(format!("json error: {value}"))
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppErrorDto {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port_conflict: Option<PortConflict>,
}

impl From<AppError> for AppErrorDto {
    fn from(value: AppError) -> Self {
        let (code, port_conflict) = match &value {
            AppError::Message(_) => ("message", None),
            AppError::Database(_) => ("database", None),
            AppError::Io(_) => ("io", None),
            AppError::NotFound(_) => ("not_found", None),
            AppError::Conflict(_) => ("conflict", None),
            AppError::InvalidArgument(_) => ("invalid_argument", None),
            AppError::PortConflict(conflict) => ("port_conflict", Some(conflict.clone())),
        };
        Self {
            code: code.to_string(),
            message: value.user_message(),
            port_conflict,
        }
    }
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        AppErrorDto::from(self.clone_for_serialize()).serialize(serializer)
    }
}

impl AppError {
    fn clone_for_serialize(&self) -> Self {
        match self {
            Self::Message(v) => Self::Message(v.clone()),
            Self::Database(v) => Self::Database(v.clone()),
            Self::Io(v) => Self::Io(v.clone()),
            Self::NotFound(v) => Self::NotFound(v.clone()),
            Self::Conflict(v) => Self::Conflict(v.clone()),
            Self::InvalidArgument(v) => Self::InvalidArgument(v.clone()),
            Self::PortConflict(v) => Self::PortConflict(v.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_docker_missing() {
        let err = AppError::message(
            "docker not found on login-shell PATH; install Docker Desktop / OrbStack",
        );
        assert!(err.user_message().contains("Docker Desktop or OrbStack"));
    }

    #[test]
    fn maps_docker_daemon_down() {
        let err = AppError::message(
            "Cannot connect to the Docker daemon at unix:///var/run/docker.sock. Is the docker daemon running?",
        );
        assert!(err.user_message().contains("daemon is not running"));
    }

    #[test]
    fn maps_missing_start_command() {
        let err = AppError::invalid("no start command configured or inferred for this project");
        assert!(err.user_message().contains("No start command"));
    }

    #[test]
    fn maps_spawn_failure() {
        let err = AppError::Io(
            "failed to spawn `npm run dev` via /bin/zsh: No such file or directory".into(),
        );
        let message = err.user_message();
        assert!(message.contains("Could not start the project"));
        assert!(message.contains("PATH"));
    }

    #[test]
    fn maps_compose_failure() {
        let err = AppError::message("compose failed: Error response from daemon: postgres");
        assert_eq!(
            err.user_message(),
            "Docker Compose failed: Error response from daemon: postgres"
        );
    }

    #[test]
    fn strips_io_prefix_from_display_path() {
        let err = AppError::Io("failed to spawn `foo` via /bin/zsh: boom".into());
        let dto = AppErrorDto::from(err);
        assert!(!dto.message.starts_with("io error:"));
        assert!(dto.message.contains("Could not start the project"));
    }
}
