//! 配置 schema：定义持久化形状与组合配置。

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use ts_rs::TS;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub enum FireKeyMode {
    #[default]
    Hold,
    Toggle,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct KeyBinding {
    pub vk: u16,
    pub interval_ms: u16,
    #[serde(default)]
    pub mode: FireKeyMode,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub enum EffectRule {
    #[default]
    GlobalAndClass,
    ClassOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct ComboDefinition {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub trigger_vk: Option<u16>,
    #[serde(default)]
    pub actions: Vec<ComboAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "camelCase")]
#[ts(tag = "type", rename_all = "camelCase")]
pub enum ComboAction {
    #[serde(rename = "tap")]
    #[serde(rename_all = "camelCase")]
    Tap {
        #[serde(default)]
        id: String,
        #[serde(default)]
        label: String,
        #[serde(default)]
        vk: Option<u16>,
        #[serde(default = "super::defaults::default_combo_hold_ms", alias = "hold_ms")]
        hold_ms: u16,
        #[serde(
            default = "super::defaults::default_combo_wait_after_ms",
            alias = "wait_after_ms"
        )]
        wait_after_ms: u16,
    },
    #[serde(rename = "command")]
    #[serde(rename_all = "camelCase")]
    Command {
        #[serde(default)]
        id: String,
        #[serde(default)]
        label: String,
        #[serde(default)]
        keys: Vec<u16>,
        #[serde(
            default = "super::defaults::default_combo_hold_ms",
            alias = "key_hold_ms"
        )]
        key_hold_ms: u16,
        #[serde(
            default = "super::defaults::default_combo_gap_ms",
            alias = "key_gap_ms"
        )]
        key_gap_ms: u16,
        #[serde(
            default = "super::defaults::default_combo_wait_after_ms",
            alias = "wait_after_ms"
        )]
        wait_after_ms: u16,
    },
}

#[derive(Debug, Clone, Copy, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub enum ComboValidationField {
    Name,
    Trigger,
    Actions,
    TapKey,
    CommandKeys,
    HoldMs,
    KeyHoldMs,
    KeyGapMs,
    WaitAfterMs,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct ComboValidationIssue {
    pub combo_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_id: Option<String>,
    pub field: ComboValidationField,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct ClassConfig {
    #[serde(default)]
    pub enabled_keys: Vec<KeyBinding>,
    #[serde(default)]
    pub effect_rule: EffectRule,
    #[serde(default)]
    pub combo_defs: Vec<ComboDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct CustomConfig {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub enabled_keys: Vec<KeyBinding>,
    #[serde(default)]
    pub effect_rule: EffectRule,
    #[serde(default)]
    pub combo_defs: Vec<ComboDefinition>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub enum DetectionNoMatchPolicy {
    #[default]
    Current,
    Global,
}

impl std::fmt::Display for DetectionNoMatchPolicy {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Current => "current",
            Self::Global => "global",
        };
        formatter.write_str(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct DetectionSettings {
    #[serde(default = "super::defaults::default_detection_enabled")]
    pub enabled: bool,
    #[serde(default = "super::defaults::default_detection_interval_ms")]
    #[ts(type = "number")]
    pub interval_ms: u64,
    #[serde(default)]
    pub no_match_policy: DetectionNoMatchPolicy,
}

impl Default for DetectionSettings {
    fn default() -> Self {
        Self {
            enabled: super::defaults::default_detection_enabled(),
            interval_ms: super::defaults::default_detection_interval_ms(),
            no_match_policy: DetectionNoMatchPolicy::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub enum LogLevelSetting {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Off,
}

impl Default for LogLevelSetting {
    fn default() -> Self {
        if cfg!(debug_assertions) {
            Self::Debug
        } else {
            Self::Info
        }
    }
}

impl std::fmt::Display for LogLevelSetting {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Trace => "trace",
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
            Self::Off => "off",
        };
        formatter.write_str(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    #[serde(default)]
    pub launch_at_startup: bool,
    #[serde(default)]
    pub start_minimized: bool,
    #[serde(default)]
    pub minimize_to_tray: bool,
    #[serde(default)]
    pub close_button_minimizes: bool,
    #[serde(default)]
    pub open_floating_control_on_start: bool,
    #[serde(default)]
    pub auto_run_enabled: bool,
    #[serde(default = "super::defaults::default_auto_run_left_vk")]
    pub auto_run_left_vk: u16,
    #[serde(default = "super::defaults::default_auto_run_right_vk")]
    pub auto_run_right_vk: u16,
    #[serde(default = "super::defaults::default_auto_run_pulse_delay_ms")]
    pub auto_run_pulse_delay_ms: u64,
    #[serde(default)]
    pub log_level: LogLevelSetting,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            launch_at_startup: false,
            start_minimized: false,
            minimize_to_tray: false,
            close_button_minimizes: false,
            open_floating_control_on_start: false,
            auto_run_enabled: false,
            auto_run_left_vk: super::defaults::default_auto_run_left_vk(),
            auto_run_right_vk: super::defaults::default_auto_run_right_vk(),
            auto_run_pulse_delay_ms: super::defaults::default_auto_run_pulse_delay_ms(),
            log_level: LogLevelSetting::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct Hotkey {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub vk: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyAppConfig {
    pub version: u32,
    pub global_keys: Vec<KeyBinding>,
    #[serde(default)]
    pub combo_defs: Vec<ComboDefinition>,
    pub classes: BTreeMap<String, ClassConfig>,
    #[serde(default)]
    pub custom_configs: BTreeMap<String, CustomConfig>,
    #[serde(default)]
    pub hidden_class_ids: Vec<String>,
    #[serde(default)]
    pub active_class_id: Option<String>,
    pub toggle_hotkey: Option<Hotkey>,
    #[serde(default)]
    pub detection: DetectionSettings,
    #[serde(default)]
    pub settings: AppSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct SettingsConfig {
    #[serde(default = "super::defaults::default_settings_config_version")]
    pub version: u32,
    #[serde(default)]
    pub launch_at_startup: bool,
    #[serde(default)]
    pub start_minimized: bool,
    #[serde(default)]
    pub minimize_to_tray: bool,
    #[serde(default)]
    pub close_button_minimizes: bool,
    #[serde(default)]
    pub open_floating_control_on_start: bool,
    #[serde(default)]
    pub log_level: LogLevelSetting,
    #[serde(default)]
    pub toggle_hotkey: Option<Hotkey>,
    #[serde(default)]
    pub detection: DetectionSettings,
    #[serde(default)]
    pub floating_control: FloatingControlSettings,
}

impl Default for SettingsConfig {
    fn default() -> Self {
        Self {
            version: super::defaults::SETTINGS_CONFIG_VERSION,
            launch_at_startup: false,
            start_minimized: false,
            minimize_to_tray: false,
            close_button_minimizes: false,
            open_floating_control_on_start: false,
            log_level: LogLevelSetting::default(),
            toggle_hotkey: Some(Hotkey {
                ctrl: true,
                alt: false,
                shift: false,
                vk: 0x77,
            }),
            detection: DetectionSettings::default(),
            floating_control: FloatingControlSettings::default(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct FloatingControlSettings {
    #[serde(default)]
    pub position: Option<WindowPosition>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct WindowPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct AutoRunConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "super::defaults::default_auto_run_left_vk")]
    pub left_vk: u16,
    #[serde(default = "super::defaults::default_auto_run_right_vk")]
    pub right_vk: u16,
    #[serde(default = "super::defaults::default_auto_run_pulse_delay_ms")]
    #[ts(type = "number")]
    pub pulse_delay_ms: u64,
}

impl Default for AutoRunConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            left_vk: super::defaults::default_auto_run_left_vk(),
            right_vk: super::defaults::default_auto_run_right_vk(),
            pulse_delay_ms: super::defaults::default_auto_run_pulse_delay_ms(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct ProfilesConfig {
    #[serde(default = "super::defaults::default_profiles_config_version")]
    pub version: u32,
    #[serde(default)]
    pub global_keys: Vec<KeyBinding>,
    #[serde(default)]
    pub combo_defs: Vec<ComboDefinition>,
    #[serde(default)]
    #[ts(type = "Record<string, ClassConfig>")]
    pub classes: BTreeMap<String, ClassConfig>,
    #[serde(default)]
    #[ts(type = "Record<string, CustomConfig>")]
    pub custom_configs: BTreeMap<String, CustomConfig>,
    #[serde(default)]
    pub hidden_class_ids: Vec<String>,
    #[serde(default)]
    pub active_class_id: Option<String>,
    #[serde(default)]
    pub auto_run: AutoRunConfig,
}

impl Default for ProfilesConfig {
    fn default() -> Self {
        Self {
            version: super::defaults::PROFILES_CONFIG_VERSION,
            global_keys: vec![KeyBinding {
                vk: 0x58,
                interval_ms: super::defaults::DEFAULT_INTERVAL_MS,
                mode: FireKeyMode::Hold,
            }],
            combo_defs: Vec::new(),
            classes: BTreeMap::new(),
            custom_configs: BTreeMap::new(),
            hidden_class_ids: Vec::new(),
            active_class_id: None,
            auto_run: AutoRunConfig::default(),
        }
    }
}

impl From<&LegacyAppConfig> for SettingsConfig {
    fn from(config: &LegacyAppConfig) -> Self {
        Self {
            version: super::defaults::SETTINGS_CONFIG_VERSION,
            launch_at_startup: config.settings.launch_at_startup,
            start_minimized: config.settings.start_minimized,
            minimize_to_tray: config.settings.minimize_to_tray,
            close_button_minimizes: config.settings.close_button_minimizes,
            open_floating_control_on_start: config.settings.open_floating_control_on_start,
            log_level: config.settings.log_level,
            toggle_hotkey: config.toggle_hotkey.clone(),
            detection: config.detection.clone(),
            floating_control: FloatingControlSettings::default(),
        }
    }
}

impl From<&LegacyAppConfig> for ProfilesConfig {
    fn from(config: &LegacyAppConfig) -> Self {
        Self {
            version: super::defaults::PROFILES_CONFIG_VERSION,
            global_keys: config.global_keys.clone(),
            combo_defs: config.combo_defs.clone(),
            classes: config.classes.clone(),
            custom_configs: config.custom_configs.clone(),
            hidden_class_ids: config.hidden_class_ids.clone(),
            active_class_id: config.active_class_id.clone(),
            auto_run: AutoRunConfig {
                enabled: config.settings.auto_run_enabled,
                left_vk: config.settings.auto_run_left_vk,
                right_vk: config.settings.auto_run_right_vk,
                pulse_delay_ms: config.settings.auto_run_pulse_delay_ms,
            },
        }
    }
}
