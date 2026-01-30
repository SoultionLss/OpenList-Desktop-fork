use crate::utils::path::get_rclone_binary_path;

#[tauri::command]
pub async fn check_rclone_available() -> Result<bool, String> {
    get_rclone_binary_path().map(|_| true).or(Ok(false))
}
