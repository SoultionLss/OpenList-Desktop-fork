use crate::conf::rclone::RcloneConfig;
use crate::{conf::core::OpenListCoreConfig, utils::path::app_config_file_path};

use super::app::AppConfig;
use serde::{Deserialize, Serialize};

#[allow(unused)]
pub static OPENLIST_CORE_CONFIG: &str = "data/config.json";
#[allow(unused)]
pub static OPENLIST_DESKTOP_SETTINGS_FILE_NAME: &str = "settings.json";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MergedSettings {
    pub openlist: OpenListCoreConfig,
    pub rclone: RcloneConfig,
    pub app: AppConfig,
}

impl MergedSettings {
    pub fn new() -> Self {
        Self {
            openlist: OpenListCoreConfig::new(),
            rclone: RcloneConfig::new(),
            app: AppConfig::new(),
        }
    }

    pub fn load() -> Result<Self, String> {
        let path = app_config_file_path().map_err(|e| e.to_string())?;
        if !path.exists() {
            std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
            std::fs::write(
                &path,
                serde_json::to_string_pretty(&Self::new()).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
            return Ok(Self::new());
        }
        let config = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        let merged_settings: MergedSettings =
            serde_json::from_str(&config).map_err(|e| e.to_string())?;
        Ok(merged_settings)
    }

    pub fn default() -> Self {
        Self::new()
    }
}
