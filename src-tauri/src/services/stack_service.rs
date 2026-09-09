use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::domain::{
    ComposeDbService, ComposeFileInfo, DbEndpoint, PortConflict, PortOccupant, ProjectStack,
    ReconcileAction,
};
use crate::error::AppError;
use crate::repositories::ProjectRepository;
use crate::services::catalog_service::CatalogService;
use crate::services::compose_detect::load_primary_compose;
use crate::services::compose_service::{stop_docker_container, ComposeService};
use crate::services::port_occupancy_service::PortOccupancyService;
use crate::services::uri_util::{
    build_db_uri, env_key_for_kind, mask_uri, parse_dotenv, rewrite_url_port, upsert_dotenv_value,
};

pub struct StackService {
    catalog: Arc<CatalogService>,
    repository: Arc<ProjectRepository>,
    occupancy: Arc<PortOccupancyService>,
    compose: Arc<ComposeService>,
}

impl StackService {
    pub fn new(
        catalog: Arc<CatalogService>,
        repository: Arc<ProjectRepository>,
        occupancy: Arc<PortOccupancyService>,
        compose: Arc<ComposeService>,
    ) -> Self {
        Self {
            catalog,
            repository,
            occupancy,
            compose,
        }
    }

    pub fn peek_stack(&self, project_id: &str) -> Result<ProjectStack, AppError> {
        let project = self.catalog.get_project(project_id)?;
        let Some(info) = load_primary_compose(Path::new(&project.path)) else {
            return Ok(ProjectStack {
                compose_file: None,
                endpoints: Vec::new(),
            });
        };
        let endpoints = self.endpoints_for(&project.path, project_id, &info, None)?;
        Ok(ProjectStack {
            compose_file: Some(info.relative_path),
            endpoints,
        })
    }

    pub fn get_stack(&self, project_id: &str) -> Result<ProjectStack, AppError> {
        let project = self.catalog.get_project(project_id)?;
        let Some(info) = load_primary_compose(Path::new(&project.path)) else {
            return Ok(ProjectStack {
                compose_file: None,
                endpoints: Vec::new(),
            });
        };
        let overlay = self.overlay_if_present(project_id);
        let running = self
            .compose
            .running_services(&project.path, &info.relative_path, overlay.as_deref())
            .unwrap_or_default();
        let endpoints = self.endpoints_for(&project.path, project_id, &info, Some(&running))?;
        Ok(ProjectStack {
            compose_file: Some(info.relative_path),
            endpoints,
        })
    }

    pub fn compose_logs(
        &self,
        project_id: &str,
        tail: u32,
    ) -> Result<Option<String>, AppError> {
        let project = self.catalog.get_project(project_id)?;
        let Some(info) = load_primary_compose(Path::new(&project.path)) else {
            return Ok(None);
        };
        let overlay = self.overlay_if_present(project_id);
        let output = self.compose.logs(
            &project.path,
            &info.relative_path,
            overlay.as_deref(),
            tail,
        )?;
        Ok(Some(output))
    }

    pub fn ensure_databases(&self, project_id: &str) -> Result<(), AppError> {
        let project = self.catalog.get_project(project_id)?;
        let Some(info) = load_primary_compose(Path::new(&project.path)) else {
            return Ok(());
        };
        if info.services.is_empty() {
            return Ok(());
        }
        // Always rewrite overlay from SQLite so older merge-style files
        // (which still published the original host port) get fixed.
        self.rewrite_overlay_file(project_id)?;
        if let Some(conflict) = self.find_conflict(project_id, &info)? {
            return Err(AppError::port_conflict(conflict));
        }
        let overlay = self.overlay_if_present(project_id);
        self.compose.up_databases(
            &project.path,
            &info.relative_path,
            overlay.as_deref(),
            &info.services,
        )
    }

    pub fn start_stack(&self, project_id: &str) -> Result<ProjectStack, AppError> {
        self.ensure_databases(project_id)?;
        self.get_stack(project_id)
    }

    pub fn stop_stack(&self, project_id: &str) -> Result<ProjectStack, AppError> {
        let project = self.catalog.get_project(project_id)?;
        let Some(info) = load_primary_compose(Path::new(&project.path)) else {
            return self.get_stack(project_id);
        };
        let overlay = self.overlay_if_present(project_id);
        self.compose.stop_databases(
            &project.path,
            &info.relative_path,
            overlay.as_deref(),
            &info.services,
        )?;
        self.get_stack(project_id)
    }

    pub fn resolve_from_scan(
        &self,
        project_id: &str,
        action: ReconcileAction,
        write_to_repo: bool,
        confirm_native: bool,
        conflict: Option<PortConflict>,
    ) -> Result<ProjectStack, AppError> {
        let project = self.catalog.get_project(project_id)?;
        let info = load_primary_compose(Path::new(&project.path))
            .ok_or_else(|| AppError::message("no compose file to reconcile"))?;
        let conflict = match conflict {
            Some(value) if value.project_id == project_id => value,
            _ => self.find_conflict(project_id, &info)?.ok_or_else(|| {
                AppError::message("no port conflict to resolve")
            })?,
        };
        self.resolve_conflict(
            project_id,
            &conflict,
            action,
            write_to_repo,
            confirm_native,
        )
    }

    pub fn resolve_conflict(
        &self,
        project_id: &str,
        conflict: &PortConflict,
        action: ReconcileAction,
        write_to_repo: bool,
        confirm_native: bool,
    ) -> Result<ProjectStack, AppError> {
        match action {
            ReconcileAction::StopOccupant => {
                self.stop_occupant(&conflict.occupant, confirm_native)?;
            }
            ReconcileAction::Remap => {
                self.apply_remap(project_id, conflict, write_to_repo)?;
            }
        }
        self.ensure_databases(project_id)?;
        self.get_stack(project_id)
    }

    pub fn spawn_export_prefix(&self, project_id: &str) -> Result<String, AppError> {
        let stack = self.peek_stack(project_id)?;
        if stack.endpoints.is_empty() {
            return Ok(String::new());
        }
        let mut parts = Vec::new();
        for endpoint in stack.endpoints {
            let key = env_key_for_kind(endpoint.kind);
            parts.push(format!(
                "export {}={}",
                key,
                shell_quote(&endpoint.uri)
            ));
        }
        Ok(parts.join("; "))
    }

    fn find_conflict(
        &self,
        project_id: &str,
        info: &ComposeFileInfo,
    ) -> Result<Option<PortConflict>, AppError> {
        let overrides = self.override_map(project_id)?;
        for service in &info.services {
            let host_port = overrides
                .get(&service.name)
                .map(|(host, _)| *host)
                .unwrap_or(service.mapping.host_port);
            let occupant = self.occupancy.identify_occupant(host_port)?;
            if matches!(occupant, PortOccupant::Unknown) {
                continue;
            }
            if let PortOccupant::CatalogProject {
                project_id: owner, ..
            } = &occupant
            {
                if owner == project_id {
                    continue;
                }
            }
            let suggested = self.occupancy.next_free_host_port(host_port)?;
            return Ok(Some(PortConflict {
                project_id: project_id.to_string(),
                service: service.name.clone(),
                host_port,
                suggested_port: suggested,
                occupant,
            }));
        }
        Ok(None)
    }

    fn stop_occupant(
        &self,
        occupant: &PortOccupant,
        confirm_native: bool,
    ) -> Result<(), AppError> {
        match occupant {
            PortOccupant::CatalogProject {
                project_id,
                service,
                ..
            } => {
                let project = self.catalog.get_project(project_id)?;
                let Some(info) = load_primary_compose(Path::new(&project.path)) else {
                    return Err(AppError::message(
                        "occupant project has no compose file to stop",
                    ));
                };
                let overlay = self.overlay_if_present(project_id);
                self.compose.stop_named_service(
                    &project.path,
                    &info.relative_path,
                    overlay.as_deref(),
                    service,
                )
            }
            PortOccupant::DockerOther { container_name, .. } => {
                stop_docker_container(container_name)
            }
            PortOccupant::NativeProcess { pid, command } => {
                if !confirm_native {
                    return Err(AppError::invalid(format!(
                        "port is held by native process pid {pid} ({command}); confirm to stop it"
                    )));
                }
                stop_native_process(*pid)
            }
            PortOccupant::Unknown => Err(AppError::message("cannot stop unknown occupant")),
        }
    }

    fn apply_remap(
        &self,
        project_id: &str,
        conflict: &PortConflict,
        write_to_repo: bool,
    ) -> Result<(), AppError> {
        let project = self.catalog.get_project(project_id)?;
        let info = load_primary_compose(Path::new(&project.path))
            .ok_or_else(|| AppError::message("no compose file for remap"))?;
        let service = info
            .services
            .iter()
            .find(|item| item.name == conflict.service)
            .ok_or_else(|| AppError::not_found(format!("service {}", conflict.service)))?;
        let host = conflict.suggested_port;
        let container = service.mapping.container_port;
        self.repository
            .upsert_port_override(project_id, &service.name, host, container)?;
        self.rewrite_overlay_file(project_id)?;
        if write_to_repo {
            write_repo_port_changes(&project.path, &info, service, host)?;
        }
        Ok(())
    }

    fn rewrite_overlay_file(&self, project_id: &str) -> Result<(), AppError> {
        let mappings = self.repository.list_port_overrides(project_id)?;
        self.compose.write_port_overlays(project_id, &mappings)?;
        Ok(())
    }

    fn endpoints_for(
        &self,
        project_path: &str,
        project_id: &str,
        info: &ComposeFileInfo,
        running: Option<&[String]>,
    ) -> Result<Vec<DbEndpoint>, AppError> {
        let overrides = self.override_map(project_id)?;
        let dotenv = read_dotenv(project_path);
        let mut endpoints = Vec::new();
        for service in &info.services {
            let port = overrides
                .get(&service.name)
                .map(|(host, _)| *host)
                .unwrap_or(service.mapping.host_port);
            let uri = effective_uri(service, port, &dotenv);
            let is_running = running
                .map(|names| names.iter().any(|name| name == &service.name))
                .unwrap_or(false);
            endpoints.push(DbEndpoint {
                service: service.name.clone(),
                kind: service.kind.clone(),
                host: "localhost".to_string(),
                port,
                user: service.user.clone(),
                database: service.database.clone(),
                uri_masked: mask_uri(&uri),
                uri,
                running: is_running,
            });
        }
        Ok(endpoints)
    }

    fn override_map(&self, project_id: &str) -> Result<HashMap<String, (u16, u16)>, AppError> {
        let mut map = HashMap::new();
        for (service, host, container) in self.repository.list_port_overrides(project_id)? {
            map.insert(service, (host, container));
        }
        Ok(map)
    }

    fn overlay_if_present(&self, project_id: &str) -> Option<PathBuf> {
        let path = self.compose.overlay_path(project_id);
        path.is_file().then_some(path)
    }
}

fn effective_uri(
    service: &ComposeDbService,
    port: u16,
    dotenv: &HashMap<String, String>,
) -> String {
    let key = env_key_for_kind(service.kind.clone());
    if let Some(raw) = dotenv.get(key) {
        return rewrite_url_port(raw, port);
    }
    build_db_uri(
        service.kind.clone(),
        service.user.as_deref(),
        service.password.as_deref(),
        port,
        service.database.as_deref(),
    )
}

fn read_dotenv(project_path: &str) -> HashMap<String, String> {
    let path = Path::new(project_path).join(".env");
    let Ok(raw) = fs::read_to_string(path) else {
        return HashMap::new();
    };
    parse_dotenv(&raw).into_iter().collect()
}

fn write_repo_port_changes(
    project_path: &str,
    info: &ComposeFileInfo,
    service: &ComposeDbService,
    host_port: u16,
) -> Result<(), AppError> {
    let env_path = Path::new(project_path).join(".env");
    if env_path.is_file() {
        let raw = fs::read_to_string(&env_path)?;
        let key = env_key_for_kind(service.kind.clone());
        let uri = if let Some((_, value)) = parse_dotenv(&raw).into_iter().find(|(k, _)| k == key) {
            rewrite_url_port(&value, host_port)
        } else {
            build_db_uri(
                service.kind.clone(),
                service.user.as_deref(),
                service.password.as_deref(),
                host_port,
                service.database.as_deref(),
            )
        };
        fs::write(&env_path, upsert_dotenv_value(&raw, key, &uri))?;
    }
    let compose_path = Path::new(project_path).join(&info.relative_path);
    if compose_path.is_file() {
        let raw = fs::read_to_string(&compose_path)?;
        let old = format!(
            "{}:{}",
            service.mapping.host_port, service.mapping.container_port
        );
        let next = format!("{host_port}:{}", service.mapping.container_port);
        if raw.contains(&old) {
            fs::write(&compose_path, raw.replacen(&old, &next, 1))?;
        }
    }
    Ok(())
}

fn stop_native_process(pid: i32) -> Result<(), AppError> {
    let result = unsafe { libc::kill(pid, libc::SIGTERM) };
    if result == 0 {
        Ok(())
    } else {
        Err(AppError::Io(format!(
            "failed to signal pid {pid}: {}",
            std::io::Error::last_os_error()
        )))
    }
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
