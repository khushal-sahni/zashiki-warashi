use serde::Serialize;
use thiserror::Error;

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
}

impl AppError {
    pub fn message(msg: impl Into<String>) -> Self {
        Self::Message(msg.into())
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppErrorDto {
    pub code: String,
    pub message: String,
}

impl From<AppError> for AppErrorDto {
    fn from(value: AppError) -> Self {
        let code = match &value {
            AppError::Message(_) => "message",
            AppError::Database(_) => "database",
            AppError::Io(_) => "io",
            AppError::NotFound(_) => "not_found",
        };
        Self {
            code: code.to_string(),
            message: value.to_string(),
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
        }
    }
}
