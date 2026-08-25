use std::fs;
use std::path::Path;

/// Infer a default start command from project files.
/// Prefer package.json scripts `dev` then `start`; else docker compose;
/// else make targets; else cargo/go.
pub fn infer_start_command(project_path: &Path) -> Option<String> {
    if let Some(cmd) = infer_from_package_json(project_path) {
        return Some(cmd);
    }
    if has_compose(project_path) {
        return Some("docker compose up".to_string());
    }
    if let Some(cmd) = infer_from_makefile(project_path) {
        return Some(cmd);
    }
    if project_path.join("Cargo.toml").is_file() {
        return Some("cargo run".to_string());
    }
    if project_path.join("go.mod").is_file() {
        return Some("go run .".to_string());
    }
    None
}

pub fn is_project_candidate(path: &Path) -> bool {
    if !path.is_dir() {
        return false;
    }
    path.join("package.json").is_file()
        || path.join("docker-compose.yml").is_file()
        || path.join("docker-compose.yaml").is_file()
        || path.join("compose.yml").is_file()
        || path.join("compose.yaml").is_file()
        || path.join("Cargo.toml").is_file()
        || path.join("go.mod").is_file()
        || path.join("Makefile").is_file()
        || path.join("makefile").is_file()
        || path.join(".git").exists()
}

fn has_compose(project_path: &Path) -> bool {
    project_path.join("docker-compose.yml").is_file()
        || project_path.join("docker-compose.yaml").is_file()
        || project_path.join("compose.yml").is_file()
        || project_path.join("compose.yaml").is_file()
}

fn infer_from_package_json(project_path: &Path) -> Option<String> {
    let path = project_path.join("package.json");
    let raw = fs::read_to_string(path).ok()?;
    let value: serde_json::Value = serde_json::from_str(&raw).ok()?;
    let scripts = value.get("scripts")?.as_object()?;
    if scripts.contains_key("dev") {
        return Some("npm run dev".to_string());
    }
    if scripts.contains_key("start") {
        return Some("npm start".to_string());
    }
    None
}

fn infer_from_makefile(project_path: &Path) -> Option<String> {
    let makefile = ["Makefile", "makefile"]
        .iter()
        .map(|name| project_path.join(name))
        .find(|path| path.is_file())?;
    let raw = fs::read_to_string(makefile).ok()?;
    if makefile_has_target(&raw, "dev") {
        return Some("make dev".to_string());
    }
    if makefile_has_target(&raw, "run") {
        return Some("make run".to_string());
    }
    None
}

fn makefile_has_target(contents: &str, target: &str) -> bool {
    let needle = format!("{target}:");
    contents.lines().any(|line| {
        let trimmed = line.trim_start();
        !trimmed.starts_with('#') && trimmed.starts_with(&needle)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_project(name: &str) -> std::path::PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("zashiki-infer-{name}-{stamp}"));
        fs::create_dir_all(&dir).expect("mkdir");
        dir
    }

    #[test]
    fn prefers_npm_dev_script() {
        let dir = temp_project("npm-dev");
        fs::write(
            dir.join("package.json"),
            r#"{"scripts":{"dev":"vite","start":"node index.js"}}"#,
        )
        .unwrap();
        assert_eq!(
            infer_start_command(&dir).as_deref(),
            Some("npm run dev")
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn falls_back_to_npm_start() {
        let dir = temp_project("npm-start");
        fs::write(
            dir.join("package.json"),
            r#"{"scripts":{"start":"node index.js"}}"#,
        )
        .unwrap();
        assert_eq!(infer_start_command(&dir).as_deref(), Some("npm start"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn detects_compose() {
        let dir = temp_project("compose");
        fs::write(dir.join("compose.yaml"), "services: {}").unwrap();
        assert_eq!(
            infer_start_command(&dir).as_deref(),
            Some("docker compose up")
        );
        assert!(is_project_candidate(&dir));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn detects_makefile_dev() {
        let dir = temp_project("make");
        fs::write(dir.join("Makefile"), "dev:\n\techo hi\n").unwrap();
        assert_eq!(infer_start_command(&dir).as_deref(), Some("make dev"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn detects_cargo() {
        let dir = temp_project("cargo");
        fs::write(dir.join("Cargo.toml"), "[package]\nname=\"x\"\n").unwrap();
        assert_eq!(infer_start_command(&dir).as_deref(), Some("cargo run"));
        let _ = fs::remove_dir_all(dir);
    }
}
