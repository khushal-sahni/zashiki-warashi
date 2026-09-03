use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum DbKind {
    Postgres,
    Mysql,
    Mongo,
    Redis,
    Search,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PortMapping {
    pub host_port: u16,
    pub container_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ComposeDbService {
    pub name: String,
    pub kind: DbKind,
    pub image: String,
    pub mapping: PortMapping,
    pub user: Option<String>,
    pub database: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ComposeFileInfo {
    pub path: String,
    pub relative_path: String,
    pub services: Vec<ComposeDbService>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PortOccupant {
    CatalogProject {
        project_id: String,
        name: String,
        service: String,
    },
    DockerOther {
        container_name: String,
        service: Option<String>,
    },
    NativeProcess {
        pid: i32,
        command: String,
    },
    Unknown,
}

impl PortOccupant {
    pub fn label(&self) -> String {
        match self {
            Self::CatalogProject { name, service, .. } => format!("{name} ({service})"),
            Self::DockerOther {
                container_name,
                service,
            } => match service {
                Some(svc) => format!("{container_name} ({svc})"),
                None => container_name.clone(),
            },
            Self::NativeProcess { pid, command } => format!("pid {pid} ({command})"),
            Self::Unknown => "unknown process".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PortConflict {
    pub project_id: String,
    pub service: String,
    pub host_port: u16,
    pub suggested_port: u16,
    pub occupant: PortOccupant,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DbEndpoint {
    pub service: String,
    pub kind: DbKind,
    pub host: String,
    pub port: u16,
    pub user: Option<String>,
    pub database: Option<String>,
    pub uri_masked: String,
    pub uri: String,
    pub running: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectStack {
    pub compose_file: Option<String>,
    pub endpoints: Vec<DbEndpoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ReconcileAction {
    StopOccupant,
    Remap,
}
