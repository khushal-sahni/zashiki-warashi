use std::path::{Path, PathBuf};

use crate::error::AppError;

/// Matches `identifier` in `tauri.conf.json`; also used as the launchd label prefix.
pub const APP_IDENTIFIER: &str = "com.zashiki.warashi";
pub const SUPERVISOR_LABEL: &str = "com.zashiki.warashi.jobs";
pub const WAKE_HELPER_LABEL: &str = "com.zashiki.warashi.wake";
pub const JOB_LABEL_PREFIX: &str = "com.zashiki.warashi.job.";

/// Filesystem layout for jobs, shared by the app and the `job` subcommands.
#[derive(Debug, Clone)]
pub struct JobPaths {
    data_dir: PathBuf,
}

impl JobPaths {
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        Self { data_dir: data_dir.into() }
    }

    /// Resolve the app data dir without Tauri (for launchd-started subcommands).
    pub fn from_home() -> Result<Self, AppError> {
        let home = std::env::var("HOME").map_err(|_| AppError::message("HOME is not set"))?;
        Ok(Self::new(
            Path::new(&home).join("Library/Application Support").join(APP_IDENTIFIER),
        ))
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub fn job_dir(&self, job_id: &str) -> PathBuf {
        self.data_dir.join("jobs").join(job_id)
    }

    pub fn run_log(&self, job_id: &str, run_id: &str) -> PathBuf {
        self.job_dir(job_id).join("runs").join(format!("{run_id}.log"))
    }

    pub fn socket(&self) -> PathBuf {
        self.data_dir.join("jobs.sock")
    }

    pub fn wake_requests(&self) -> PathBuf {
        self.data_dir.join("wake-requests.json")
    }
}

pub fn launch_agents_dir() -> Result<PathBuf, AppError> {
    let home = std::env::var("HOME").map_err(|_| AppError::message("HOME is not set"))?;
    Ok(Path::new(&home).join("Library/LaunchAgents"))
}

pub fn job_label(job_id: &str) -> String {
    format!("{JOB_LABEL_PREFIX}{job_id}")
}

pub fn wake_helper_plist() -> PathBuf {
    PathBuf::from(format!("/Library/LaunchDaemons/{WAKE_HELPER_LABEL}.plist"))
}
