use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RunState {
    Stopped,
    Starting,
    Running,
    Failed,
}

impl RunState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Stopped => "stopped",
            Self::Starting => "starting",
            Self::Running => "running",
            Self::Failed => "failed",
        }
    }

    pub fn parse(value: &str) -> Self {
        match value {
            "starting" => Self::Starting,
            "running" => Self::Running,
            "failed" => Self::Failed,
            _ => Self::Stopped,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectRun {
    pub project_id: String,
    pub pid: Option<i32>,
    pub pgid: Option<i32>,
    pub started_at: Option<String>,
    pub started_at_unix: Option<i64>,
    pub status: RunState,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub path: String,
    /// User override; null means infer at runtime.
    pub start_command: Option<String>,
    /// User override; null means process-group signal stop.
    pub stop_command: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub inferred_start_command: Option<String>,
    pub run: ProjectRun,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanCandidate {
    pub name: String,
    pub path: String,
    pub inferred_start_command: Option<String>,
    pub already_registered: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub scan_roots: Vec<String>,
}
