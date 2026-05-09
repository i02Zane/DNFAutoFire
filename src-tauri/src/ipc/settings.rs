use crate::app::state::AppState;
use crate::config::SettingsConfig;
use crate::error::AppResult;
use crate::runtime::AppStateSnapshot;
use tauri::State;

#[tauri::command]
pub(crate) fn save_settings(
    settings: SettingsConfig,
    app: tauri::AppHandle,
    state: State<AppState>,
) -> AppResult<AppStateSnapshot> {
    tracing::debug!("请求保存 settings.json");
    state.runtime_supervisor.save_settings(&app, settings)
}
