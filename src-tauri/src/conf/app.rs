use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub theme: Option<String>,
    pub monitor_interval: Option<u64>,
    pub service_api_token: Option<String>,
    pub service_port: Option<u64>,
    pub auto_update_enabled: Option<bool>,
}

impl AppConfig {
    pub fn new() -> Self {
        Self {
            theme: Some("zh".into()),
            monitor_interval: Some(5),
            service_api_token: Some("yeM6PCcZGaCpapyBKAbjTp2YAhcku6cUr".into()),
            service_port: Some(53211),
            auto_update_enabled: Some(true),
        }
    }
}
