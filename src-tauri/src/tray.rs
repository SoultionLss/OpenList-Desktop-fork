use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};

use crate::cmd;
use crate::object::structs::AppState;

pub fn create_tray(app_handle: &AppHandle) -> tauri::Result<()> {
    let quit_i = MenuItem::with_id(app_handle, "quit", "退出", true, None::<&str>)?;
    let show_i = MenuItem::with_id(app_handle, "show", "显示窗口", true, None::<&str>)?;
    let hide_i = MenuItem::with_id(app_handle, "hide", "隐藏窗口", true, None::<&str>)?;
    let restart_i = MenuItem::with_id(app_handle, "restart", "重启应用", true, None::<&str>)?;

    let start_service_i = MenuItem::with_id(
        app_handle,
        "start_service",
        "启动OpenList",
        true,
        None::<&str>,
    )?;
    let stop_service_i = MenuItem::with_id(
        app_handle,
        "stop_service",
        "停止OpenList",
        true,
        None::<&str>,
    )?;
    let restart_service_i = MenuItem::with_id(
        app_handle,
        "restart_service",
        "重启OpenList",
        true,
        None::<&str>,
    )?;
    let service_submenu = Submenu::with_id_and_items(
        app_handle,
        "service",
        "核心控制",
        true,
        &[&start_service_i, &stop_service_i, &restart_service_i],
    )?;

    let menu = Menu::with_items(
        app_handle,
        &[
            &show_i,
            &hide_i,
            &PredefinedMenuItem::separator(app_handle)?,
            &service_submenu,
            &PredefinedMenuItem::separator(app_handle)?,
            &restart_i,
            &quit_i,
        ],
    )?;

    let _tray = TrayIconBuilder::with_id("main-tray")
        .tooltip("OpenList Desktop")
        .icon(app_handle.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            let app_handle = tray.app_handle();
            match event {
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } => {
                    handle_tray_click(app_handle);
                }
                TrayIconEvent::Enter { .. } => {
                    log::debug!("Mouse entered tray icon area");
                }
                TrayIconEvent::Leave { .. } => {
                    log::debug!("Mouse left tray icon area");
                }
                _ => {
                    log::debug!("Other tray event: {event:?}");
                }
            }
        })
        .on_menu_event(|app_handle, event| {
            let app_handle = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                handle_menu_event(&app_handle, event).await;
            });
        })
        .build(app_handle)?;

    Ok(())
}

fn handle_tray_click(app_handle: &AppHandle) {
    if let Some(window) = app_handle.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

async fn handle_menu_event(app_handle: &AppHandle, event: tauri::menu::MenuEvent) {
    match event.id().as_ref() {
        "quit" => {
            log::info!("Quit menu item clicked");
            app_handle.exit(0);
        }
        "show" => {
            if let Some(window) = app_handle.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        "hide" => {
            if let Some(window) = app_handle.get_webview_window("main") {
                let _ = window.hide();
            }
        }
        "restart" => {
            log::info!("Restart menu item clicked");
            app_handle.restart();
        }
        "start_service" => {
            log::info!("Start service menu item clicked");
            if let Err(e) = handle_core_action(app_handle, "start").await {
                log::error!("Failed to start OpenList Core: {e}");
            } else {
                update_tray_menu(app_handle, true).ok();
            }
        }
        "stop_service" => {
            log::info!("Stop service menu item clicked");
            if let Err(e) = handle_core_action(app_handle, "stop").await {
                log::error!("Failed to stop OpenList Core: {e}");
            } else {
                update_tray_menu(app_handle, false).ok();
            }
        }
        "restart_service" => {
            log::info!("Restart service menu item clicked");
            if let Err(e) = handle_core_action(app_handle, "restart").await {
                log::error!("Failed to restart OpenList Core: {e}");
            } else {
                update_tray_menu(app_handle, true).ok();
            }
        }
        _ => {
            log::debug!("Unknown menu item clicked: {:?}", event.id());
        }
    }
}

pub fn update_tray_menu(app_handle: &AppHandle, service_running: bool) -> tauri::Result<()> {
    if let Some(tray) = app_handle.tray_by_id("main-tray") {
        let start_service_i = MenuItem::with_id(
            app_handle,
            "start_service",
            "启动OpenList",
            !service_running,
            None::<&str>,
        )?;
        let stop_service_i = MenuItem::with_id(
            app_handle,
            "stop_service",
            "停止OpenList",
            service_running,
            None::<&str>,
        )?;
        let restart_service_i = MenuItem::with_id(
            app_handle,
            "restart_service",
            "重启OpenList",
            service_running,
            None::<&str>,
        )?;

        let service_submenu = Submenu::with_id_and_items(
            app_handle,
            "service",
            "核心控制",
            true,
            &[&start_service_i, &stop_service_i, &restart_service_i],
        )?;

        let quit_i = MenuItem::with_id(app_handle, "quit", "退出", true, None::<&str>)?;
        let show_i = MenuItem::with_id(app_handle, "show", "显示窗口", true, None::<&str>)?;
        let hide_i = MenuItem::with_id(app_handle, "hide", "隐藏窗口", true, None::<&str>)?;
        let restart_i = MenuItem::with_id(app_handle, "restart", "重启应用", true, None::<&str>)?;

        let menu = Menu::with_items(
            app_handle,
            &[
                &show_i,
                &hide_i,
                &PredefinedMenuItem::separator(app_handle)?,
                &service_submenu,
                &PredefinedMenuItem::separator(app_handle)?,
                &restart_i,
                &quit_i,
            ],
        )?;

        tray.set_menu(Some(menu))?;
        log::debug!("Tray menu updated with service_running: {service_running}");
    }
    Ok(())
}

async fn handle_core_action(app_handle: &AppHandle, action: &str) -> Result<(), String> {
    let state = app_handle.state::<AppState>();
    match action {
        "start" => {
            cmd::openlist_core::start_openlist_core(state.clone()).await?;
        }
        "stop" => {
            cmd::openlist_core::stop_openlist_core(state.clone()).await?;
        }
        "restart" => {
            cmd::openlist_core::restart_openlist_core(state.clone()).await?;
        }
        _ => {
            log::warn!("Unknown core action requested from tray menu: {action}");
        }
    }
    Ok(())
}
