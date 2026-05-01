//! Tauri 后端入口：注册命令、托盘、共享状态，并处理主窗口启动行为。

mod assistant;
mod commands;
mod config;
mod core;
mod hotkey;
mod logging;
mod notify;
mod startup;
mod state;
mod tray;

use commands::{
    is_assistant_running, is_elevated, is_running, load_app_config, register_toggle_hotkey,
    restart_as_admin, save_app_config, set_launch_at_startup, set_log_level, set_runtime_keys,
    set_runtime_profile, show_error_message, start_assistant, start_autofire, stop_assistant,
    stop_autofire, update_tray_current_config,
};
use state::AppState;
use tauri::Manager;
use tray::create_tray_icon;

pub(crate) const FLOATING_CONTROL_TOGGLE_REQUEST_EVENT: &str = "floating-control:toggle-request";
pub(crate) const FLOATING_CONTROL_VISIBILITY_EVENT: &str = "floating-control:visibility-changed";
pub(crate) const FLOATING_CONTROL_WINDOW_LABEL: &str = "floating-control";
pub(crate) const APP_NAME: &str = "DNF按键助手";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let logging_state = logging::initialize();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if let tauri::WindowEvent::CloseRequested { .. } = event {
                    window.app_handle().exit(0);
                }
            }
        })
        .manage(AppState::new())
        .setup(move |app| {
            tracing::info!(
                version = env!("CARGO_PKG_VERSION"),
                build_mode = logging_state.build_mode,
                log_level = %logging_state.log_level,
                process_id = std::process::id(),
                elevated = is_elevated(),
                log_dir = %logging_state.log_dir.display(),
                log_file = %logging_state
                    .log_file
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|| "<stderr>".to_string()),
                config_path = %crate::config::config_path().display(),
                "应用启动完成"
            );

            #[cfg(windows)]
            remove_tauri_keyboard_raw_input_registration();

            create_tray_icon(app)?;
            let config = app.state::<AppState>().config_store.current();
            tracing::info!(
                global_key_count = config.global_keys.len(),
                class_count = config.classes.len(),
                custom_config_count = config.custom_configs.len(),
                active_class_id = config.active_class_id.as_deref().unwrap_or("-"),
                start_minimized = config.settings.start_minimized,
                minimize_to_tray = config.settings.minimize_to_tray,
                open_floating_control_on_start = config.settings.open_floating_control_on_start,
                launch_at_startup = config.settings.launch_at_startup,
                "运行配置已加载"
            );
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_title(APP_NAME);
                // 启动显示策略完全由配置决定，托盘只负责后续显隐入口。
                if !config.settings.start_minimized {
                    let _ = window.show();
                    let _ = window.set_focus();
                } else if config.settings.minimize_to_tray {
                    let _ = window.hide();
                } else {
                    let _ = window.minimize();
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_app_config,
            save_app_config,
            set_log_level,
            set_runtime_keys,
            set_runtime_profile,
            start_assistant,
            stop_assistant,
            is_assistant_running,
            start_autofire,
            stop_autofire,
            is_running,
            update_tray_current_config,
            register_toggle_hotkey,
            is_elevated,
            show_error_message,
            restart_as_admin,
            set_launch_at_startup,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(windows)]
fn remove_tauri_keyboard_raw_input_registration() {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::Input::{RegisterRawInputDevices, RAWINPUTDEVICE, RIDEV_REMOVE};

    let device = RAWINPUTDEVICE {
        usUsagePage: 0x01,
        usUsage: 0x06,
        dwFlags: RIDEV_REMOVE,
        hwndTarget: HWND::default(),
    };

    if let Err(error) =
        unsafe { RegisterRawInputDevices(&[device], std::mem::size_of::<RAWINPUTDEVICE>() as u32) }
    {
        tracing::warn!(error = %error, "移除键盘 Raw Input 失败");
    }
}
