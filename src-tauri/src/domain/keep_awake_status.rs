use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct KeepAwakeStatus {
    pub enabled: bool,
    pub lid_closed_armed: bool,
    pub caffeinate_pid: Option<i32>,
}

impl KeepAwakeStatus {
    pub fn off() -> Self {
        Self {
            enabled: false,
            lid_closed_armed: false,
            caffeinate_pid: None,
        }
    }
}
