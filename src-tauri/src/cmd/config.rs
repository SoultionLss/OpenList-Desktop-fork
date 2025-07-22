use std::fs;
use std::path::PathBuf;

use tauri::State;
use tokio::time::{Duration, sleep};

use crate::cmd::http_api::{delete_process, get_process_list, start_process, stop_process};
use crate::cmd::openlist_core::create_openlist_core_process;
use crate::conf::config::MergedSettings;
use crate::object::structs::AppState;
use crate::utils::path::{app_config_file_path, get_default_openlist_data_dir};

fn write_json_to_file<T: serde::Serialize>(path: PathBuf, value: &T) -> Result<(), String> {
    let json = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

fn persist_app_settings(settings: &MergedSettings) -> Result<(), String> {
    let path = app_config_file_path().map_err(|e| e.to_string())?;
    write_json_to_file(path, settings)
}

fn update_data_config(port: u16, data_dir: Option<&str>) -> Result<(), String> {
    let data_config_path = if let Some(dir) = data_dir.filter(|d| !d.is_empty()) {
        PathBuf::from(dir).join("config.json")
    } else {
        get_default_openlist_data_dir()?.join("config.json")
    };

    if let Some(parent) = data_config_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let mut cfg_value = if data_config_path.exists() {
        let s = fs::read_to_string(&data_config_path).map_err(|e| e.to_string())?;
        serde_json::from_str::<serde_json::Value>(&s).map_err(|e| e.to_string())?
    } else {
        serde_json::json!({ "scheme": { "http_port": port } })
    };

    let scheme = cfg_value.get_mut("scheme").and_then(|v| v.as_object_mut());
    if let Some(obj) = scheme {
        obj.insert("http_port".into(), serde_json::json!(port));
    } else {
        cfg_value["scheme"] = serde_json::json!({ "http_port": port });
    }

    write_json_to_file(data_config_path, &cfg_value)
}

async fn restart_openlist_core(state: State<'_, AppState>) -> Result<(), String> {
    let procs = get_process_list(state.clone()).await?;
    if let Some(proc) = procs
        .into_iter()
        .find(|p| p.config.name == "single_openlist_core_process")
    {
        let id = proc.config.id.clone();
        let _ = stop_process(id.clone(), state.clone()).await;
        sleep(Duration::from_millis(1_000)).await;
        start_process(id, state)
            .await
            .map_err(|e| format!("Failed to restart OpenList core: {e}"))?;
    }
    Ok(())
}

async fn recreate_openlist_core_process(state: State<'_, AppState>) -> Result<(), String> {
    let procs = get_process_list(state.clone()).await?;
    if let Some(proc) = procs
        .into_iter()
        .find(|p| p.config.name == "single_openlist_core_process")
    {
        let id = proc.config.id.clone();
        let _ = stop_process(id.clone(), state.clone()).await;
        sleep(Duration::from_millis(1000)).await;
        let _ = delete_process(id, state.clone()).await;
        sleep(Duration::from_millis(1000)).await;

        let auto_launch = state
            .app_settings
            .read()
            .clone()
            .map(|settings| settings.openlist.auto_launch)
            .unwrap_or(false);

        create_openlist_core_process(auto_launch, state.clone()).await?;
    }
    Ok(())
}

#[tauri::command]
pub async fn load_settings(state: State<'_, AppState>) -> Result<Option<MergedSettings>, String> {
    state.load_settings()?;
    Ok(state.get_settings())
}

#[tauri::command]
pub async fn save_settings(
    settings: MergedSettings,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    state.update_settings(settings.clone());
    persist_app_settings(&settings)?;
    log::info!("Settings saved successfully");
    Ok(true)
}

#[tauri::command]
pub async fn save_settings_with_update_port(
    settings: MergedSettings,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let old_settings = state.get_settings();
    let needs_process_recreation = if let Some(old) = old_settings {
        old.openlist.data_dir != settings.openlist.data_dir
    } else {
        false
    };

    state.update_settings(settings.clone());
    persist_app_settings(&settings)?;
    let data_dir = if settings.openlist.data_dir.is_empty() {
        None
    } else {
        Some(settings.openlist.data_dir.as_str())
    };
    update_data_config(settings.openlist.port, data_dir)?;

    if needs_process_recreation {
        if let Err(e) = recreate_openlist_core_process(state.clone()).await {
            log::error!("{e}");
            return Err(e);
        }
        log::info!(
            "Settings saved and OpenList core recreated with new data directory successfully"
        );
    } else {
        if let Err(e) = restart_openlist_core(state.clone()).await {
            log::error!("{e}");
            return Err(e);
        }
        log::info!("Settings saved and OpenList core restarted with new port successfully");
    }

    Ok(true)
}

#[tauri::command]
pub async fn reset_settings(state: State<'_, AppState>) -> Result<Option<MergedSettings>, String> {
    let base_settings = MergedSettings::default();
    state.update_settings(base_settings.clone());
    persist_app_settings(&base_settings)?;
    log::info!("Settings reset to default");
    Ok(Some(base_settings))
}
