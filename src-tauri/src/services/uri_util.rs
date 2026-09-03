use crate::domain::DbKind;

pub fn mask_secret(value: &str) -> String {
    if value.is_empty() {
        return value.to_string();
    }
    "••••".to_string()
}

pub fn rewrite_url_port(url: &str, port: u16) -> String {
    if let Some(at) = url.rfind('@') {
        let (head, tail) = url.split_at(at + 1);
        if let Some(slash) = tail.find('/') {
            let (hostport, rest) = tail.split_at(slash);
            return format!("{head}{}{rest}", replace_port(hostport, port));
        }
        return format!("{head}{}", replace_port(tail, port));
    }
    url.to_string()
}

pub fn build_db_uri(
    kind: DbKind,
    user: Option<&str>,
    password: Option<&str>,
    port: u16,
    database: Option<&str>,
) -> String {
    match kind {
        DbKind::Postgres => format!(
            "postgresql://{}:{}@localhost:{}/{}",
            user.unwrap_or("postgres"),
            password.unwrap_or(""),
            port,
            database.unwrap_or("postgres")
        ),
        DbKind::Mysql => format!(
            "mysql://{}:{}@localhost:{}/{}",
            user.unwrap_or("root"),
            password.unwrap_or(""),
            port,
            database.unwrap_or("mysql")
        ),
        DbKind::Mongo => match (user, password) {
            (Some(u), Some(p)) => format!("mongodb://{u}:{p}@localhost:{port}"),
            _ => format!("mongodb://localhost:{port}"),
        },
        DbKind::Redis => match password {
            Some(p) if !p.is_empty() => format!("redis://:{p}@localhost:{port}"),
            _ => format!("redis://localhost:{port}"),
        },
        DbKind::Search => format!("http://localhost:{port}"),
    }
}

pub fn mask_uri(uri: &str) -> String {
    if let Some(scheme_end) = uri.find("://") {
        let rest = &uri[scheme_end + 3..];
        if let Some(at) = rest.find('@') {
            let creds = &rest[..at];
            if let Some(colon) = creds.find(':') {
                let user = &creds[..colon];
                let masked = format!(
                    "{}{}{user}:{}@{}",
                    &uri[..scheme_end + 3],
                    "",
                    mask_secret("x"),
                    &rest[at + 1..]
                );
                return masked;
            }
        }
    }
    uri.to_string()
}

pub fn env_key_for_kind(kind: DbKind) -> &'static str {
    match kind {
        DbKind::Postgres => "DATABASE_URL",
        DbKind::Mysql => "DATABASE_URL",
        DbKind::Mongo => "MONGODB_URI",
        DbKind::Redis => "REDIS_URL",
        DbKind::Search => "ELASTICSEARCH_URL",
    }
}

pub fn parse_dotenv(raw: &str) -> Vec<(String, String)> {
    raw.lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                return None;
            }
            let (key, value) = trimmed.split_once('=')?;
            let value = value
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();
            Some((key.trim().to_string(), value))
        })
        .collect()
}

pub fn upsert_dotenv_value(raw: &str, key: &str, value: &str) -> String {
    let mut replaced = false;
    let mut lines: Vec<String> = raw
        .lines()
        .map(|line| {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') || !trimmed.starts_with(&format!("{key}=")) {
                return line.to_string();
            }
            replaced = true;
            format!("{key}={value}")
        })
        .collect();
    if !replaced {
        if !raw.is_empty() && !raw.ends_with('\n') {
            lines.push(String::new());
        }
        lines.push(format!("{key}={value}"));
    }
    let mut out = lines.join("\n");
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

fn replace_port(hostport: &str, port: u16) -> String {
    if let Some((host, _)) = hostport.rsplit_once(':') {
        if host.starts_with('[') {
            return format!("{host}]:{port}");
        }
        return format!("{host}:{port}");
    }
    format!("{hostport}:{port}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masks_password_in_uri() {
        let uri = "postgresql://aurum:secret@localhost:5432/aurum";
        let masked = mask_uri(uri);
        assert!(masked.contains("aurum:••••@localhost:5432/aurum"));
        assert!(!masked.contains("secret"));
    }

    #[test]
    fn rewrite_url_port_changes_host_port() {
        let next = rewrite_url_port("postgresql://aurum:aurum@localhost:5432/aurum", 5433);
        assert_eq!(next, "postgresql://aurum:aurum@localhost:5433/aurum");
    }

    #[test]
    fn upsert_dotenv_replaces_existing() {
        let raw = "FOO=1\nDATABASE_URL=postgresql://a:b@localhost:5432/db\n";
        let next = upsert_dotenv_value(raw, "DATABASE_URL", "postgresql://a:b@localhost:5433/db");
        assert!(next.contains("localhost:5433"));
        assert!(!next.contains("localhost:5432"));
    }
}
