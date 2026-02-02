use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tauri::State;

use crate::conf::rclone_config::{RcloneConfigFile, WebDavRemoteConfig};
use crate::core::process_manager::{PROCESS_MANAGER, ProcessConfig, ProcessInfo};
use crate::object::structs::{AppState, RcloneMountInfo};
use crate::utils::args::split_args_vec;
use crate::utils::path::{
    get_app_logs_dir, get_rclone_binary_path_with_custom, get_rclone_config_path_with_custom,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RcloneWebdavConfigInput {
    pub url: String,
    pub vendor: Option<String>,
    pub user: String,
    pub pass: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MountProcessInput {
    pub id: String,
    pub name: String,
    pub args: Vec<String>,
    pub auto_start: Option<bool>,
}

fn get_mount_process_id(remote_name: &str) -> String {
    format!("rclone_mount_{remote_name}_process")
}

#[tauri::command]
pub async fn rclone_list_config(remote_type: String) -> Result<Value, String> {
    let config = RcloneConfigFile::load()?;

    let filtered: HashMap<String, Value> = config
        .remotes
        .iter()
        .filter(|(_, remote)| remote_type.is_empty() || remote.remote_type == remote_type)
        .map(|(name, remote)| {
            let mut obj = serde_json::Map::new();
            obj.insert("type".to_string(), json!(remote.remote_type));
            for (key, value) in &remote.options {
                obj.insert(key.clone(), json!(value));
            }
            (name.clone(), Value::Object(obj))
        })
        .collect();

    Ok(json!(filtered))
}

#[tauri::command]
pub async fn rclone_list_remotes() -> Result<Vec<String>, String> {
    let config = RcloneConfigFile::load()?;
    Ok(config.list_remotes())
}

#[tauri::command]
pub async fn rclone_create_remote(
    name: String,
    r#type: String,
    config: RcloneWebdavConfigInput,
) -> Result<bool, String> {
    let mut rclone_config = RcloneConfigFile::load()?;

    if rclone_config.has_remote(&name) {
        return Err(format!("Remote '{name}' already exists"));
    }

    let webdav = WebDavRemoteConfig {
        name: name.clone(),
        url: config.url,
        vendor: config.vendor,
        user: config.user,
        pass: config.pass,
    };

    // For now, we only support webdav type
    if r#type != "webdav" {
        return Err(format!("Unsupported remote type: {}", r#type));
    }

    rclone_config.set_remote(webdav.to_rclone_config());
    rclone_config.save()?;

    Ok(true)
}

#[tauri::command]
pub async fn rclone_update_remote(
    name: String,
    r#type: String,
    config: RcloneWebdavConfigInput,
) -> Result<bool, String> {
    let mut rclone_config = RcloneConfigFile::load()?;

    if !rclone_config.has_remote(&name) {
        return Err(format!("Remote '{name}' does not exist"));
    }

    if r#type != "webdav" {
        return Err(format!("Unsupported remote type: {}", r#type));
    }

    let webdav = WebDavRemoteConfig {
        name: name.clone(),
        url: config.url,
        vendor: config.vendor,
        user: config.user,
        pass: config.pass,
    };

    rclone_config.set_remote(webdav.to_rclone_config());
    rclone_config.save()?;

    Ok(true)
}

#[tauri::command]
pub async fn rclone_delete_remote(name: String) -> Result<bool, String> {
    let mut rclone_config = RcloneConfigFile::load()?;

    let process_id = get_mount_process_id(&name);
    if PROCESS_MANAGER.is_registered(&process_id) {
        let _ = PROCESS_MANAGER.stop(&process_id);
        let _ = PROCESS_MANAGER.remove(&process_id);
    }

    if rclone_config.remove_remote(&name).is_none() {
        return Err(format!("Remote '{name}' does not exist"));
    }

    rclone_config.save()?;
    Ok(true)
}

#[tauri::command]
pub async fn create_rclone_mount_remote_process(
    config: MountProcessInput,
    state: State<'_, AppState>,
) -> Result<ProcessInfo, String> {
    let settings = state
        .app_settings
        .read()
        .clone()
        .ok_or("Failed to read app settings")?;

    let custom_rclone_binary = settings.rclone.binary_path;
    let custom_rclone_config = settings.rclone.rclone_conf_path;

    let binary_path = get_rclone_binary_path_with_custom(custom_rclone_binary.as_deref())
        .map_err(|e| format!("Failed to get rclone binary path: {e}"))?;
    let log_dir =
        get_app_logs_dir().map_err(|e| format!("Failed to get app logs directory: {e}"))?;
    let rclone_conf_path = get_rclone_config_path_with_custom(custom_rclone_config.as_deref())
        .map_err(|e| format!("Failed to get rclone config path: {e}"))?;

    let args_vec = split_args_vec(config.args.clone());

    let mount_point_opt = args_vec.iter().filter(|arg| !arg.starts_with('-')).nth(1);

    if let Some(mount_point) = mount_point_opt {
        let mount_path = Path::new(mount_point);
        if !mount_path.exists() {
            if let Err(e) = fs::create_dir_all(mount_path) {
                return Err(format!(
                    "Failed to create mount point directory '{}': {}",
                    mount_point, e
                ));
            }
        }
    }

    let mut args: Vec<String> = vec![
        "mount".into(),
        "--config".into(),
        rclone_conf_path.to_string_lossy().into_owned(),
    ];
    args.extend(args_vec);

    let log_file = log_dir.join(format!("{}.log", config.id));

    let process_config = ProcessConfig {
        id: config.id.clone(),
        name: config.name.clone(),
        bin_path: binary_path.to_string_lossy().into_owned(),
        args,
        log_file: log_file.to_string_lossy().into_owned(),
        working_dir: binary_path
            .parent()
            .map(|p| p.to_string_lossy().into_owned()),
        env_vars: None,
    };

    if PROCESS_MANAGER.is_registered(&config.id) {
        let info = PROCESS_MANAGER.get_status(&config.id)?;
        if config.auto_start.unwrap_or(true) && !info.is_running {
            return PROCESS_MANAGER.start(&config.id);
        }
        return Ok(info);
    }

    if config.auto_start.unwrap_or(true) {
        PROCESS_MANAGER.register_and_start(process_config)
    } else {
        PROCESS_MANAGER.register(process_config)
    }
}

#[tauri::command]
pub async fn start_mount_process(process_id: String) -> Result<ProcessInfo, String> {
    if !PROCESS_MANAGER.is_registered(&process_id) {
        return Err(format!("Mount process '{process_id}' is not registered"));
    }
    PROCESS_MANAGER.start(&process_id)
}

#[tauri::command]
pub async fn stop_mount_process(process_id: String) -> Result<ProcessInfo, String> {
    if !PROCESS_MANAGER.is_registered(&process_id) {
        return Err(format!("Mount process '{process_id}' is not registered"));
    }
    PROCESS_MANAGER.stop(&process_id)
}

#[tauri::command]
pub async fn unmount_remote(name: String) -> Result<bool, String> {
    let process_id = get_mount_process_id(&name);

    if !PROCESS_MANAGER.is_registered(&process_id) {
        return Ok(true); // Already not mounted
    }

    let info = PROCESS_MANAGER.get_status(&process_id)?;
    if info.is_running {
        PROCESS_MANAGER.stop(&process_id)?;
    }

    let _ = PROCESS_MANAGER.remove(&process_id);

    Ok(true)
}

#[tauri::command]
pub async fn check_mount_status(mount_point: String) -> Result<bool, String> {
    let path = Path::new(&mount_point);

    if !path.exists() {
        return Ok(false);
    }

    #[cfg(target_os = "windows")]
    {
        if mount_point.len() == 2 && mount_point.ends_with(':') {
            let drive_path = format!("{mount_point}\\");
            return Ok(fs::read_dir(&drive_path).is_ok());
        }
        Ok(fs::read_dir(&mount_point).is_ok())
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        Ok(fs::read_dir(&mount_point).is_ok())
    }
}

async fn check_mount_status_internal(mount_point: &str) -> Result<bool, String> {
    check_mount_status(mount_point.to_string()).await
}

#[tauri::command]
pub async fn get_mount_info_list(
    _state: State<'_, AppState>,
) -> Result<Vec<RcloneMountInfo>, String> {
    let process_list = PROCESS_MANAGER.list();
    let mut mount_infos = Vec::new();

    for process in process_list {
        if !process.name.starts_with("rclone_mount_") {
            continue;
        }

        let args = &process.config.args;
        let non_flag_args: Vec<&String> = args.iter().filter(|arg| !arg.starts_with('-')).collect();

        if non_flag_args.len() >= 3 && non_flag_args[0] == "mount" {
            let remote_path = non_flag_args[1].clone();
            let mount_point = non_flag_args[2].clone();

            let mount_status = match check_mount_status_internal(&mount_point).await {
                Ok(is_accessible) => {
                    if process.is_running {
                        if is_accessible { "mounted" } else { "mounting" }
                    } else {
                        "unmounted"
                    }
                }
                Err(_) => "error",
            };

            let remote_name = remote_path.split(':').next().unwrap_or("").to_string();

            mount_infos.push(RcloneMountInfo {
                name: remote_name,
                process_id: process.id,
                remote_path,
                mount_point,
                status: mount_status.to_string(),
            });
        }
    }

    Ok(mount_infos)
}

#[tauri::command]
pub async fn get_mount_process_logs(
    process_id: String,
    lines: Option<usize>,
) -> Result<Vec<String>, String> {
    if !PROCESS_MANAGER.is_registered(&process_id) {
        return Err(format!("Mount process '{process_id}' is not registered"));
    }
    PROCESS_MANAGER.read_logs(&process_id, lines.unwrap_or(100))
}
