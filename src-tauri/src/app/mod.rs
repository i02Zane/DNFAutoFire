//! Tauri 应用装配层：注册命令、托盘、共享状态，并处理主窗口启动行为。

pub mod events;
pub mod state;

use crate::platform::tray::create_tray_icon;
use events::APP_NAME;
use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let logging_state = crate::platform::logging::initialize();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    let app_handle = window.app_handle();
                    let state = app_handle.state::<AppState>();
                    if state.config_store.settings().close_button_minimizes {
                        api.prevent_close();
                        if let Err(error) =
                            state.runtime_supervisor.minimize_main_window(app_handle)
                        {
                            tracing::warn!(error = %error, "关闭按钮最小化主窗口失败");
                        }
                        return;
                    }
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
                elevated = crate::ipc::system::is_elevated(),
                log_dir = %logging_state.log_dir.display(),
                log_file = %logging_state
                    .log_file
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|| "<stderr>".to_string()),
                legacy_config_path = %crate::config::config_path().display(),
                settings_config_path = %crate::config::settings_config_path().display(),
                profiles_config_path = %crate::config::profiles_config_path().display(),
                "应用启动完成"
            );

            #[cfg(windows)]
            remove_tauri_keyboard_raw_input_registration();

            create_tray_icon(app)?;
            app.state::<AppState>()
                .runtime_supervisor
                .initialize(app.handle());
            let settings = app.state::<AppState>().config_store.settings();
            let profiles = app.state::<AppState>().config_store.profiles();
            tracing::info!(
                global_key_count = profiles.global_keys.len(),
                class_count = profiles.classes.len(),
                custom_config_count = profiles.custom_configs.len(),
                active_class_id = profiles.active_class_id.as_deref().unwrap_or("-"),
                start_minimized = settings.start_minimized,
                minimize_to_tray = settings.minimize_to_tray,
                open_floating_control_on_start = settings.open_floating_control_on_start,
                auto_run_enabled = profiles.auto_run.enabled,
                launch_at_startup = settings.launch_at_startup,
                "运行配置已加载"
            );
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_title(APP_NAME);
                // 启动显示策略完全由配置决定，托盘只负责后续显隐入口。
                if !settings.start_minimized {
                    let _ = window.show();
                    let _ = window.set_focus();
                } else if settings.minimize_to_tray {
                    let _ = window.hide();
                } else {
                    let _ = window.minimize();
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            crate::ipc::bootstrap::load_bootstrap,
            crate::ipc::diagnostics::load_runtime_diagnostics,
            crate::ipc::profiles::add_custom_config,
            crate::ipc::profiles::add_global_key,
            crate::ipc::profiles::add_profile_key,
            crate::ipc::profiles::delete_custom_config,
            crate::ipc::profiles::set_class_hidden,
            crate::ipc::profiles::update_auto_run,
            crate::ipc::profiles::update_global_keys,
            crate::ipc::profiles::update_profile_combos,
            crate::ipc::profiles::update_profile_effect_rule,
            crate::ipc::profiles::update_profile_keys,
            crate::ipc::profiles::validate_combo_defs,
            crate::ipc::runtime::close_main_window,
            crate::ipc::runtime::minimize_main_window,
            crate::ipc::runtime::select_active_config,
            crate::ipc::runtime::set_assistant_running,
            crate::ipc::runtime::set_floating_control_visible,
            crate::ipc::runtime::update_floating_control_position,
            crate::ipc::settings::save_settings,
            crate::ipc::system::is_elevated,
            crate::ipc::system::show_error_message,
            crate::ipc::system::restart_as_admin,
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
