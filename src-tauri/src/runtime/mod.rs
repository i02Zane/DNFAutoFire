//! 助手运行时：统一管理当前生效快照、启动/停止、热键切换和失败回滚。

use crate::app::events::{emit_app_config_changed, emit_runtime_error, emit_runtime_state_changed};
use crate::config::{
    AutoRunConfig, ComboDefinition, ConfigRepository, EffectRule, KeyBinding, ProfilesConfig,
    SettingsConfig, WindowPosition,
};
use crate::domain::{
    compute_effective_combos, compute_effective_keys, profile_display_snapshot,
    ProfileDisplaySnapshot,
};
use crate::engines::{AutoFireEngine, AutoRunEngine, ComboEngine};
use crate::error::{AppError, AppResult};
use crate::platform::floating_control::FloatingControlRuntime;
use crate::platform::hotkey::{register_windows_hotkey, validate_hotkey, HotkeyRegistration};
use parking_lot::Mutex;
use serde::Serialize;
use services::{to_fire_key_configs, ProfileService, SettingsService, WindowService};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tauri::AppHandle;
use ts_rs::TS;

mod services;

pub(crate) const EMPTY_ASSISTANT_PROFILE_ERROR: &str = "请至少配置一个连发按键或一键连招";

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub(crate) struct RuntimeEffectiveProfile {
    pub keys: Vec<KeyBinding>,
    pub combos: Vec<ComboDefinition>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub(crate) struct RuntimeStateSnapshot {
    #[ts(type = "number")]
    pub revision: u64,
    pub assistant_running: bool,
    pub detection_running: bool,
    pub floating_control_visible: bool,
    pub active_toggle_keys: Vec<u16>,
    pub effective_profile: RuntimeEffectiveProfile,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub(crate) struct RuntimeStatusSnapshot {
    #[ts(type = "number")]
    pub revision: u64,
    pub assistant_running: bool,
    pub detection_running: bool,
    pub floating_control_visible: bool,
    pub active_toggle_keys: Vec<u16>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub(crate) struct AppStateSnapshot {
    #[ts(type = "number")]
    pub revision: u64,
    pub settings: SettingsConfig,
    pub profiles: ProfilesConfig,
    pub profile_display: ProfileDisplaySnapshot,
    pub runtime: RuntimeStatusSnapshot,
    pub effective_profile: RuntimeEffectiveProfile,
}

#[derive(Clone)]
pub(crate) struct AssistantProfile {
    keys: Vec<KeyBinding>,
    combos: Vec<ComboDefinition>,
}

impl AssistantProfile {
    pub(crate) fn new(keys: Vec<KeyBinding>, combos: Vec<ComboDefinition>) -> Self {
        Self { keys, combos }
    }

    fn is_empty(&self) -> bool {
        self.keys.is_empty() && self.combos.is_empty()
    }
}

impl Default for AssistantProfile {
    fn default() -> Self {
        Self::new(Vec::new(), Vec::new())
    }
}

impl AssistantProfile {
    fn from_profiles(profiles: &ProfilesConfig) -> Self {
        Self::new(
            compute_effective_keys(profiles),
            compute_effective_combos(profiles),
        )
    }
}

#[derive(Clone)]
pub(crate) struct AssistantRuntime {
    engine: Arc<Mutex<AutoFireEngine>>,
    combo_engine: Arc<Mutex<ComboEngine>>,
    auto_run_runtime: Arc<Mutex<AutoRunEngine>>,
    config_store: Arc<ConfigRepository>,
    profile: Arc<Mutex<AssistantProfile>>,
}

#[derive(Clone)]
pub(crate) struct RuntimeSupervisor {
    assistant_runtime: AssistantRuntime,
    detection_runtime: Arc<Mutex<crate::vision::DetectionRuntime>>,
    config_store: Arc<ConfigRepository>,
    settings_service: SettingsService,
    profile_service: ProfileService,
    window_service: WindowService,
    hotkey_registration: Arc<Mutex<Option<HotkeyRegistration>>>,
    revision: Arc<AtomicU64>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub(crate) struct AssistantRuntimeSnapshot {
    pub running: bool,
    pub profile_key_count: usize,
    pub profile_combo_count: usize,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub(crate) struct AssistantEngineSnapshots {
    pub assistant: AssistantRuntimeSnapshot,
    pub autofire: crate::engines::autofire::AutoFireSnapshot,
    pub combo: crate::engines::combo::ComboSnapshot,
    pub auto_run: crate::engines::autorun::AutoRunSnapshot,
}

impl AssistantRuntime {
    pub(crate) fn new(
        engine: Arc<Mutex<AutoFireEngine>>,
        combo_engine: Arc<Mutex<ComboEngine>>,
        auto_run_runtime: Arc<Mutex<AutoRunEngine>>,
        config_store: Arc<ConfigRepository>,
    ) -> Self {
        Self {
            engine,
            combo_engine,
            auto_run_runtime,
            config_store,
            profile: Arc::new(Mutex::new(AssistantProfile::default())),
        }
    }

    pub(crate) fn refresh_from_current_config(&self) -> AppResult<()> {
        let profiles = self.config_store.profiles();
        let profile = AssistantProfile::from_profiles(&profiles);

        if self.is_running() && profile.is_empty() && !profiles.auto_run.enabled {
            tracing::warn!("刷新运行时快照失败：当前配置为空且一键奔跑未启用");
            self.stop();
            return Err(AppError::runtime(EMPTY_ASSISTANT_PROFILE_ERROR));
        }

        self.store_runtime_profile(profile.clone());
        if self.is_running() {
            tracing::info!(
                key_count = profile.keys.len(),
                combo_count = profile.combos.len(),
                "运行中刷新运行时快照"
            );
            self.apply_profile(profile)?;
            self.sync_auto_run()?;
        }

        Ok(())
    }

    pub(crate) fn start_with_profile(&self, profile: AssistantProfile) -> AppResult<()> {
        if profile.is_empty() && !self.config_store.profiles().auto_run.enabled {
            tracing::warn!("尝试启动助手，但当前运行时快照为空且一键奔跑未启用");
            self.stop();
            return Err(AppError::runtime(EMPTY_ASSISTANT_PROFILE_ERROR));
        }

        let profiles = self.config_store.profiles();
        tracing::info!(
            key_count = profile.keys.len(),
            combo_count = profile.combos.len(),
            auto_run = profiles.auto_run.enabled,
            "启动助手"
        );
        self.store_runtime_profile(profile.clone());
        self.apply_profile(profile)?;
        self.sync_auto_run()?;

        Ok(())
    }

    pub(crate) fn toggle_from_runtime_profile(&self) -> AppResult<()> {
        if self.is_running() {
            tracing::info!("收到全局快捷键，准备停止助手");
            self.stop();
            return Ok(());
        }

        let profile = self.current_config_profile();
        tracing::info!(
            key_count = profile.keys.len(),
            combo_count = profile.combos.len(),
            "收到全局快捷键，准备启动助手"
        );
        self.start_with_profile(profile)
    }

    pub(crate) fn stop(&self) {
        let was_running = self.is_running() || self.auto_run_runtime.lock().is_running();
        if was_running {
            tracing::info!("停止助手");
        } else {
            tracing::debug!("助手已经处于停止状态");
        }
        self.engine.lock().stop();
        self.combo_engine.lock().stop();
        self.auto_run_runtime.lock().stop();
    }

    pub(crate) fn is_running(&self) -> bool {
        self.engine.lock().is_running()
            || self.combo_engine.lock().is_running()
            || self.auto_run_runtime.lock().is_running()
    }

    pub(crate) fn snapshot(&self) -> AssistantRuntimeSnapshot {
        let profile = self.profile.lock();
        AssistantRuntimeSnapshot {
            running: self.is_running(),
            profile_key_count: profile.keys.len(),
            profile_combo_count: profile.combos.len(),
        }
    }

    pub(crate) fn active_toggle_keys(&self) -> Vec<u16> {
        self.engine.lock().active_toggle_keys()
    }

    pub(crate) fn engine_snapshots(&self) -> AssistantEngineSnapshots {
        AssistantEngineSnapshots {
            assistant: self.snapshot(),
            autofire: self.engine.lock().snapshot(),
            combo: self.combo_engine.lock().snapshot(),
            auto_run: self.auto_run_runtime.lock().snapshot(),
        }
    }

    fn store_runtime_profile(&self, profile: AssistantProfile) {
        *self.profile.lock() = profile;
    }

    fn current_config_profile(&self) -> AssistantProfile {
        let profiles = self.config_store.profiles();
        AssistantProfile::from_profiles(&profiles)
    }

    fn apply_profile(&self, profile: AssistantProfile) -> AppResult<()> {
        let AssistantProfile { keys, combos } = profile;
        tracing::debug!(
            key_count = keys.len(),
            combo_count = combos.len(),
            "应用运行时快照到引擎"
        );

        {
            let mut engine = self.engine.lock();
            if keys.is_empty() {
                engine.stop();
            } else {
                engine.set_key_configs(to_fire_key_configs(keys));
                if let Err(error) = engine.start() {
                    tracing::warn!(error = %error, "启动连发引擎失败");
                    drop(engine);
                    self.stop();
                    return Err(error);
                }
            }
        }

        {
            let mut combo_engine = self.combo_engine.lock();
            let has_combos = !combos.is_empty();
            combo_engine.set_combo_configs(combos);
            if has_combos {
                if let Err(error) = combo_engine.start() {
                    tracing::warn!(error = %error, "启动一键连招引擎失败");
                    drop(combo_engine);
                    self.stop();
                    return Err(error);
                }
            } else {
                combo_engine.stop();
            }
        }

        tracing::info!("运行时快照已生效");
        Ok(())
    }

    fn sync_auto_run(&self) -> AppResult<()> {
        let profiles = self.config_store.profiles();
        let mut auto_run = self.auto_run_runtime.lock();
        auto_run.set_settings(
            profiles.auto_run.left_vk,
            profiles.auto_run.right_vk,
            profiles.auto_run.pulse_delay_ms,
        );
        if profiles.auto_run.enabled {
            if let Err(error) = auto_run.start() {
                tracing::warn!(error = %error, "启动一键奔跑失败");
                drop(auto_run);
                self.stop();
                return Err(error);
            }
        } else {
            auto_run.stop();
        }
        Ok(())
    }
}

impl RuntimeStateSnapshot {
    pub(crate) fn from_runtime(
        assistant_runtime: &AssistantRuntime,
        profiles: &ProfilesConfig,
        detection_running: bool,
        floating_control_visible: bool,
        revision: u64,
    ) -> Self {
        Self {
            revision,
            assistant_running: assistant_runtime.is_running(),
            detection_running,
            floating_control_visible,
            active_toggle_keys: assistant_runtime.active_toggle_keys(),
            effective_profile: RuntimeEffectiveProfile {
                keys: compute_effective_keys(profiles),
                combos: compute_effective_combos(profiles),
            },
        }
    }
}

impl RuntimeStatusSnapshot {
    fn from_runtime_snapshot(snapshot: &RuntimeStateSnapshot) -> Self {
        Self {
            revision: snapshot.revision,
            assistant_running: snapshot.assistant_running,
            detection_running: snapshot.detection_running,
            floating_control_visible: snapshot.floating_control_visible,
            active_toggle_keys: snapshot.active_toggle_keys.clone(),
        }
    }
}

impl AppStateSnapshot {
    pub(crate) fn from_parts(
        settings: &SettingsConfig,
        profiles: &ProfilesConfig,
        runtime: RuntimeStateSnapshot,
    ) -> Self {
        Self {
            revision: runtime.revision,
            settings: settings.clone(),
            profiles: profiles.clone(),
            profile_display: profile_display_snapshot(profiles),
            runtime: RuntimeStatusSnapshot::from_runtime_snapshot(&runtime),
            effective_profile: runtime.effective_profile,
        }
    }
}

impl RuntimeSupervisor {
    pub(crate) fn new(
        assistant_runtime: AssistantRuntime,
        detection_runtime: Arc<Mutex<crate::vision::DetectionRuntime>>,
        floating_control_runtime: Arc<Mutex<FloatingControlRuntime>>,
        config_store: Arc<ConfigRepository>,
        hotkey_registration: Arc<Mutex<Option<HotkeyRegistration>>>,
    ) -> Self {
        let settings_service = SettingsService::new(config_store.clone());
        let profile_service = ProfileService::new(config_store.clone());
        let window_service =
            WindowService::new(config_store.clone(), floating_control_runtime.clone());
        Self {
            assistant_runtime,
            detection_runtime,
            config_store,
            settings_service,
            profile_service,
            window_service,
            hotkey_registration,
            revision: Arc::new(AtomicU64::new(1)),
        }
    }

    pub(crate) fn snapshot(&self) -> RuntimeStateSnapshot {
        self.snapshot_at(self.current_revision())
    }

    fn snapshot_at(&self, revision: u64) -> RuntimeStateSnapshot {
        let profiles = self.config_store.profiles();
        RuntimeStateSnapshot::from_runtime(
            &self.assistant_runtime,
            &profiles,
            self.detection_runtime.lock().is_running(),
            self.window_service.floating_control_visible(),
            revision,
        )
    }

    pub(crate) fn app_state_snapshot(&self) -> AppStateSnapshot {
        self.app_state_snapshot_at(self.current_revision())
    }

    fn app_state_snapshot_at(&self, revision: u64) -> AppStateSnapshot {
        let settings = self.config_store.settings();
        let profiles = self.config_store.profiles();
        AppStateSnapshot::from_parts(&settings, &profiles, self.snapshot_at(revision))
    }

    pub(crate) fn emit_snapshot(&self, app: &AppHandle) {
        let revision = self.bump_revision();
        self.emit_runtime_snapshot_at(app, revision);
    }

    pub(crate) fn emit_error(&self, app: &AppHandle, error: AppError) {
        emit_runtime_error(app, &error);
    }

    pub(crate) fn initialize(&self, app: &AppHandle) {
        let settings = self.config_store.settings();
        if let Err(error) = self.settings_service.sync_current_launch_at_startup() {
            self.emit_error(app, error);
        }
        if let Err(error) = self.sync_hotkey(app) {
            self.emit_error(app, error);
        }
        if let Err(error) = self.sync_detection(app) {
            self.emit_error(app, error);
        }
        if let Err(error) = self.window_service.set_floating_control_visible(
            app,
            settings.open_floating_control_on_start,
            settings.floating_control.position,
        ) {
            self.emit_error(app, error);
        }
        self.emit_runtime_snapshot_at(app, self.current_revision());
    }

    pub(crate) fn save_settings(
        &self,
        app: &AppHandle,
        settings: crate::config::SettingsConfig,
    ) -> AppResult<AppStateSnapshot> {
        self.settings_service.save_settings(settings)?;
        crate::platform::tray::update_tray_current_config_item(app);
        if let Err(error) = self.sync_hotkey(app) {
            self.emit_error(app, error);
        }
        if let Err(error) = self.sync_detection(app) {
            self.emit_error(app, error);
        }
        if let Err(error) = self.assistant_runtime.refresh_from_current_config() {
            self.emit_error(app, error);
        }
        Ok(self.emit_config_and_runtime_changed(app))
    }

    pub(crate) fn set_floating_control_visible(
        &self,
        app: &AppHandle,
        visible: bool,
    ) -> AppResult<AppStateSnapshot> {
        self.window_service.set_floating_control_visible(
            app,
            visible,
            self.config_store.settings().floating_control.position,
        )?;
        let revision = self.bump_revision();
        let snapshot = self.app_state_snapshot_at(revision);
        self.emit_runtime_snapshot_at(app, revision);
        Ok(snapshot)
    }

    pub(crate) fn update_floating_control_position(
        &self,
        app: &AppHandle,
        position: WindowPosition,
    ) -> AppResult<AppStateSnapshot> {
        self.settings_service
            .update_floating_control_position(position)?;
        Ok(self.emit_config_and_runtime_changed(app))
    }

    pub(crate) fn minimize_main_window(&self, app: &AppHandle) -> AppResult<()> {
        self.window_service.minimize_main_window(app)
    }

    pub(crate) fn close_main_window(&self, app: &AppHandle) -> AppResult<()> {
        self.window_service.close_main_window(app)
    }

    pub(crate) fn toggle_floating_control_visible(
        &self,
        app: &AppHandle,
    ) -> AppResult<AppStateSnapshot> {
        self.window_service.toggle_floating_control_visible(app)?;
        let revision = self.bump_revision();
        let snapshot = self.app_state_snapshot_at(revision);
        self.emit_runtime_snapshot_at(app, revision);
        Ok(snapshot)
    }

    pub(crate) fn update_global_keys(
        &self,
        app: &AppHandle,
        keys: Vec<KeyBinding>,
    ) -> AppResult<AppStateSnapshot> {
        self.profile_service.update_global_keys(keys)?;
        Ok(self.after_profiles_changed(app))
    }

    pub(crate) fn update_profile_keys(
        &self,
        app: &AppHandle,
        config_id: String,
        keys: Vec<KeyBinding>,
    ) -> AppResult<AppStateSnapshot> {
        self.profile_service.update_profile_keys(config_id, keys)?;
        Ok(self.after_profiles_changed(app))
    }

    pub(crate) fn update_profile_effect_rule(
        &self,
        app: &AppHandle,
        config_id: String,
        effect_rule: EffectRule,
    ) -> AppResult<AppStateSnapshot> {
        self.profile_service
            .update_profile_effect_rule(config_id, effect_rule)?;
        Ok(self.after_profiles_changed(app))
    }

    pub(crate) fn update_profile_combos(
        &self,
        app: &AppHandle,
        config_id: String,
        combos: Vec<ComboDefinition>,
    ) -> AppResult<AppStateSnapshot> {
        self.profile_service
            .update_profile_combos(config_id, combos)?;
        Ok(self.after_profiles_changed(app))
    }

    pub(crate) fn update_auto_run(
        &self,
        app: &AppHandle,
        auto_run: AutoRunConfig,
    ) -> AppResult<AppStateSnapshot> {
        self.profile_service.update_auto_run(auto_run)?;
        Ok(self.after_profiles_changed(app))
    }

    pub(crate) fn add_custom_config(
        &self,
        app: &AppHandle,
        name: String,
    ) -> AppResult<AppStateSnapshot> {
        self.profile_service.add_custom_config(name)?;
        Ok(self.after_profiles_changed(app))
    }

    pub(crate) fn delete_custom_config(
        &self,
        app: &AppHandle,
        config_id: String,
    ) -> AppResult<AppStateSnapshot> {
        self.profile_service.delete_custom_config(config_id)?;
        Ok(self.after_profiles_changed(app))
    }

    pub(crate) fn set_class_hidden(
        &self,
        app: &AppHandle,
        class_id: String,
        hidden: bool,
    ) -> AppResult<AppStateSnapshot> {
        self.profile_service.set_class_hidden(class_id, hidden)?;
        Ok(self.after_profiles_changed(app))
    }

    pub(crate) fn select_active_config(
        &self,
        app: &AppHandle,
        active_class_id: Option<String>,
    ) -> AppResult<AppStateSnapshot> {
        let changed = self.profile_service.select_active_config(active_class_id)?;
        if changed {
            return Ok(self.after_profiles_changed(app));
        }
        Ok(self.app_state_snapshot())
    }

    pub(crate) fn set_assistant_running(&self, app: &AppHandle, running: bool) -> AppResult<bool> {
        if running {
            let profiles = self.config_store.profiles();
            let profile = AssistantProfile::from_profiles(&profiles);
            self.assistant_runtime.start_with_profile(profile)?;
        } else {
            self.assistant_runtime.stop();
        }
        self.emit_snapshot(app);
        Ok(true)
    }

    pub(crate) fn toggle_assistant(&self, app: &AppHandle) -> AppResult<()> {
        self.assistant_runtime.toggle_from_runtime_profile()?;
        self.emit_snapshot(app);
        Ok(())
    }

    pub(crate) fn sync_hotkey(&self, app: &AppHandle) -> AppResult<()> {
        let hotkey = self.config_store.settings().toggle_hotkey;
        if let Some(ref hotkey) = hotkey {
            validate_hotkey(hotkey)?;
        }

        *self.hotkey_registration.lock() = None;

        if let Some(hotkey) = hotkey {
            #[cfg(windows)]
            {
                let registration = register_windows_hotkey(hotkey, self.clone(), app.clone())?;
                *self.hotkey_registration.lock() = Some(registration);
            }

            #[cfg(not(windows))]
            {
                let _ = (hotkey, app);
            }
        }

        Ok(())
    }

    pub(crate) fn sync_detection(&self, app: &AppHandle) -> AppResult<()> {
        let settings = self.config_store.settings();
        let mut detection_runtime = self.detection_runtime.lock();
        if settings.detection.enabled {
            detection_runtime.start(app.clone(), settings.detection.interval_ms, self.clone())?;
        } else {
            detection_runtime.stop();
        }
        Ok(())
    }

    pub(crate) fn select_active_config_from_detection(
        &self,
        app: &AppHandle,
        active_class_id: Option<String>,
    ) -> AppResult<()> {
        let changed = self.profile_service.select_active_config(active_class_id)?;
        if changed {
            self.after_profiles_changed(app);
        }
        Ok(())
    }

    fn after_profiles_changed(&self, app: &AppHandle) -> AppStateSnapshot {
        crate::platform::tray::update_tray_current_config_item(app);
        if let Err(error) = self.assistant_runtime.refresh_from_current_config() {
            self.emit_error(app, error);
        }
        self.emit_config_and_runtime_changed(app)
    }

    fn emit_config_and_runtime_changed(&self, app: &AppHandle) -> AppStateSnapshot {
        let revision = self.bump_revision();
        let snapshot = self.app_state_snapshot_at(revision);
        emit_app_config_changed(app, &snapshot);
        self.emit_runtime_snapshot_at(app, revision);
        snapshot
    }

    fn emit_runtime_snapshot_at(&self, app: &AppHandle, revision: u64) {
        emit_runtime_state_changed(app, &self.snapshot_at(revision));
    }

    fn current_revision(&self) -> u64 {
        self.revision.load(Ordering::SeqCst)
    }

    fn bump_revision(&self) -> u64 {
        self.revision.fetch_add(1, Ordering::SeqCst) + 1
    }

    #[cfg(test)]
    fn apply_profiles_changed_for_test(&self) -> AppResult<AppStateSnapshot> {
        self.assistant_runtime.refresh_from_current_config()?;
        let revision = self.bump_revision();
        Ok(self.app_state_snapshot_at(revision))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        ClassConfig, FireKeyMode, LogLevelSetting, SettingsConfig, PROFILES_CONFIG_FILE_NAME,
        SETTINGS_CONFIG_FILE_NAME,
    };
    use crate::runtime::services::{SettingsService, SettingsSideEffects};
    use std::fs;

    const BLADE_MASTER_ID: &str = "male_slayer_blade_master";

    #[test]
    fn settings_service_rolls_back_when_side_effect_fails() {
        let harness = RuntimeHarness::new("settings-service-rollback");
        let service = SettingsService::with_side_effects(
            harness.config_store.clone(),
            Arc::new(FailingSettingsSideEffects),
        );
        let mut settings = harness.config_store.settings();
        settings.launch_at_startup = true;
        settings.log_level = LogLevelSetting::Error;

        let error = service.save_settings(settings).unwrap_err();

        assert!(error.message.contains("side effect failed"));
        let cached = harness.config_store.settings();
        assert!(!cached.launch_at_startup);
        assert_ne!(cached.log_level, LogLevelSetting::Error);
        let file_settings: SettingsConfig = serde_json::from_str(
            &fs::read_to_string(harness.dir.join(SETTINGS_CONFIG_FILE_NAME)).unwrap(),
        )
        .unwrap();
        assert!(!file_settings.launch_at_startup);
        assert_ne!(file_settings.log_level, LogLevelSetting::Error);
    }

    #[test]
    fn update_profile_keys_refreshes_effective_profile_and_runtime_snapshot() {
        let harness = RuntimeHarness::new("update-profile-keys-refresh");
        harness.set_profiles_for_blade_master(Vec::new(), Some(BLADE_MASTER_ID.to_string()));
        let before_revision = harness.supervisor.snapshot().revision;

        harness
            .supervisor
            .profile_service
            .update_profile_keys(
                BLADE_MASTER_ID.to_string(),
                vec![KeyBinding {
                    vk: 0x41,
                    interval_ms: 25,
                    mode: FireKeyMode::Toggle,
                }],
            )
            .unwrap();
        let snapshot = harness
            .supervisor
            .apply_profiles_changed_for_test()
            .unwrap();

        assert_eq!(snapshot.revision, before_revision + 1);
        assert_eq!(snapshot.effective_profile.keys.len(), 1);
        assert_eq!(snapshot.effective_profile.keys[0].vk, 0x41);
        assert_eq!(snapshot.runtime.revision, snapshot.revision);
        assert_eq!(
            harness
                .supervisor
                .assistant_runtime
                .snapshot()
                .profile_key_count,
            1
        );
        assert_eq!(
            harness.config_store.profiles().classes[BLADE_MASTER_ID].enabled_keys[0].interval_ms,
            25
        );
        assert!(harness.dir.join(PROFILES_CONFIG_FILE_NAME).exists());
    }

    #[test]
    fn select_active_config_bumps_revision_and_recomputes_effective_profile() {
        let harness = RuntimeHarness::new("select-active-config-revision");
        harness.set_profiles_for_blade_master(
            vec![KeyBinding {
                vk: 0x58,
                interval_ms: 20,
                mode: FireKeyMode::Hold,
            }],
            None,
        );
        let before = harness.supervisor.app_state_snapshot();

        let changed = harness
            .supervisor
            .profile_service
            .select_active_config(Some(BLADE_MASTER_ID.to_string()))
            .unwrap();
        let snapshot = harness
            .supervisor
            .apply_profiles_changed_for_test()
            .unwrap();

        assert!(changed);
        assert_eq!(snapshot.revision, before.revision + 1);
        assert_eq!(
            snapshot.profiles.active_class_id,
            Some(BLADE_MASTER_ID.to_string())
        );
        assert_eq!(snapshot.effective_profile.keys.len(), 1);
        assert_eq!(snapshot.effective_profile.keys[0].vk, 0x5A);
        assert_eq!(snapshot.runtime.revision, snapshot.revision);
    }

    struct RuntimeHarness {
        dir: std::path::PathBuf,
        config_store: Arc<ConfigRepository>,
        supervisor: RuntimeSupervisor,
    }

    impl RuntimeHarness {
        fn new(name: &str) -> Self {
            let dir = unique_temp_dir(name);
            let config_store = Arc::new(ConfigRepository::from_path(dir.join("app-config.json")));
            let engine = Arc::new(Mutex::new(AutoFireEngine::new()));
            let combo_engine = Arc::new(Mutex::new(ComboEngine::new()));
            let auto_run_runtime = Arc::new(Mutex::new(AutoRunEngine::new()));
            let floating_control_runtime = Arc::new(Mutex::new(FloatingControlRuntime::new()));
            let assistant_runtime =
                AssistantRuntime::new(engine, combo_engine, auto_run_runtime, config_store.clone());
            let detection_runtime = Arc::new(Mutex::new(crate::vision::DetectionRuntime::new(
                config_store.clone(),
            )));
            let supervisor = RuntimeSupervisor::new(
                assistant_runtime,
                detection_runtime,
                floating_control_runtime,
                config_store.clone(),
                Arc::new(Mutex::new(None)),
            );

            Self {
                dir,
                config_store,
                supervisor,
            }
        }

        fn set_profiles_for_blade_master(
            &self,
            global_keys: Vec<KeyBinding>,
            active_class_id: Option<String>,
        ) {
            let mut profiles = ProfilesConfig {
                global_keys,
                active_class_id,
                ..ProfilesConfig::default()
            };
            profiles.classes.insert(
                BLADE_MASTER_ID.to_string(),
                ClassConfig {
                    enabled_keys: vec![KeyBinding {
                        vk: 0x5A,
                        interval_ms: 30,
                        mode: FireKeyMode::Hold,
                    }],
                    effect_rule: EffectRule::ClassOnly,
                    combo_defs: Vec::new(),
                },
            );
            self.config_store
                .replace_profiles_for_import(profiles)
                .unwrap();
        }
    }

    impl Drop for RuntimeHarness {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.dir);
        }
    }

    struct FailingSettingsSideEffects;

    impl SettingsSideEffects for FailingSettingsSideEffects {
        fn apply(
            &self,
            _previous_settings: &SettingsConfig,
            _next_settings: &SettingsConfig,
        ) -> AppResult<()> {
            Err(AppError::startup("side effect failed"))
        }
    }

    fn unique_temp_dir(name: &str) -> std::path::PathBuf {
        let mut dir = std::env::temp_dir();
        let unique_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        dir.push(format!(
            "dnfautofire-runtime-{name}-{}-{unique_id}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }
}
