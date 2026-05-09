use crate::error::AppError;
use serde::Serialize;
use tauri::Emitter;

pub(crate) const FLOATING_CONTROL_WINDOW_LABEL: &str = "floating-control";
pub(crate) const APP_CONFIG_CHANGED_EVENT: &str = "app-config:changed";
pub(crate) const RUNTIME_STATE_CHANGED_EVENT: &str = "runtime-state:changed";
pub(crate) const RUNTIME_ERROR_EVENT: &str = "runtime-error";
pub(crate) const APP_NAME: &str = "DNF按键助手";

pub(crate) fn emit_app_config_changed<T>(app: &tauri::AppHandle, payload: &T)
where
    T: Serialize,
{
    if let Err(error) = app.emit(APP_CONFIG_CHANGED_EVENT, payload) {
        tracing::warn!(error = %error, "发送配置变更事件失败");
    }
}

pub(crate) fn emit_runtime_state_changed<T>(app: &tauri::AppHandle, payload: &T)
where
    T: Serialize,
{
    if let Err(error) = app.emit(RUNTIME_STATE_CHANGED_EVENT, payload) {
        tracing::warn!(error = %error, "发送运行态变更事件失败");
    }
}

pub(crate) fn emit_runtime_error(app: &tauri::AppHandle, error_payload: &AppError) {
    if let Err(error) = app.emit(RUNTIME_ERROR_EVENT, error_payload) {
        tracing::warn!(error = %error, "发送运行态错误事件失败");
    }
}
