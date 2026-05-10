//! 运行时 service：封装具体配置写入和窗口操作，RuntimeSupervisor 只负责编排。

use crate::config::{
    validate_profiles_config, AutoRunConfig, ClassConfig, ComboDefinition, ConfigRepository,
    CustomConfig, EffectRule, FireKeyMode, KeyBinding, ProfilesConfig, SettingsConfig,
    WindowPosition, AUTOFIRE_KEY_CANDIDATE_VKS, DEFAULT_INTERVAL_MS,
};
use crate::engines::FireKeyConfig;
use crate::error::{AppError, AppResult};
use crate::platform::floating_control::FloatingControlRuntime;
use crate::platform::logging::update_log_level;
#[cfg(windows)]
use crate::platform::startup::set_windows_launch_at_startup;
use parking_lot::Mutex;
use std::sync::Arc;
use tauri::{AppHandle, Manager};

#[derive(Clone)]
pub(crate) struct SettingsService {
    config_store: Arc<ConfigRepository>,
    side_effects: Arc<dyn SettingsSideEffects>,
}

#[derive(Clone)]
pub(crate) struct ProfileService {
    config_store: Arc<ConfigRepository>,
}

#[derive(Clone)]
pub(crate) struct WindowService {
    config_store: Arc<ConfigRepository>,
    floating_control_runtime: Arc<Mutex<FloatingControlRuntime>>,
}

impl SettingsService {
    pub(crate) fn new(config_store: Arc<ConfigRepository>) -> Self {
        Self {
            config_store,
            side_effects: Arc::new(SystemSettingsSideEffects),
        }
    }

    #[cfg(test)]
    pub(crate) fn with_side_effects(
        config_store: Arc<ConfigRepository>,
        side_effects: Arc<dyn SettingsSideEffects>,
    ) -> Self {
        Self {
            config_store,
            side_effects,
        }
    }

    pub(crate) fn save_settings(&self, mut settings: SettingsConfig) -> AppResult<()> {
        let previous_settings = self.config_store.settings();
        settings.detection.interval_ms =
            crate::config::normalize_detection_interval_ms(settings.detection.interval_ms);
        self.config_store.save_settings(settings)?;
        let saved_settings = self.config_store.settings();
        if let Err(error) = self.side_effects.apply(&previous_settings, &saved_settings) {
            if let Err(rollback_error) = self.config_store.save_settings(previous_settings.clone())
            {
                tracing::error!(error = %rollback_error, "settings.json 回滚失败");
            }
            if let Err(restore_error) = self.side_effects.apply(&saved_settings, &previous_settings)
            {
                tracing::error!(error = %restore_error, "设置副作用回滚失败");
            }
            return Err(error);
        }
        Ok(())
    }

    pub(crate) fn sync_current_launch_at_startup(&self) -> AppResult<()> {
        sync_launch_at_startup_setting(self.config_store.settings().launch_at_startup)
    }

    pub(crate) fn update_floating_control_position(
        &self,
        position: WindowPosition,
    ) -> AppResult<()> {
        let mut settings = self.config_store.settings();
        settings.floating_control.position = Some(position);
        self.save_settings(settings)
    }
}

impl ProfileService {
    pub(crate) fn new(config_store: Arc<ConfigRepository>) -> Self {
        Self { config_store }
    }

    #[allow(dead_code)]
    pub(crate) fn replace_profiles_for_import(&self, profiles: ProfilesConfig) -> AppResult<()> {
        self.config_store.replace_profiles_for_import(profiles)
    }

    pub(crate) fn update_global_keys(&self, keys: Vec<KeyBinding>) -> AppResult<()> {
        self.update_profiles(|profiles| {
            profiles.global_keys = normalize_key_bindings(keys);
            Ok(())
        })
    }

    pub(crate) fn add_global_key(&self) -> AppResult<()> {
        self.update_profiles(|profiles| {
            let Some(keys) = first_valid_keys(profiles, |candidate_profiles, candidate_key| {
                candidate_profiles.global_keys.push(candidate_key);
            }) else {
                return Err(AppError::validation("没有可用的连发按键。"));
            };
            profiles.global_keys = keys.global_keys;
            Ok(())
        })
    }

    pub(crate) fn update_profile_keys(
        &self,
        config_id: String,
        keys: Vec<KeyBinding>,
    ) -> AppResult<()> {
        self.update_profiles(|profiles| {
            if let Some(custom_config) = profiles.custom_configs.get_mut(&config_id) {
                custom_config.enabled_keys = normalize_key_bindings(keys);
                return Ok(());
            }

            let next_config = {
                let class_config = profiles
                    .classes
                    .entry(config_id.clone())
                    .or_insert_with(empty_class_config);
                class_config.enabled_keys = normalize_key_bindings(keys);
                class_config.clone()
            };
            if !class_config_has_data(&next_config) {
                profiles.classes.remove(&config_id);
            }
            Ok(())
        })
    }

    pub(crate) fn add_profile_key(&self, config_id: String) -> AppResult<()> {
        self.update_profiles(|profiles| {
            let Some(keys) = first_valid_keys(profiles, |candidate_profiles, candidate_key| {
                push_profile_key(candidate_profiles, &config_id, candidate_key);
            }) else {
                return Err(AppError::validation("没有可用的连发按键。"));
            };
            replace_profile_keys(profiles, &config_id, keys);
            Ok(())
        })
    }

    pub(crate) fn update_profile_effect_rule(
        &self,
        config_id: String,
        effect_rule: EffectRule,
    ) -> AppResult<()> {
        self.update_profiles(|profiles| {
            if let Some(custom_config) = profiles.custom_configs.get_mut(&config_id) {
                custom_config.effect_rule = effect_rule;
                return Ok(());
            }

            profiles
                .classes
                .entry(config_id)
                .or_insert_with(empty_class_config)
                .effect_rule = effect_rule;
            Ok(())
        })
    }

    pub(crate) fn update_profile_combos(
        &self,
        config_id: String,
        combos: Vec<ComboDefinition>,
    ) -> AppResult<()> {
        self.update_profiles(|profiles| {
            if let Some(custom_config) = profiles.custom_configs.get_mut(&config_id) {
                custom_config.combo_defs = combos;
                return Ok(());
            }

            let next_config = {
                let class_config = profiles
                    .classes
                    .entry(config_id.clone())
                    .or_insert_with(empty_class_config);
                class_config.combo_defs = combos;
                class_config.clone()
            };
            if !class_config_has_data(&next_config) {
                profiles.classes.remove(&config_id);
            }
            Ok(())
        })
    }

    pub(crate) fn update_auto_run(&self, auto_run: AutoRunConfig) -> AppResult<()> {
        self.update_profiles(|profiles| {
            profiles.auto_run = auto_run;
            Ok(())
        })
    }

    pub(crate) fn add_custom_config(&self, name: String) -> AppResult<()> {
        let trimmed_name = name.trim().to_string();
        if trimmed_name.is_empty() {
            return Err(AppError::validation("自定义配置名称不能为空"));
        }

        self.update_profiles(|profiles| {
            let config_id = next_custom_config_id(profiles);
            profiles.custom_configs.insert(
                config_id,
                CustomConfig {
                    name: trimmed_name,
                    enabled_keys: Vec::new(),
                    effect_rule: EffectRule::default(),
                    combo_defs: Vec::new(),
                },
            );
            Ok(())
        })
    }

    pub(crate) fn delete_custom_config(&self, config_id: String) -> AppResult<()> {
        self.update_profiles(|profiles| {
            profiles.custom_configs.remove(&config_id);
            if profiles.active_class_id.as_deref() == Some(config_id.as_str()) {
                profiles.active_class_id = None;
            }
            Ok(())
        })
    }

    pub(crate) fn set_class_hidden(&self, class_id: String, hidden: bool) -> AppResult<()> {
        self.update_profiles(|profiles| {
            if profiles
                .classes
                .get(&class_id)
                .is_some_and(class_config_has_data)
            {
                return Ok(());
            }

            profiles.hidden_class_ids.retain(|item| item != &class_id);
            if hidden {
                profiles.hidden_class_ids.push(class_id);
            }
            Ok(())
        })
    }

    pub(crate) fn select_active_config(&self, active_class_id: Option<String>) -> AppResult<bool> {
        self.config_store.select_active_config(active_class_id)
    }

    fn update_profiles<F>(&self, apply: F) -> AppResult<()>
    where
        F: FnOnce(&mut ProfilesConfig) -> AppResult<()>,
    {
        self.config_store.update_profiles(apply)
    }
}

impl WindowService {
    pub(crate) fn new(
        config_store: Arc<ConfigRepository>,
        floating_control_runtime: Arc<Mutex<FloatingControlRuntime>>,
    ) -> Self {
        Self {
            config_store,
            floating_control_runtime,
        }
    }

    pub(crate) fn floating_control_visible(&self) -> bool {
        self.floating_control_runtime.lock().is_visible()
    }

    pub(crate) fn set_floating_control_visible(
        &self,
        app: &AppHandle,
        visible: bool,
        position: Option<WindowPosition>,
    ) -> AppResult<()> {
        self.floating_control_runtime
            .lock()
            .set_visible(app, visible, position)
    }

    pub(crate) fn toggle_floating_control_visible(&self, app: &AppHandle) -> AppResult<()> {
        self.floating_control_runtime
            .lock()
            .toggle(app, self.config_store.settings().floating_control.position)
    }

    pub(crate) fn minimize_main_window(&self, app: &AppHandle) -> AppResult<()> {
        let window = app
            .get_webview_window("main")
            .ok_or_else(|| AppError::window("主窗口不存在"))?;
        if self.config_store.settings().minimize_to_tray {
            window
                .hide()
                .map_err(|error| AppError::window(error.to_string()))
        } else {
            window
                .minimize()
                .map_err(|error| AppError::window(error.to_string()))
        }
    }

    pub(crate) fn close_main_window(&self, app: &AppHandle) -> AppResult<()> {
        if self.config_store.settings().close_button_minimizes {
            return self.minimize_main_window(app);
        }

        self.set_floating_control_visible(
            app,
            false,
            self.config_store.settings().floating_control.position,
        )?;
        let window = app
            .get_webview_window("main")
            .ok_or_else(|| AppError::window("主窗口不存在"))?;
        window
            .close()
            .map_err(|error| AppError::window(error.to_string()))
    }
}

pub(crate) fn to_fire_key_configs(keys: Vec<KeyBinding>) -> Vec<FireKeyConfig> {
    keys.into_iter()
        .map(|key| FireKeyConfig {
            vk: key.vk,
            interval_ms: key.interval_ms,
            mode: key.mode,
        })
        .collect()
}

pub(crate) trait SettingsSideEffects: Send + Sync {
    fn apply(
        &self,
        previous_settings: &SettingsConfig,
        next_settings: &SettingsConfig,
    ) -> AppResult<()>;
}

struct SystemSettingsSideEffects;

impl SettingsSideEffects for SystemSettingsSideEffects {
    fn apply(
        &self,
        previous_settings: &SettingsConfig,
        next_settings: &SettingsConfig,
    ) -> AppResult<()> {
        if previous_settings.launch_at_startup != next_settings.launch_at_startup {
            sync_launch_at_startup_setting(next_settings.launch_at_startup)?;
        }
        if previous_settings.log_level != next_settings.log_level {
            update_log_level(next_settings.log_level)?;
        }
        Ok(())
    }
}

fn sync_launch_at_startup_setting(enabled: bool) -> AppResult<()> {
    tracing::info!(enabled, "同步开机启动设置");
    #[cfg(windows)]
    {
        set_windows_launch_at_startup(enabled)
    }

    #[cfg(not(windows))]
    {
        if enabled {
            return Err(AppError::startup("开机自启动当前仅支持 Windows。"));
        }
        Ok(())
    }
}

fn empty_class_config() -> ClassConfig {
    ClassConfig {
        enabled_keys: Vec::new(),
        effect_rule: EffectRule::default(),
        combo_defs: Vec::new(),
    }
}

fn class_config_has_data(config: &ClassConfig) -> bool {
    !config.enabled_keys.is_empty() || !config.combo_defs.is_empty()
}

fn normalize_key_bindings(keys: Vec<KeyBinding>) -> Vec<KeyBinding> {
    keys.into_iter()
        .map(|key| KeyBinding {
            interval_ms: key.interval_ms.clamp(
                crate::config::MIN_INTERVAL_MS,
                crate::config::MAX_INTERVAL_MS,
            ),
            ..key
        })
        .collect()
}

fn first_valid_keys(
    profiles: &ProfilesConfig,
    push_candidate: impl Fn(&mut ProfilesConfig, KeyBinding),
) -> Option<ProfilesConfig> {
    for vk in AUTOFIRE_KEY_CANDIDATE_VKS {
        let mut candidate_profiles = profiles.clone();
        push_candidate(&mut candidate_profiles, default_autofire_key(vk));
        if validate_profiles_config(&candidate_profiles).is_ok() {
            return Some(candidate_profiles);
        }
    }
    None
}

fn default_autofire_key(vk: u16) -> KeyBinding {
    KeyBinding {
        vk,
        interval_ms: DEFAULT_INTERVAL_MS,
        mode: FireKeyMode::Hold,
    }
}

fn push_profile_key(profiles: &mut ProfilesConfig, config_id: &str, key: KeyBinding) {
    if let Some(custom_config) = profiles.custom_configs.get_mut(config_id) {
        custom_config.enabled_keys.push(key);
        return;
    }

    profiles
        .classes
        .entry(config_id.to_string())
        .or_insert_with(empty_class_config)
        .enabled_keys
        .push(key);
}

fn replace_profile_keys(profiles: &mut ProfilesConfig, config_id: &str, source: ProfilesConfig) {
    if let Some(source_config) = source.custom_configs.get(config_id) {
        if let Some(custom_config) = profiles.custom_configs.get_mut(config_id) {
            custom_config.enabled_keys = source_config.enabled_keys.clone();
        }
        return;
    }

    if let Some(source_config) = source.classes.get(config_id) {
        profiles
            .classes
            .entry(config_id.to_string())
            .or_insert_with(empty_class_config)
            .enabled_keys = source_config.enabled_keys.clone();
    }
}

fn next_custom_config_id(profiles: &ProfilesConfig) -> String {
    let mut index = profiles.custom_configs.len() + 1;
    loop {
        let config_id = format!("custom-{index}");
        if !profiles.custom_configs.contains_key(&config_id) {
            return config_id;
        }
        index += 1;
    }
}
