use crate::app::state::AppState;
use crate::config::WindowPosition;
use crate::error::AppResult;
use crate::runtime::AppStateSnapshot;
use tauri::State;

#[tauri::command]
pub(crate) fn set_assistant_running(
    running: bool,
    app: tauri::AppHandle,
    state: State<AppState>,
) -> AppResult<bool> {
    state
        .runtime_supervisor
        .set_assistant_running(&app, running)
}

#[tauri::command]
pub(crate) fn select_active_config(
    active_class_id: Option<String>,
    app: tauri::AppHandle,
    state: State<AppState>,
) -> AppResult<AppStateSnapshot> {
    state
        .runtime_supervisor
        .select_active_config(&app, active_class_id)
}

#[tauri::command]
pub(crate) fn set_floating_control_visible(
    visible: bool,
    app: tauri::AppHandle,
    state: State<AppState>,
) -> AppResult<AppStateSnapshot> {
    state
        .runtime_supervisor
        .set_floating_control_visible(&app, visible)
}

#[tauri::command]
pub(crate) fn update_floating_control_position(
    x: i32,
    y: i32,
    app: tauri::AppHandle,
    state: State<AppState>,
) -> AppResult<AppStateSnapshot> {
    state
        .runtime_supervisor
        .update_floating_control_position(&app, WindowPosition { x, y })
}

#[tauri::command]
pub(crate) fn minimize_main_window(app: tauri::AppHandle, state: State<AppState>) -> AppResult<()> {
    state.runtime_supervisor.minimize_main_window(&app)
}

#[tauri::command]
pub(crate) fn close_main_window(app: tauri::AppHandle, state: State<AppState>) -> AppResult<()> {
    state.runtime_supervisor.close_main_window(&app)
}
