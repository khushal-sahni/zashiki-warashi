use std::fs;
use std::path::{Path, PathBuf};

use serde_yaml::Value;

use crate::domain::{ComposeDbService, ComposeFileInfo, DbKind, PortMapping};

const COMPOSE_NAMES: &[&str] = &[
    "compose.yaml",
    "compose.yml",
    "docker-compose.yaml",
    "docker-compose.yml",
];

const SKIP_DIRS: &[&str] = &["node_modules", ".git", "dist", "target", ".next"];

pub fn has_compose(project_path: &Path) -> bool {
    primary_compose_file(project_path).is_some()
}

pub fn primary_compose_file(project_path: &Path) -> Option<PathBuf> {
    find_compose_files(project_path).into_iter().next()
}

pub fn find_compose_files(project_path: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    push_compose_in_dir(project_path, &mut files);
    let Ok(entries) = fs::read_dir(project_path) else {
        return files;
    };
    let mut children: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir() && !is_skipped_dir(path))
        .collect();
    children.sort();
    for child in children {
        push_compose_in_dir(&child, &mut files);
    }
    files
}

pub fn parse_compose_file(project_path: &Path, compose_path: &Path) -> Option<ComposeFileInfo> {
    let raw = fs::read_to_string(compose_path).ok()?;
    let value: Value = serde_yaml::from_str(&raw).ok()?;
    let services = parse_db_services(&value);
    let relative = compose_path
        .strip_prefix(project_path)
        .unwrap_or(compose_path)
        .to_string_lossy()
        .to_string();
    Some(ComposeFileInfo {
        path: compose_path.to_string_lossy().to_string(),
        relative_path: relative,
        services,
    })
}

pub fn load_primary_compose(project_path: &Path) -> Option<ComposeFileInfo> {
    let path = primary_compose_file(project_path)?;
    parse_compose_file(project_path, &path)
}

pub fn classify_db_kind(image: &str, service: &str) -> Option<DbKind> {
    let hay = format!("{image} {service}").to_lowercase();
    if hay.contains("postgres") || hay.contains("postgis") {
        return Some(DbKind::Postgres);
    }
    if hay.contains("mysql") || hay.contains("mariadb") {
        return Some(DbKind::Mysql);
    }
    if hay.contains("mongo") {
        return Some(DbKind::Mongo);
    }
    if hay.contains("redis") || hay.contains("keydb") || hay.contains("valkey") {
        return Some(DbKind::Redis);
    }
    if hay.contains("elasticsearch") || hay.contains("opensearch") {
        return Some(DbKind::Search);
    }
    None
}

pub fn parse_port_mapping(spec: &str) -> Option<PortMapping> {
    let trimmed = spec.trim().trim_matches('"').trim_end_matches("/tcp");
    let parts: Vec<&str> = trimmed.split(':').collect();
    match parts.as_slice() {
        [host, container] => Some(PortMapping {
            host_port: host.parse().ok()?,
            container_port: container.parse().ok()?,
        }),
        [_, host, container] => Some(PortMapping {
            host_port: host.parse().ok()?,
            container_port: container.parse().ok()?,
        }),
        [port] => {
            let parsed = port.parse().ok()?;
            Some(PortMapping {
                host_port: parsed,
                container_port: parsed,
            })
        }
        _ => None,
    }
}

fn push_compose_in_dir(dir: &Path, files: &mut Vec<PathBuf>) {
    for name in COMPOSE_NAMES {
        let candidate = dir.join(name);
        if candidate.is_file() {
            files.push(candidate);
            return;
        }
    }
}

fn is_skipped_dir(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| SKIP_DIRS.contains(&name))
}

fn parse_db_services(root: &Value) -> Vec<ComposeDbService> {
    let Some(services) = root.get("services").and_then(Value::as_mapping) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (key, spec) in services {
        let Some(name) = key.as_str() else {
            continue;
        };
        let image = spec
            .get("image")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let Some(kind) = classify_db_kind(&image, name) else {
            continue;
        };
        let Some(mapping) = first_port_mapping(spec) else {
            continue;
        };
        let env = flatten_environment(spec);
        out.push(ComposeDbService {
            name: name.to_string(),
            kind,
            image,
            mapping,
            user: env_value(&env, kind_user_keys(kind)),
            database: env_value(&env, kind_db_keys(kind)),
            password: env_value(&env, kind_password_keys(kind)),
        });
    }
    out
}

fn first_port_mapping(spec: &Value) -> Option<PortMapping> {
    let ports = spec.get("ports")?;
    if let Some(seq) = ports.as_sequence() {
        for item in seq {
            if let Some(mapping) = port_from_value(item) {
                return Some(mapping);
            }
        }
    }
    None
}

fn port_from_value(value: &Value) -> Option<PortMapping> {
    if let Some(text) = value.as_str() {
        return parse_port_mapping(text);
    }
    if let Some(number) = value.as_u64() {
        let port = u16::try_from(number).ok()?;
        return Some(PortMapping {
            host_port: port,
            container_port: port,
        });
    }
    let map = value.as_mapping()?;
    let published = map.get(Value::from("published"))?;
    let target = map
        .get(Value::from("target"))
        .or_else(|| map.get(Value::from("container_port")));
    let host = yaml_u16(published)?;
    let container = target.and_then(yaml_u16).unwrap_or(host);
    Some(PortMapping {
        host_port: host,
        container_port: container,
    })
}

fn yaml_u16(value: &Value) -> Option<u16> {
    if let Some(n) = value.as_u64() {
        return u16::try_from(n).ok();
    }
    value.as_str()?.parse().ok()
}

fn flatten_environment(spec: &Value) -> Vec<(String, String)> {
    let Some(env) = spec.get("environment") else {
        return Vec::new();
    };
    if let Some(map) = env.as_mapping() {
        return map
            .iter()
            .filter_map(|(key, value)| {
                let name = key.as_str()?.to_string();
                let val = match value {
                    Value::String(text) => text.clone(),
                    Value::Number(num) => num.to_string(),
                    Value::Bool(flag) => flag.to_string(),
                    _ => return None,
                };
                Some((name, val))
            })
            .collect();
    }
    if let Some(seq) = env.as_sequence() {
        return seq
            .iter()
            .filter_map(Value::as_str)
            .filter_map(|item| {
                let (name, val) = item.split_once('=')?;
                Some((name.to_string(), val.to_string()))
            })
            .collect();
    }
    Vec::new()
}

fn env_value(env: &[(String, String)], keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some((_, value)) = env.iter().find(|(name, _)| name == key) {
            if !value.is_empty() {
                return Some(value.clone());
            }
        }
    }
    None
}

fn kind_user_keys(kind: DbKind) -> &'static [&'static str] {
    match kind {
        DbKind::Postgres => &["POSTGRES_USER"],
        DbKind::Mysql => &["MYSQL_USER", "MARIADB_USER"],
        DbKind::Mongo => &["MONGO_INITDB_ROOT_USERNAME"],
        DbKind::Redis | DbKind::Search => &[],
    }
}

fn kind_db_keys(kind: DbKind) -> &'static [&'static str] {
    match kind {
        DbKind::Postgres => &["POSTGRES_DB"],
        DbKind::Mysql => &["MYSQL_DATABASE", "MARIADB_DATABASE"],
        DbKind::Mongo => &["MONGO_INITDB_DATABASE"],
        DbKind::Redis | DbKind::Search => &[],
    }
}

fn kind_password_keys(kind: DbKind) -> &'static [&'static str] {
    match kind {
        DbKind::Postgres => &["POSTGRES_PASSWORD"],
        DbKind::Mysql => &["MYSQL_PASSWORD", "MYSQL_ROOT_PASSWORD", "MARIADB_PASSWORD"],
        DbKind::Mongo => &["MONGO_INITDB_ROOT_PASSWORD"],
        DbKind::Redis => &["REDIS_PASSWORD"],
        DbKind::Search => &[],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_project(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("zashiki-compose-{name}-{stamp}"));
        fs::create_dir_all(&dir).expect("mkdir");
        dir
    }

    #[test]
    fn finds_nested_local_compose() {
        let dir = temp_project("nested");
        let local = dir.join("local");
        fs::create_dir_all(&local).unwrap();
        fs::write(local.join("docker-compose.yml"), "services: {}\n").unwrap();
        let found = find_compose_files(&dir);
        assert_eq!(found.len(), 1);
        assert!(found[0].ends_with("local/docker-compose.yml"));
        assert!(has_compose(&dir));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn skips_node_modules_compose() {
        let dir = temp_project("skip");
        let nested = dir.join("node_modules").join("pkg");
        fs::create_dir_all(&nested).unwrap();
        fs::write(nested.join("compose.yml"), "services: {}\n").unwrap();
        assert!(find_compose_files(&dir).is_empty());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn classifies_postgres_and_ignores_web() {
        assert_eq!(
            classify_db_kind("postgres:16-alpine", "postgres"),
            Some(DbKind::Postgres)
        );
        assert_eq!(classify_db_kind("node:22", "web"), None);
    }

    #[test]
    fn parses_aurum_style_compose() {
        let dir = temp_project("aurum-style");
        let local = dir.join("local");
        fs::create_dir_all(&local).unwrap();
        fs::write(
            local.join("docker-compose.yml"),
            r#"
services:
  postgres:
    image: postgres:16-alpine
    ports:
      - "5432:5432"
    environment:
      POSTGRES_USER: aurum
      POSTGRES_PASSWORD: aurum
      POSTGRES_DB: aurum
  web:
    image: node:22
    ports:
      - "3000:3000"
"#,
        )
        .unwrap();
        let info = load_primary_compose(&dir).expect("parse");
        assert_eq!(info.services.len(), 1);
        assert_eq!(info.services[0].name, "postgres");
        assert_eq!(info.services[0].mapping.host_port, 5432);
        assert_eq!(info.services[0].mapping.container_port, 5432);
        assert_eq!(info.services[0].user.as_deref(), Some("aurum"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn parse_port_mapping_host_container() {
        let mapping = parse_port_mapping("127.0.0.1:5433:5432").unwrap();
        assert_eq!(mapping.host_port, 5433);
        assert_eq!(mapping.container_port, 5432);
    }
}
