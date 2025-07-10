use std::path::PathBuf;
use std::{env, fs};

pub static APP_ID: &str = "io.github.openlistteam.openlist.desktop";

fn get_app_dir() -> Result<PathBuf, String> {
    let app_dir = env::current_exe()
        .map_err(|e| format!("Failed to get current exe path: {e}"))?
        .parent()
        .ok_or("Failed to get parent directory")?
        .to_path_buf();
    if !app_dir.exists() {
        return Err(format!("Application directory does not exist: {app_dir:?}"));
    }

    Ok(app_dir)
}

fn get_binary_path(binary: &str, service_name: &str) -> Result<PathBuf, String> {
    let mut name = binary.to_string();
    if cfg!(target_os = "windows") {
        name.push_str(".exe");
    }

    let path = get_app_dir()?.join(&name);
    if !path.exists() {
        return Err(format!(
            "{service_name} service binary not found at: {path:?}"
        ));
    }
    Ok(path)
}

pub fn get_openlist_binary_path() -> Result<PathBuf, String> {
    get_binary_path("openlist", "OpenList")
}

pub fn get_rclone_binary_path() -> Result<PathBuf, String> {
    get_binary_path("rclone", "Rclone")
}

pub fn get_app_config_dir() -> Result<PathBuf, String> {
    get_app_dir()
}

pub fn app_config_file_path() -> Result<PathBuf, String> {
    Ok(get_app_config_dir()?.join("settings.json"))
}

pub fn get_app_logs_dir() -> Result<PathBuf, String> {
    let logs = get_app_dir()?.join("logs");
    fs::create_dir_all(&logs).map_err(|e| e.to_string())?;
    Ok(logs)
}
