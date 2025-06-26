use crate::object::structs::AppState;

use std::{collections::HashMap, env};

use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProcessConfig {
    pub id: String,
    pub name: String,
    pub bin_path: String,
    pub args: Vec<String>,
    pub log_file: String,
    pub working_dir: Option<String>,
    pub env_vars: Option<HashMap<String, String>>,
    pub auto_restart: bool,
    pub auto_start: bool,
    pub run_as_admin: bool,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CreateProcessResponse {
    pub success: bool,
    pub data: ProcessConfig,
    pub error: Option<String>,
    pub timestamp: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ListProcessResponse {
    pub success: bool,
    pub data: Vec<ProcessStatus>,
    pub error: Option<String>,
    pub timestamp: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProcessStatus {
    pub id: String,
    pub name: String,
    pub is_running: bool,
    pub pid: Option<u32>,
    pub started_at: Option<u64>,
    pub restart_count: u32,
    pub last_exit_code: Option<i32>,
    pub config: ProcessConfig,
}

const DEFAULT_API_KEY: &str = "yeM6PCcZGaCpapyBKAbjTp2YAhcku6cUr";
const DEFAULT_HTTP_SERVER_PORT: u16 = 53211;

pub fn get_server_port() -> u16 {
    env::var("PROCESS_MANAGER_PORT")
        .ok()
        .and_then(|port_str| port_str.parse().ok())
        .unwrap_or_else(|| DEFAULT_HTTP_SERVER_PORT)
}

pub fn get_api_key(state: State<'_, AppState>) -> String {
    let app_settings = state
        .app_settings
        .read()
        .clone()
        .ok_or_else(|| "Failed to read app settings".to_string())
        .unwrap();
    let openlist_config = app_settings.openlist;
    if openlist_config.api_token != "" {
        return openlist_config.api_token.clone();
    }
    let api_key =
        env::var("PROCESS_MANAGER_API_KEY").unwrap_or_else(|_| DEFAULT_API_KEY.to_string());
    api_key
}
