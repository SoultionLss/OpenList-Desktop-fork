use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenListCoreConfig {
    pub port: u16,
    pub data_dir: String,
    pub auto_launch: bool,
    pub ssl_enabled: bool,
}

impl OpenListCoreConfig {
    pub fn new() -> Self {
        Self {
            port: 5244,
            data_dir: "".to_string(),
            auto_launch: false,
            ssl_enabled: false,
        }
    }
}
