use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub theme: Option<String>,
    pub monitor_interval: Option<u64>,
    pub auto_update_enabled: Option<bool>,
    pub gh_proxy: Option<String>,
    pub gh_proxy_api: Option<bool>,
}

impl AppConfig {
    pub fn new() -> Self {
        Self {
            theme: Some("light".to_string()),
            monitor_interval: Some(5),
            auto_update_enabled: Some(true),
            gh_proxy: None,
            gh_proxy_api: Some(false),
        }
    }
}
