#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;

use anyhow::Context;
use tauri::{
    menu::{MenuBuilder, MenuEvent},
    AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};

use crate::desktop_runtime::{DesktopRuntimeSupervisor, PortalDesktopRuntimeSnapshot};
use crate::desktop_runtime_config::DesktopRuntimeAccessMode;

mod api_key_setup;
mod desktop_runtime;
mod desktop_runtime_config;
mod desktop_shell;

const SERVICE_START_HIDDEN_ENV: &str = "SDKWORK_ROUTER_PORTAL_START_HIDDEN";
const SERVICE_MODE_ENV: &str = "SDKWORK_ROUTER_SERVICE_MODE";

// The portal desktop shell supervises the bundled router-product-service sidecar.
struct RuntimeState {
    supervisor: Mutex<DesktopRuntimeSupervisor>,
}

struct TrayState {
    _tray: tauri::tray::TrayIcon,
}

#[derive(Clone, Copy, Debug, Default)]
struct DesktopLaunchConfig {
    start_hidden: bool,
}

impl DesktopLaunchConfig {
    fn from_environment() -> Self {
        let start_hidden = env_flag_is_truthy(SERVICE_START_HIDDEN_ENV)
            || env_flag_is_truthy(SERVICE_MODE_ENV)
            || std::env::args()
                .skip(1)
                .any(|arg| matches!(arg.as_str(), "--service" | "--start-hidden"));

        Self { start_hidden }
    }
}

#[tauri::command]
fn runtime_base_url(state: tauri::State<'_, RuntimeState>) -> Result<String, String> {
    current_runtime_snapshot(&state)?
        .public_base_url
        .ok_or_else(|| "Desktop runtime did not expose a public base URL.".to_string())
}

#[tauri::command]
fn runtime_desktop_snapshot(
    state: tauri::State<'_, RuntimeState>,
) -> Result<PortalDesktopRuntimeSnapshot, String> {
    current_runtime_snapshot(&state)
}

#[tauri::command]
fn restart_product_runtime(
    app: AppHandle,
    state: tauri::State<'_, RuntimeState>,
) -> Result<PortalDesktopRuntimeSnapshot, String> {
    let snapshot = {
        let mut supervisor = state
            .supervisor
            .lock()
            .map_err(|_| "Desktop runtime state is unavailable.".to_string())?;
        supervisor
            .restart()
            .map_err(|error| format!("desktop runtime restart failed: {error}"))?
    };

    refresh_runtime_windows(&app, &snapshot)
        .map_err(|error| format!("desktop runtime window refresh failed: {error}"))?;

    Ok(snapshot)
}

#[allow(non_snake_case)]
#[tauri::command]
fn update_desktop_runtime_access_mode(
    app: AppHandle,
    accessMode: DesktopRuntimeAccessMode,
    state: tauri::State<'_, RuntimeState>,
) -> Result<PortalDesktopRuntimeSnapshot, String> {
    let snapshot = {
        let mut supervisor = state
            .supervisor
            .lock()
            .map_err(|_| "Desktop runtime state is unavailable.".to_string())?;
        supervisor
            .update_access_mode(accessMode)
            .map_err(|error| format!("desktop runtime access mode update failed: {error}"))?
    };

    refresh_runtime_windows(&app, &snapshot)
        .map_err(|error| format!("desktop runtime window refresh failed: {error}"))?;

    Ok(snapshot)
}

fn main() {
    let launch_config = DesktopLaunchConfig::from_environment();

    tauri::Builder::default()
        .setup(move |app| {
            let runtime = DesktopRuntimeSupervisor::bootstrap(&app.handle()).map_err(box_setup_error)?;
            app.manage(RuntimeState {
                supervisor: Mutex::new(runtime),
            });

            install_desktop_shell(app).map_err(box_setup_error)?;

            let app_handle = app.handle().clone();
            if launch_config.start_hidden {
                #[cfg(target_os = "macos")]
                let _ = app_handle.set_dock_visibility(false);
                hide_main_window(&app_handle).map_err(box_setup_error)?;
            } else {
                #[cfg(target_os = "macos")]
                let _ = app_handle.set_dock_visibility(true);
                show_main_window(&app_handle).map_err(box_setup_error)?;
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            runtime_base_url,
            runtime_desktop_snapshot,
            restart_product_runtime,
            update_desktop_runtime_access_mode,
            api_key_setup::install_api_router_client_setup,
            api_key_setup::list_api_key_instances
        ])
        .run(tauri::generate_context!())
        .expect("error while running sdkwork-router-portal tauri application");
}

fn install_desktop_shell(app: &mut tauri::App) -> anyhow::Result<()> {
    let menu = build_tray_menu(app)?;
    let app_handle = app.handle().clone();

    if let Some(window) = app_handle.get_webview_window(desktop_shell::PORTAL_WINDOW_LABEL) {
        let app_handle = app_handle.clone();
        window.on_window_event(move |event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = hide_window_by_label(&app_handle, desktop_shell::PORTAL_WINDOW_LABEL);
            }
        });
    }

    let tray = TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(false)
        .icon(
            app.default_window_icon()
                .cloned()
                .context("desktop tray icon is unavailable")?,
        )
        .on_tray_icon_event({
            let app_handle = app_handle.clone();
            move |_, event| {
                if let TrayIconEvent::Click {
                    button,
                    button_state,
                    ..
                } = event
                {
                    if button == MouseButton::Left && button_state == MouseButtonState::Up {
                        let _ = toggle_main_window_visibility(&app_handle);
                    }
                }
            }
        })
        .on_menu_event({
            let app_handle = app_handle.clone();
            move |_, event| {
                handle_tray_menu_event(&app_handle, event);
            }
        })
        .build(app)
        .context("failed to build portal tray icon")?;

    app.manage(TrayState { _tray: tray });
    Ok(())
}

fn build_tray_menu(app: &tauri::App) -> anyhow::Result<tauri::menu::Menu<tauri::Wry>> {
    let menu = MenuBuilder::new(app)
        .text(desktop_shell::TRAY_ACTION_SHOW_WINDOW, "Show window")
        .text(desktop_shell::TRAY_ACTION_HIDE_WINDOW, "Hide window")
        .separator()
        .text(desktop_shell::TRAY_ACTION_OPEN_PORTAL, "Open portal")
        .text(desktop_shell::TRAY_ACTION_OPEN_ADMIN, "Open admin")
        .text(desktop_shell::TRAY_ACTION_OPEN_GATEWAY, "Open gateway")
        .separator()
        .text(desktop_shell::TRAY_ACTION_RESTART_RUNTIME, "Restart runtime")
        .separator()
        .text(desktop_shell::TRAY_ACTION_QUIT_APP, "Quit")
        .build()
        .context("failed to build portal tray menu")?;

    Ok(menu)
}

fn handle_tray_menu_event(app_handle: &AppHandle, event: MenuEvent) {
    match event.id().as_ref() {
        desktop_shell::TRAY_ACTION_SHOW_WINDOW => {
            let _ = show_main_window(app_handle);
        }
        desktop_shell::TRAY_ACTION_HIDE_WINDOW => {
            let _ = hide_main_window(app_handle);
        }
        desktop_shell::TRAY_ACTION_OPEN_PORTAL => {
            let _ = show_main_window(app_handle);
        }
        desktop_shell::TRAY_ACTION_OPEN_ADMIN => {
            let _ = open_runtime_window(
                app_handle,
                desktop_shell::ADMIN_WINDOW_LABEL,
                "SDKWork Router Admin",
                desktop_shell::admin_url,
                1440.0,
                960.0,
            );
        }
        desktop_shell::TRAY_ACTION_OPEN_GATEWAY => {
            let _ = open_runtime_window(
                app_handle,
                desktop_shell::GATEWAY_WINDOW_LABEL,
                "SDKWork Router Gateway",
                desktop_shell::gateway_url,
                1440.0,
                960.0,
            );
        }
        desktop_shell::TRAY_ACTION_RESTART_RUNTIME => {
            let _ = restart_runtime_from_shell(app_handle);
        }
        desktop_shell::TRAY_ACTION_QUIT_APP => {
            app_handle.exit(0);
        }
        _ => {}
    }
}

fn show_main_window(app_handle: &AppHandle) -> anyhow::Result<()> {
    let window = app_handle
        .get_webview_window(desktop_shell::PORTAL_WINDOW_LABEL)
        .context("main portal window is unavailable")?;
    show_window(&window)
}

fn hide_main_window(app_handle: &AppHandle) -> anyhow::Result<()> {
    let window = app_handle
        .get_webview_window(desktop_shell::PORTAL_WINDOW_LABEL)
        .context("main portal window is unavailable")?;
    hide_window(&window)
}

fn toggle_main_window_visibility(app_handle: &AppHandle) -> anyhow::Result<()> {
    let window = app_handle
        .get_webview_window(desktop_shell::PORTAL_WINDOW_LABEL)
        .context("main portal window is unavailable")?;
    if window.is_visible().unwrap_or(false) {
        hide_window(&window)
    } else {
        show_window(&window)
    }
}

fn open_runtime_window(
    app_handle: &AppHandle,
    window_label: &'static str,
    title: &str,
    build_path: fn(&str) -> Option<String>,
    width: f64,
    height: f64,
) -> anyhow::Result<()> {
    let base_url = runtime_public_base_url(app_handle)
        .context("desktop runtime did not expose a public base URL")?;
    let url = build_path(&base_url).context("failed to build runtime URL")?;
    let target_url = url.parse().context("failed to parse runtime URL")?;

    if let Some(window) = app_handle.get_webview_window(window_label) {
        window.navigate(target_url)?;
        return show_window(&window);
    }

    let window =
        WebviewWindowBuilder::new(app_handle, window_label, WebviewUrl::External(target_url))
            .title(title)
            .center()
            .inner_size(width, height)
            .resizable(true)
            .decorations(false)
            .skip_taskbar(false)
            .visible(false)
            .build()
            .context("failed to build runtime window")?;

    install_window_close_hides(&window, window_label);
    show_window(&window)?;
    Ok(())
}

fn install_window_close_hides(window: &WebviewWindow, window_label: &'static str) {
    let app_handle = window.app_handle().clone();
    let window_label = window_label.to_owned();

    window.on_window_event(move |event| {
        if let WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            let _ = hide_window_by_label(&app_handle, &window_label);
        }
    });
}

fn hide_window_by_label(app_handle: &AppHandle, window_label: &str) -> anyhow::Result<()> {
    let window = app_handle
        .get_webview_window(window_label)
        .with_context(|| format!("{window_label} window is unavailable"))?;
    hide_window(&window)
}

fn show_window(window: &WebviewWindow) -> anyhow::Result<()> {
    let _ = window.set_skip_taskbar(false);
    window.show()?;
    window.unminimize()?;
    window.set_focus()?;
    Ok(())
}

fn hide_window(window: &WebviewWindow) -> anyhow::Result<()> {
    let _ = window.set_skip_taskbar(true);
    window.hide()?;
    Ok(())
}

fn restart_runtime_from_shell(app_handle: &AppHandle) -> anyhow::Result<()> {
    let runtime_state = app_handle.state::<RuntimeState>();
    let snapshot = {
        let mut supervisor = runtime_state
            .supervisor
            .lock()
            .map_err(|_| anyhow::anyhow!("Desktop runtime state is unavailable."))?;
        supervisor.restart()?
    };

    refresh_runtime_windows(app_handle, &snapshot)?;
    Ok(())
}

fn refresh_runtime_windows(
    app_handle: &AppHandle,
    snapshot: &PortalDesktopRuntimeSnapshot,
) -> anyhow::Result<()> {
    let base_url = snapshot
        .public_base_url
        .as_deref()
        .context("desktop runtime did not expose a public base URL")?;

    refresh_window(
        app_handle,
        desktop_shell::PORTAL_WINDOW_LABEL,
        desktop_shell::portal_url(base_url),
    )?;
    refresh_window(
        app_handle,
        desktop_shell::ADMIN_WINDOW_LABEL,
        desktop_shell::admin_url(base_url),
    )?;
    refresh_window(
        app_handle,
        desktop_shell::GATEWAY_WINDOW_LABEL,
        desktop_shell::gateway_url(base_url),
    )?;

    Ok(())
}

fn refresh_window(
    app_handle: &AppHandle,
    window_label: &'static str,
    target_url: Option<String>,
) -> anyhow::Result<()> {
    let Some(target_url) = target_url else {
        return Ok(());
    };
    let Some(window) = app_handle.get_webview_window(window_label) else {
        return Ok(());
    };

    let target_url = target_url.parse().context("failed to parse runtime URL")?;
    window.navigate(target_url)?;
    Ok(())
}

fn box_setup_error(error: anyhow::Error) -> Box<dyn std::error::Error> {
    Box::new(std::io::Error::new(
        std::io::ErrorKind::Other,
        error.to_string(),
    ))
}

fn current_runtime_snapshot(
    state: &tauri::State<'_, RuntimeState>,
) -> Result<PortalDesktopRuntimeSnapshot, String> {
    state
        .supervisor
        .lock()
        .map_err(|_| "Desktop runtime state is unavailable.".to_string())
        .map(|supervisor| supervisor.snapshot())
}

fn runtime_public_base_url(app_handle: &AppHandle) -> Option<String> {
    let state = app_handle.state::<RuntimeState>();
    current_runtime_snapshot(&state)
        .ok()
        .and_then(|snapshot| snapshot.public_base_url)
}

fn env_flag_is_truthy(name: &str) -> bool {
    matches!(
        std::env::var(name)
            .ok()
            .as_deref()
            .map(str::trim)
            .unwrap_or_default(),
        "1" | "true" | "TRUE" | "yes" | "YES" | "on" | "ON"
    )
}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;

    use super::box_setup_error;

    #[test]
    fn box_setup_error_preserves_context_message() {
        let error = box_setup_error(anyhow!("desktop runtime boot failed"));
        assert_eq!(error.to_string(), "desktop runtime boot failed");
    }
}
