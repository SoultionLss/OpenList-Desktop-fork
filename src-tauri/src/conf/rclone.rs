use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RcloneConfig {
    pub config: serde_json::Value,
    pub binary_path: Option<String>,
    pub rclone_conf_path: Option<String>,
}

impl Default for RcloneConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl RcloneConfig {
    pub fn new() -> Self {
        Self {
            config: serde_json::Value::Object(Default::default()),
            binary_path: None,
            rclone_conf_path: None,
        }
    }
}
