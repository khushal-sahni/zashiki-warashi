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
        let (code, message, port_conflict) = match &value {
            AppError::Message(v) => ("message", v.clone(), None),
            AppError::Database(v) => ("database", format!("database error: {v}"), None),
            AppError::Io(v) => ("io", format!("io error: {v}"), None),
            AppError::NotFound(v) => ("not_found", format!("not found: {v}"), None),
            AppError::Conflict(v) => ("conflict", format!("conflict: {v}"), None),
            AppError::InvalidArgument(v) => {
                ("invalid_argument", format!("invalid argument: {v}"), None)
            }
            AppError::PortConflict(conflict) => (
                "port_conflict",
                format!(
                    "port {} is already in use by {}",
                    conflict.host_port,
                    conflict.occupant.label()
                ),
                Some(conflict.clone()),
            ),
        };
        Self {
            code: code.to_string(),
            message,
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
