use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::app::AppConfig;
use crate::conf::core::OpenListCoreConfig;
use crate::conf::rclone::RcloneConfig;
use crate::utils::path::app_config_file_path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MergedSettings {
    pub openlist: OpenListCoreConfig,
    pub rclone: RcloneConfig,
    pub app: AppConfig,
}

impl Default for MergedSettings {
    fn default() -> Self {
        Self::new()
    }
}

impl MergedSettings {
    pub fn new() -> Self {
        Self {
            openlist: OpenListCoreConfig::new(),
            rclone: RcloneConfig::new(),
            app: AppConfig::new(),
        }
    }

    pub fn get_data_config_path_for_dir(data_dir: Option<&str>) -> Result<PathBuf, String> {
        if let Some(dir) = data_dir.filter(|d| !d.is_empty()) {
            Ok(PathBuf::from(dir).join("config.json"))
        } else {
            let exe = std::env::current_exe()
                .map_err(|e| format!("Failed to get current exe path: {e}"))?;
            let dir = exe
                .parent()
                .ok_or_else(|| "Failed to get executable parent directory".to_string())?;
            Ok(dir.join("data").join("config.json"))
        }
    }

    pub fn read_data_config_for_dir(data_dir: Option<&str>) -> Result<serde_json::Value, String> {
        let path = Self::get_data_config_path_for_dir(data_dir)?;
        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())
    }

    fn get_port_from_data_config_for_dir(data_dir: Option<&str>) -> Result<Option<u16>, String> {
        let config = Self::read_data_config_for_dir(data_dir)?;
        Ok(config
            .get("scheme")
            .and_then(|s| s.get("http_port"))
            .and_then(|p| p.as_u64())
            .map(|p| p as u16))
    }

    pub fn save(&self) -> Result<(), String> {
        let path = app_config_file_path().map_err(|e| e.to_string())?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        let file = std::fs::File::create(&path).map_err(|e| e.to_string())?;
        serde_json::to_writer_pretty(file, &self).map_err(|e| e.to_string())
    }

    pub fn load() -> Result<Self, String> {
        let path = app_config_file_path().map_err(|e| e.to_string())?;

        let mut settings = if path.exists() {
            let data = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
            serde_json::from_str(&data).map_err(|e| e.to_string())?
        } else {
            let default = Self::new();
            if let Some(dir) = path.parent() {
                std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
            }
            default.save()?;
            default
        };

        let data_dir = if settings.openlist.data_dir.is_empty() {
            None
        } else {
            Some(settings.openlist.data_dir.as_str())
        };

        if let Ok(Some(port)) = Self::get_port_from_data_config_for_dir(data_dir)
            && settings.openlist.port != port
        {
            settings.openlist.port = port;
            settings.save()?;
        }

        Ok(settings)
    }
}
