use tauri::State;

use crate::object::structs::AppState;
use crate::utils::path::get_rclone_binary_path_with_custom;

#[tauri::command]
pub async fn check_rclone_available(state: State<'_, AppState>) -> Result<bool, String> {
    let settings = state
        .app_settings
        .read()
        .clone()
        .ok_or("Failed to read app settings")?;

    get_rclone_binary_path_with_custom(settings.rclone.binary_path.as_deref())
        .map(|_| true)
        .or(Ok(false))
}
