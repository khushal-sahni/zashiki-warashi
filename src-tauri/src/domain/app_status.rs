use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStatus {
    pub name: String,
    pub version: String,
    pub database_ready: bool,
    pub app_data_dir: String,
    pub schema_version: i64,
}
