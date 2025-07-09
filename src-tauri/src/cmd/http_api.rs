use reqwest::Client;
use tauri::State;

use crate::object::structs::AppState;
use crate::utils::api::{ListProcessResponse, ProcessStatus, get_api_key, get_server_port};
use crate::utils::args::split_args_vec;

fn create_client() -> (Client, String, u16) {
    let client = Client::new();
    let api_key = get_api_key();
    let port = get_server_port();
    (client, api_key, port)
}

async fn process_operation(id: &str, operation: &str) -> Result<bool, String> {
    let (client, api_key, port) = create_client();
    let url = match operation {
        "start" => format!("http://127.0.0.1:{port}/api/v1/processes/{id}/start"),
        "stop" => format!("http://127.0.0.1:{port}/api/v1/processes/{id}/stop"),
        "delete" => format!("http://127.0.0.1:{port}/api/v1/processes/{id}"),
        _ => return Err("Invalid operation".to_string()),
    };

    let request = match operation {
        "delete" => client.delete(&url),
        _ => client.post(&url),
    };

    let response = request
        .header("Authorization", format!("Bearer {api_key}"))
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {e}"))?;

    if response.status().is_success() {
        Ok(true)
    } else {
        Err(format!(
            "Failed to {operation} process: {}",
            response.status()
        ))
    }
}

#[tauri::command]
pub async fn get_process_list(_state: State<'_, AppState>) -> Result<Vec<ProcessStatus>, String> {
    let (client, api_key, port) = create_client();
    let response = client
        .get(format!("http://127.0.0.1:{port}/api/v1/processes"))
        .header("Authorization", format!("Bearer {api_key}"))
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("Failed to get process list: {}", response.status()));
    }

    let response_text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response text: {e}"))?;

    serde_json::from_str::<ListProcessResponse>(&response_text)
        .map(|process_list| process_list.data)
        .map_err(|e| format!("Failed to parse response: {e}, response: {response_text}"))
}

#[tauri::command]
pub async fn start_process(id: String, _state: State<'_, AppState>) -> Result<bool, String> {
    process_operation(&id, "start").await
}

#[tauri::command]
pub async fn stop_process(id: String, _state: State<'_, AppState>) -> Result<bool, String> {
    process_operation(&id, "stop").await
}

#[tauri::command]
pub async fn restart_process(id: String, _state: State<'_, AppState>) -> Result<bool, String> {
    process_operation(&id, "stop")
        .await
        .map_err(|e| format!("Failed to stop OpenList Core process: {e}"))?;

    process_operation(&id, "start")
        .await
        .map_err(|e| format!("Failed to start OpenList Core process: {e}"))
}

#[tauri::command]
pub async fn update_process(
    id: String,
    update_config: serde_json::Value,
    _state: State<'_, AppState>,
) -> Result<bool, String> {
    let (client, api_key, port) = create_client();

    let mut processed_config = update_config;
    if let Some(args) = processed_config.get("args").and_then(|v| v.as_array()) {
        let args_strings: Vec<String> = args
            .iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.to_string())
            .collect();
        processed_config["args"] = serde_json::json!(split_args_vec(args_strings));
    }

    let response = client
        .put(format!("http://127.0.0.1:{port}/api/v1/processes/{id}"))
        .header("Authorization", format!("Bearer {api_key}"))
        .json(&processed_config)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {e}"))?;

    if response.status().is_success() {
        Ok(true)
    } else {
        Err(format!("Failed to update process: {}", response.status()))
    }
}

#[tauri::command]
pub async fn delete_process(id: String, _state: State<'_, AppState>) -> Result<bool, String> {
    process_operation(&id, "delete").await
}
