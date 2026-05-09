//! 配置迁移和旧版兼容：读取 legacy 配置、拆分文件迁移和归一化。

use super::defaults::*;
use super::paths::*;
use super::schema::*;
use super::validation::{
    normalize_auto_run_pulse_delay_ms, normalize_detection_interval_ms, validate_legacy_config,
    validate_profiles_config, validate_settings_config,
};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LoadedConfigStatus {
    Existing,
    Migrated,
    Missing,
    Invalid,
}

#[derive(Debug, Clone)]
pub(crate) struct LoadedConfig<T> {
    pub(crate) value: T,
    status: LoadedConfigStatus,
}

impl<T> LoadedConfig<T> {
    pub(crate) fn should_persist(&self) -> bool {
        matches!(
            self.status,
            LoadedConfigStatus::Migrated | LoadedConfigStatus::Missing
        )
    }
}

pub(crate) fn normalize_active_config_id(
    profiles: &ProfilesConfig,
    active_class_id: Option<String>,
) -> Option<String> {
    let active_class_id = active_class_id?;
    let active_class_id = active_class_id.trim().to_string();
    if active_class_id.is_empty() {
        return None;
    }
    is_active_profile_id(profiles, &active_class_id).then_some(active_class_id)
}

fn is_active_profile_id(profiles: &ProfilesConfig, active_id: &str) -> bool {
    profiles
        .classes
        .get(active_id)
        .is_some_and(has_class_config)
        || profiles
            .custom_configs
            .get(active_id)
            .is_some_and(has_custom_config)
}

pub(crate) fn default_config() -> LegacyAppConfig {
    LegacyAppConfig {
        version: CONFIG_VERSION,
        global_keys: vec![KeyBinding {
            vk: 0x58,
            interval_ms: DEFAULT_INTERVAL_MS,
            mode: FireKeyMode::Hold,
        }],
        combo_defs: Vec::new(),
        classes: BTreeMap::new(),
        custom_configs: BTreeMap::new(),
        hidden_class_ids: Vec::new(),
        active_class_id: None,
        toggle_hotkey: Some(Hotkey {
            ctrl: true,
            alt: false,
            shift: false,
            vk: 0x77,
        }),
        detection: DetectionSettings::default(),
        settings: AppSettings {
            launch_at_startup: false,
            start_minimized: false,
            minimize_to_tray: false,
            open_floating_control_on_start: false,
            auto_run_enabled: false,
            auto_run_left_vk: default_auto_run_left_vk(),
            auto_run_right_vk: default_auto_run_right_vk(),
            auto_run_pulse_delay_ms: default_auto_run_pulse_delay_ms(),
            log_level: LogLevelSetting::default(),
        },
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LogLevelConfigFile {
    #[serde(default)]
    log_level: Option<LogLevelSetting>,
    #[serde(default)]
    settings: LogLevelConfigSettings,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LogLevelConfigSettings {
    #[serde(default)]
    log_level: LogLevelSetting,
}

pub(crate) fn read_log_level_setting(path: &Path) -> LogLevelSetting {
    fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str::<LogLevelConfigFile>(&content).ok())
        .and_then(|config| config.log_level.or(Some(config.settings.log_level)))
        .unwrap_or_default()
}

pub(crate) fn load_config_from_path(path: &PathBuf) -> LegacyAppConfig {
    if let Ok(content) = fs::read_to_string(path) {
        if let Ok(mut config) = serde_json::from_str::<LegacyAppConfig>(&content) {
            normalize_config(&mut config);
            if validate_legacy_config(&config).is_ok() {
                tracing::info!(
                    path = %path.display(),
                    version = config.version,
                    global_key_count = config.global_keys.len(),
                    class_count = config.classes.len(),
                    custom_config_count = config.custom_configs.len(),
                    "配置加载成功"
                );
                return config;
            }
            tracing::warn!(path = %path.display(), "配置校验失败，准备迁移或回退默认配置");
        } else {
            tracing::warn!(path = %path.display(), "配置解析失败");
        }
    } else if path.exists() {
        tracing::warn!(path = %path.display(), "读取配置失败");
    } else {
        tracing::debug!(path = %path.display(), "配置文件不存在，尝试迁移旧版 profile");
    }

    // 读取失败或校验失败时尝试吸收旧版单 profile 文件，最后再回到默认配置。
    if let Some(config) = migrate_legacy_profile(path) {
        tracing::info!(path = %path.display(), "已从旧版 profile 迁移配置");
        return config;
    }

    tracing::warn!(path = %path.display(), "回退到默认配置");
    default_config()
}

fn normalize_config(config: &mut LegacyAppConfig) {
    let original_version = config.version;
    config.version = CONFIG_VERSION;
    normalize_keys(&mut config.global_keys);
    normalize_combos(&mut config.combo_defs, "global");
    for (class_id, class_config) in &mut config.classes {
        normalize_keys(&mut class_config.enabled_keys);
        normalize_combos(&mut class_config.combo_defs, class_id);
    }
    for (config_id, custom_config) in &mut config.custom_configs {
        normalize_keys(&mut custom_config.enabled_keys);
        normalize_combos(&mut custom_config.combo_defs, config_id);
    }
    normalize_detection_settings(config, original_version);
    normalize_app_settings(config);
    normalize_active_profile(config);
}

fn normalize_detection_settings(config: &mut LegacyAppConfig, original_version: u32) {
    if original_version < CONFIG_VERSION {
        config.detection.enabled = default_detection_enabled();
    }

    config.detection.interval_ms = normalize_detection_interval_ms(config.detection.interval_ms);
}

fn normalize_app_settings(config: &mut LegacyAppConfig) {
    config.settings.auto_run_pulse_delay_ms =
        normalize_auto_run_pulse_delay_ms(config.settings.auto_run_pulse_delay_ms);
}

pub(crate) fn normalize_settings_config(settings: &mut SettingsConfig) {
    settings.version = SETTINGS_CONFIG_VERSION;
    settings.detection.interval_ms =
        normalize_detection_interval_ms(settings.detection.interval_ms);
}

pub(crate) fn normalize_profiles_config(profiles: &mut ProfilesConfig) {
    profiles.version = PROFILES_CONFIG_VERSION;
    normalize_keys(&mut profiles.global_keys);
    normalize_combos(&mut profiles.combo_defs, "global");
    for (class_id, class_config) in &mut profiles.classes {
        normalize_keys(&mut class_config.enabled_keys);
        normalize_combos(&mut class_config.combo_defs, class_id);
    }
    for (config_id, custom_config) in &mut profiles.custom_configs {
        normalize_keys(&mut custom_config.enabled_keys);
        normalize_combos(&mut custom_config.combo_defs, config_id);
    }
    profiles.auto_run.pulse_delay_ms =
        normalize_auto_run_pulse_delay_ms(profiles.auto_run.pulse_delay_ms);
    let active_class_id = profiles.active_class_id.clone();
    profiles.active_class_id = normalize_active_config_id(profiles, active_class_id);
}

fn normalize_keys(keys: &mut [KeyBinding]) {
    for key in keys {
        key.interval_ms = key.interval_ms.clamp(MIN_INTERVAL_MS, MAX_INTERVAL_MS);
    }
}

fn normalize_combos(combos: &mut [ComboDefinition], owner_id: &str) {
    for (combo_index, combo) in combos.iter_mut().enumerate() {
        if combo.id.trim().is_empty() {
            combo.id = format!("{owner_id}-combo-{}", combo_index + 1);
        }
        for (action_index, action) in combo.actions.iter_mut().enumerate() {
            match action {
                ComboAction::Tap {
                    id,
                    hold_ms,
                    wait_after_ms,
                    ..
                } => {
                    if id.trim().is_empty() {
                        *id = format!("{}-action-{}", combo.id, action_index + 1);
                    }
                    *hold_ms = (*hold_ms).clamp(MIN_COMBO_HOLD_MS, MAX_COMBO_HOLD_MS);
                    *wait_after_ms = (*wait_after_ms).min(MAX_COMBO_WAIT_MS);
                }
                ComboAction::Command {
                    id,
                    key_hold_ms,
                    key_gap_ms,
                    wait_after_ms,
                    ..
                } => {
                    if id.trim().is_empty() {
                        *id = format!("{}-action-{}", combo.id, action_index + 1);
                    }
                    *key_hold_ms = (*key_hold_ms).clamp(MIN_COMBO_HOLD_MS, MAX_COMBO_HOLD_MS);
                    *key_gap_ms = (*key_gap_ms).min(MAX_COMBO_GAP_MS);
                    *wait_after_ms = (*wait_after_ms).min(MAX_COMBO_WAIT_MS);
                }
            }
        }
    }
}

fn normalize_active_profile(config: &mut LegacyAppConfig) {
    let Some(active_id) = config.active_class_id.as_deref() else {
        return;
    };

    if !is_active_profile_id_in_config(config, active_id) {
        config.active_class_id = None;
    }
}

fn is_active_profile_id_in_config(config: &LegacyAppConfig, active_id: &str) -> bool {
    config.classes.get(active_id).is_some_and(has_class_config)
        || config
            .custom_configs
            .get(active_id)
            .is_some_and(has_custom_config)
}

fn has_class_config(config: &ClassConfig) -> bool {
    !config.enabled_keys.is_empty() || !config.combo_defs.is_empty()
}

fn has_custom_config(config: &CustomConfig) -> bool {
    !config.enabled_keys.is_empty() || !config.combo_defs.is_empty()
}

pub(crate) fn load_settings_config_from_path(
    path: &Path,
    legacy_config: Option<&LegacyAppConfig>,
) -> LoadedConfig<SettingsConfig> {
    if let Ok(content) = fs::read_to_string(path) {
        match serde_json::from_str::<SettingsConfig>(&content) {
            Ok(mut settings) => {
                normalize_settings_config(&mut settings);
                if validate_settings_config(&settings).is_ok() {
                    tracing::info!(
                        path = %path.display(),
                        version = settings.version,
                        "settings.json 加载成功"
                    );
                    return LoadedConfig {
                        value: settings,
                        status: LoadedConfigStatus::Existing,
                    };
                }
                tracing::warn!(path = %path.display(), "settings.json 校验失败，回退默认值");
            }
            Err(error) => {
                tracing::warn!(path = %path.display(), error = %error, "settings.json 解析失败");
            }
        }
        return LoadedConfig {
            value: SettingsConfig::default(),
            status: LoadedConfigStatus::Invalid,
        };
    }

    if path.exists() {
        tracing::warn!(path = %path.display(), "settings.json 读取失败");
        return LoadedConfig {
            value: SettingsConfig::default(),
            status: LoadedConfigStatus::Invalid,
        };
    }

    if let Some(legacy_config) = legacy_config {
        let mut settings = SettingsConfig::from(legacy_config);
        normalize_settings_config(&mut settings);
        tracing::info!(path = %path.display(), "已从旧版配置迁移 settings.json");
        return LoadedConfig {
            value: settings,
            status: LoadedConfigStatus::Migrated,
        };
    }

    tracing::debug!(path = %path.display(), "settings.json 不存在，使用默认配置");
    LoadedConfig {
        value: SettingsConfig::default(),
        status: LoadedConfigStatus::Missing,
    }
}

pub(crate) fn load_profiles_config_from_path(
    path: &Path,
    legacy_config: Option<&LegacyAppConfig>,
) -> LoadedConfig<ProfilesConfig> {
    if let Ok(content) = fs::read_to_string(path) {
        match serde_json::from_str::<ProfilesConfig>(&content) {
            Ok(mut profiles) => {
                normalize_profiles_config(&mut profiles);
                if validate_profiles_config(&profiles).is_ok() {
                    tracing::info!(
                        path = %path.display(),
                        version = profiles.version,
                        "profiles.json 加载成功"
                    );
                    return LoadedConfig {
                        value: profiles,
                        status: LoadedConfigStatus::Existing,
                    };
                }
                tracing::warn!(path = %path.display(), "profiles.json 校验失败，回退默认值");
            }
            Err(error) => {
                tracing::warn!(path = %path.display(), error = %error, "profiles.json 解析失败");
            }
        }
        return LoadedConfig {
            value: ProfilesConfig::default(),
            status: LoadedConfigStatus::Invalid,
        };
    }

    if path.exists() {
        tracing::warn!(path = %path.display(), "profiles.json 读取失败");
        return LoadedConfig {
            value: ProfilesConfig::default(),
            status: LoadedConfigStatus::Invalid,
        };
    }

    if let Some(legacy_config) = legacy_config {
        let mut profiles = ProfilesConfig::from(legacy_config);
        normalize_profiles_config(&mut profiles);
        tracing::info!(path = %path.display(), "已从旧版配置迁移 profiles.json");
        return LoadedConfig {
            value: profiles,
            status: LoadedConfigStatus::Migrated,
        };
    }

    tracing::debug!(path = %path.display(), "profiles.json 不存在，使用默认配置");
    LoadedConfig {
        value: ProfilesConfig::default(),
        status: LoadedConfigStatus::Missing,
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyProfile {
    enabled_keys: Vec<u16>,
}

fn migrate_legacy_profile(path: &Path) -> Option<LegacyAppConfig> {
    let dir = path.parent()?;
    let entries = fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !is_legacy_profile_candidate(&path) {
            continue;
        }

        // 旧版 profile 只有 enabledKeys，没有职业配置和间隔，迁移为全局默认间隔。
        let content = fs::read_to_string(&path).ok()?;
        let profile = serde_json::from_str::<LegacyProfile>(&content).ok()?;
        let mut config = default_config();
        config.global_keys = profile
            .enabled_keys
            .into_iter()
            .map(|vk| KeyBinding {
                vk,
                interval_ms: DEFAULT_INTERVAL_MS,
                mode: FireKeyMode::Hold,
            })
            .collect();
        tracing::info!(
            path = %path.display(),
            key_count = config.global_keys.len(),
            "检测到旧版 profile，已迁移为当前配置"
        );
        return Some(config);
    }
    None
}

pub(crate) fn is_legacy_profile_candidate(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };

    file_name != LEGACY_CONFIG_FILE_NAME
        && file_name != SETTINGS_CONFIG_FILE_NAME
        && file_name != PROFILES_CONFIG_FILE_NAME
        && path.extension().and_then(|ext| ext.to_str()) == Some("json")
}
