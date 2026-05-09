use crate::app::state::AppState;
use crate::config::{
    validate_combo_defs_for_profile, AutoRunConfig, ComboDefinition, ComboValidationIssue,
    EffectRule, KeyBinding,
};
use crate::error::AppResult;
use crate::runtime::AppStateSnapshot;
use serde::Deserialize;
use tauri::State;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AutoRunPatch {
    enabled: Option<bool>,
    left_vk: Option<u16>,
    right_vk: Option<u16>,
    pulse_delay_ms: Option<u64>,
}

#[tauri::command]
pub(crate) fn update_global_keys(
    keys: Vec<KeyBinding>,
    app: tauri::AppHandle,
    state: State<AppState>,
) -> AppResult<AppStateSnapshot> {
    tracing::debug!(key_count = keys.len(), "请求更新全局连发键");
    state.runtime_supervisor.update_global_keys(&app, keys)
}

#[tauri::command]
pub(crate) fn update_profile_keys(
    config_id: String,
    keys: Vec<KeyBinding>,
    app: tauri::AppHandle,
    state: State<AppState>,
) -> AppResult<AppStateSnapshot> {
    tracing::debug!(config_id, key_count = keys.len(), "请求更新配置连发键");
    state
        .runtime_supervisor
        .update_profile_keys(&app, config_id, keys)
}

#[tauri::command]
pub(crate) fn update_profile_effect_rule(
    config_id: String,
    effect_rule: EffectRule,
    app: tauri::AppHandle,
    state: State<AppState>,
) -> AppResult<AppStateSnapshot> {
    tracing::debug!(config_id, ?effect_rule, "请求更新配置生效规则");
    state
        .runtime_supervisor
        .update_profile_effect_rule(&app, config_id, effect_rule)
}

#[tauri::command]
pub(crate) fn update_profile_combos(
    config_id: String,
    combos: Vec<ComboDefinition>,
    app: tauri::AppHandle,
    state: State<AppState>,
) -> AppResult<AppStateSnapshot> {
    tracing::debug!(config_id, combo_count = combos.len(), "请求更新配置连招");
    state
        .runtime_supervisor
        .update_profile_combos(&app, config_id, combos)
}

#[tauri::command]
pub(crate) fn update_auto_run(
    patch: AutoRunPatch,
    app: tauri::AppHandle,
    state: State<AppState>,
) -> AppResult<AppStateSnapshot> {
    let mut auto_run: AutoRunConfig = state.config_store.profiles().auto_run;
    if let Some(enabled) = patch.enabled {
        auto_run.enabled = enabled;
    }
    if let Some(left_vk) = patch.left_vk {
        auto_run.left_vk = left_vk;
    }
    if let Some(right_vk) = patch.right_vk {
        auto_run.right_vk = right_vk;
    }
    if let Some(pulse_delay_ms) = patch.pulse_delay_ms {
        auto_run.pulse_delay_ms = pulse_delay_ms;
    }
    tracing::debug!(enabled = auto_run.enabled, "请求更新一键奔跑配置");
    state.runtime_supervisor.update_auto_run(&app, auto_run)
}

#[tauri::command]
pub(crate) fn add_custom_config(
    name: String,
    app: tauri::AppHandle,
    state: State<AppState>,
) -> AppResult<AppStateSnapshot> {
    tracing::debug!(name, "请求新增自定义配置");
    state.runtime_supervisor.add_custom_config(&app, name)
}

#[tauri::command]
pub(crate) fn delete_custom_config(
    config_id: String,
    app: tauri::AppHandle,
    state: State<AppState>,
) -> AppResult<AppStateSnapshot> {
    tracing::debug!(config_id, "请求删除自定义配置");
    state
        .runtime_supervisor
        .delete_custom_config(&app, config_id)
}

#[tauri::command]
pub(crate) fn set_class_hidden(
    class_id: String,
    hidden: bool,
    app: tauri::AppHandle,
    state: State<AppState>,
) -> AppResult<AppStateSnapshot> {
    tracing::debug!(class_id, hidden, "请求更新职业显示状态");
    state
        .runtime_supervisor
        .set_class_hidden(&app, class_id, hidden)
}

#[tauri::command]
pub(crate) fn validate_combo_defs(
    config_id: String,
    combos: Vec<ComboDefinition>,
    state: State<AppState>,
) -> Vec<ComboValidationIssue> {
    validate_combo_defs_for_profile(&state.config_store.profiles(), &config_id, &combos)
}
