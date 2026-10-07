use serde::{Deserialize, Serialize};

/// When a job fires. Every variant maps onto launchd `StartCalendarInterval`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum JobSchedule {
    /// Must divide 60, or be a whole number of hours that divides 24.
    Interval { minutes: u32 },
    Daily { hour: u8, minute: u8 },
    /// Weekdays use launchd numbering: 0 = Sunday … 6 = Saturday.
    Weekly { weekdays: Vec<u8>, hour: u8, minute: u8 },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum JobPolicy {
    /// Run on time if awake, otherwise once on the next wake.
    Optimistic,
    /// Arm a hardware wake shortly before each fire.
    Exact,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum NetworkPolicy {
    /// Wait for a route, then run anyway.
    BestEffort,
    /// Wait for a route, then skip the run if still offline.
    Required,
    /// Do not wait for the network.
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    pub id: String,
    pub name: String,
    pub command: String,
    pub working_dir: Option<String>,
    pub enabled: bool,
    pub schedule: JobSchedule,
    pub policy: JobPolicy,
    pub network: NetworkPolicy,
    pub network_grace_seconds: u32,
    pub max_runtime_seconds: Option<u32>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Fields a caller may set when creating or updating a job.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct JobInput {
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub working_dir: Option<String>,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    pub schedule: JobSchedule,
    #[serde(default = "default_policy")]
    pub policy: JobPolicy,
    #[serde(default = "default_network")]
    pub network: NetworkPolicy,
    #[serde(default = "default_grace")]
    pub network_grace_seconds: u32,
    #[serde(default)]
    pub max_runtime_seconds: Option<u32>,
}

fn default_enabled() -> bool {
    true
}

fn default_policy() -> JobPolicy {
    JobPolicy::Optimistic
}

fn default_network() -> NetworkPolicy {
    NetworkPolicy::BestEffort
}

fn default_grace() -> u32 {
    90
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RunTrigger {
    Calendar,
    CatchUp,
    Manual,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum JobRunStatus {
    Running,
    Succeeded,
    Failed,
    TimedOut,
    SkippedOffline,
    Interrupted,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum NetworkOutcome {
    Online,
    Offline,
    NotChecked,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PowerSource {
    Ac,
    Battery,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct JobRun {
    pub id: String,
    pub job_id: String,
    pub trigger: RunTrigger,
    pub status: JobRunStatus,
    pub scheduled_for_unix: Option<i64>,
    pub started_at_unix: i64,
    pub finished_at_unix: Option<i64>,
    pub exit_code: Option<i32>,
    pub network: Option<NetworkOutcome>,
    pub power_source: Option<PowerSource>,
    pub runner_pid: Option<i32>,
    pub log_path: String,
    pub message: Option<String>,
}

/// A job plus derived, display-ready state.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct JobSummary {
    #[serde(flatten)]
    pub job: Job,
    pub next_fire_unix: Option<i64>,
    pub last_run: Option<JobRun>,
}

/// A user LaunchAgent that Zashiki does not manage. Read-only.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ForeignAgent {
    pub label: String,
    pub program: Option<String>,
    pub plist_path: String,
    pub schedule_hint: Option<String>,
    pub loaded: bool,
    pub pid: Option<i32>,
    pub last_exit: Option<i32>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct JobSystemStatus {
    pub supervisor_installed: bool,
    pub wake_helper_installed: bool,
    pub power_source: PowerSource,
    pub online: bool,
    pub socket_path: String,
    pub mcp_command: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct JobLogTail {
    pub run_id: String,
    pub lines: Vec<String>,
}

macro_rules! str_enum {
    ($ty:ty { $($variant:ident => $text:literal),+ $(,)? }) => {
        impl $ty {
            pub fn as_str(&self) -> &'static str {
                match self { $(Self::$variant => $text),+ }
            }

            pub fn parse(value: &str) -> Option<Self> {
                match value { $($text => Some(Self::$variant),)+ _ => None }
            }
        }
    };
}

str_enum!(JobPolicy { Optimistic => "optimistic", Exact => "exact" });
str_enum!(NetworkPolicy { BestEffort => "best_effort", Required => "required", None => "none" });
str_enum!(RunTrigger { Calendar => "calendar", CatchUp => "catch_up", Manual => "manual" });
str_enum!(JobRunStatus {
    Running => "running",
    Succeeded => "succeeded",
    Failed => "failed",
    TimedOut => "timed_out",
    SkippedOffline => "skipped_offline",
    Interrupted => "interrupted",
});
str_enum!(NetworkOutcome { Online => "online", Offline => "offline", NotChecked => "not_checked" });
str_enum!(PowerSource { Ac => "ac", Battery => "battery", Unknown => "unknown" });
