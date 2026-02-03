use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RcloneMountConfig {
    pub name: String,
    pub r#type: String,
    pub url: String,
    pub vendor: Option<String>,
    pub user: String,
    pub pass: String,
    #[serde(rename = "mountPoint")]
    pub mount_point: Option<String>,
    #[serde(rename = "volumeName")]
    pub volume_name: Option<String>,
    #[serde(rename = "extraFlags")]
    pub extra_flags: Option<Vec<String>>,
    #[serde(rename = "autoMount")]
    pub auto_mount: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RcloneConfig {
    pub mount_config: Option<HashMap<String, RcloneMountConfig>>,
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
            mount_config: Some(HashMap::new()),
            binary_path: None,
            rclone_conf_path: None,
        }
    }
}
