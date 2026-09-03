use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::domain::ComposeDbService;
use crate::error::AppError;

pub struct ComposeService {
    overrides_root: PathBuf,
}

impl ComposeService {
    pub fn new(overrides_root: PathBuf) -> Self {
        let _ = fs::create_dir_all(&overrides_root);
        Self { overrides_root }
    }

    pub fn overlay_path(&self, project_id: &str) -> PathBuf {
        self.overrides_root.join(format!("{project_id}.yml"))
    }

    /// Write a Compose override that *replaces* published ports.
    /// Plain `ports:` merges/appends with the base file and still binds the old host port.
    pub fn write_port_overlays(
        &self,
        project_id: &str,
        mappings: &[(String, u16, u16)],
    ) -> Result<PathBuf, AppError> {
        let path = self.overlay_path(project_id);
        if mappings.is_empty() {
            let _ = fs::remove_file(&path);
            return Ok(path);
        }
        let mut yaml = String::from("services:\n");
        for (service, host_port, container_port) in mappings {
            yaml.push_str(&format!(
                "  {service}:\n    ports: !override\n      - \"{host_port}:{container_port}\"\n"
            ));
        }
        fs::write(&path, yaml)?;
        Ok(path)
    }

    pub fn up_databases(
        &self,
        project_path: &str,
        compose_rel: &str,
        overlay: Option<&Path>,
        services: &[ComposeDbService],
    ) -> Result<(), AppError> {
        if services.is_empty() {
            return Ok(());
        }
        let names = service_names(services);
        // Force recreate so a previously-created container still bound to the old
        // host port (from a failed start) picks up the remapped publish.
        let flags = if overlay.is_some() {
            "up -d --wait --force-recreate"
        } else {
            "up -d --wait"
        };
        let command = compose_command(compose_rel, overlay, &format!("{flags} {names}"));
        run_compose(project_path, &command)
    }

    pub fn stop_databases(
        &self,
        project_path: &str,
        compose_rel: &str,
        overlay: Option<&Path>,
        services: &[ComposeDbService],
    ) -> Result<(), AppError> {
        if services.is_empty() {
            return Ok(());
        }
        let names = service_names(services);
        let command = compose_command(compose_rel, overlay, &format!("stop {names}"));
        run_compose(project_path, &command)
    }

    pub fn stop_named_service(
        &self,
        project_path: &str,
        compose_rel: &str,
        overlay: Option<&Path>,
        service: &str,
    ) -> Result<(), AppError> {
        let command = compose_command(compose_rel, overlay, &format!("stop {service}"));
        run_compose(project_path, &command)
    }

    pub fn running_services(
        &self,
        project_path: &str,
        compose_rel: &str,
        overlay: Option<&Path>,
    ) -> Result<Vec<String>, AppError> {
        let command = compose_command(compose_rel, overlay, "ps --status running --format json");
        let output = run_compose_capture(project_path, &command)?;
        Ok(parse_running_service_names(&output))
    }

    pub fn logs_command(compose_rel: &str, overlay: Option<&Path>, tail: u32) -> String {
        compose_command(compose_rel, overlay, &format!("logs --no-color --tail {tail}"))
    }
}

fn service_names(services: &[ComposeDbService]) -> String {
    services
        .iter()
        .map(|service| service.name.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

fn compose_command(compose_rel: &str, overlay: Option<&Path>, rest: &str) -> String {
    let mut command = format!("docker compose -f {}", shell_quote(compose_rel));
    if let Some(path) = overlay {
        command.push_str(" -f ");
        command.push_str(&shell_quote(&path.to_string_lossy()));
    }
    command.push(' ');
    command.push_str(rest);
    command
}

fn run_compose(cwd: &str, command: &str) -> Result<(), AppError> {
    let output = run_compose_capture(cwd, command)?;
    if output.trim().is_empty() {
        return Ok(());
    }
    Ok(())
}

fn run_compose_capture(cwd: &str, command: &str) -> Result<String, AppError> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
    let output = Command::new(&shell)
        .args(["-lc", command])
        .current_dir(cwd)
        .stdin(Stdio::null())
        .output()
        .map_err(|err| AppError::Io(format!("failed to run `{command}`: {err}")))?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    if output.status.success() {
        return Ok(stdout);
    }
    let detail = [stderr.trim(), stdout.trim()]
        .into_iter()
        .find(|text| !text.is_empty())
        .unwrap_or("docker compose failed");
    Err(AppError::message(detail.to_string()))
}

fn parse_running_service_names(raw: &str) -> Vec<String> {
    let mut names = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) {
            if let Some(name) = value
                .get("Service")
                .or_else(|| value.get("Name"))
                .and_then(|item| item.as_str())
            {
                names.push(name.to_string());
            }
        }
    }
    names
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

pub fn stop_docker_container(name: &str) -> Result<(), AppError> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
    let command = format!("docker stop {}", shell_quote(name));
    let status = Command::new(&shell)
        .args(["-lc", &command])
        .stdin(Stdio::null())
        .status()
        .map_err(|err| AppError::Io(format!("docker stop failed: {err}")))?;
    if status.success() {
        Ok(())
    } else {
        Err(AppError::message(format!(
            "docker stop {name} exited with {status}"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compose_command_includes_overlay() {
        let overlay = PathBuf::from("/tmp/override.yml");
        let command = compose_command("local/docker-compose.yml", Some(&overlay), "up -d --wait postgres");
        assert!(command.contains("-f 'local/docker-compose.yml'"));
        assert!(command.contains("-f '/tmp/override.yml'"));
        assert!(command.contains("up -d --wait postgres"));
    }

    #[test]
    fn port_overlay_uses_override_tag() {
        let root = std::env::temp_dir().join(format!(
            "zashiki-overlay-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        let service = ComposeService::new(root.clone());
        let path = service
            .write_port_overlays("proj", &[("postgres".into(), 5433, 5432)])
            .unwrap();
        let body = fs::read_to_string(path).unwrap();
        assert!(body.contains("ports: !override"));
        assert!(body.contains("\"5433:5432\""));
        assert!(!body.contains("5432:5432"));
        let _ = fs::remove_dir_all(root);
    }
}
