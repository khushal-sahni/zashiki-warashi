use std::sync::Arc;

use rusqlite::{params, params_from_iter, OptionalExtension, Row};

use crate::domain::{
    Job, JobPolicy, JobRun, JobRunStatus, JobSchedule, NetworkOutcome, NetworkPolicy, PowerSource,
    RunTrigger,
};
use crate::error::AppError;
use crate::repositories::Database;

const JOB_COLUMNS: &str = "id, name, command, working_dir, enabled, schedule_json, policy, network, network_grace_seconds, max_runtime_seconds, created_at, updated_at";
const RUN_COLUMNS: &str = "id, job_id, trigger, status, scheduled_for_unix, started_at_unix, finished_at_unix, exit_code, network, power_source, runner_pid, log_path, message";

pub struct JobRepository {
    database: Arc<Database>,
}

impl JobRepository {
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    pub fn list_jobs(&self) -> Result<Vec<Job>, AppError> {
        self.database.with_conn(|conn| {
            let sql = format!("SELECT {JOB_COLUMNS} FROM jobs ORDER BY name COLLATE NOCASE");
            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt.query_map([], read_job_row)?;
            rows.map(|row| row.map_err(AppError::from).and_then(job_from_raw))
                .collect()
        })
    }

    pub fn get_job(&self, id: &str) -> Result<Option<Job>, AppError> {
        self.database.with_conn(|conn| {
            let sql = format!("SELECT {JOB_COLUMNS} FROM jobs WHERE id = ?1");
            let raw = conn.query_row(&sql, params![id], read_job_row).optional()?;
            raw.map(job_from_raw).transpose()
        })
    }

    pub fn upsert_job(&self, job: &Job) -> Result<(), AppError> {
        let schedule_json = serde_json::to_string(&job.schedule)?;
        self.database.with_conn(|conn| {
            let sql = format!(
                "INSERT INTO jobs ({JOB_COLUMNS}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                 ON CONFLICT(id) DO UPDATE SET name = excluded.name, command = excluded.command,
                   working_dir = excluded.working_dir, enabled = excluded.enabled,
                   schedule_json = excluded.schedule_json, policy = excluded.policy,
                   network = excluded.network, network_grace_seconds = excluded.network_grace_seconds,
                   max_runtime_seconds = excluded.max_runtime_seconds, updated_at = excluded.updated_at"
            );
            conn.execute(
                &sql,
                params![
                    job.id,
                    job.name,
                    job.command,
                    job.working_dir,
                    job.enabled,
                    schedule_json,
                    job.policy.as_str(),
                    job.network.as_str(),
                    job.network_grace_seconds,
                    job.max_runtime_seconds,
                    job.created_at,
                    job.updated_at,
                ],
            )?;
            Ok(())
        })
    }

    pub fn delete_job(&self, id: &str) -> Result<bool, AppError> {
        self.database.with_conn(|conn| {
            conn.execute("DELETE FROM job_runs WHERE job_id = ?1", params![id])?;
            let removed = conn.execute("DELETE FROM jobs WHERE id = ?1", params![id])?;
            Ok(removed > 0)
        })
    }

    pub fn insert_run(&self, run: &JobRun) -> Result<(), AppError> {
        self.database.with_conn(|conn| {
            let sql = format!(
                "INSERT INTO job_runs ({RUN_COLUMNS}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)"
            );
            conn.execute(&sql, params_from_iter(run_params(run)))?;
            Ok(())
        })
    }

    pub fn update_run(&self, run: &JobRun) -> Result<(), AppError> {
        self.database.with_conn(|conn| {
            conn.execute(
                "UPDATE job_runs SET status = ?2, finished_at_unix = ?3, exit_code = ?4,
                   network = ?5, power_source = ?6, runner_pid = ?7, message = ?8
                 WHERE id = ?1",
                params![
                    run.id,
                    run.status.as_str(),
                    run.finished_at_unix,
                    run.exit_code,
                    run.network.map(|value| value.as_str()),
                    run.power_source.map(|value| value.as_str()),
                    run.runner_pid,
                    run.message,
                ],
            )?;
            Ok(())
        })
    }

    pub fn get_run(&self, id: &str) -> Result<Option<JobRun>, AppError> {
        self.database.with_conn(|conn| {
            let sql = format!("SELECT {RUN_COLUMNS} FROM job_runs WHERE id = ?1");
            Ok(conn.query_row(&sql, params![id], read_run_row).optional()?)
        })
    }

    pub fn list_runs(&self, job_id: &str, limit: u32) -> Result<Vec<JobRun>, AppError> {
        self.database.with_conn(|conn| {
            let sql = format!(
                "SELECT {RUN_COLUMNS} FROM job_runs WHERE job_id = ?1
                 ORDER BY started_at_unix DESC, rowid DESC LIMIT ?2"
            );
            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt.query_map(params![job_id, limit], read_run_row)?;
            Ok(rows.collect::<Result<Vec<_>, _>>()?)
        })
    }

    pub fn list_running_runs(&self) -> Result<Vec<JobRun>, AppError> {
        self.database.with_conn(|conn| {
            let sql = format!("SELECT {RUN_COLUMNS} FROM job_runs WHERE status = 'running'");
            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt.query_map([], read_run_row)?;
            Ok(rows.collect::<Result<Vec<_>, _>>()?)
        })
    }
}

struct RawJob {
    id: String,
    name: String,
    command: String,
    working_dir: Option<String>,
    enabled: bool,
    schedule_json: String,
    policy: String,
    network: String,
    network_grace_seconds: u32,
    max_runtime_seconds: Option<u32>,
    created_at: i64,
    updated_at: i64,
}

fn read_job_row(row: &Row<'_>) -> rusqlite::Result<RawJob> {
    Ok(RawJob {
        id: row.get(0)?,
        name: row.get(1)?,
        command: row.get(2)?,
        working_dir: row.get(3)?,
        enabled: row.get(4)?,
        schedule_json: row.get(5)?,
        policy: row.get(6)?,
        network: row.get(7)?,
        network_grace_seconds: row.get(8)?,
        max_runtime_seconds: row.get(9)?,
        created_at: row.get(10)?,
        updated_at: row.get(11)?,
    })
}

fn job_from_raw(raw: RawJob) -> Result<Job, AppError> {
    let schedule: JobSchedule = serde_json::from_str(&raw.schedule_json)?;
    Ok(Job {
        policy: JobPolicy::parse(&raw.policy).unwrap_or(JobPolicy::Optimistic),
        network: NetworkPolicy::parse(&raw.network).unwrap_or(NetworkPolicy::BestEffort),
        id: raw.id,
        name: raw.name,
        command: raw.command,
        working_dir: raw.working_dir,
        enabled: raw.enabled,
        schedule,
        network_grace_seconds: raw.network_grace_seconds,
        max_runtime_seconds: raw.max_runtime_seconds,
        created_at: raw.created_at,
        updated_at: raw.updated_at,
    })
}

fn read_run_row(row: &Row<'_>) -> rusqlite::Result<JobRun> {
    let trigger: String = row.get(2)?;
    let status: String = row.get(3)?;
    let network: Option<String> = row.get(8)?;
    let power: Option<String> = row.get(9)?;
    Ok(JobRun {
        id: row.get(0)?,
        job_id: row.get(1)?,
        trigger: RunTrigger::parse(&trigger).unwrap_or(RunTrigger::Manual),
        status: JobRunStatus::parse(&status).unwrap_or(JobRunStatus::Interrupted),
        scheduled_for_unix: row.get(4)?,
        started_at_unix: row.get(5)?,
        finished_at_unix: row.get(6)?,
        exit_code: row.get(7)?,
        network: network.as_deref().and_then(NetworkOutcome::parse),
        power_source: power.as_deref().and_then(PowerSource::parse),
        runner_pid: row.get(10)?,
        log_path: row.get(11)?,
        message: row.get(12)?,
    })
}

fn run_params(run: &JobRun) -> Vec<Box<dyn rusqlite::ToSql + '_>> {
    vec![
        Box::new(&run.id),
        Box::new(&run.job_id),
        Box::new(run.trigger.as_str()),
        Box::new(run.status.as_str()),
        Box::new(run.scheduled_for_unix),
        Box::new(run.started_at_unix),
        Box::new(run.finished_at_unix),
        Box::new(run.exit_code),
        Box::new(run.network.map(|value| value.as_str())),
        Box::new(run.power_source.map(|value| value.as_str())),
        Box::new(run.runner_pid),
        Box::new(&run.log_path),
        Box::new(&run.message),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_repo() -> (JobRepository, std::path::PathBuf) {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("zashiki-jobs-{stamp}"));
        let db = Arc::new(Database::open(&dir).expect("open"));
        (JobRepository::new(db), dir)
    }

    fn sample_job() -> Job {
        Job {
            id: "job-1".into(),
            name: "Ingest".into(),
            command: "echo hi".into(),
            working_dir: None,
            enabled: true,
            schedule: JobSchedule::Daily { hour: 3, minute: 0 },
            policy: JobPolicy::Exact,
            network: NetworkPolicy::Required,
            network_grace_seconds: 90,
            max_runtime_seconds: Some(3600),
            created_at: 1,
            updated_at: 1,
        }
    }

    fn sample_run(id: &str, started: i64) -> JobRun {
        JobRun {
            id: id.into(),
            job_id: "job-1".into(),
            trigger: RunTrigger::Calendar,
            status: JobRunStatus::Running,
            scheduled_for_unix: Some(started),
            started_at_unix: started,
            finished_at_unix: None,
            exit_code: None,
            network: None,
            power_source: Some(PowerSource::Ac),
            runner_pid: Some(123),
            log_path: "/tmp/x.log".into(),
            message: None,
        }
    }

    #[test]
    fn round_trips_job_and_runs() {
        let (repo, dir) = temp_repo();
        let job = sample_job();
        repo.upsert_job(&job).expect("upsert");
        assert_eq!(repo.get_job("job-1").expect("get"), Some(job.clone()));

        repo.insert_run(&sample_run("r1", 10)).expect("run 1");
        let mut second = sample_run("r2", 20);
        repo.insert_run(&second).expect("run 2");
        second.status = JobRunStatus::Succeeded;
        second.exit_code = Some(0);
        repo.update_run(&second).expect("update");

        let runs = repo.list_runs("job-1", 10).expect("runs");
        assert_eq!(runs.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(), ["r2", "r1"]);
        assert_eq!(runs[0].status, JobRunStatus::Succeeded);
        assert_eq!(repo.list_running_runs().expect("running").len(), 1);

        assert!(repo.delete_job("job-1").expect("delete"));
        assert!(repo.list_runs("job-1", 10).expect("runs").is_empty());
        let _ = std::fs::remove_dir_all(dir);
    }
}
