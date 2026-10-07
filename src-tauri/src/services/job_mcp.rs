use std::io::{BufRead, Write};
use std::path::Path;

use serde_json::{json, Value};
use tracing::warn;

use crate::error::AppError;
use crate::services::job_socket::call;

const PROTOCOL_VERSION: &str = "2025-06-18";

const SCHEDULE_SCHEMA: &str = r#"{
  "type": "object",
  "description": "kind=interval needs minutes (5,10,15,20,30,60,120,180,240,360,480,720,1440). kind=daily needs hour+minute. kind=weekly needs weekdays (0=Sun..6=Sat), hour, minute. Local time.",
  "properties": {
    "kind": { "type": "string", "enum": ["interval", "daily", "weekly"] },
    "minutes": { "type": "integer" },
    "hour": { "type": "integer", "minimum": 0, "maximum": 23 },
    "minute": { "type": "integer", "minimum": 0, "maximum": 59 },
    "weekdays": { "type": "array", "items": { "type": "integer", "minimum": 0, "maximum": 6 } }
  },
  "required": ["kind"]
}"#;

/// Tool name → (socket method, description, input schema).
fn tools() -> Vec<(&'static str, &'static str, &'static str, Value)> {
    let schedule: Value = serde_json::from_str(SCHEDULE_SCHEMA).unwrap_or(Value::Null);
    let id_only = json!({ "type": "object", "properties": { "id": { "type": "string" } }, "required": ["id"] });
    vec![
        ("list_jobs", "list_jobs", "List scheduled jobs managed by Zashiki with next fire time and last run.", json!({ "type": "object", "properties": {} })),
        (
            "upsert_job",
            "upsert_job",
            "Create a job (omit id) or replace an existing one. Zashiki owns the schedule, wake, network wait and logs; the command owns the work. policy: optimistic (run on next wake if asleep) or exact (wake the Mac ~90s early; closed-lid only on AC). network: best_effort, required, or none.",
            json!({
                "type": "object",
                "properties": {
                    "id": { "type": "string" },
                    "job": {
                        "type": "object",
                        "properties": {
                            "name": { "type": "string" },
                            "command": { "type": "string", "description": "Run with the user's login shell (-lc)." },
                            "workingDir": { "type": "string" },
                            "enabled": { "type": "boolean" },
                            "schedule": schedule,
                            "policy": { "type": "string", "enum": ["optimistic", "exact"] },
                            "network": { "type": "string", "enum": ["bestEffort", "required", "none"] },
                            "networkGraceSeconds": { "type": "integer", "minimum": 0, "maximum": 600 },
                            "maxRuntimeSeconds": { "type": "integer", "minimum": 1 }
                        },
                        "required": ["name", "command", "schedule"]
                    }
                },
                "required": ["job"]
            }),
        ),
        ("set_job_enabled", "set_job_enabled", "Enable or pause a job.", json!({ "type": "object", "properties": { "id": { "type": "string" }, "enabled": { "type": "boolean" } }, "required": ["id", "enabled"] })),
        ("run_job", "run_job", "Start a job now, outside its schedule.", id_only),
        ("job_runs", "list_runs", "Recent runs for a job, newest first.", json!({ "type": "object", "properties": { "id": { "type": "string" }, "limit": { "type": "integer" } }, "required": ["id"] })),
        ("job_log", "run_log", "Tail of one run's log.", json!({ "type": "object", "properties": { "runId": { "type": "string" }, "lines": { "type": "integer" } }, "required": ["runId"] })),
    ]
}

fn tool_list() -> Value {
    let tools: Vec<Value> = tools()
        .into_iter()
        .map(|(name, _, description, schema)| json!({ "name": name, "description": description, "inputSchema": schema }))
        .collect();
    json!({ "tools": tools })
}

fn call_tool(socket: &Path, params: &Value) -> Value {
    let name = params.get("name").and_then(Value::as_str).unwrap_or_default();
    let arguments = params.get("arguments").cloned().unwrap_or_else(|| json!({}));
    let Some((_, method, _, _)) = tools().into_iter().find(|(tool, ..)| *tool == name) else {
        return tool_text(&format!("Unknown tool `{name}`"), true);
    };
    match call(socket, method, arguments) {
        Ok(result) => tool_text(&serde_json::to_string_pretty(&result).unwrap_or_default(), false),
        Err(err) => tool_text(&err.user_message(), true),
    }
}

fn tool_text(text: &str, is_error: bool) -> Value {
    json!({ "content": [{ "type": "text", "text": text }], "isError": is_error })
}

/// Handle one JSON-RPC message. `None` for notifications.
pub fn handle_message(socket: &Path, message: &Value) -> Option<Value> {
    let id = message.get("id")?.clone();
    let method = message.get("method").and_then(Value::as_str).unwrap_or_default();
    let params = message.get("params").cloned().unwrap_or(Value::Null);
    let result = match method {
        "initialize" => json!({
            "protocolVersion": params.get("protocolVersion").cloned().unwrap_or(json!(PROTOCOL_VERSION)),
            "capabilities": { "tools": {} },
            "serverInfo": { "name": "zashiki-jobs", "version": env!("CARGO_PKG_VERSION") }
        }),
        "ping" => json!({}),
        "tools/list" => tool_list(),
        "tools/call" => call_tool(socket, &params),
        other => {
            return Some(json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32601, "message": format!("method not found: {other}") } }))
        }
    };
    Some(json!({ "jsonrpc": "2.0", "id": id, "result": result }))
}

/// stdio MCP server: newline-delimited JSON-RPC on stdin/stdout.
pub fn serve_stdio(socket: &Path) -> Result<(), AppError> {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let message: Value = match serde_json::from_str(&line) {
            Ok(message) => message,
            Err(err) => {
                warn!(error = %err, "ignoring malformed MCP message");
                continue;
            }
        };
        if let Some(response) = handle_message(socket, &message) {
            writeln!(stdout, "{response}")?;
            stdout.flush()?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize_and_list_tools() {
        let socket = Path::new("/nonexistent.sock");
        let init = handle_message(socket, &json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {} })).expect("reply");
        assert_eq!(init["result"]["serverInfo"]["name"], "zashiki-jobs");
        let list = handle_message(socket, &json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" })).expect("reply");
        let names: Vec<&str> = list["result"]["tools"].as_array().expect("tools").iter().filter_map(|t| t["name"].as_str()).collect();
        assert!(names.contains(&"upsert_job"));
        assert!(names.contains(&"job_log"));
    }

    #[test]
    fn notifications_get_no_reply_and_offline_socket_is_tool_error() {
        let socket = Path::new("/nonexistent.sock");
        assert!(handle_message(socket, &json!({ "jsonrpc": "2.0", "method": "notifications/initialized" })).is_none());
        let reply = handle_message(
            socket,
            &json!({ "jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": { "name": "list_jobs", "arguments": {} } }),
        )
        .expect("reply");
        assert_eq!(reply["result"]["isError"], true);
    }
}
