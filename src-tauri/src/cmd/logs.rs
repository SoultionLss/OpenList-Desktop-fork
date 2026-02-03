use std::env;
use std::path::PathBuf;
use std::process::Command;

use rand::Rng;
use rand::distr::Alphanumeric;
use tauri::State;

use crate::object::structs::AppState;
use crate::utils::path::{get_app_logs_dir, get_default_openlist_data_dir};

fn generate_random_password(length: usize) -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

async fn execute_openlist_admin_set(
    password: &str,
    state: &State<'_, AppState>,
) -> Result<(), String> {
    let exe_path =
        env::current_exe().map_err(|e| format!("Failed to determine executable path: {e}"))?;
    let app_dir = exe_path
        .parent()
        .ok_or("Executable has no parent directory")?;

    let possible_names = ["openlist", "openlist.exe"];

    let mut openlist_exe = None;
    for name in &possible_names {
        let exe_path = app_dir.join(name);
        if exe_path.exists() {
            openlist_exe = Some(exe_path);
            break;
        }
    }

    let openlist_exe = openlist_exe.ok_or_else(|| {
        format!(
            "OpenList executable not found. Searched for: {:?} in {}",
            possible_names,
            app_dir.display()
        )
    })?;

    log::info!(
        "Setting new admin password using: {}",
        openlist_exe.display()
    );

    let mut cmd = Command::new(&openlist_exe);
    cmd.args(["admin", "set", password]);
    cmd.current_dir(app_dir);

    let effective_data_dir = if let Some(settings) = state.get_settings()
        && !settings.openlist.data_dir.is_empty()
    {
        settings.openlist.data_dir
    } else {
        get_default_openlist_data_dir()
            .map_err(|e| format!("Failed to get default data directory: {e}"))?
            .to_string_lossy()
            .to_string()
    };

    cmd.arg("--data");
    cmd.arg(&effective_data_dir);
    log::info!("Using data directory: {effective_data_dir}");
    log::info!("Executing command: {cmd:?}");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    let output = cmd
        .output()
        .map_err(|e| format!("Failed to execute openlist command: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        log::error!("OpenList admin set command failed. stdout: {stdout}, stderr: {stderr}");
        return Err(format!("OpenList admin set command failed: {stderr}"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    log::info!("Successfully set admin password. Output: {stdout}");

    Ok(())
}

fn resolve_log_paths(source: Option<&str>, data_dir: Option<&str>) -> Result<Vec<PathBuf>, String> {
    let logs_dir = get_app_logs_dir()?;

    let openlist_log_base = if let Some(dir) = data_dir.filter(|d| !d.is_empty()) {
        PathBuf::from(dir)
    } else {
        get_default_openlist_data_dir()
            .map_err(|e| format!("Failed to get default data directory: {e}"))?
    };

    let mut paths = Vec::new();
    match source {
        Some("openlist") => paths.push(openlist_log_base.join("log/log.log")),
        Some("app") => paths.push(logs_dir.join("app.log")),
        Some("rclone") => paths.push(logs_dir.join("process_rclone.log")),
        Some("all") => {
            paths.push(openlist_log_base.join("log/log.log"));
            paths.push(logs_dir.join("app.log"));
            paths.push(logs_dir.join("process_rclone.log"));
        }
        _ => return Err("Invalid log source".into()),
    }
    Ok(paths)
}

#[tauri::command]
pub async fn get_admin_password(state: State<'_, AppState>) -> Result<String, String> {
    if let Some(settings) = state.get_settings()
        && let Some(ref stored_password) = settings.app.admin_password
        && !stored_password.is_empty()
    {
        log::info!("Found admin password in local settings");
        return Ok(stored_password.clone());
    }

    let new_password = generate_random_password(16);

    if let Err(e) = execute_openlist_admin_set(&new_password, &state).await {
        return Err(format!("Failed to set new admin password: {e}"));
    }

    log::info!("Successfully generated and set new admin password");

    if let Some(mut settings) = state.get_settings() {
        settings.app.admin_password = Some(new_password.clone());
        state.update_settings(settings.clone());

        if let Err(e) = settings.save() {
            log::warn!("Failed to save new admin password to settings: {e}");
        }
    }

    Ok(new_password)
}

#[tauri::command]
pub async fn reset_admin_password(state: State<'_, AppState>) -> Result<String, String> {
    log::info!("Forcing admin password reset");
    let new_password = generate_random_password(16);
    if let Err(e) = execute_openlist_admin_set(&new_password, &state).await {
        return Err(format!("Failed to set new admin password: {e}"));
    }
    log::info!("Successfully generated and set new admin password via force reset");

    if let Some(mut settings) = state.get_settings() {
        settings.app.admin_password = Some(new_password.clone());
        state.update_settings(settings.clone());

        if let Err(e) = settings.save() {
            log::warn!("Failed to save new admin password to settings: {e}");
        }
    }

    Ok(new_password)
}

#[tauri::command]
pub async fn set_admin_password(
    password: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    log::info!("Setting custom admin password");

    if let Err(e) = execute_openlist_admin_set(&password, &state).await {
        return Err(format!("Failed to set admin password: {e}"));
    }

    log::info!("Successfully set custom admin password");

    if let Some(mut settings) = state.get_settings() {
        settings.app.admin_password = Some(password.clone());
        state.update_settings(settings.clone());

        if let Err(e) = settings.save() {
            log::warn!("Failed to save admin password to settings: {e}");
        }
    }

    Ok(password)
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
        if path.exists() {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read {path:?}: {e}"))?;
            logs.extend(content.lines().map(str::to_string));
        } else {
            log::info!("Log file does not exist: {path:?}");
        }
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
