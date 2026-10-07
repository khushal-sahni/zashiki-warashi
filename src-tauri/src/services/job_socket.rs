use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{info, warn};

use crate::domain::JobInput;
use crate::error::AppError;
use crate::services::job_service::JobService;

const CLIENT_TIMEOUT: Duration = Duration::from_secs(15);

/// One JSON line in. `params` depends on `method`.
#[derive(Debug, Deserialize, Serialize)]
pub struct SocketRequest {
    #[serde(default)]
    pub id: Value,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

/// One JSON line out.
#[derive(Debug, Deserialize, Serialize)]
pub struct SocketResponse {
    pub id: Value,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct IdParams {
    id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpsertParams {
    #[serde(default)]
    id: Option<String>,
    job: JobInput,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct EnableParams {
    id: String,
    enabled: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RunsParams {
    id: String,
    #[serde(default)]
    limit: Option<u32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LogParams {
    run_id: String,
    #[serde(default)]
    lines: Option<usize>,
}

pub const METHODS: &[&str] = &[
    "list_jobs", "get_job", "upsert_job", "delete_job", "set_job_enabled", "run_job", "list_runs", "run_log",
];

fn parse<T: for<'de> Deserialize<'de>>(params: Value) -> Result<T, AppError> {
    serde_json::from_value(params).map_err(|err| AppError::invalid(format!("bad params: {err}")))
}

/// Route one request to `JobService`. Shared by the socket server and tests.
pub fn dispatch(service: &JobService, method: &str, params: Value) -> Result<Value, AppError> {
    let value = match method {
        "list_jobs" => serde_json::to_value(service.list()?)?,
        "get_job" => serde_json::to_value(service.get(&parse::<IdParams>(params)?.id)?)?,
        "upsert_job" => {
            let p: UpsertParams = parse(params)?;
            let saved = match p.id {
                Some(id) => service.update(&id, p.job)?,
                None => service.create(p.job)?,
            };
            serde_json::to_value(saved)?
        }
        "delete_job" => {
            service.delete(&parse::<IdParams>(params)?.id)?;
            json!({ "deleted": true })
        }
        "set_job_enabled" => {
            let p: EnableParams = parse(params)?;
            serde_json::to_value(service.set_enabled(&p.id, p.enabled)?)?
        }
        "run_job" => {
            service.run_now(&parse::<IdParams>(params)?.id)?;
            json!({ "started": true })
        }
        "list_runs" => {
            let p: RunsParams = parse(params)?;
            serde_json::to_value(service.list_runs(&p.id, p.limit)?)?
        }
        "run_log" => {
            let p: LogParams = parse(params)?;
            serde_json::to_value(service.log_tail(&p.run_id, p.lines)?)?
        }
        other => return Err(AppError::invalid(format!("unknown method `{other}`; try one of {}", METHODS.join(", ")))),
    };
    Ok(value)
}

fn respond(service: &JobService, line: &str) -> SocketResponse {
    let request: SocketRequest = match serde_json::from_str(line) {
        Ok(request) => request,
        Err(err) => {
            return SocketResponse { id: Value::Null, ok: false, result: None, error: Some(format!("invalid JSON: {err}")) }
        }
    };
    match dispatch(service, &request.method, request.params) {
        Ok(result) => SocketResponse { id: request.id, ok: true, result: Some(result), error: None },
        Err(err) => SocketResponse { id: request.id, ok: false, result: None, error: Some(err.user_message()) },
    }
}

/// Bind the socket (owner-only) and serve forever on background threads.
pub fn serve(service: Arc<JobService>, socket: &Path) -> Result<(), AppError> {
    if socket.exists() {
        fs::remove_file(socket)?;
    }
    let listener = UnixListener::bind(socket)?;
    fs::set_permissions(socket, fs::Permissions::from_mode(0o600))?;
    info!(socket = %socket.display(), "job socket listening");
    thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let service = service.clone();
                    thread::spawn(move || handle_connection(&service, stream));
                }
                Err(err) => warn!(error = %err, "job socket accept failed"),
            }
        }
    });
    Ok(())
}

fn handle_connection(service: &JobService, stream: UnixStream) {
    let Ok(mut writer) = stream.try_clone() else {
        return;
    };
    for line in BufReader::new(stream).lines().map_while(Result::ok) {
        if line.trim().is_empty() {
            continue;
        }
        let response = respond(service, &line);
        let Ok(body) = serde_json::to_string(&response) else {
            return;
        };
        if writeln!(writer, "{body}").is_err() {
            return;
        }
    }
}

/// Send one request and wait for its response.
pub fn call(socket: &Path, method: &str, params: Value) -> Result<Value, AppError> {
    let mut stream = UnixStream::connect(socket).map_err(|err| {
        AppError::message(format!(
            "Zashiki's job supervisor is not reachable at {} ({err}). Open Zashiki once so it can install it.",
            socket.display()
        ))
    })?;
    stream.set_read_timeout(Some(CLIENT_TIMEOUT))?;
    let request = SocketRequest { id: json!(1), method: method.to_string(), params };
    writeln!(stream, "{}", serde_json::to_string(&request)?)?;
    let mut line = String::new();
    BufReader::new(stream).read_line(&mut line)?;
    let response: SocketResponse = serde_json::from_str(&line)?;
    if response.ok {
        Ok(response.result.unwrap_or(Value::Null))
    } else {
        Err(AppError::message(response.error.unwrap_or_else(|| "request failed".into())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::{Database, JobRepository};
    use crate::services::job_paths::JobPaths;
    use crate::services::launchd_service::{LaunchctlRunner, LaunchdService};
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct NoopLaunchctl;

    impl LaunchctlRunner for NoopLaunchctl {
        fn bootstrap(&self, _plist: &Path) -> Result<(), AppError> {
            Ok(())
        }
        fn bootout(&self, _label: &str) {}
        fn list(&self) -> Result<String, AppError> {
            Ok(String::new())
        }
    }

    #[test]
    fn socket_round_trip_creates_and_lists() {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).expect("time").as_nanos();
        let dir = PathBuf::from(format!("/tmp/zw-sock-{stamp}"));
        let paths = JobPaths::new(&dir);
        let repo = Arc::new(JobRepository::new(Arc::new(Database::open(&dir).expect("db"))));
        let launchd = LaunchdService::new(dir.join("agents"), paths.clone(), Box::new(NoopLaunchctl));
        let service = Arc::new(JobService::new(repo, paths.clone(), launchd, PathBuf::from("/bin/z")));
        serve(service, &paths.socket()).expect("serve");

        let mode = fs::metadata(paths.socket()).expect("meta").permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);

        let created = call(
            &paths.socket(),
            "upsert_job",
            json!({ "job": { "name": "radar", "command": "echo hi", "schedule": { "kind": "daily", "hour": 6, "minute": 0 } } }),
        )
        .expect("create");
        assert_eq!(created["policy"], "optimistic");
        let listed = call(&paths.socket(), "list_jobs", Value::Null).expect("list");
        assert_eq!(listed.as_array().map(Vec::len), Some(1));
        assert!(call(&paths.socket(), "nope", Value::Null).is_err());
        let _ = fs::remove_dir_all(dir);
    }
}
