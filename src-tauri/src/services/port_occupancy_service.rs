use std::net::TcpListener;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use crate::domain::PortOccupant;
use crate::error::AppError;
use crate::repositories::ProjectRepository;

#[derive(Debug, Clone)]
pub struct ListenerInfo {
    pub pid: i32,
    pub command: String,
}

#[derive(Debug, Clone)]
pub struct DockerPublish {
    pub container_name: String,
    pub service: Option<String>,
    pub working_dir: Option<String>,
    pub host_port: u16,
}

pub trait OccupancyProbe: Send + Sync {
    fn listeners_on_port(&self, port: u16) -> Result<Vec<ListenerInfo>, AppError>;
    fn docker_publishes(&self) -> Result<Vec<DockerPublish>, AppError>;
}

pub struct SystemOccupancyProbe;

impl OccupancyProbe for SystemOccupancyProbe {
    fn listeners_on_port(&self, port: u16) -> Result<Vec<ListenerInfo>, AppError> {
        lsof_listeners(port)
    }

    fn docker_publishes(&self) -> Result<Vec<DockerPublish>, AppError> {
        docker_ps_publishes()
    }
}

pub struct PortOccupancyService {
    probe: Arc<dyn OccupancyProbe>,
    repository: Arc<ProjectRepository>,
}

impl PortOccupancyService {
    pub fn new(repository: Arc<ProjectRepository>) -> Self {
        Self {
            probe: Arc::new(SystemOccupancyProbe),
            repository,
        }
    }

    #[cfg(test)]
    pub fn with_probe(repository: Arc<ProjectRepository>, probe: Arc<dyn OccupancyProbe>) -> Self {
        Self { probe, repository }
    }

    pub fn is_port_free(&self, port: u16) -> Result<bool, AppError> {
        Ok(self.probe.listeners_on_port(port)?.is_empty())
    }

    pub fn next_free_host_port(&self, preferred: u16) -> Result<u16, AppError> {
        let start = preferred.saturating_add(1).max(1024);
        for port in start..=65_000 {
            if self.is_port_free(port)? && can_bind_locally(port) {
                return Ok(port);
            }
        }
        Err(AppError::message("no free host port available"))
    }

    pub fn identify_occupant(&self, port: u16) -> Result<PortOccupant, AppError> {
        let publishes = self.probe.docker_publishes()?;
        if let Some(docker) = publishes.into_iter().find(|item| item.host_port == port) {
            return self.match_docker_occupant(docker);
        }
        let listeners = self.probe.listeners_on_port(port)?;
        if let Some(listener) = listeners.first() {
            return Ok(PortOccupant::NativeProcess {
                pid: listener.pid,
                command: listener.command.clone(),
            });
        }
        Ok(PortOccupant::Unknown)
    }

    fn match_docker_occupant(&self, docker: DockerPublish) -> Result<PortOccupant, AppError> {
        if let Some(working_dir) = docker.working_dir.as_ref() {
            if let Some(project) = self.match_catalog_path(working_dir)? {
                return Ok(PortOccupant::CatalogProject {
                    project_id: project.0,
                    name: project.1,
                    service: docker
                        .service
                        .clone()
                        .unwrap_or_else(|| "database".to_string()),
                });
            }
        }
        Ok(PortOccupant::DockerOther {
            container_name: docker.container_name,
            service: docker.service,
        })
    }

    fn match_catalog_path(&self, working_dir: &str) -> Result<Option<(String, String)>, AppError> {
        let needle = canonicalize_lossy(Path::new(working_dir));
        for project in self.repository.list_projects()? {
            let project_path = canonicalize_lossy(Path::new(&project.path));
            if needle == project_path
                || needle.starts_with(&format!("{project_path}/"))
                || project_path.starts_with(&format!("{needle}/"))
            {
                return Ok(Some((project.id, project.name)));
            }
        }
        Ok(None)
    }
}

fn can_bind_locally(port: u16) -> bool {
    TcpListener::bind(("127.0.0.1", port)).is_ok()
}

fn canonicalize_lossy(path: &Path) -> String {
    path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .to_string()
}

fn lsof_listeners(port: u16) -> Result<Vec<ListenerInfo>, AppError> {
    let output = Command::new("lsof")
        .args([
            "-nP",
            &format!("-iTCP:{port}"),
            "-sTCP:LISTEN",
            "-Fpc",
        ])
        .output()
        .map_err(|err| AppError::Io(format!("lsof failed: {err}")))?;
    if !output.status.success() && output.stdout.is_empty() {
        return Ok(Vec::new());
    }
    Ok(parse_lsof_output(&String::from_utf8_lossy(&output.stdout)))
}

fn parse_lsof_output(raw: &str) -> Vec<ListenerInfo> {
    let mut out = Vec::new();
    let mut pid: Option<i32> = None;
    let mut command = String::new();
    for line in raw.lines() {
        if let Some(rest) = line.strip_prefix('p') {
            if let (Some(prev_pid), true) = (pid, !command.is_empty()) {
                out.push(ListenerInfo {
                    pid: prev_pid,
                    command: command.clone(),
                });
            }
            pid = rest.parse().ok();
            command.clear();
        } else if let Some(rest) = line.strip_prefix('c') {
            command = rest.to_string();
        }
    }
    if let (Some(prev_pid), true) = (pid, !command.is_empty()) {
        out.push(ListenerInfo {
            pid: prev_pid,
            command,
        });
    }
    out
}

fn docker_ps_publishes() -> Result<Vec<DockerPublish>, AppError> {
    let Ok(mut command) = crate::services::docker_bin::docker_command() else {
        return Ok(Vec::new());
    };
    let format = "{{.Names}}\t{{.Label \"com.docker.compose.service\"}}\t{{.Label \"com.docker.compose.project.working_dir\"}}\t{{.Ports}}";
    let output = command
        .args(["ps", "--format", format])
        .output()
        .map_err(|err| AppError::Io(format!("docker ps failed: {err}")))?;
    if !output.status.success() {
        return Ok(Vec::new());
    }
    Ok(parse_docker_ps(&String::from_utf8_lossy(&output.stdout)))
}

fn parse_docker_ps(raw: &str) -> Vec<DockerPublish> {
    let mut out = Vec::new();
    for line in raw.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 4 {
            continue;
        }
        let name = parts[0].trim();
        if name.is_empty() {
            continue;
        }
        let service = non_empty(parts[1]);
        let working_dir = non_empty(parts[2]);
        for host_port in extract_published_ports(parts[3]) {
            out.push(DockerPublish {
                container_name: name.to_string(),
                service: service.clone(),
                working_dir: working_dir.clone(),
                host_port,
            });
        }
    }
    out
}

fn non_empty(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn extract_published_ports(ports: &str) -> Vec<u16> {
    let mut found = Vec::new();
    for chunk in ports.split(',') {
        let chunk = chunk.trim();
        if let Some((left, _)) = chunk.split_once("->") {
            let host = left.rsplit(':').next().unwrap_or(left);
            if let Ok(port) = host.parse::<u16>() {
                found.push(port);
            }
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::Database;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct FakeProbe {
        listeners: Mutex<HashMap<u16, Vec<ListenerInfo>>>,
        docker: Mutex<Vec<DockerPublish>>,
    }

    impl OccupancyProbe for FakeProbe {
        fn listeners_on_port(&self, port: u16) -> Result<Vec<ListenerInfo>, AppError> {
            Ok(self
                .listeners
                .lock()
                .unwrap()
                .get(&port)
                .cloned()
                .unwrap_or_default())
        }

        fn docker_publishes(&self) -> Result<Vec<DockerPublish>, AppError> {
            Ok(self.docker.lock().unwrap().clone())
        }
    }

    fn temp_repo() -> (std::path::PathBuf, Arc<ProjectRepository>) {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("zashiki-occ-{stamp}"));
        std::fs::create_dir_all(&dir).unwrap();
        let db = Arc::new(Database::open(&dir).unwrap());
        (dir, Arc::new(ProjectRepository::new(db)))
    }

    #[test]
    fn next_free_host_port_skips_occupied() {
        let (dir, repo) = temp_repo();
        let probe = Arc::new(FakeProbe {
            listeners: Mutex::new(HashMap::from([(
                5433,
                vec![ListenerInfo {
                    pid: 1,
                    command: "postgres".into(),
                }],
            )])),
            docker: Mutex::new(Vec::new()),
        });
        let service = PortOccupancyService::with_probe(repo, probe);
        let next = service.next_free_host_port(5432).unwrap();
        assert_ne!(next, 5433);
        assert!(next > 5432);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn matches_catalog_project_from_working_dir() {
        let (dir, repo) = temp_repo();
        let project_dir = dir.join("job-radar");
        std::fs::create_dir_all(&project_dir).unwrap();
        let row = crate::repositories::ProjectRow {
            id: "proj-1".into(),
            name: "job-radar".into(),
            path: project_dir.to_string_lossy().to_string(),
            start_command: None,
            stop_command: None,
            created_at: "1".into(),
            updated_at: "1".into(),
        };
        repo.insert_project(&row).unwrap();
        let probe = Arc::new(FakeProbe {
            listeners: Mutex::new(HashMap::new()),
            docker: Mutex::new(vec![DockerPublish {
                container_name: "job-radar-postgres".into(),
                service: Some("postgres".into()),
                working_dir: Some(project_dir.to_string_lossy().to_string()),
                host_port: 5432,
            }]),
        });
        let service = PortOccupancyService::with_probe(repo, probe);
        let occupant = service.identify_occupant(5432).unwrap();
        assert!(matches!(
            occupant,
            PortOccupant::CatalogProject {
                ref name,
                ref service,
                ..
            } if name == "job-radar" && service == "postgres"
        ));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn parse_lsof_and_docker_helpers() {
        let listeners = parse_lsof_output("p42\ncpostgres\np99\ncnode\n");
        assert_eq!(listeners.len(), 2);
        assert_eq!(listeners[0].pid, 42);
        let docker = parse_docker_ps(
            "aurum-postgres\tpostgres\t/tmp/aurum\t0.0.0.0:5432->5432/tcp, :::5432->5432/tcp\n",
        );
        assert_eq!(docker[0].host_port, 5432);
        assert_eq!(docker[0].service.as_deref(), Some("postgres"));
    }
}
