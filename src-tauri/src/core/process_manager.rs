use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::utils::path::get_user_data_dir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessConfig {
    pub id: String,
    pub name: String,
    pub bin_path: String,
    pub args: Vec<String>,
    pub log_file: String,
    pub working_dir: Option<String>,
    pub env_vars: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub id: String,
    pub name: String,
    pub is_running: bool,
    pub pid: Option<u32>,
    pub started_at: Option<u64>,
    pub config: ProcessConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedProcessState {
    pub id: String,
    pub pid: u32,
    pub started_at: u64,
    pub config: ProcessConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct PersistedState {
    pub processes: Vec<PersistedProcessState>,
}

#[derive(Debug)]
struct ManagedProcess {
    config: ProcessConfig,
    child: Option<Child>,
    external_pid: Option<u32>,
    started_at: Option<u64>,
}

pub struct ProcessManager {
    processes: RwLock<HashMap<String, ManagedProcess>>,
    state_file: PathBuf,
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessManager {
    pub fn new() -> Self {
        let state_file = Self::get_state_file_path();
        let manager = Self {
            processes: RwLock::new(HashMap::new()),
            state_file,
        };
        manager.recover_persisted_state();
        manager
    }

    fn get_state_file_path() -> PathBuf {
        let data_dir = get_user_data_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("process_manager");
        let _ = std::fs::create_dir_all(&data_dir);
        data_dir.join("process_state.json")
    }

    fn is_process_alive(pid: u32) -> bool {
        #[cfg(target_os = "windows")]
        {
            const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;
            const STILL_ACTIVE: u32 = 259;

            unsafe {
                let handle = windows_sys::Win32::System::Threading::OpenProcess(
                    PROCESS_QUERY_LIMITED_INFORMATION,
                    0,
                    pid,
                );
                if handle.is_null() {
                    return false;
                }
                let mut exit_code: u32 = 0;
                let result = windows_sys::Win32::System::Threading::GetExitCodeProcess(
                    handle,
                    &mut exit_code as *mut u32,
                );
                windows_sys::Win32::Foundation::CloseHandle(handle);
                result != 0 && exit_code == STILL_ACTIVE
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            unsafe { libc::kill(pid as i32, 0) == 0 }
        }
    }

    fn kill_process_by_pid(pid: u32) {
        #[cfg(target_os = "windows")]
        {
            const PROCESS_TERMINATE: u32 = 0x0001;
            unsafe {
                let handle =
                    windows_sys::Win32::System::Threading::OpenProcess(PROCESS_TERMINATE, 0, pid);
                if !handle.is_null() {
                    windows_sys::Win32::System::Threading::TerminateProcess(handle, 1);
                    windows_sys::Win32::Foundation::CloseHandle(handle);
                }
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            unsafe {
                libc::kill(pid as i32, libc::SIGTERM);
                std::thread::sleep(std::time::Duration::from_millis(500));
                if libc::kill(pid as i32, 0) == 0 {
                    libc::kill(pid as i32, libc::SIGKILL);
                }
            }
        }
    }

    /// Recover persisted process state from disk
    fn recover_persisted_state(&self) {
        if !self.state_file.exists() {
            log::debug!("No persisted process state file found");
            return;
        }

        let state: PersistedState = match std::fs::read_to_string(&self.state_file) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(state) => state,
                Err(e) => {
                    log::warn!("Failed to parse persisted state: {e}");
                    return;
                }
            },
            Err(e) => {
                log::warn!("Failed to read persisted state file: {e}");
                return;
            }
        };

        let mut processes = self.processes.write();
        let mut recovered_count = 0;
        let mut removed_count = 0;

        for persisted in state.processes {
            if Self::is_process_alive(persisted.pid) {
                log::info!(
                    "Recovered running process '{}' (pid: {})",
                    persisted.id,
                    persisted.pid
                );
                processes.insert(
                    persisted.id.clone(),
                    ManagedProcess {
                        config: persisted.config,
                        child: None, // We don't have the Child handle, but we track the PID
                        external_pid: Some(persisted.pid),
                        started_at: Some(persisted.started_at),
                    },
                );
                recovered_count += 1;
            } else {
                log::info!(
                    "Process '{}' (pid: {}) is no longer running, removing from state",
                    persisted.id,
                    persisted.pid
                );
                removed_count += 1;
            }
        }

        drop(processes);

        log::info!(
            "Process state recovery complete: {} recovered, {} removed",
            recovered_count,
            removed_count
        );

        self.persist_state();
    }

    fn persist_state(&self) {
        let processes = self.processes.read();
        let mut state = PersistedState::default();

        for managed in processes.values() {
            let pid = if let Some(ref child) = managed.child {
                Some(child.id())
            } else {
                managed.external_pid
            };

            if let (Some(pid), Some(started_at)) = (pid, managed.started_at) {
                state.processes.push(PersistedProcessState {
                    id: managed.config.id.clone(),
                    pid,
                    started_at,
                    config: managed.config.clone(),
                });
            }
        }

        drop(processes);

        match serde_json::to_string_pretty(&state) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&self.state_file, json) {
                    log::error!("Failed to persist process state: {e}");
                } else {
                    log::debug!("Process state persisted successfully");
                }
            }
            Err(e) => {
                log::error!("Failed to serialize process state: {e}");
            }
        }
    }

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    fn rotate_log_if_needed(log_path: &PathBuf) -> Result<(), String> {
        const MAX_LOG_SIZE: u64 = 10 * 1024 * 1024;
        const MAX_ARCHIVES: usize = 3;

        if !log_path.exists() {
            return Ok(());
        }

        let metadata = std::fs::metadata(log_path)
            .map_err(|e| format!("Failed to read log file metadata: {e}"))?;

        if metadata.len() < MAX_LOG_SIZE {
            return Ok(());
        }

        log::info!(
            "Rotating log file '{}' (size: {} bytes)",
            log_path.display(),
            metadata.len()
        );

        let log_dir = log_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));
        let log_file_name = log_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("log");
        let log_extension = log_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("log");

        let oldest_archive = log_dir.join(format!(
            "{}.{}.{}",
            log_file_name, MAX_ARCHIVES, log_extension
        ));
        if oldest_archive.exists() {
            let _ = std::fs::remove_file(&oldest_archive);
        }

        for i in (1..MAX_ARCHIVES).rev() {
            let from = log_dir.join(format!("{}.{}.{}", log_file_name, i, log_extension));
            let to = log_dir.join(format!("{}.{}.{}", log_file_name, i + 1, log_extension));
            if from.exists() {
                std::fs::rename(&from, &to)
                    .map_err(|e| format!("Failed to rotate archive {}: {e}", i))?;
            }
        }

        let archive = log_dir.join(format!("{}.1.{}", log_file_name, log_extension));
        std::fs::rename(log_path, &archive)
            .map_err(|e| format!("Failed to archive current log: {e}"))?;

        log::info!("Log file rotated successfully");
        Ok(())
    }

    pub fn register(&self, config: ProcessConfig) -> Result<ProcessInfo, String> {
        let mut processes = self.processes.write();
        log::info!("Registering process '{}'", config.id);

        if let Some(managed) = processes.get_mut(&config.id) {
            let (is_running, pid) = if let Some(ref child) = managed.child {
                (true, Some(child.id()))
            } else if let Some(ext_pid) = managed.external_pid {
                if Self::is_process_alive(ext_pid) {
                    (true, Some(ext_pid))
                } else {
                    managed.external_pid = None;
                    managed.started_at = None;
                    (false, None)
                }
            } else {
                (false, None)
            };

            managed.config = config.clone();

            return Ok(ProcessInfo {
                id: managed.config.id.clone(),
                name: managed.config.name.clone(),
                is_running,
                pid,
                started_at: managed.started_at,
                config: managed.config.clone(),
            });
        }

        let managed = ManagedProcess {
            config: config.clone(),
            child: None,
            external_pid: None,
            started_at: None,
        };

        let info = ProcessInfo {
            id: config.id.clone(),
            name: config.name.clone(),
            is_running: false,
            pid: None,
            started_at: None,
            config: config.clone(),
        };

        processes.insert(config.id.clone(), managed);
        log::info!(
            "Process list after registering: {:?}",
            processes.keys().collect::<Vec<_>>()
        );
        Ok(info)
    }

    pub fn register_and_start(&self, config: ProcessConfig) -> Result<ProcessInfo, String> {
        let id = config.id.clone();
        self.register(config)?;
        self.start(&id)
    }

    pub fn start(&self, id: &str) -> Result<ProcessInfo, String> {
        let mut processes = self.processes.write();

        let managed = processes
            .get_mut(id)
            .ok_or_else(|| format!("Process with id '{id}' not found"))?;

        if let Some(ref mut child) = managed.child {
            match child.try_wait() {
                Ok(Some(_)) => {
                    managed.child = None;
                    managed.external_pid = None;
                }
                Ok(None) => {
                    return Ok(ProcessInfo {
                        id: managed.config.id.clone(),
                        name: managed.config.name.clone(),
                        is_running: true,
                        pid: Some(child.id()),
                        started_at: managed.started_at,
                        config: managed.config.clone(),
                    });
                }
                Err(e) => {
                    log::warn!("Error checking process status: {e}");
                    managed.child = None;
                    managed.external_pid = None;
                }
            }
        }

        if let Some(ext_pid) = managed.external_pid {
            if Self::is_process_alive(ext_pid) {
                return Ok(ProcessInfo {
                    id: managed.config.id.clone(),
                    name: managed.config.name.clone(),
                    is_running: true,
                    pid: Some(ext_pid),
                    started_at: managed.started_at,
                    config: managed.config.clone(),
                });
            } else {
                managed.external_pid = None;
                managed.started_at = None;
            }
        }

        let config = &managed.config;

        let config_bin_path = config.bin_path.clone();
        let config_args = config.args.clone();

        let log_path = PathBuf::from(&config.log_file);
        if let Some(parent) = log_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create log directory: {e}"))?;
        }

        Self::rotate_log_if_needed(&log_path)?;

        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .map_err(|e| format!("Failed to open log file: {e}"))?;

        let stderr_file = log_file
            .try_clone()
            .map_err(|e| format!("Failed to clone log file handle: {e}"))?;

        let mut cmd = Command::new(&config.bin_path);
        cmd.args(&config.args)
            .stdout(Stdio::from(log_file))
            .stderr(Stdio::from(stderr_file));

        if let Some(ref working_dir) = config.working_dir {
            cmd.current_dir(working_dir);
        }

        if let Some(ref env_vars) = config.env_vars {
            for (key, value) in env_vars {
                cmd.env(key, value);
            }
        }

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }

        let child = cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn process: {e}"))?;

        let pid = child.id();
        let started_at = Self::current_timestamp();

        managed.child = Some(child);
        managed.external_pid = None;
        managed.started_at = Some(started_at);

        let info = ProcessInfo {
            id: managed.config.id.clone(),
            name: managed.config.name.clone(),
            is_running: true,
            pid: Some(pid),
            started_at: Some(started_at),
            config: managed.config.clone(),
        };

        drop(processes);

        log::info!(
            "Started process '{}' (pid: {}) with command: {} {}",
            id,
            pid,
            config_bin_path,
            config_args.join(" ")
        );

        self.persist_state();

        Ok(info)
    }

    /// Stop a running process
    pub fn stop(&self, id: &str) -> Result<ProcessInfo, String> {
        let mut processes = self.processes.write();

        let managed = processes
            .get_mut(id)
            .ok_or_else(|| format!("Process with id '{id}' not found"))?;

        // Stop via Child handle if available
        if let Some(ref mut child) = managed.child {
            // Try graceful termination first
            #[cfg(target_os = "windows")]
            {
                let _ = child.kill();
            }

            #[cfg(not(target_os = "windows"))]
            {
                // Send SIGTERM first
                unsafe {
                    libc::kill(child.id() as i32, libc::SIGTERM);
                }
                // Give it a moment to terminate gracefully
                std::thread::sleep(std::time::Duration::from_millis(500));
                // If still running, force kill
                let _ = child.kill();
            }

            // Wait for the process to fully terminate
            let _ = child.wait();
            log::info!("Stopped process '{}' via Child handle", id);
        }
        // Stop via external PID (recovered process without Child handle)
        else if let Some(ext_pid) = managed.external_pid {
            if Self::is_process_alive(ext_pid) {
                Self::kill_process_by_pid(ext_pid);
                log::info!("Stopped process '{}' via external PID {}", id, ext_pid);
            }
        }

        managed.child = None;
        managed.external_pid = None;
        managed.started_at = None;

        let info = ProcessInfo {
            id: managed.config.id.clone(),
            name: managed.config.name.clone(),
            is_running: false,
            pid: None,
            started_at: None,
            config: managed.config.clone(),
        };

        drop(processes);

        // Persist state after stopping
        self.persist_state();

        Ok(info)
    }

    /// Restart a process
    #[allow(dead_code)]
    pub fn restart(&self, id: &str) -> Result<ProcessInfo, String> {
        self.stop(id)?;
        std::thread::sleep(std::time::Duration::from_millis(200));
        self.start(id)
    }

    /// Get status of a specific process
    pub fn get_status(&self, id: &str) -> Result<ProcessInfo, String> {
        let mut processes = self.processes.write();

        let managed = processes
            .get_mut(id)
            .ok_or_else(|| format!("Process with id '{id}' not found"))?;

        let (is_running, pid) = if let Some(ref mut child) = managed.child {
            match child.try_wait() {
                Ok(Some(_)) => {
                    // Process has exited
                    managed.child = None;
                    managed.external_pid = None;
                    managed.started_at = None;
                    (false, None)
                }
                Ok(None) => (true, Some(child.id())),
                Err(_) => {
                    managed.child = None;
                    managed.external_pid = None;
                    managed.started_at = None;
                    (false, None)
                }
            }
        } else if let Some(ext_pid) = managed.external_pid {
            // Check if external process is still alive
            if Self::is_process_alive(ext_pid) {
                (true, Some(ext_pid))
            } else {
                managed.external_pid = None;
                managed.started_at = None;
                (false, None)
            }
        } else {
            (false, None)
        };

        Ok(ProcessInfo {
            id: managed.config.id.clone(),
            name: managed.config.name.clone(),
            is_running,
            pid,
            started_at: managed.started_at,
            config: managed.config.clone(),
        })
    }

    /// List all registered processes
    pub fn list(&self) -> Vec<ProcessInfo> {
        let mut processes = self.processes.write();
        let mut result = Vec::new();

        for managed in processes.values_mut() {
            let (is_running, pid) = if let Some(ref mut child) = managed.child {
                match child.try_wait() {
                    Ok(Some(_)) => {
                        managed.child = None;
                        managed.external_pid = None;
                        managed.started_at = None;
                        (false, None)
                    }
                    Ok(None) => (true, Some(child.id())),
                    Err(_) => {
                        managed.child = None;
                        managed.external_pid = None;
                        managed.started_at = None;
                        (false, None)
                    }
                }
            } else if let Some(ext_pid) = managed.external_pid {
                // Check if external process is still alive
                if Self::is_process_alive(ext_pid) {
                    (true, Some(ext_pid))
                } else {
                    managed.external_pid = None;
                    managed.started_at = None;
                    (false, None)
                }
            } else {
                (false, None)
            };

            result.push(ProcessInfo {
                id: managed.config.id.clone(),
                name: managed.config.name.clone(),
                is_running,
                pid,
                started_at: managed.started_at,
                config: managed.config.clone(),
            });
        }

        result
    }

    /// Remove a process from management (must be stopped first)
    pub fn remove(&self, id: &str) -> Result<(), String> {
        let mut processes = self.processes.write();

        if let Some(mut managed) = processes.remove(id) {
            // Make sure to stop it if running via Child handle
            if let Some(ref mut child) = managed.child {
                let _ = child.kill();
                let _ = child.wait();
            }
            // Also stop if running via external PID
            if let Some(ext_pid) = managed.external_pid {
                if Self::is_process_alive(ext_pid) {
                    Self::kill_process_by_pid(ext_pid);
                }
            }
            drop(processes);
            self.persist_state();
            Ok(())
        } else {
            Err(format!("Process with id '{id}' not found"))
        }
    }

    /// Update process configuration (must be stopped first to take effect)
    #[allow(dead_code)]
    pub fn update_config(&self, id: &str, config: ProcessConfig) -> Result<ProcessInfo, String> {
        let mut processes = self.processes.write();

        let managed = processes
            .get_mut(id)
            .ok_or_else(|| format!("Process with id '{id}' not found"))?;

        // Check if running via Child handle
        if let Some(ref mut child) = managed.child {
            if child.try_wait().map_or(false, |status| status.is_none()) {
                return Err("Cannot update config while process is running. Stop it first.".into());
            }
        }
        // Check if running via external PID
        if let Some(ext_pid) = managed.external_pid {
            if Self::is_process_alive(ext_pid) {
                return Err("Cannot update config while process is running. Stop it first.".into());
            }
        }

        managed.config = config;

        Ok(ProcessInfo {
            id: managed.config.id.clone(),
            name: managed.config.name.clone(),
            is_running: false,
            pid: None,
            started_at: None,
            config: managed.config.clone(),
        })
    }

    /// Read recent log lines for a process
    pub fn read_logs(&self, id: &str, lines: usize) -> Result<Vec<String>, String> {
        let processes = self.processes.read();

        let managed = processes
            .get(id)
            .ok_or_else(|| format!("Process with id '{id}' not found"))?;

        let log_path = PathBuf::from(&managed.config.log_file);
        if !log_path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&log_path).map_err(|e| format!("Failed to open log file: {e}"))?;
        let reader = BufReader::new(file);

        let all_lines: Vec<String> = reader.lines().filter_map(|l| l.ok()).collect();
        let start = all_lines.len().saturating_sub(lines);

        Ok(all_lines[start..].to_vec())
    }

    /// Check if a process is registered
    pub fn is_registered(&self, id: &str) -> bool {
        self.processes.read().contains_key(id)
    }

    /// Check if a process is running
    pub fn is_running(&self, id: &str) -> bool {
        self.get_status(id).map_or(false, |info| info.is_running)
    }

    /// Stop all managed processes (useful for cleanup on app exit)
    #[allow(dead_code)]
    pub fn stop_all(&self) {
        let mut processes = self.processes.write();

        for (id, managed) in processes.iter_mut() {
            // Stop via Child handle
            if let Some(ref mut child) = managed.child {
                log::info!("Stopping process '{}' during cleanup", id);
                let _ = child.kill();
                let _ = child.wait();
            }
            // Also stop via external PID
            else if let Some(ext_pid) = managed.external_pid {
                if Self::is_process_alive(ext_pid) {
                    log::info!(
                        "Stopping process '{}' (external pid: {}) during cleanup",
                        id,
                        ext_pid
                    );
                    Self::kill_process_by_pid(ext_pid);
                }
            }
            managed.child = None;
            managed.external_pid = None;
            managed.started_at = None;
        }

        drop(processes);
        self.persist_state();
    }
}

// Global process manager instance
lazy_static::lazy_static! {
    pub static ref PROCESS_MANAGER: Arc<ProcessManager> = Arc::new(ProcessManager::new());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_manager_register() {
        let pm = ProcessManager::new();
        let config = ProcessConfig {
            id: "test".into(),
            name: "Test Process".into(),
            bin_path: "/bin/echo".into(),
            args: vec!["hello".into()],
            log_file: "/tmp/test.log".into(),
            working_dir: None,
            env_vars: None,
        };

        let result = pm.register(config.clone());
        assert!(result.is_ok());

        // Registering same ID should fail
        let result = pm.register(config);
        assert!(result.is_err());
    }
}
