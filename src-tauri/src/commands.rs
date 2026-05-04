//! Tauri 命令入口：前端所有后端能力都从这里进入，再分发到状态、配置和核心模块。

use crate::assistant::{AssistantProfile, EMPTY_ASSISTANT_PROFILE_ERROR};
use crate::config::{
    validate_detection_interval_ms, validate_keys, validate_runtime_profile, AppConfig,
    ComboDefinition, Hotkey, KeyBinding, LogLevelSetting,
};
use crate::core::FireKeyConfig;
use crate::hotkey::{register_windows_hotkey, validate_hotkey};
use crate::logging::{format_hotkey, update_log_level};
use crate::notify::show_error_message_box;
use crate::startup::set_windows_launch_at_startup;
use crate::state::AppState;
use crate::tray::update_tray_current_config_item;
use tauri::Emitter;
use tauri::State;

const EMPTY_AUTOFIRE_KEYS_ERROR: &str = "请至少配置一个连发按键";

#[tauri::command]
pub(crate) fn load_app_config(state: State<AppState>) -> AppConfig {
    state.config_store.current()
}

#[tauri::command]
pub(crate) fn save_app_config(
    config: AppConfig,
    state: State<AppState>,
) -> Result<AppConfig, String> {
    tracing::debug!(
        global_key_count = config.global_keys.len(),
        class_count = config.classes.len(),
        custom_config_count = config.custom_configs.len(),
        "请求保存配置"
    );
    // 保存和内存缓存统一交给 AppConfigStore，避免命令各自维护配置副本。
    state.config_store.save(config)
}

#[tauri::command]
pub(crate) fn set_log_level(log_level: LogLevelSetting) -> Result<bool, String> {
    tracing::info!(log_level = %log_level, "鍒囨崲鏃ュ織绾у埆");
    update_log_level(log_level)?;
    Ok(true)
}

#[tauri::command]
pub(crate) fn update_tray_current_config(label: String, state: State<AppState>) -> bool {
    let label = label.trim();
    if label.is_empty() {
        return false;
    }

    tracing::debug!(label = %label, "同步托盘当前配置");
    *state.tray_current_config_label.lock() = label.to_string();
    update_tray_current_config_item(&state);
    true
}

#[tauri::command]
pub(crate) fn set_runtime_keys(
    keys: Vec<KeyBinding>,
    state: State<AppState>,
) -> Result<bool, String> {
    if let Err(error) = validate_keys(&keys) {
        tracing::warn!(error = %error, key_count = keys.len(), "设置连发按键失败");
        return Err(error);
    }
    state.assistant_runtime.set_runtime_keys(keys);
    tracing::debug!("已同步连发按键快照");
    Ok(true)
}

#[tauri::command]
pub(crate) fn set_runtime_profile(
    keys: Vec<KeyBinding>,
    combos: Vec<ComboDefinition>,
    state: State<AppState>,
) -> Result<bool, String> {
    if let Err(error) = validate_runtime_profile(&keys, &combos) {
        tracing::warn!(
            error = %error,
            key_count = keys.len(),
            combo_count = combos.len(),
            "设置运行时快照失败"
        );
        return Err(error);
    }
    state
        .assistant_runtime
        .set_runtime_profile(AssistantProfile::new(keys, combos))?;
    tracing::debug!("已同步运行时快照");
    Ok(true)
}

#[tauri::command]
pub(crate) fn start_autofire(
    keys: Vec<KeyBinding>,
    state: State<AppState>,
) -> Result<bool, String> {
    tracing::debug!(key_count = keys.len(), "请求启动连发引擎");
    if let Err(error) = validate_keys(&keys) {
        tracing::warn!(error = %error, key_count = keys.len(), "启动连发引擎失败");
        return Err(error);
    }
    let mut engine = state.engine.lock();
    if keys.is_empty() {
        // 空配置不能启动；若之前已运行，先确保引擎停下。
        engine.stop();
        tracing::warn!("启动连发引擎失败：未配置任何按键");
        return Err(EMPTY_AUTOFIRE_KEYS_ERROR.to_string());
    }

    engine.set_key_configs(
        keys.into_iter()
            .map(|key| FireKeyConfig {
                vk: key.vk,
                interval_ms: key.interval_ms,
            })
            .collect(),
    );
    engine.start()?;
    Ok(true)
}

#[tauri::command]
pub(crate) fn start_assistant(
    keys: Vec<KeyBinding>,
    combos: Vec<ComboDefinition>,
    state: State<AppState>,
) -> Result<bool, String> {
    tracing::debug!(
        key_count = keys.len(),
        combo_count = combos.len(),
        "请求启动助手"
    );
    if let Err(error) = validate_runtime_profile(&keys, &combos) {
        tracing::warn!(
            error = %error,
            key_count = keys.len(),
            combo_count = combos.len(),
            "启动助手失败"
        );
        return Err(error);
    }
    if keys.is_empty() && combos.is_empty() {
        state.assistant_runtime.stop();
        tracing::warn!("启动助手失败：运行时快照为空");
        return Err(EMPTY_ASSISTANT_PROFILE_ERROR.to_string());
    }

    state
        .assistant_runtime
        .start_with_profile(AssistantProfile::new(keys, combos))?;
    Ok(true)
}

#[tauri::command]
pub(crate) fn stop_autofire(state: State<AppState>) -> bool {
    tracing::info!("请求停止连发引擎");
    state.engine.lock().stop();
    true
}

#[tauri::command]
pub(crate) fn start_detection(
    interval_ms: u64,
    app: tauri::AppHandle,
    state: State<AppState>,
) -> Result<bool, String> {
    tracing::debug!(interval_ms, "请求启动职业识别引擎");
    validate_detection_interval_ms(interval_ms)?;

    let mut runtime = state.detection_runtime.lock();
    runtime.start(app.clone(), interval_ms)?;
    drop(runtime);
    emit_detection_running_changed(&app, true);
    Ok(true)
}

#[tauri::command]
pub(crate) fn stop_detection(app: tauri::AppHandle, state: State<AppState>) -> bool {
    tracing::info!("请求停止职业识别引擎");
    state.detection_runtime.lock().stop();
    emit_detection_running_changed(&app, false);
    true
}

#[tauri::command]
pub(crate) fn is_detection_running(state: State<AppState>) -> bool {
    state.detection_runtime.lock().is_running()
}

#[tauri::command]
pub(crate) fn is_running(state: State<AppState>) -> bool {
    state.engine.lock().is_running()
}

#[tauri::command]
pub(crate) fn stop_assistant(state: State<AppState>) -> bool {
    tracing::info!("请求停止助手");
    state.assistant_runtime.stop();
    true
}

#[tauri::command]
pub(crate) fn is_assistant_running(state: State<AppState>) -> bool {
    state.assistant_runtime.is_running()
}

#[tauri::command]
pub(crate) fn register_toggle_hotkey(
    hotkey: Option<Hotkey>,
    state: State<AppState>,
) -> Result<bool, String> {
    if let Some(ref hotkey) = hotkey {
        if let Err(error) = validate_hotkey(hotkey) {
            tracing::warn!(error = %error, "注册启动/停止快捷键失败");
            return Err(error);
        }
    }

    match &hotkey {
        Some(hotkey) => tracing::info!(
            hotkey = %format_hotkey(hotkey.ctrl, hotkey.alt, hotkey.shift, hotkey.vk),
            "更新全局启动/停止快捷键"
        ),
        None => tracing::info!("清除全局启动/停止快捷键"),
    }

    // 先 drop 旧注册，确保 Windows 端不会同时存在两组相同 id 的热键。
    *state.hotkey_registration.lock() = None;

    if let Some(hotkey) = hotkey {
        #[cfg(windows)]
        {
            let registration = register_windows_hotkey(hotkey, state.assistant_runtime.clone())?;
            *state.hotkey_registration.lock() = Some(registration);
        }

        #[cfg(not(windows))]
        {
            let _ = hotkey;
        }
    }

    Ok(true)
}

#[tauri::command]
pub(crate) fn is_elevated() -> bool {
    #[cfg(windows)]
    {
        use windows::Win32::Foundation::{CloseHandle, HANDLE};
        use windows::Win32::Security::{
            GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
        };
        use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

        unsafe {
            let mut token_handle = HANDLE::default();
            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token_handle).is_err() {
                return false;
            }

            let mut elevation = TOKEN_ELEVATION::default();
            let mut size = std::mem::size_of::<TOKEN_ELEVATION>() as u32;
            let result = GetTokenInformation(
                token_handle,
                TokenElevation,
                Some(&mut elevation as *mut _ as *mut _),
                size,
                &mut size,
            );
            let _ = CloseHandle(token_handle);
            result.is_ok() && elevation.TokenIsElevated != 0
        }
    }

    #[cfg(not(windows))]
    {
        false
    }
}

#[tauri::command]
pub(crate) fn show_error_message(message: String) -> bool {
    show_error_message_box(&message)
}

#[tauri::command]
pub(crate) fn restart_as_admin() -> bool {
    tracing::info!("请求以管理员权限重启");
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use windows::core::PCWSTR;
        use windows::Win32::UI::Shell::ShellExecuteW;
        use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

        let exe_path = std::env::current_exe().unwrap_or_default();
        let exe_str: Vec<u16> = exe_path.as_os_str().encode_wide().chain(Some(0)).collect();
        let verb: Vec<u16> = "runas\0".encode_utf16().collect();

        // ShellExecuteW 的 runas 会弹出 UAC，由用户确认后再退出当前进程。
        unsafe {
            let result = ShellExecuteW(
                None,
                PCWSTR(verb.as_ptr()),
                PCWSTR(exe_str.as_ptr()),
                PCWSTR::null(),
                PCWSTR::null(),
                SW_SHOWNORMAL,
            );

            if result.0 as usize > 32 {
                tracing::info!("已发起管理员重启");
                std::process::exit(0);
            }
            tracing::warn!("管理员重启失败");
            false
        }
    }

    #[cfg(not(windows))]
    {
        false
    }
}

#[tauri::command]
pub(crate) fn set_launch_at_startup(enabled: bool) -> Result<bool, String> {
    tracing::info!(enabled, "请求更新开机启动设置");
    #[cfg(windows)]
    {
        set_windows_launch_at_startup(enabled)?;
    }

    #[cfg(not(windows))]
    {
        let _ = enabled;
        return Err("开机自启动当前仅支持 Windows。".to_string());
    }

    Ok(true)
}

fn emit_detection_running_changed(app: &tauri::AppHandle, running: bool) {
    if let Err(error) = app.emit(crate::DETECTION_RUNNING_CHANGED_EVENT, running) {
        tracing::warn!(error = %error, running, "发送职业识别运行状态事件失败");
    }
}
