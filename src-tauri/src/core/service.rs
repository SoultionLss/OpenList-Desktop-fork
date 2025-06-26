use log::error;

use std::env;
use std::process::Command as StdCommand;

#[cfg(target_os = "windows")]
pub async fn install_service() -> Result<bool, Box<dyn std::error::Error>> {
    use deelevate::{PrivilegeLevel, Token};
    use runas::Command as RunasCommand;
    use std::os::windows::process::CommandExt;

    let app_dir = env::current_exe().unwrap().parent().unwrap().to_path_buf();
    let install_path = app_dir.join("install-openlist-service.exe");

    if !install_path.exists() {
        error!("Service installer not found at {}", install_path.display());
        return Err(Box::from(format!(
            "Service installer not found at {}",
            install_path.display()
        )));
    }
    let token = Token::with_current_process()?;
    let level = token.privilege_level()?;
    let status = match level {
        PrivilegeLevel::NotPrivileged => RunasCommand::new(install_path).show(false).status()?,
        _ => StdCommand::new(install_path)
            .creation_flags(0x08000000)
            .status()?,
    };
    if status.success() {
        Ok(true)
    } else {
        Err(Box::from(format!(
            "Failed to install service, exit status: {}",
            status
        )))
    }
}

#[cfg(target_os = "windows")]
pub async fn uninstall_service() -> Result<bool, Box<dyn std::error::Error>> {
    use deelevate::{PrivilegeLevel, Token};
    use runas::Command as RunasCommand;
    use std::os::windows::process::CommandExt;

    let app_dir = env::current_exe().unwrap().parent().unwrap().to_path_buf();
    let uninstall_path = app_dir.join("uninstall-openlist-service.exe");

    if !uninstall_path.exists() {
        error!("Uninstaller not found: {:?}", uninstall_path);
        return Err(Box::from(format!(
            "uninstaller not found: {uninstall_path:?}"
        )));
    }

    let token = Token::with_current_process()?;
    let level = token.privilege_level()?;
    let status = match level {
        PrivilegeLevel::NotPrivileged => RunasCommand::new(uninstall_path).show(false).status()?,
        _ => StdCommand::new(uninstall_path)
            .creation_flags(0x08000000)
            .status()?,
    };

    if status.success() {
        Ok(true)
    } else {
        Err(Box::from(format!(
            "Failed to uninstall service, exit status: {}",
            status
        )))
    }
}

#[cfg(target_os = "linux")]
pub async fn install_service() -> Result<bool, Box<dyn std::error::Error>> {
    use users::get_effective_uid;

    let app_dir = env::current_exe().unwrap().parent().unwrap().to_path_buf();
    let install_path = app_dir.join("install-openlist-service");

    if !install_path.exists() {
        error!("Service installer not found at {}", install_path.display());
        return Err(Box::from(format!(
            "Service installer not found at {}",
            install_path.display()
        )));
    }

    let install_shell: String = install_path.to_string_lossy().replace(" ", "\\ ");

    let elevator = linux_elevator();
    let status = match get_effective_uid() {
        0 => StdCommand::new(install_shell).status()?,
        _ => StdCommand::new(elevator.clone())
            .arg("sh")
            .arg("-c")
            .arg(install_shell)
            .status()?,
    };
    if status.success() {
        Ok(true)
    } else {
        Err(Box::from(format!(
            "Failed to install service, exit status: {}",
            status
        )))
    }
}

#[cfg(target_os = "linux")]
pub async fn uninstall_service() -> Result<bool, Box<dyn std::error::Error>> {
    use users::get_effective_uid;

    let app_dir = env::current_exe().unwrap().parent().unwrap().to_path_buf();
    let uninstall_path = app_dir.join("uninstall-openlist-service");

    if !uninstall_path.exists() {
        error!("Uninstaller not found: {:?}", uninstall_path);
        return Err(Box::from(format!(
            "Uninstaller not found: {:?}",
            uninstall_path
        )));
    }

    let uninstall_shell: String = uninstall_path.to_string_lossy().replace(" ", "\\ ");

    let elevator = linux_elevator();
    let status = match get_effective_uid() {
        0 => StdCommand::new(uninstall_shell).status()?,
        _ => StdCommand::new(elevator.clone())
            .arg("sh")
            .arg("-c")
            .arg(uninstall_shell)
            .status()?,
    };

    if status.success() {
        Ok(true)
    } else {
        Err(Box::from(format!(
            "Failed to uninstall service, exit status: {}",
            status
        )))
    }
}

#[cfg(target_os = "linux")]
pub fn linux_elevator() -> String {
    match StdCommand::new("which").arg("pkexec").output() {
        Ok(output) => {
            if !output.stdout.is_empty() {
                if let Ok(path) = std::str::from_utf8(&output.stdout) {
                    path.trim().to_string()
                } else {
                    "sudo".to_string()
                }
            } else {
                "sudo".to_string()
            }
        }
        Err(_) => "sudo".to_string(),
    }
}

#[cfg(target_os = "windows")]
fn start_service_with_elevation(service_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    use deelevate::{PrivilegeLevel, Token};
    use runas::Command as RunasCommand;
    use std::os::windows::process::CommandExt;

    let token = Token::with_current_process()?;
    let level = token.privilege_level()?;

    let powershell_cmd = format!("Start-Service -Name '{}'", service_name);

    let status = match level {
        PrivilegeLevel::NotPrivileged => {
            log::info!("Running without admin privileges, using runas for elevation");
            RunasCommand::new("powershell.exe")
                .args(&["-Command", &powershell_cmd])
                .show(false)
                .status()?
        }
        _ => {
            log::info!("Already have admin privileges, running directly");
            StdCommand::new("powershell.exe")
                .args(&["-Command", &powershell_cmd])
                .creation_flags(0x08000000)
                .status()?
        }
    };

    if status.success() {
        log::info!("Service started successfully via PowerShell");
        Ok(true)
    } else {
        log::error!(
            "Failed to start service via PowerShell, exit code: {}",
            status
        );
        Ok(false)
    }
}

#[cfg(not(target_os = "windows"))]
pub async fn start_service() -> Result<bool, Box<dyn std::error::Error>> {
    log::info!("Service start not implemented for this platform");
    Ok(false)
}

#[cfg(target_os = "macos")]
pub async fn install_service() -> Result<bool, Box<dyn std::error::Error>> {
    let app_dir = env::current_exe()?.parent().unwrap().to_path_buf();
    let install_path = app_dir.join("install-openlist-service");

    if !install_path.exists() {
        error!("Service installer not found at {}", install_path.display());
        return Err(Box::from(format!(
            "Service installer not found at {}",
            install_path.display()
        )));
    }

    let status = StdCommand::new(&install_path).status()?;

    if status.success() {
        Ok(true)
    } else {
        Err(Box::from(format!(
            "Failed to install service, exit status: {}",
            status
        )))
    }
}

#[cfg(target_os = "macos")]
pub async fn uninstall_service() -> Result<bool, Box<dyn std::error::Error>> {
    let app_dir = env::current_exe()?.parent().unwrap().to_path_buf();
    let uninstall_path = app_dir.join("uninstall-openlist-service");

    if !uninstall_path.exists() {
        error!("Uninstaller not found: {:?}", uninstall_path);
        return Err(Box::from(format!(
            "Uninstaller not found: {:?}",
            uninstall_path
        )));
    }
    let status = StdCommand::new(&uninstall_path).status()?;

    if status.success() {
        Ok(true)
    } else {
        Err(Box::from(format!(
            "Failed to uninstall service, exit status: {}",
            status
        )))
    }
}

#[cfg(target_os = "windows")]
pub async fn check_service_status() -> Result<bool, Box<dyn std::error::Error>> {
    use windows_service::service::{ServiceAccess, ServiceState};
    use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};
    let service_name = "openlist_desktop_service";

    let manager = match ServiceManager::local_computer(
        None::<&str>,
        ServiceManagerAccess::CONNECT | ServiceManagerAccess::ENUMERATE_SERVICE,
    ) {
        Ok(mgr) => mgr,
        Err(_) => ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)?,
    };
    let service = match manager.open_service(service_name, ServiceAccess::QUERY_STATUS) {
        Ok(svc) => svc,
        Err(e) => {
            log::error!("Failed to open service '{}': {:?}", service_name, e);
            return Ok(false);
        }
    };
    match service.query_status() {
        Ok(status) => {
            match status.current_state {
                ServiceState::Running => {
                    return Ok(true);
                }
                ServiceState::StartPending => {
                    return Ok(true);
                }
                ServiceState::StopPending => {
                    std::thread::sleep(std::time::Duration::from_millis(1000));
                }
                _ => {
                    log::info!(
                        "Service is in state: {:?}, attempting to start...",
                        status.current_state
                    );
                }
            }

            match start_service_with_elevation(service_name) {
                Ok(true) => Ok(true),
                Ok(false) => Ok(false),
                Err(e) => {
                    log::error!("Error during service elevation: {:?}", e);
                    Ok(false)
                }
            }
        }
        Err(e) => {
            log::error!("Failed to query service status: {:?}", e);
            match start_service_with_elevation(service_name) {
                Ok(true) => Ok(true),
                Ok(false) => {
                    log::error!("Failed to start service with elevation.");
                    Ok(false)
                }
                Err(elev_err) => {
                    log::error!("Error during service elevation: {:?}", elev_err);
                    Ok(false)
                }
            }
        }
    }
}

#[cfg(target_os = "linux")]
pub async fn check_service_status() -> Result<bool, Box<dyn std::error::Error>> {
    const SERVICE_NAME: &str = "openlist-desktop-service";

    log::info!("Checking Linux service status for: {}", SERVICE_NAME);

    let init_system = detect_linux_init_system();

    match init_system.as_str() {
        "systemd" => check_systemd_service_status(SERVICE_NAME).await,
        "openrc" => check_openrc_service_status(SERVICE_NAME).await,
        _ => {
            log::warn!("Unknown init system: {}, assuming systemd", init_system);
            check_systemd_service_status(SERVICE_NAME).await
        }
    }
}

#[cfg(target_os = "linux")]
fn detect_linux_init_system() -> String {
    if std::path::Path::new("/run/systemd/system").exists() {
        return "systemd".to_string();
    }

    if std::path::Path::new("/run/openrc").exists() {
        return "openrc".to_string();
    }

    if let Ok(output) = StdCommand::new("which").arg("systemctl").output() {
        if output.status.success() && !output.stdout.is_empty() {
            return "systemd".to_string();
        }
    }

    if let Ok(output) = StdCommand::new("which").arg("rc-service").output() {
        if output.status.success() && !output.stdout.is_empty() {
            return "openrc".to_string();
        }
    }

    "systemd".to_string()
}

#[cfg(target_os = "linux")]
async fn check_systemd_service_status(
    service_name: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    log::info!("Checking systemd service status for: {}", service_name);

    let status_output = StdCommand::new("systemctl")
        .args(&["is-active", service_name])
        .output();

    match status_output {
        Ok(output) => {
            let status = String::from_utf8_lossy(&output.stdout)
                .trim()
                .to_lowercase();
            log::info!("Service {} status: {}", service_name, status);

            match status.as_str() {
                "active" | "activating" => {
                    log::info!("Service is active and running");
                    return Ok(true);
                }
                "inactive" | "failed" => {
                    log::info!("Service is {}, attempting to start", status);
                    return start_systemd_service(service_name).await;
                }
                "unknown" => {
                    log::warn!("Service status unknown, checking if service exists");
                    let exists_output = StdCommand::new("systemctl")
                        .args(&["list-unit-files", &format!("{}.service", service_name)])
                        .output();

                    match exists_output {
                        Ok(output) if output.status.success() => {
                            let output_str = String::from_utf8_lossy(&output.stdout);
                            if output_str.contains(service_name) {
                                log::info!("Service exists but not active, attempting to start");
                                return start_systemd_service(service_name).await;
                            } else {
                                log::error!("Service {} not found", service_name);
                                return Ok(false);
                            }
                        }
                        _ => {
                            log::error!("Failed to check if service exists");
                            return Ok(false);
                        }
                    }
                }
                _ => {
                    log::warn!("Unknown service status: {}, attempting to start", status);
                    return start_systemd_service(service_name).await;
                }
            }
        }
        Err(e) => {
            log::error!("Failed to check systemd service status: {}", e);
            return start_systemd_service(service_name).await;
        }
    }
}

#[cfg(target_os = "linux")]
async fn start_systemd_service(service_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    use users::get_effective_uid;

    log::info!("Attempting to start systemd service: {}", service_name);

    let status = match get_effective_uid() {
        0 => StdCommand::new("systemctl")
            .args(&["start", service_name])
            .status()?,
        _ => {
            let elevator = linux_elevator();
            log::info!("Using {} for elevation", elevator);

            StdCommand::new(&elevator)
                .args(&["systemctl", "start", service_name])
                .status()?
        }
    };

    if status.success() {
        log::info!("Service started successfully");
        std::thread::sleep(std::time::Duration::from_millis(1000));

        let verify_output = StdCommand::new("systemctl")
            .args(&["is-active", service_name])
            .output()?;

        let verify_status_str = String::from_utf8_lossy(&verify_output.stdout);
        let verify_status = verify_status_str.trim();
        let is_running = verify_status == "active" || verify_status == "activating";

        if is_running {
            log::info!("Service verified as running");
        } else {
            log::warn!(
                "Service start command succeeded but service is not active: {}",
                verify_status
            );
        }

        Ok(is_running)
    } else {
        log::error!("Failed to start service, exit code: {}", status);
        Ok(false)
    }
}

#[cfg(target_os = "linux")]
async fn check_openrc_service_status(
    service_name: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    log::info!("Checking OpenRC service status for: {}", service_name);

    let status_output = StdCommand::new("rc-service")
        .args(&[service_name, "status"])
        .output();

    match status_output {
        Ok(output) => {
            let status_str = String::from_utf8_lossy(&output.stdout).to_lowercase();
            let stderr_str = String::from_utf8_lossy(&output.stderr).to_lowercase();

            log::info!("OpenRC service status output: {}", status_str);

            if status_str.contains("started") || status_str.contains("running") {
                log::info!("Service is running");
                return Ok(true);
            } else if status_str.contains("stopped") || status_str.contains("inactive") {
                log::info!("Service is stopped, attempting to start");
                return start_openrc_service(service_name).await;
            } else if stderr_str.contains("does not exist") {
                log::error!("Service {} does not exist", service_name);
                return Ok(false);
            } else {
                log::warn!("Unknown service status, attempting to start");
                return start_openrc_service(service_name).await;
            }
        }
        Err(e) => {
            log::error!("Failed to check OpenRC service status: {}", e);
            return start_openrc_service(service_name).await;
        }
    }
}

#[cfg(target_os = "linux")]
async fn start_openrc_service(service_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    use users::get_effective_uid;

    log::info!("Attempting to start OpenRC service: {}", service_name);
    let status = match get_effective_uid() {
        0 => StdCommand::new("rc-service")
            .args(&[service_name, "start"])
            .status()?,
        _ => {
            let elevator = linux_elevator();
            log::info!("Using {} for elevation", elevator);

            StdCommand::new(&elevator)
                .args(&["rc-service", service_name, "start"])
                .status()?
        }
    };

    if status.success() {
        log::info!("Service start command completed");
        std::thread::sleep(std::time::Duration::from_millis(1000));

        let verify_output = StdCommand::new("rc-service")
            .args(&[service_name, "status"])
            .output()?;

        let verify_status = String::from_utf8_lossy(&verify_output.stdout).to_lowercase();
        let is_running = verify_status.contains("started") || verify_status.contains("running");

        if is_running {
            log::info!("Service verified as running");
        } else {
            log::warn!(
                "Service start command succeeded but service is not running: {}",
                verify_status
            );
        }

        Ok(is_running)
    } else {
        log::error!("Failed to start OpenRC service, exit code: {}", status);
        Ok(false)
    }
}

#[cfg(target_os = "macos")]
pub async fn check_service_status() -> Result<bool, Box<dyn std::error::Error>> {
    const SERVICE_IDENTIFIER: &str = "io.github.openlistteam.openlist.service";

    log::info!("Checking macOS service status for: {}", SERVICE_IDENTIFIER);

    let status_output = StdCommand::new("launchctl")
        .args(&["list", SERVICE_IDENTIFIER])
        .output();

    match status_output {
        Ok(output) => {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                log::info!("launchctl list output: {}", output_str);

                if let Some(pid_value) = extract_plist_value(&output_str, "PID") {
                    log::info!("Extracted PID value: {}", pid_value);
                    if let Ok(pid) = pid_value.parse::<i32>() {
                        if pid > 0 {
                            log::info!("Service is running with PID: {}", pid);
                            return Ok(true);
                        }
                    }
                }

                if let Some(exit_status) = extract_plist_value(&output_str, "LastExitStatus") {
                    if let Ok(status) = exit_status.parse::<i32>() {
                        if status == 0 {
                            log::info!(
                                "Service is loaded but not running (clean exit), attempting to start"
                            );
                            return start_macos_service(SERVICE_IDENTIFIER).await;
                        } else {
                            log::warn!(
                                "Service has non-zero exit status: {}, attempting to restart",
                                status
                            );
                            return restart_macos_service(SERVICE_IDENTIFIER).await;
                        }
                    }
                }

                log::info!("Service appears to be loaded but status unclear, attempting to start");
                return start_macos_service(SERVICE_IDENTIFIER).await;
            } else {
                let stderr_str = String::from_utf8_lossy(&output.stderr);
                if stderr_str.contains("Could not find service") {
                    log::error!("Service {} is not loaded", SERVICE_IDENTIFIER);
                    return Ok(false);
                } else {
                    log::warn!("launchctl list failed, attempting to start service anyway");
                    return start_macos_service(SERVICE_IDENTIFIER).await;
                }
            }
        }
        Err(e) => {
            log::error!("Failed to check macOS service status: {}", e);
            return start_macos_service(SERVICE_IDENTIFIER).await;
        }
    }
}

#[cfg(target_os = "macos")]
async fn start_macos_service(service_identifier: &str) -> Result<bool, Box<dyn std::error::Error>> {
    log::info!("Attempting to start macOS service: {}", service_identifier);

    let status = StdCommand::new("launchctl")
        .args(&["start", service_identifier])
        .status()?;

    if status.success() {
        log::info!("Service start command completed");
        std::thread::sleep(std::time::Duration::from_millis(2000));

        let verify_output = StdCommand::new("launchctl")
            .args(&["list", service_identifier])
            .output()?;

        if verify_output.status.success() {
            let output_str = String::from_utf8_lossy(&verify_output.stdout);
            log::info!("Verification output: {}", output_str);

            if let Some(pid_value) = extract_plist_value(&output_str, "PID") {
                if let Ok(pid) = pid_value.parse::<i32>() {
                    if pid > 0 {
                        log::info!("Service verified as running with PID: {}", pid);
                        return Ok(true);
                    } else {
                        log::warn!("Service has invalid PID: {}", pid);
                        return Ok(false);
                    }
                }
            }

            if output_str.contains("Label") && output_str.contains(service_identifier) {
                log::warn!("Service is loaded but PID could not be determined");
                return Ok(false);
            }
        }

        log::warn!("Could not verify service status after start");
        Ok(false)
    } else {
        log::error!("Failed to start macOS service, exit code: {}", status);
        Ok(false)
    }
}

#[cfg(target_os = "macos")]
fn extract_plist_value(plist_output: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\"", key);

    for line in plist_output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&pattern) {
            if let Some(equals_pos) = trimmed.find('=') {
                let value_part = &trimmed[equals_pos + 1..];
                let value_trimmed = value_part.trim();

                let value_clean = if value_trimmed.ends_with(';') {
                    &value_trimmed[..value_trimmed.len() - 1]
                } else {
                    value_trimmed
                };

                return Some(value_clean.trim().to_string());
            }
        }
    }

    None
}

#[cfg(target_os = "macos")]
async fn restart_macos_service(
    service_identifier: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    log::info!(
        "Attempting to restart macOS service: {}",
        service_identifier
    );

    let _ = StdCommand::new("launchctl")
        .args(&["stop", service_identifier])
        .status();

    std::thread::sleep(std::time::Duration::from_millis(500));

    start_macos_service(service_identifier).await
}

#[cfg(target_os = "windows")]
pub async fn stop_service() -> Result<bool, Box<dyn std::error::Error>> {
    use deelevate::{PrivilegeLevel, Token};
    use runas::Command as RunasCommand;
    use std::os::windows::process::CommandExt;

    let service_name = "openlist_desktop_service";
    log::info!("Attempting to stop Windows service: {}", service_name);

    let token = Token::with_current_process()?;
    let level = token.privilege_level()?;

    let powershell_cmd = format!("Stop-Service -Name '{}' -Force", service_name);

    let status = match level {
        PrivilegeLevel::NotPrivileged => {
            log::info!("Running without admin privileges, using runas for elevation");
            RunasCommand::new("powershell.exe")
                .args(&["-Command", &powershell_cmd])
                .show(false)
                .status()?
        }
        _ => {
            log::info!("Already have admin privileges, running directly");
            StdCommand::new("powershell.exe")
                .args(&["-Command", &powershell_cmd])
                .creation_flags(0x08000000)
                .status()?
        }
    };

    if status.success() {
        log::info!("Service stopped successfully");
        std::thread::sleep(std::time::Duration::from_secs(1));
        Ok(true)
    } else {
        log::error!("Failed to stop service, exit code: {}", status);
        Ok(false)
    }
}

#[cfg(target_os = "linux")]
pub async fn stop_service() -> Result<bool, Box<dyn std::error::Error>> {
    const SERVICE_NAME: &str = "openlist-desktop-service";

    log::info!("Attempting to stop Linux service: {}", SERVICE_NAME);

    let init_system = detect_linux_init_system();

    match init_system.as_str() {
        "systemd" => stop_systemd_service(SERVICE_NAME).await,
        "openrc" => stop_openrc_service(SERVICE_NAME).await,
        _ => {
            log::warn!("Unknown init system: {}, assuming systemd", init_system);
            stop_systemd_service(SERVICE_NAME).await
        }
    }
}

#[cfg(target_os = "linux")]
async fn stop_systemd_service(service_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    use users::get_effective_uid;

    log::info!("Attempting to stop systemd service: {}", service_name);

    let status = match get_effective_uid() {
        0 => StdCommand::new("systemctl")
            .args(&["stop", service_name])
            .status()?,
        _ => {
            let elevator = linux_elevator();
            log::info!("Using {} for elevation", elevator);

            StdCommand::new(&elevator)
                .args(&["systemctl", "stop", service_name])
                .status()?
        }
    };

    if status.success() {
        log::info!("Service stop command completed");
        std::thread::sleep(std::time::Duration::from_secs(1));

        let verify_output = StdCommand::new("systemctl")
            .args(&["is-active", service_name])
            .output()?;

        let verify_status_str = String::from_utf8_lossy(&verify_output.stdout);
        let verify_status = verify_status_str.trim();
        let is_stopped = verify_status == "inactive" || verify_status == "failed";

        if is_stopped {
            log::info!("Service verified as stopped");
        } else {
            log::warn!(
                "Service stop command succeeded but service is still active: {}",
                verify_status
            );
        }

        Ok(is_stopped)
    } else {
        log::error!("Failed to stop systemd service, exit code: {}", status);
        Ok(false)
    }
}

#[cfg(target_os = "linux")]
async fn stop_openrc_service(service_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    use users::get_effective_uid;

    log::info!("Attempting to stop OpenRC service: {}", service_name);
    let status = match get_effective_uid() {
        0 => StdCommand::new("rc-service")
            .args(&[service_name, "stop"])
            .status()?,
        _ => {
            let elevator = linux_elevator();
            log::info!("Using {} for elevation", elevator);

            StdCommand::new(&elevator)
                .args(&["rc-service", service_name, "stop"])
                .status()?
        }
    };

    if status.success() {
        log::info!("Service stop command completed");
        std::thread::sleep(std::time::Duration::from_secs(1));

        let verify_output = StdCommand::new("rc-service")
            .args(&[service_name, "status"])
            .output()?;

        let verify_status = String::from_utf8_lossy(&verify_output.stdout).to_lowercase();
        let is_stopped = verify_status.contains("stopped") || verify_status.contains("inactive");

        if is_stopped {
            log::info!("Service verified as stopped");
        } else {
            log::warn!(
                "Service stop command succeeded but service is still running: {}",
                verify_status
            );
        }

        Ok(is_stopped)
    } else {
        log::error!("Failed to stop OpenRC service, exit code: {}", status);
        Ok(false)
    }
}

#[cfg(target_os = "macos")]
pub async fn stop_service() -> Result<bool, Box<dyn std::error::Error>> {
    const SERVICE_IDENTIFIER: &str = "io.github.openlistteam.openlist.service";

    log::info!("Attempting to stop macOS service: {}", SERVICE_IDENTIFIER);

    let status = StdCommand::new("launchctl")
        .args(&["stop", SERVICE_IDENTIFIER])
        .status()?;

    if status.success() {
        log::info!("Service stop command completed");
        std::thread::sleep(std::time::Duration::from_secs(1));

        let verify_output = StdCommand::new("launchctl")
            .args(&["list", SERVICE_IDENTIFIER])
            .output()?;

        if verify_output.status.success() {
            let output_str = String::from_utf8_lossy(&verify_output.stdout);
            log::info!("Verification output after stop: {}", output_str);

            if let Some(pid_value) = extract_plist_value(&output_str, "PID") {
                if let Ok(pid) = pid_value.parse::<i32>() {
                    let is_stopped = pid <= 0;

                    if is_stopped {
                        log::info!("Service verified as stopped");
                    } else {
                        log::warn!(
                            "Service stop command succeeded but service is still running with PID: {}",
                            pid
                        );
                    }

                    return Ok(is_stopped);
                }
            }

            log::info!("No PID found in output, service appears to be stopped");
            return Ok(true);
        }

        log::info!("Could not verify service status after stop, assuming success");
        Ok(true)
    } else {
        log::error!("Failed to stop macOS service, exit code: {}", status);
        Ok(false)
    }
}

#[cfg(target_os = "windows")]
pub async fn restart_service() -> Result<bool, Box<dyn std::error::Error>> {
    use deelevate::{PrivilegeLevel, Token};
    use runas::Command as RunasCommand;
    use std::os::windows::process::CommandExt;

    let service_name = "openlist_desktop_service";
    log::info!("Attempting to restart Windows service: {}", service_name);

    let powershell_cmd = format!("Restart-Service -Name '{}' -Force", service_name);

    let status = {
        let token = Token::with_current_process()?;
        let level = token.privilege_level()?;

        match level {
            PrivilegeLevel::NotPrivileged => {
                log::info!("Running without admin privileges, using runas for elevation");
                RunasCommand::new("powershell.exe")
                    .args(&["-Command", &powershell_cmd])
                    .show(false)
                    .status()?
            }
            _ => {
                log::info!("Already have admin privileges, running directly");
                StdCommand::new("powershell.exe")
                    .args(&["-Command", &powershell_cmd])
                    .creation_flags(0x08000000)
                    .status()?
            }
        }
    };

    if status.success() {
        log::info!("Service restart command completed");
        std::thread::sleep(std::time::Duration::from_secs(2));

        match check_service_status().await {
            Ok(true) => {
                log::info!("Service verified as running after restart");
                Ok(true)
            }
            Ok(false) => {
                log::warn!("Service restart command succeeded but service is not running");
                Ok(false)
            }
            Err(e) => {
                log::error!("Error verifying service status after restart: {}", e);
                Ok(false)
            }
        }
    } else {
        log::error!("Failed to restart service, exit code: {}", status);
        Ok(false)
    }
}

#[cfg(target_os = "linux")]
pub async fn restart_service() -> Result<bool, Box<dyn std::error::Error>> {
    const SERVICE_NAME: &str = "openlist-desktop-service";

    log::info!("Attempting to restart Linux service: {}", SERVICE_NAME);

    let init_system = detect_linux_init_system();

    match init_system.as_str() {
        "systemd" => restart_systemd_service(SERVICE_NAME).await,
        "openrc" => restart_openrc_service(SERVICE_NAME).await,
        _ => {
            log::warn!("Unknown init system: {}, assuming systemd", init_system);
            restart_systemd_service(SERVICE_NAME).await
        }
    }
}

#[cfg(target_os = "linux")]
async fn restart_systemd_service(service_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    use users::get_effective_uid;

    log::info!("Attempting to restart systemd service: {}", service_name);
    let status = match get_effective_uid() {
        0 => StdCommand::new("systemctl")
            .args(&["restart", service_name])
            .status()?,
        _ => {
            let elevator = linux_elevator();
            log::info!("Using {} for elevation", elevator);

            StdCommand::new(&elevator)
                .args(&["systemctl", "restart", service_name])
                .status()?
        }
    };

    if status.success() {
        log::info!("Service restart command completed");
        std::thread::sleep(std::time::Duration::from_secs(2));

        let verify_output = StdCommand::new("systemctl")
            .args(&["is-active", service_name])
            .output()?;

        let verify_status_str = String::from_utf8_lossy(&verify_output.stdout);
        let verify_status = verify_status_str.trim();
        let is_running = verify_status == "active" || verify_status == "activating";

        if is_running {
            log::info!("Service verified as running after restart");
        } else {
            log::warn!(
                "Service restart command succeeded but service is not active: {}",
                verify_status
            );
        }

        Ok(is_running)
    } else {
        log::error!("Failed to restart systemd service, exit code: {}", status);
        Ok(false)
    }
}

#[cfg(target_os = "linux")]
async fn restart_openrc_service(service_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    use users::get_effective_uid;

    log::info!("Attempting to restart OpenRC service: {}", service_name);
    let status = match get_effective_uid() {
        0 => StdCommand::new("rc-service")
            .args(&[service_name, "restart"])
            .status()?,
        _ => {
            let elevator = linux_elevator();
            log::info!("Using {} for elevation", elevator);

            StdCommand::new(&elevator)
                .args(&["rc-service", service_name, "restart"])
                .status()?
        }
    };

    if status.success() {
        log::info!("Service restart command completed");
        std::thread::sleep(std::time::Duration::from_secs(2));

        let verify_output = StdCommand::new("rc-service")
            .args(&[service_name, "status"])
            .output()?;

        let verify_status = String::from_utf8_lossy(&verify_output.stdout).to_lowercase();
        let is_running = verify_status.contains("started") || verify_status.contains("running");

        if is_running {
            log::info!("Service verified as running after restart");
        } else {
            log::warn!(
                "Service restart command succeeded but service is not running: {}",
                verify_status
            );
        }

        Ok(is_running)
    } else {
        log::error!("Failed to restart OpenRC service, exit code: {}", status);
        Ok(false)
    }
}

#[cfg(target_os = "macos")]
pub async fn restart_service() -> Result<bool, Box<dyn std::error::Error>> {
    const SERVICE_IDENTIFIER: &str = "io.github.openlistteam.openlist.service";

    log::info!(
        "Attempting to restart macOS service: {}",
        SERVICE_IDENTIFIER
    );

    let stop_result = stop_service().await?;
    if !stop_result {
        log::warn!("Failed to stop service, but continuing with restart attempt");
    }

    std::thread::sleep(std::time::Duration::from_millis(500));

    start_macos_service(SERVICE_IDENTIFIER).await
}
