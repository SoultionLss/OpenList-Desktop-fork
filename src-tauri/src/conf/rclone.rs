use serde::{Deserialize, Serialize};

/// Rclone configuration stored in settings (mainly for api_port which is no
/// longer needed) Kept for backward compatibility with existing settings files
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RcloneConfig {
    pub config: serde_json::Value,
    #[serde(default = "default_api_port")]
    pub api_port: u16,
}

fn default_api_port() -> u16 {
    45572
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
            api_port: 45572,
        }
    }
}
