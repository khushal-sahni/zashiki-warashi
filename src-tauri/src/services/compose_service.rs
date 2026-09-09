use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use crate::domain::ComposeDbService;
use crate::error::AppError;
use crate::services::docker_bin::docker_command;

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
        let mut rest = if overlay.is_some() {
            vec![
                "up".into(),
                "-d".into(),
                "--wait".into(),
                "--force-recreate".into(),
            ]
        } else {
            vec!["up".into(), "-d".into(), "--wait".into()]
        };
        rest.extend(names);
        run_compose(project_path, compose_rel, overlay, &rest)?;
        Ok(())
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
        let mut rest = vec!["stop".into()];
        rest.extend(service_names(services));
        run_compose(project_path, compose_rel, overlay, &rest)?;
        Ok(())
    }

    pub fn stop_named_service(
        &self,
        project_path: &str,
        compose_rel: &str,
        overlay: Option<&Path>,
        service: &str,
    ) -> Result<(), AppError> {
        run_compose(
            project_path,
            compose_rel,
            overlay,
            &["stop".into(), service.into()],
        )?;
        Ok(())
    }

    pub fn running_services(
        &self,
        project_path: &str,
        compose_rel: &str,
        overlay: Option<&Path>,
    ) -> Result<Vec<String>, AppError> {
        let output = run_compose(
            project_path,
            compose_rel,
            overlay,
            &[
                "ps".into(),
                "--status".into(),
                "running".into(),
                "--format".into(),
                "json".into(),
            ],
        )?;
        Ok(parse_running_service_names(&output))
    }

    pub fn logs(
        &self,
        project_path: &str,
        compose_rel: &str,
        overlay: Option<&Path>,
        tail: u32,
    ) -> Result<String, AppError> {
        run_compose(
            project_path,
            compose_rel,
            overlay,
            &[
                "logs".into(),
                "--no-color".into(),
                format!("--tail={tail}"),
            ],
        )
    }
}

fn service_names(services: &[ComposeDbService]) -> Vec<String> {
    services
        .iter()
        .map(|service| service.name.clone())
        .collect()
}

fn compose_args(compose_rel: &str, overlay: Option<&Path>, rest: &[String]) -> Vec<String> {
    let mut args = vec!["compose".into(), "-f".into(), compose_rel.to_string()];
    if let Some(path) = overlay {
        args.push("-f".into());
        args.push(path.to_string_lossy().into_owned());
    }
    args.extend(rest.iter().cloned());
    args
}

fn run_compose(
    cwd: &str,
    compose_rel: &str,
    overlay: Option<&Path>,
    rest: &[String],
) -> Result<String, AppError> {
    let args = compose_args(compose_rel, overlay, rest);
    let mut command = docker_command()?;
    let output = command
        .args(&args)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .output()
        .map_err(|err| AppError::Io(format!("failed to run docker {}: {err}", args.join(" "))))?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    if output.status.success() {
        return Ok(stdout);
    }
    let detail = [stderr.trim(), stdout.trim()]
        .into_iter()
        .find(|text| !text.is_empty())
        .unwrap_or("docker compose failed");
    Err(AppError::message(format_compose_failure(detail)))
}

fn format_compose_failure(detail: &str) -> String {
    format!("compose failed: {detail}")
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

pub fn stop_docker_container(name: &str) -> Result<(), AppError> {
    let mut command = docker_command()?;
    let status = command
        .args(["stop", name])
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
    fn compose_args_includes_overlay() {
        let overlay = PathBuf::from("/tmp/override.yml");
        let args = compose_args(
            "local/docker-compose.yml",
            Some(&overlay),
            &["up".into(), "-d".into(), "--wait".into(), "postgres".into()],
        );
        assert_eq!(
            args,
            vec![
                "compose",
                "-f",
                "local/docker-compose.yml",
                "-f",
                "/tmp/override.yml",
                "up",
                "-d",
                "--wait",
                "postgres",
            ]
        );
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
