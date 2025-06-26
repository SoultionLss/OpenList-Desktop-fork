use std::fs;
use tauri::State;

use crate::conf::config::MergedSettings;
use crate::object::structs::AppState;
use crate::utils::path::app_config_file_path;

#[tauri::command]
pub async fn save_settings(
    settings: MergedSettings,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    state.update_settings(settings.clone());
    let settings_path = app_config_file_path().map_err(|e| e.to_string())?;
    let settings_json = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    fs::write(settings_path, settings_json).map_err(|e| e.to_string())?;

    log::info!("Settings saved successfully");
    Ok(true)
}

#[tauri::command]
pub async fn load_settings(state: State<'_, AppState>) -> Result<Option<MergedSettings>, String> {
    state.load_settings()?;
    Ok(state.get_settings())
}

#[tauri::command]
pub async fn reset_settings(state: State<'_, AppState>) -> Result<Option<MergedSettings>, String> {
    let default_settings = MergedSettings::default();
    state.update_settings(default_settings.clone());

    let settings_path = app_config_file_path().map_err(|e| e.to_string())?;
    let settings_json =
        serde_json::to_string_pretty(&default_settings).map_err(|e| e.to_string())?;
    fs::write(settings_path, settings_json).map_err(|e| e.to_string())?;

    log::info!("Settings reset to default");
    Ok(Some(default_settings))
}
