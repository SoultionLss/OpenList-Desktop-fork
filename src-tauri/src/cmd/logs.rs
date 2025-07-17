use std::env;
use std::path::PathBuf;

use once_cell::sync::Lazy;
use regex::Regex;
use tauri::State;

use crate::object::structs::AppState;

static ADMIN_PWD_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"Successfully created the admin user and the initial password is: (\w+)")
        .expect("Invalid regex pattern")
});

fn resolve_log_paths(source: Option<&str>, data_dir: Option<&str>) -> Result<Vec<PathBuf>, String> {
    let exe_path =
        env::current_exe().map_err(|e| format!("Failed to determine executable path: {e}"))?;
    let app_dir = exe_path
        .parent()
        .ok_or("Executable has no parent directory")?
        .to_path_buf();

    let openlist_log_base = if let Some(dir) = data_dir.filter(|d| !d.is_empty()) {
        PathBuf::from(dir)
    } else {
        app_dir.join("data")
    };

    let mut paths = Vec::new();
    match source {
        Some("openlist") => paths.push(openlist_log_base.join("log/log.log")),
        Some("app") => paths.push(app_dir.join("logs/app.log")),
        Some("rclone") => paths.push(app_dir.join("logs/process_rclone.log")),
        Some("openlist_core") => paths.push(app_dir.join("logs/process_openlist_core.log")),
        None => {
            paths.push(openlist_log_base.join("log/log.log"));
            paths.push(app_dir.join("logs/app.log"));
            paths.push(app_dir.join("logs/process_rclone.log"));
            paths.push(app_dir.join("logs/process_openlist_core.log"));
        }
        _ => return Err("Invalid log source".into()),
    }
    Ok(paths)
}

#[tauri::command]
pub async fn get_admin_password(state: State<'_, AppState>) -> Result<String, String> {
    let data_dir = state
        .get_settings()
        .map(|s| s.openlist.data_dir)
        .filter(|d| !d.is_empty());

    let paths = resolve_log_paths(Some("openlist_core"), data_dir.as_deref())?;
    let content =
        std::fs::read_to_string(&paths[0]).map_err(|e| format!("Failed to read log file: {e}"))?;

    ADMIN_PWD_REGEX
        .captures_iter(&content)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .last()
        .ok_or_else(|| "No admin password found in logs".into())
}

#[tauri::command]
pub async fn get_logs(
    source: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let data_dir = state
        .get_settings()
        .map(|s| s.openlist.data_dir)
        .filter(|d| !d.is_empty());

    let paths = resolve_log_paths(source.as_deref(), data_dir.as_deref())?;
    let mut logs = Vec::new();

    for path in paths {
        let content =
            std::fs::read_to_string(&path).map_err(|e| format!("Failed to read {path:?}: {e}"))?;
        logs.extend(content.lines().map(str::to_string));
    }
    Ok(logs)
}

#[tauri::command]
pub async fn clear_logs(
    source: Option<String>,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let data_dir = state
        .get_settings()
        .map(|s| s.openlist.data_dir)
        .filter(|d| !d.is_empty());

    let paths = resolve_log_paths(source.as_deref(), data_dir.as_deref())?;
    let mut cleared_count = 0;

    for path in paths {
        if path.exists() {
            std::fs::write(&path, "").map_err(|e| format!("Failed to clear {path:?}: {e}"))?;
            cleared_count += 1;
        }
    }

    if cleared_count == 0 {
        Err("No log files found to clear".into())
    } else {
        Ok(true)
    }
}
