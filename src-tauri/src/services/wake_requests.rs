use std::fs;
use std::path::Path;

use chrono::{Local, TimeZone};
use serde::{Deserialize, Serialize};

use crate::domain::{Job, JobPolicy};
use crate::error::AppError;
use crate::services::job_schedule::next_fire;

/// Wake this long before an exact job fires so Wi-Fi can rejoin.
pub const WAKE_LEAD_SECONDS: i64 = 90;
pub const MAX_WAKE_HORIZON_SECONDS: i64 = 8 * 24 * 3600;
pub const MAX_WAKES: usize = 16;

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct WakeRequests {
    pub wakes: Vec<i64>,
}

/// Next wake timestamp for every enabled exact job.
pub fn desired_wakes(jobs: &[Job], now_unix: i64) -> Vec<i64> {
    let Some(now) = Local.timestamp_opt(now_unix, 0).single() else {
        return Vec::new();
    };
    let mut wakes: Vec<i64> = jobs
        .iter()
        .filter(|job| job.enabled && job.policy == JobPolicy::Exact)
        .filter_map(|job| next_fire(&job.schedule, &(now + chrono::Duration::seconds(WAKE_LEAD_SECONDS))))
        .map(|fire| fire.timestamp() - WAKE_LEAD_SECONDS)
        .collect();
    sanitize(&mut wakes, now_unix);
    wakes
}

/// Keep only future wakes inside the horizon, sorted, unique, capped.
pub fn sanitize(wakes: &mut Vec<i64>, now_unix: i64) {
    wakes.retain(|wake| *wake > now_unix && *wake <= now_unix + MAX_WAKE_HORIZON_SECONDS);
    wakes.sort_unstable();
    wakes.dedup();
    wakes.truncate(MAX_WAKES);
}

pub fn write_requests(path: &Path, wakes: &[i64]) -> Result<(), AppError> {
    let body = serde_json::to_string(&WakeRequests { wakes: wakes.to_vec() })?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, body)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

/// Read and validate requests. Anything malformed means "no wakes".
pub fn read_requests(path: &Path, now_unix: i64) -> Vec<i64> {
    let parsed = fs::read_to_string(path)
        .ok()
        .and_then(|body| serde_json::from_str::<WakeRequests>(&body).ok())
        .unwrap_or_default();
    let mut wakes = parsed.wakes;
    sanitize(&mut wakes, now_unix);
    wakes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{JobSchedule, NetworkPolicy};

    fn job(policy: JobPolicy, enabled: bool) -> Job {
        Job {
            id: "j".into(),
            name: "j".into(),
            command: "true".into(),
            working_dir: None,
            enabled,
            schedule: JobSchedule::Interval { minutes: 60 },
            policy,
            network: NetworkPolicy::Required,
            network_grace_seconds: 90,
            max_runtime_seconds: None,
            created_at: 0,
            updated_at: 0,
        }
    }

    #[test]
    fn only_enabled_exact_jobs_request_wakes() {
        let now = Local::now().timestamp();
        let wakes = desired_wakes(
            &[job(JobPolicy::Exact, true), job(JobPolicy::Optimistic, true), job(JobPolicy::Exact, false)],
            now,
        );
        assert_eq!(wakes.len(), 1);
        assert!(wakes[0] > now);
        assert_eq!((wakes[0] + WAKE_LEAD_SECONDS) % 60, 0);
    }

    #[test]
    fn sanitize_drops_past_far_and_duplicates() {
        let now = 1_000_000;
        let mut wakes = vec![now - 5, now + 10, now + 10, now + MAX_WAKE_HORIZON_SECONDS + 1];
        sanitize(&mut wakes, now);
        assert_eq!(wakes, vec![now + 10]);
    }

    #[test]
    fn malformed_request_file_means_no_wakes() {
        let path = std::env::temp_dir().join(format!("zashiki-wake-{}.json", std::process::id()));
        fs::write(&path, "{\"wakes\": [\"rm -rf\"]}").expect("write");
        assert!(read_requests(&path, 0).is_empty());
        write_requests(&path, &[100]).expect("write");
        assert_eq!(read_requests(&path, 50), vec![100]);
        let _ = fs::remove_file(path);
    }
}
