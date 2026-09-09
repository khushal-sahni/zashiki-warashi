use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tauri::{AppHandle, Emitter};
use tracing::warn;

use crate::domain::{LogChunk, LogSource};
use crate::error::AppError;

const DEFAULT_TAIL_LINES: usize = 2_000;
const READ_WINDOW_BYTES: u64 = 2 * 1024 * 1024;
const TAIL_INTERVAL: Duration = Duration::from_millis(200);
const EMIT_BATCH: usize = 500;

pub struct LogService {
    logs_root: PathBuf,
    offsets: Arc<Mutex<HashMap<String, u64>>>,
    emitter: Arc<Mutex<Option<AppHandle>>>,
}

impl LogService {
    pub fn new(logs_root: PathBuf) -> Self {
        let _ = fs::create_dir_all(&logs_root);
        Self {
            logs_root,
            offsets: Arc::new(Mutex::new(HashMap::new())),
            emitter: Arc::new(Mutex::new(None)),
        }
    }

    pub fn prepare_session(&self, project_id: &str) -> Result<PathBuf, AppError> {
        validate_project_id(project_id)?;
        let dir = self.project_dir(project_id);
        fs::create_dir_all(&dir)?;
        let current = current_log_path(&dir);
        rotate_if_needed(&current)?;
        File::create(&current)?;
        self.set_offset(project_id, 0);
        Ok(current)
    }

    pub fn read_tail(&self, project_id: &str, max_lines: Option<u32>) -> Result<LogChunk, AppError> {
        validate_project_id(project_id)?;
        let limit = max_lines
            .map(|value| value as usize)
            .unwrap_or(DEFAULT_TAIL_LINES)
            .max(1);
        let path = current_log_path(&self.project_dir(project_id));
        let (lines, truncated) = read_file_tail(&path, limit)?;
        Ok(LogChunk {
            project_id: project_id.to_string(),
            source: LogSource::Process,
            lines,
            truncated,
        })
    }

    pub fn clear(&self, project_id: &str) -> Result<(), AppError> {
        validate_project_id(project_id)?;
        let path = current_log_path(&self.project_dir(project_id));
        if path.exists() {
            OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(&path)?;
        }
        self.set_offset(project_id, 0);
        Ok(())
    }

    pub fn compose_logs_from_output(
        &self,
        project_id: &str,
        output: &str,
        max_lines: Option<u32>,
    ) -> Result<LogChunk, AppError> {
        let limit = max_lines.unwrap_or(DEFAULT_TAIL_LINES as u32).max(1);
        let (lines, truncated) = last_n_lines(output, limit as usize);
        Ok(LogChunk {
            project_id: project_id.to_string(),
            source: LogSource::Compose,
            lines,
            truncated,
        })
    }

    pub fn start_tailer(&self, app_handle: AppHandle) {
        if let Ok(mut slot) = self.emitter.lock() {
            *slot = Some(app_handle);
        }
        let root = self.logs_root.clone();
        let offsets = self.offsets.clone();
        let emitter = self.emitter.clone();
        thread::Builder::new()
            .name("zashiki-log-tailer".into())
            .spawn(move || run_tailer_loop(root, offsets, emitter))
            .ok();
    }

    fn project_dir(&self, project_id: &str) -> PathBuf {
        self.logs_root.join(project_id)
    }

    fn set_offset(&self, project_id: &str, offset: u64) {
        if let Ok(mut map) = self.offsets.lock() {
            map.insert(project_id.to_string(), offset);
        }
    }
}

fn run_tailer_loop(
    root: PathBuf,
    offsets: Arc<Mutex<HashMap<String, u64>>>,
    emitter: Arc<Mutex<Option<AppHandle>>>,
) {
    loop {
        thread::sleep(TAIL_INTERVAL);
        let handle = match emitter.lock() {
            Ok(guard) => guard.clone(),
            Err(_) => continue,
        };
        let Some(handle) = handle else {
            continue;
        };
        for (project_id, path) in list_current_logs(&root) {
            emit_new_lines(&handle, &offsets, &project_id, &path);
        }
    }
}

fn emit_new_lines(
    handle: &AppHandle,
    offsets: &Arc<Mutex<HashMap<String, u64>>>,
    project_id: &str,
    path: &Path,
) {
    let current_offset = offsets
        .lock()
        .ok()
        .and_then(|map| map.get(project_id).copied())
        .unwrap_or(0);
    let Ok((lines, next_offset)) = read_new_lines(path, current_offset) else {
        return;
    };
    if let Ok(mut map) = offsets.lock() {
        map.insert(project_id.to_string(), next_offset);
    }
    if lines.is_empty() {
        return;
    }
    for chunk in lines.chunks(EMIT_BATCH) {
        let payload = LogChunk {
            project_id: project_id.to_string(),
            source: LogSource::Process,
            lines: chunk.to_vec(),
            truncated: false,
        };
        if let Err(err) = handle.emit("project-log", &payload) {
            warn!(error = %err, "failed to emit project-log");
            break;
        }
    }
}

fn list_current_logs(root: &Path) -> Vec<(String, PathBuf)> {
    let Ok(entries) = fs::read_dir(root) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        if !kind.is_dir() {
            continue;
        }
        let id = entry.file_name().to_string_lossy().to_string();
        let path = current_log_path(&entry.path());
        if path.is_file() {
            out.push((id, path));
        }
    }
    out
}

fn current_log_path(dir: &Path) -> PathBuf {
    dir.join("current.log")
}

fn rotate_if_needed(current: &Path) -> Result<(), AppError> {
    if !current.exists() {
        return Ok(());
    }
    let len = fs::metadata(current)?.len();
    if len == 0 {
        return Ok(());
    }
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let archive = current
        .parent()
        .unwrap_or(current)
        .join(format!("{stamp}.log"));
    fs::rename(current, archive)?;
    Ok(())
}

fn read_file_tail(path: &Path, max_lines: usize) -> Result<(Vec<String>, bool), AppError> {
    if !path.exists() {
        return Ok((Vec::new(), false));
    }
    let mut file = File::open(path)?;
    let len = file.metadata()?.len();
    let start = len.saturating_sub(READ_WINDOW_BYTES);
    file.seek(SeekFrom::Start(start))?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    if start > 0 {
        if let Some(idx) = buf.find('\n') {
            buf = buf[idx + 1..].to_string();
        }
    }
    let (lines, line_truncated) = last_n_lines(&buf, max_lines);
    Ok((lines, start > 0 || line_truncated))
}

fn last_n_lines(text: &str, n: usize) -> (Vec<String>, bool) {
    let mut lines: Vec<&str> = text.split('\n').collect();
    if lines.last() == Some(&"") {
        lines.pop();
    }
    let truncated = lines.len() > n;
    let start = lines.len().saturating_sub(n);
    let kept = lines[start..].iter().map(|line| (*line).to_string()).collect();
    (kept, truncated)
}

fn read_new_lines(path: &Path, offset: u64) -> Result<(Vec<String>, u64), AppError> {
    let mut file = File::open(path)?;
    let len = file.metadata()?.len();
    if len < offset {
        return read_new_lines(path, 0);
    }
    if len == offset {
        return Ok((Vec::new(), offset));
    }
    file.seek(SeekFrom::Start(offset))?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    let Some(last_nl) = buf.iter().rposition(|byte| *byte == b'\n') else {
        return Ok((Vec::new(), offset));
    };
    let used = &buf[..=last_nl];
    let text = String::from_utf8_lossy(used);
    let lines = text.lines().map(str::to_string).collect();
    Ok((lines, offset + used.len() as u64))
}

fn validate_project_id(project_id: &str) -> Result<(), AppError> {
    if project_id.is_empty()
        || project_id.contains('/')
        || project_id.contains('\\')
        || project_id.contains("..")
    {
        return Err(AppError::invalid("invalid project id"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_logs(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("zashiki-logs-{name}-{stamp}"));
        fs::create_dir_all(&dir).expect("mkdir");
        dir
    }

    #[test]
    fn prepare_session_rotates_non_empty_current() {
        let root = temp_logs("rotate");
        let service = LogService::new(root.clone());
        let first = service.prepare_session("proj-a").expect("first");
        fs::write(&first, "old session\n").unwrap();
        let second = service.prepare_session("proj-a").expect("second");
        assert_eq!(first, second);
        assert_eq!(fs::read_to_string(&second).unwrap(), "");
        let dir = root.join("proj-a");
        let archives: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .ends_with(".log")
                    && entry.file_name() != "current.log"
            })
            .collect();
        assert_eq!(archives.len(), 1);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn read_tail_returns_last_n_lines() {
        let root = temp_logs("tail");
        let service = LogService::new(root.clone());
        let path = service.prepare_session("proj-b").expect("session");
        let body = (1..=10)
            .map(|n| format!("line-{n}"))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        fs::write(path, body).unwrap();
        let chunk = service.read_tail("proj-b", Some(3)).expect("tail");
        assert_eq!(chunk.lines, vec!["line-8", "line-9", "line-10"]);
        assert!(chunk.truncated);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn clear_truncates_current_and_keeps_archive() {
        let root = temp_logs("clear");
        let service = LogService::new(root.clone());
        let path = service.prepare_session("proj-c").expect("session");
        fs::write(&path, "keep me archived\n").unwrap();
        service.prepare_session("proj-c").expect("rotate");
        let current = root.join("proj-c").join("current.log");
        fs::write(&current, "wipe this\n").unwrap();
        service.clear("proj-c").expect("clear");
        assert_eq!(fs::read_to_string(&current).unwrap(), "");
        let archives: Vec<_> = fs::read_dir(root.join("proj-c"))
            .unwrap()
            .flatten()
            .filter(|entry| entry.file_name() != "current.log")
            .collect();
        assert_eq!(archives.len(), 1);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn last_n_lines_handles_trailing_newline() {
        let (lines, truncated) = last_n_lines("a\nb\nc\n", 10);
        assert_eq!(lines, vec!["a", "b", "c"]);
        assert!(!truncated);
    }

    #[test]
    fn rejects_path_traversal_ids() {
        let root = temp_logs("id");
        let service = LogService::new(root.clone());
        assert!(service.prepare_session("../etc").is_err());
        let _ = fs::remove_dir_all(root);
    }
}
