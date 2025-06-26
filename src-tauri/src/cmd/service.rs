use crate::core::service::check_service_status as check_service_status_impl;
use crate::core::service::install_service as install_service_impl;
use crate::core::service::restart_service as restart_service_impl;
use crate::core::service::stop_service as stop_service_impl;
use crate::core::service::uninstall_service as uninstall_service_impl;

#[tauri::command]
pub async fn check_service_status() -> Result<bool, String> {
    check_service_status_impl().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn install_service() -> Result<bool, String> {
    install_service_impl().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn uninstall_service() -> Result<bool, String> {
    uninstall_service_impl().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_service() -> Result<bool, String> {
    stop_service_impl().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn restart_service() -> Result<bool, String> {
    restart_service_impl().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_service() -> Result<bool, String> {
    check_service_status_impl().await.map_err(|e| e.to_string())
}
