#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
#[cfg(target_os = "windows")]
use std::process::{Command, ExitStatus};

#[cfg(target_os = "windows")]
use deelevate::{PrivilegeLevel, Token};
#[cfg(target_os = "windows")]
use runas::Command as RunasCommand;
use tauri::State;

use crate::object::structs::AppState;

#[cfg(target_os = "windows")]
const RULE: &str = "OpenList Core";

#[cfg(target_os = "windows")]
fn netsh_with_elevation(args: &[String]) -> Result<ExitStatus, String> {
    let token = Token::with_current_process().map_err(|e| format!("token: {e}"))?;
    let elevated = !matches!(
        token
            .privilege_level()
            .map_err(|e| format!("privilege: {e}"))?,
        PrivilegeLevel::NotPrivileged
    );

    if elevated {
        Command::new("netsh")
            .args(args)
            .creation_flags(0x08000000)
            .status()
    } else {
        RunasCommand::new("netsh").args(args).show(false).status()
    }
    .map_err(|e| format!("netsh: {e}"))
}

#[cfg(target_os = "windows")]
fn firewall_rule(verb: &str, port: Option<u16>) -> Result<bool, String> {
    let mut args: Vec<String> = vec![
        "advfirewall".into(),
        "firewall".into(),
        verb.into(),
        "rule".into(),
        format!("name={RULE}"),
    ];
    if let Some(p) = port {
        args.extend([
            "dir=in".into(),
            "action=allow".into(),
            "protocol=TCP".into(),
            format!("localport={p}"),
            "description=Allow OpenList Core web interface access".into(),
        ]);
    }
    Ok(netsh_with_elevation(&args)?.success())
}

#[cfg(target_os = "windows")]
fn rule_stdout() -> Result<Option<String>, String> {
    let out = Command::new("netsh")
        .args([
            "advfirewall",
            "firewall",
            "show",
            "rule",
            &format!("name={RULE}"),
        ])
        .creation_flags(0x08000000)
        .output()
        .map_err(|e| format!("netsh: {e}"))?;
    if out.status.success() {
        Ok(Some(String::from_utf8_lossy(&out.stdout).into()))
    } else {
        Ok(None)
    }
}

#[cfg(not(target_os = "windows"))]
fn firewall_rule(_: &str, _: Option<u16>) -> Result<bool, String> {
    Ok(true)
}
#[cfg(not(target_os = "windows"))]
fn rule_stdout() -> Result<Option<String>, String> {
    Ok(None)
}

#[tauri::command]
pub async fn check_firewall_rule(state: State<'_, AppState>) -> Result<bool, String> {
    let port = state
        .app_settings
        .read()
        .clone()
        .ok_or("read settings")?
        .openlist
        .port;

    if let Some(out) = rule_stdout()? {
        Ok(out.contains(&port.to_string()))
    } else {
        Ok(false)
    }
}

#[tauri::command]
pub async fn add_firewall_rule(state: State<'_, AppState>) -> Result<bool, String> {
    let port = state
        .app_settings
        .read()
        .clone()
        .ok_or("read settings")?
        .openlist
        .port;

    let _ = firewall_rule("delete", None);
    firewall_rule("add", Some(port))
}

#[tauri::command]
pub async fn remove_firewall_rule(_state: State<'_, AppState>) -> Result<bool, String> {
    firewall_rule("delete", None)
}
