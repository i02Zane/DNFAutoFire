use crate::app::state::AppState;
use crate::config::{ComboDefinition, KeyBinding, ProfilesConfig, SettingsConfig};
use crate::domain::classes::{class_categories, ClassCategory};
use crate::domain::{profile_display_snapshot, ProfileDisplaySnapshot};
use serde::Serialize;
use tauri::State;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub(crate) struct BootstrapState {
    #[ts(type = "number")]
    revision: u64,
    settings: SettingsConfig,
    profiles: ProfilesConfig,
    class_categories: Vec<ClassCategory>,
    profile_display: ProfileDisplaySnapshot,
    runtime: BootstrapRuntimeState,
    effective_profile: BootstrapEffectiveProfile,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub(crate) struct BootstrapRuntimeState {
    #[ts(type = "number")]
    revision: u64,
    assistant_running: bool,
    detection_running: bool,
    floating_control_visible: bool,
    active_toggle_keys: Vec<u16>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub(crate) struct BootstrapEffectiveProfile {
    keys: Vec<KeyBinding>,
    combos: Vec<ComboDefinition>,
}

#[tauri::command]
pub(crate) fn load_bootstrap(state: State<AppState>) -> BootstrapState {
    let settings = state.config_store.settings();
    let profiles = state.config_store.profiles();
    let profile_display = profile_display_snapshot(&profiles);
    let runtime = state.runtime_supervisor.snapshot();
    BootstrapState {
        revision: runtime.revision,
        settings,
        profiles,
        class_categories: class_categories(),
        profile_display,
        runtime: BootstrapRuntimeState {
            revision: runtime.revision,
            assistant_running: runtime.assistant_running,
            detection_running: runtime.detection_running,
            floating_control_visible: runtime.floating_control_visible,
            active_toggle_keys: runtime.active_toggle_keys,
        },
        effective_profile: BootstrapEffectiveProfile {
            keys: runtime.effective_profile.keys,
            combos: runtime.effective_profile.combos,
        },
    }
}
