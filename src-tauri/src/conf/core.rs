use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenListCoreConfig {
    pub port: u16,
    pub api_token: String,
    pub auto_launch: bool,
    pub ssl_enabled: bool,
}

impl OpenListCoreConfig {
    pub fn new() -> Self {
        Self {
            port: 5244,
            api_token: "".to_string(),
            auto_launch: false,
            ssl_enabled: false,
        }
    }
}
