//! 配置系统：定义 schema、默认值、旧配置迁移、校验和原子写入。

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use parking_lot::Mutex;

pub(crate) const CONFIG_VERSION: u32 = 6;
pub(crate) const DEFAULT_INTERVAL_MS: u16 = 20;
const MIN_INTERVAL_MS: u16 = 10;
const MAX_INTERVAL_MS: u16 = 1000;
const MIN_COMBO_HOLD_MS: u16 = 10;
const MAX_COMBO_HOLD_MS: u16 = 1000;
const MAX_COMBO_GAP_MS: u16 = 1000;
const MAX_COMBO_WAIT_MS: u16 = 5000;
const MAX_COMBO_COMMAND_DIRECTION_KEYS: usize = 4;
const COMBO_COMMAND_DIRECTION_VKS: [u16; 4] = [0x25, 0x26, 0x27, 0x28];
const COMBO_COMMAND_FINISH_VKS: [u16; 4] = [0x5A, 0x58, 0x43, 0x20];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyBinding {
    pub vk: u16,
    pub interval_ms: u16,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum EffectRule {
    #[default]
    GlobalAndClass,
    ClassOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
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
        #[serde(default = "default_combo_hold_ms", alias = "hold_ms")]
        hold_ms: u16,
        #[serde(default = "default_combo_wait_after_ms", alias = "wait_after_ms")]
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
        #[serde(default = "default_combo_hold_ms", alias = "key_hold_ms")]
        key_hold_ms: u16,
        #[serde(default = "default_combo_gap_ms", alias = "key_gap_ms")]
        key_gap_ms: u16,
        #[serde(default = "default_combo_wait_after_ms", alias = "wait_after_ms")]
        wait_after_ms: u16,
    },
}

fn default_combo_hold_ms() -> u16 {
    30
}

fn default_combo_gap_ms() -> u16 {
    20
}

fn default_combo_wait_after_ms() -> u16 {
    100
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassConfig {
    #[serde(default)]
    pub enabled_keys: Vec<KeyBinding>,
    #[serde(default)]
    pub effect_rule: EffectRule,
    #[serde(default)]
    pub combo_defs: Vec<ComboDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectionSettings {
    pub enabled: bool,
    pub interval_ms: u64,
    pub icon_database_version: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    #[serde(default)]
    pub launch_at_startup: bool,
    #[serde(default)]
    pub start_minimized: bool,
    #[serde(default)]
    pub minimize_to_tray: bool,
    #[serde(default)]
    pub open_floating_control_on_start: bool,
    #[serde(default)]
    pub log_level: LogLevelSetting,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hotkey {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub vk: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
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
    pub detection: DetectionSettings,
    #[serde(default)]
    pub settings: AppSettings,
}

pub(crate) struct AppConfigStore {
    path: PathBuf,
    config: Mutex<AppConfig>,
}

impl AppConfigStore {
    pub(crate) fn new() -> Self {
        Self::from_path(config_path())
    }

    pub(crate) fn from_path(path: PathBuf) -> Self {
        tracing::debug!(path = %path.display(), "初始化配置存储");
        let config = load_config_from_path(&path);
        Self {
            path,
            config: Mutex::new(config),
        }
    }

    pub(crate) fn current(&self) -> AppConfig {
        self.config.lock().clone()
    }

    pub(crate) fn save(&self, config: AppConfig) -> Result<AppConfig, String> {
        if let Err(error) = validate_config(&config) {
            tracing::warn!(path = %self.path.display(), error = %error, "配置校验失败，拒绝保存");
            return Err(error);
        }

        // 配置目录位于 exe 旁，便携使用时不能假设目录已经存在。
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                let message = format!("创建配置目录失败: {e}");
                tracing::error!(path = %self.path.display(), error = %message, "创建配置目录失败");
                message
            })?;
        }
        let content = serde_json::to_string_pretty(&config).map_err(|e| {
            let message = format!("序列化配置失败: {e}");
            tracing::error!(path = %self.path.display(), error = %message, "序列化配置失败");
            message
        })?;
        write_config_atomically(&self.path, content.as_bytes()).map_err(|e| {
            let message = format!("写入配置失败: {e}");
            tracing::error!(path = %self.path.display(), error = %message, "写入配置失败");
            message
        })?;

        *self.config.lock() = config.clone();
        tracing::info!(
            path = %self.path.display(),
            version = config.version,
            global_key_count = config.global_keys.len(),
            class_count = config.classes.len(),
            custom_config_count = config.custom_configs.len(),
            "配置已保存"
        );
        Ok(config)
    }
}

pub(crate) fn write_config_atomically(path: &Path, content: &[u8]) -> std::io::Result<()> {
    let temp_path = path.with_extension("json.tmp");
    {
        let mut temp_file = fs::File::create(&temp_path)?;
        temp_file.write_all(content)?;
        temp_file.write_all(b"\n")?;
        temp_file.sync_all()?;
    }

    // 先写临时文件再替换目标文件，避免崩溃或断电时留下半截 JSON。
    replace_file(&temp_path, path)?;
    if let Some(parent) = path.parent() {
        if let Ok(dir) = fs::File::open(parent) {
            let _ = dir.sync_all();
        }
    }
    Ok(())
}

#[cfg(windows)]
fn replace_file(source: &Path, target: &Path) -> std::io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows::core::PCWSTR;
    use windows::Win32::Storage::FileSystem::{
        MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
    };

    let source_wide: Vec<u16> = source.as_os_str().encode_wide().chain(Some(0)).collect();
    let target_wide: Vec<u16> = target.as_os_str().encode_wide().chain(Some(0)).collect();
    // Windows 上用 MoveFileExW 的 WRITE_THROUGH，尽量减少配置写入后丢失的窗口。
    unsafe {
        MoveFileExW(
            PCWSTR(source_wide.as_ptr()),
            PCWSTR(target_wide.as_ptr()),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    }
    .map_err(std::io::Error::from)
}

#[cfg(not(windows))]
fn replace_file(source: &Path, target: &Path) -> std::io::Result<()> {
    fs::rename(source, target)
}

fn default_config() -> AppConfig {
    AppConfig {
        version: CONFIG_VERSION,
        global_keys: vec![
            KeyBinding {
                vk: 0x58,
                interval_ms: DEFAULT_INTERVAL_MS,
            },
        ],
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
        detection: DetectionSettings {
            enabled: true,
            interval_ms: 5000,
            icon_database_version: "builtin-empty-v1".to_string(),
        },
        settings: AppSettings {
            launch_at_startup: false,
            start_minimized: false,
            minimize_to_tray: false,
            open_floating_control_on_start: false,
            log_level: LogLevelSetting::default(),
        },
    }
}

pub(crate) fn config_path() -> PathBuf {
    // 配置跟随 exe 目录，方便把整个程序目录复制到其他机器继续使用。
    let mut path = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    path.push("configs");
    path.push("app-config.json");
    path
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LogLevelConfigFile {
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
        .map(|config| config.settings.log_level)
        .unwrap_or_default()
}

pub(crate) fn load_config_from_path(path: &PathBuf) -> AppConfig {
    if let Ok(content) = fs::read_to_string(path) {
        if let Ok(mut config) = serde_json::from_str::<AppConfig>(&content) {
            normalize_config(&mut config);
            if validate_config(&config).is_ok() {
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

fn normalize_config(config: &mut AppConfig) {
    config.version = CONFIG_VERSION;
    normalize_combos(&mut config.combo_defs, "global");
    for (class_id, class_config) in &mut config.classes {
        normalize_combos(&mut class_config.combo_defs, class_id);
    }
    for (config_id, custom_config) in &mut config.custom_configs {
        normalize_combos(&mut custom_config.combo_defs, config_id);
    }
    normalize_active_profile(config);
}

fn normalize_combos(combos: &mut [ComboDefinition], owner_id: &str) {
    for (combo_index, combo) in combos.iter_mut().enumerate() {
        if combo.id.trim().is_empty() {
            combo.id = format!("{owner_id}-combo-{}", combo_index + 1);
        }
        for (action_index, action) in combo.actions.iter_mut().enumerate() {
            match action {
                ComboAction::Tap { id, .. } | ComboAction::Command { id, .. } => {
                    if id.trim().is_empty() {
                        *id = format!("{}-action-{}", combo.id, action_index + 1);
                    }
                }
            }
        }
    }
}

fn normalize_active_profile(config: &mut AppConfig) {
    let Some(active_id) = config.active_class_id.as_deref() else {
        return;
    };

    let has_active_profile = config.classes.get(active_id).is_some_and(has_class_config)
        || config
            .custom_configs
            .get(active_id)
            .is_some_and(has_custom_config);

    if !has_active_profile {
        config.active_class_id = None;
    }
}

fn has_class_config(config: &ClassConfig) -> bool {
    !config.enabled_keys.is_empty() || !config.combo_defs.is_empty()
}

fn has_custom_config(config: &CustomConfig) -> bool {
    !config.enabled_keys.is_empty() || !config.combo_defs.is_empty()
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyProfile {
    enabled_keys: Vec<u16>,
}

fn migrate_legacy_profile(path: &Path) -> Option<AppConfig> {
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

fn is_legacy_profile_candidate(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };

    file_name != "app-config.json" && path.extension().and_then(|ext| ext.to_str()) == Some("json")
}

pub(crate) fn validate_config(config: &AppConfig) -> Result<(), String> {
    validate_keys(&config.global_keys)?;
    for class_config in config.classes.values() {
        validate_keys(&class_config.enabled_keys)?;
        let effective_keys = effective_keys_for_profile(
            config,
            &class_config.enabled_keys,
            &class_config.effect_rule,
        );
        validate_combo_defs(&class_config.combo_defs, &effective_keys)?;
    }
    for custom_config in config.custom_configs.values() {
        if custom_config.name.trim().is_empty() {
            return Err("自定义配置名称不能为空".to_string());
        }
        validate_keys(&custom_config.enabled_keys)?;
        let effective_keys = effective_keys_for_profile(
            config,
            &custom_config.enabled_keys,
            &custom_config.effect_rule,
        );
        validate_combo_defs(&custom_config.combo_defs, &effective_keys)?;
    }
    Ok(())
}

pub(crate) fn validate_keys(keys: &[KeyBinding]) -> Result<(), String> {
    let mut seen = HashSet::new();
    for key in keys {
        if !(MIN_INTERVAL_MS..=MAX_INTERVAL_MS).contains(&key.interval_ms) {
            return Err(format!(
                "连发间隔必须在 {MIN_INTERVAL_MS}-{MAX_INTERVAL_MS} 毫秒之间"
            ));
        }
        if !seen.insert(key.vk) {
            return Err("同一配置中不能重复添加相同按键".to_string());
        }
    }
    Ok(())
}

pub(crate) fn validate_runtime_profile(
    keys: &[KeyBinding],
    combos: &[ComboDefinition],
) -> Result<(), String> {
    validate_keys(keys)?;
    let effective_keys = keys.iter().map(|key| key.vk).collect();
    validate_combo_defs(combos, &effective_keys)
}

pub(crate) fn validate_combo_defs(
    combos: &[ComboDefinition],
    effective_key_vks: &HashSet<u16>,
) -> Result<(), String> {
    let mut seen_triggers = HashSet::new();
    for combo in combos {
        for action in &combo.actions {
            validate_combo_action_timing(action)?;
        }

        if !combo.enabled {
            continue;
        }

        if combo.name.trim().is_empty() {
            return Err("启用的一键连招必须填写名称。".to_string());
        }

        let trigger_vk = combo
            .trigger_vk
            .ok_or_else(|| "启用的一键连招必须设置触发键。".to_string())?;
        if !seen_triggers.insert(trigger_vk) {
            return Err("同一职业的一键连招触发键不能重复。".to_string());
        }
        if effective_key_vks.contains(&trigger_vk) {
            return Err("一键连招触发键不能与当前生效连发键重复。".to_string());
        }
        if combo.actions.is_empty() {
            return Err("启用的一键连招至少需要一个动作。".to_string());
        }

        for action in &combo.actions {
            validate_enabled_combo_action(action)?;
        }
    }
    Ok(())
}

fn validate_enabled_combo_action(action: &ComboAction) -> Result<(), String> {
    match action {
        ComboAction::Tap { vk, .. } => {
            if vk.is_none() {
                return Err("快捷栏动作必须设置按键。".to_string());
            }
        }
        ComboAction::Command { keys, .. } => {
            if keys.is_empty() {
                return Err("手搓动作至少需要一个按键。".to_string());
            }
            if !keys.iter().all(|vk| is_combo_command_vk(*vk)) {
                return Err("手搓动作只能使用上下左右和 Z/X/C/空格。".to_string());
            }
            if keys
                .iter()
                .filter(|vk| COMBO_COMMAND_DIRECTION_VKS.contains(vk))
                .count()
                > MAX_COMBO_COMMAND_DIRECTION_KEYS
            {
                return Err("手搓动作最多只能包含 4 个方向键。".to_string());
            }
            if !keys
                .last()
                .is_some_and(|vk| COMBO_COMMAND_FINISH_VKS.contains(vk))
            {
                return Err("手搓动作必须以 Z/X/C/空格结束。".to_string());
            }
        }
    }
    Ok(())
}

fn is_combo_command_vk(vk: u16) -> bool {
    COMBO_COMMAND_DIRECTION_VKS.contains(&vk) || COMBO_COMMAND_FINISH_VKS.contains(&vk)
}

fn validate_combo_action_timing(action: &ComboAction) -> Result<(), String> {
    match action {
        ComboAction::Tap {
            hold_ms,
            wait_after_ms,
            ..
        } => {
            validate_combo_hold(*hold_ms)?;
            validate_combo_wait(*wait_after_ms)?;
        }
        ComboAction::Command {
            key_hold_ms,
            key_gap_ms,
            wait_after_ms,
            ..
        } => {
            validate_combo_hold(*key_hold_ms)?;
            if *key_gap_ms > MAX_COMBO_GAP_MS {
                return Err(format!("手搓按键间隔不能超过 {MAX_COMBO_GAP_MS} 毫秒"));
            }
            validate_combo_wait(*wait_after_ms)?;
        }
    }
    Ok(())
}

fn validate_combo_hold(value: u16) -> Result<(), String> {
    if !(MIN_COMBO_HOLD_MS..=MAX_COMBO_HOLD_MS).contains(&value) {
        return Err(format!(
            "连招按下时长必须在 {MIN_COMBO_HOLD_MS}-{MAX_COMBO_HOLD_MS} 毫秒之间"
        ));
    }
    Ok(())
}

fn validate_combo_wait(value: u16) -> Result<(), String> {
    if value > MAX_COMBO_WAIT_MS {
        return Err(format!("动作后等待不能超过 {MAX_COMBO_WAIT_MS} 毫秒"));
    }
    Ok(())
}

fn effective_keys_for_profile(
    config: &AppConfig,
    enabled_keys: &[KeyBinding],
    effect_rule: &EffectRule,
) -> HashSet<u16> {
    let mut effective_keys = HashSet::new();
    if *effect_rule == EffectRule::GlobalAndClass {
        effective_keys.extend(config.global_keys.iter().map(|key| key.vk));
    }
    effective_keys.extend(enabled_keys.iter().map(|key| key.vk));
    effective_keys
}

#[cfg(test)]
mod tests {
    use super::{
        is_legacy_profile_candidate, load_config_from_path, read_log_level_setting,
        validate_config, validate_runtime_profile, AppConfig, AppConfigStore, AppSettings,
        ComboAction, ComboDefinition, CustomConfig, DetectionSettings, EffectRule, KeyBinding,
        LogLevelSetting, CONFIG_VERSION,
    };
    use std::collections::BTreeMap;
    use std::fs;

    #[test]
    fn app_settings_read_current_floating_control_field() {
        let settings: AppSettings =
            serde_json::from_str(r#"{"openFloatingControlOnStart":true}"#).unwrap();

        assert!(settings.open_floating_control_on_start);
        assert_eq!(settings.log_level, LogLevelSetting::default());
    }

    #[test]
    fn app_settings_serialize_only_current_floating_control_field() {
        let value = serde_json::to_value(AppSettings {
            open_floating_control_on_start: true,
            ..AppSettings::default()
        })
        .unwrap();

        assert_eq!(value["openFloatingControlOnStart"], true);
        assert_eq!(value["logLevel"], LogLevelSetting::default().to_string());
    }

    #[test]
    fn legacy_profile_candidate_skips_unrelated_entries() {
        let dir = unique_temp_dir("legacy-profile-candidate");
        fs::create_dir_all(dir.join("backup")).unwrap();
        fs::write(dir.join("README"), "not json").unwrap();
        fs::write(dir.join("app-config.json"), "{}").unwrap();
        fs::write(dir.join("old-profile.json"), r#"{"enabledKeys":[74]}"#).unwrap();

        assert!(!is_legacy_profile_candidate(&dir.join("backup")));
        assert!(!is_legacy_profile_candidate(&dir.join("README")));
        assert!(!is_legacy_profile_candidate(&dir.join("app-config.json")));
        assert!(is_legacy_profile_candidate(&dir.join("old-profile.json")));

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn load_config_migrates_legacy_profile_with_unrelated_entries() {
        let dir = unique_temp_dir("legacy-profile-migration");
        fs::create_dir_all(dir.join("backup")).unwrap();
        fs::write(dir.join("README"), "not json").unwrap();
        fs::write(dir.join("old-profile.json"), r#"{"enabledKeys":[74,88]}"#).unwrap();

        let config = load_config_from_path(&dir.join("app-config.json"));

        assert_eq!(config.global_keys.len(), 2);
        assert_eq!(config.global_keys[0].vk, 74);
        assert_eq!(config.global_keys[1].vk, 88);

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn load_config_accepts_v3_combo_placeholder() {
        let dir = unique_temp_dir("v3-combo-placeholder");
        fs::write(
            dir.join("app-config.json"),
            r#"{
                "version":3,
                "globalKeys":[{"vk":74,"intervalMs":20}],
                "classes":{
                    "male_slayer_blade_master":{
                        "enabledKeys":[],
                        "effectRule":"globalAndClass",
                        "comboDefs":[{"name":"旧连招","steps":["A","S"]}]
                    }
                },
                "activeClassId":null,
                "toggleHotkey":{"ctrl":true,"alt":false,"shift":false,"vk":119},
                "detection":{"enabled":true,"intervalMs":5000,"iconDatabaseVersion":"builtin-empty-v1"},
                "settings":{}
            }"#,
        )
        .unwrap();

        let config = load_config_from_path(&dir.join("app-config.json"));
        let combo = &config.classes["male_slayer_blade_master"].combo_defs[0];

        assert_eq!(config.version, CONFIG_VERSION);
        assert_eq!(config.combo_defs.len(), 0);
        assert_eq!(combo.name, "旧连招");
        assert!(!combo.enabled);
        assert!(combo.trigger_vk.is_none());
        assert!(combo.actions.is_empty());
        assert!(!combo.id.is_empty());

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn load_config_defaults_v5_management_fields() {
        let dir = unique_temp_dir("v5-management-defaults");
        fs::write(
            dir.join("app-config.json"),
            r#"{
                "version":4,
                "globalKeys":[],
                "comboDefs":[],
                "classes":{},
                "activeClassId":null,
                "toggleHotkey":null,
                "detection":{"enabled":true,"intervalMs":5000,"iconDatabaseVersion":"builtin-empty-v1"},
                "settings":{}
            }"#,
        )
        .unwrap();

        let config = load_config_from_path(&dir.join("app-config.json"));

        assert_eq!(config.version, CONFIG_VERSION);
        assert!(config.custom_configs.is_empty());
        assert!(config.hidden_class_ids.is_empty());

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn load_config_clears_empty_active_custom_config() {
        let dir = unique_temp_dir("empty-active-custom-config");
        fs::write(
            dir.join("app-config.json"),
            r#"{
                "version":5,
                "globalKeys":[],
                "comboDefs":[],
                "classes":{},
                "customConfigs":{
                    "custom-empty":{
                        "name":"empty",
                        "enabledKeys":[],
                        "effectRule":"globalAndClass",
                        "comboDefs":[]
                    }
                },
                "hiddenClassIds":[],
                "activeClassId":"custom-empty",
                "toggleHotkey":null,
                "detection":{"enabled":true,"intervalMs":5000,"iconDatabaseVersion":"builtin-empty-v1"},
                "settings":{}
            }"#,
        )
        .unwrap();

        let config = load_config_from_path(&dir.join("app-config.json"));

        assert_eq!(config.active_class_id, None);
        assert!(config.custom_configs.contains_key("custom-empty"));

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn load_config_clears_empty_active_class_config() {
        let dir = unique_temp_dir("empty-active-class-config");
        fs::write(
            dir.join("app-config.json"),
            r#"{
                "version":5,
                "globalKeys":[],
                "comboDefs":[],
                "classes":{
                    "male_slayer_blade_master":{
                        "enabledKeys":[],
                        "effectRule":"globalAndClass",
                        "comboDefs":[]
                    }
                },
                "customConfigs":{},
                "hiddenClassIds":[],
                "activeClassId":"male_slayer_blade_master",
                "toggleHotkey":null,
                "detection":{"enabled":true,"intervalMs":5000,"iconDatabaseVersion":"builtin-empty-v1"},
                "settings":{}
            }"#,
        )
        .unwrap();

        let config = load_config_from_path(&dir.join("app-config.json"));

        assert_eq!(config.active_class_id, None);
        assert!(config.classes.contains_key("male_slayer_blade_master"));

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn load_config_clears_missing_active_profile() {
        let dir = unique_temp_dir("missing-active-profile");
        fs::write(
            dir.join("app-config.json"),
            r#"{
                "version":5,
                "globalKeys":[],
                "comboDefs":[],
                "classes":{},
                "customConfigs":{},
                "hiddenClassIds":[],
                "activeClassId":"missing-profile",
                "toggleHotkey":null,
                "detection":{"enabled":true,"intervalMs":5000,"iconDatabaseVersion":"builtin-empty-v1"},
                "settings":{}
            }"#,
        )
        .unwrap();

        let config = load_config_from_path(&dir.join("app-config.json"));

        assert_eq!(config.active_class_id, None);

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn load_config_accepts_camel_case_combo_action_timings() {
        let dir = unique_temp_dir("camel-case-combo-timings");
        fs::write(
            dir.join("app-config.json"),
            r#"{
                "version":4,
                "globalKeys":[],
                "comboDefs":[],
                "classes":{
                    "male_slayer_blade_master":{
                        "enabledKeys":[],
                        "effectRule":"globalAndClass",
                        "comboDefs":[{
                            "id":"combo-1",
                            "name":"测试连招",
                            "enabled":true,
                            "triggerVk":65,
                            "actions":[
                                {"id":"tap-1","type":"tap","label":"","vk":90,"holdMs":35,"waitAfterMs":120},
                                {"id":"command-1","type":"command","label":"","keys":[38,90],"keyHoldMs":40,"keyGapMs":25,"waitAfterMs":140}
                            ]
                        }]
                    }
                },
                "activeClassId":null,
                "toggleHotkey":null,
                "detection":{"enabled":true,"intervalMs":5000,"iconDatabaseVersion":"builtin-empty-v1"},
                "settings":{}
            }"#,
        )
        .unwrap();

        let config = load_config_from_path(&dir.join("app-config.json"));
        let actions = &config.classes["male_slayer_blade_master"].combo_defs[0].actions;

        assert!(matches!(
            actions[0],
            ComboAction::Tap {
                hold_ms: 35,
                wait_after_ms: 120,
                ..
            }
        ));
        assert!(matches!(
            actions[1],
            ComboAction::Command {
                key_hold_ms: 40,
                key_gap_ms: 25,
                wait_after_ms: 140,
                ..
            }
        ));

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn load_config_defaults_missing_combo_action_timings() {
        let dir = unique_temp_dir("missing-combo-timings");
        fs::write(
            dir.join("app-config.json"),
            r#"{
                "version":4,
                "globalKeys":[],
                "comboDefs":[],
                "classes":{
                    "male_slayer_blade_master":{
                        "enabledKeys":[],
                        "effectRule":"globalAndClass",
                        "comboDefs":[{
                            "id":"combo-1",
                            "name":"测试连招",
                            "enabled":false,
                            "triggerVk":null,
                            "actions":[
                                {"id":"tap-1","type":"tap","label":"","vk":null},
                                {"id":"command-1","type":"command","label":"","keys":[]}
                            ]
                        }]
                    }
                },
                "activeClassId":null,
                "toggleHotkey":null,
                "detection":{"enabled":true,"intervalMs":5000,"iconDatabaseVersion":"builtin-empty-v1"},
                "settings":{}
            }"#,
        )
        .unwrap();

        let config = load_config_from_path(&dir.join("app-config.json"));
        let actions = &config.classes["male_slayer_blade_master"].combo_defs[0].actions;

        assert!(matches!(
            actions[0],
            ComboAction::Tap {
                hold_ms: 30,
                wait_after_ms: 100,
                ..
            }
        ));
        assert!(matches!(
            actions[1],
            ComboAction::Command {
                key_hold_ms: 30,
                key_gap_ms: 20,
                wait_after_ms: 100,
                ..
            }
        ));

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn validate_runtime_profile_rejects_duplicate_combo_triggers() {
        let combos = vec![valid_combo("combo-1", 0x41), valid_combo("combo-2", 0x41)];

        let result = validate_runtime_profile(&[], &combos);

        assert!(result
            .unwrap_err()
            .contains("同一职业的一键连招触发键不能重复"));
    }

    #[test]
    fn validate_runtime_profile_rejects_trigger_autofire_overlap() {
        let keys = vec![KeyBinding {
            vk: 0x41,
            interval_ms: 20,
        }];
        let combos = vec![valid_combo("combo-1", 0x41)];

        let result = validate_runtime_profile(&keys, &combos);

        assert!(result
            .unwrap_err()
            .contains("一键连招触发键不能与当前生效连发键重复"));
    }

    #[test]
    fn validate_runtime_profile_accepts_command_combo_without_autofire_keys() {
        let result = validate_runtime_profile(&[], &[valid_command_combo()]);

        assert!(result.is_ok());
    }

    #[test]
    fn validate_runtime_profile_rejects_command_with_unrelated_key() {
        let mut combo = valid_command_combo();
        combo.actions = vec![command_action(vec![0x26, 0x41])];

        let result = validate_runtime_profile(&[], &[combo]);

        assert!(result
            .unwrap_err()
            .contains("手搓动作只能使用上下左右和 Z/X/C/空格"));
    }

    #[test]
    fn validate_runtime_profile_rejects_command_with_too_many_directions() {
        let mut combo = valid_command_combo();
        combo.actions = vec![command_action(vec![0x26, 0x28, 0x25, 0x27, 0x26, 0x5A])];

        let result = validate_runtime_profile(&[], &[combo]);

        assert!(result
            .unwrap_err()
            .contains("手搓动作最多只能包含 4 个方向键"));
    }

    #[test]
    fn validate_runtime_profile_rejects_command_without_finish_key() {
        let mut combo = valid_command_combo();
        combo.actions = vec![command_action(vec![0x26, 0x28])];

        let result = validate_runtime_profile(&[], &[combo]);

        assert!(result
            .unwrap_err()
            .contains("手搓动作必须以 Z/X/C/空格结束"));
    }

    #[test]
    fn validate_config_ignores_root_combo_defs() {
        let mut config = minimal_config();
        config.combo_defs.push(ComboDefinition {
            id: "root-invalid".to_string(),
            name: String::new(),
            enabled: true,
            trigger_vk: None,
            actions: Vec::new(),
        });

        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn validate_config_rejects_custom_duplicate_keys() {
        let mut config = minimal_config();
        config.custom_configs.insert(
            "custom-1".to_string(),
            CustomConfig {
                name: "测试配置".to_string(),
                enabled_keys: vec![
                    KeyBinding {
                        vk: 0x41,
                        interval_ms: 20,
                    },
                    KeyBinding {
                        vk: 0x41,
                        interval_ms: 25,
                    },
                ],
                effect_rule: EffectRule::GlobalAndClass,
                combo_defs: Vec::new(),
            },
        );

        assert!(validate_config(&config)
            .unwrap_err()
            .contains("同一配置中不能重复添加相同按键"));
    }

    #[test]
    fn validate_config_rejects_custom_combo_trigger_overlap() {
        let mut config = minimal_config();
        config.custom_configs.insert(
            "custom-1".to_string(),
            CustomConfig {
                name: "测试配置".to_string(),
                enabled_keys: vec![KeyBinding {
                    vk: 0x41,
                    interval_ms: 20,
                }],
                effect_rule: EffectRule::ClassOnly,
                combo_defs: vec![valid_combo("combo-1", 0x41)],
            },
        );

        assert!(validate_config(&config)
            .unwrap_err()
            .contains("一键连招触发键不能与当前生效连发键重复"));
    }

    #[test]
    fn app_config_store_save_updates_cache_and_file() {
        let dir = unique_temp_dir("app-config-store-save");
        let path = dir.join("app-config.json");
        let store = AppConfigStore::from_path(path.clone());
        let mut config = minimal_config();
        config.global_keys.push(KeyBinding {
            vk: 0x4A,
            interval_ms: 25,
        });

        let saved = store.save(config).unwrap();
        let cached = store.current();
        let file_config = load_config_from_path(&path);

        assert_eq!(saved.global_keys[0].interval_ms, 25);
        assert_eq!(cached.global_keys[0].interval_ms, 25);
        assert_eq!(file_config.global_keys[0].interval_ms, 25);

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn read_log_level_setting_reads_nested_setting() {
        let dir = unique_temp_dir("log-level-read");
        let path = dir.join("app-config.json");
        fs::write(&path, r#"{"settings":{"logLevel":"warn"}}"#).unwrap();

        assert_eq!(read_log_level_setting(&path), LogLevelSetting::Warn);

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn read_log_level_setting_falls_back_for_missing_or_invalid_content() {
        let dir = unique_temp_dir("log-level-fallback");
        let missing_path = dir.join("missing.json");
        let invalid_path = dir.join("invalid.json");
        fs::write(&invalid_path, "{not-json").unwrap();

        assert_eq!(
            read_log_level_setting(&missing_path),
            LogLevelSetting::default()
        );
        assert_eq!(
            read_log_level_setting(&invalid_path),
            LogLevelSetting::default()
        );

        fs::remove_dir_all(dir).unwrap();
    }

    fn unique_temp_dir(name: &str) -> std::path::PathBuf {
        let mut dir = std::env::temp_dir();
        let unique_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        dir.push(format!(
            "dnfautofire-{name}-{}-{unique_id}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn minimal_config() -> AppConfig {
        AppConfig {
            version: CONFIG_VERSION,
            global_keys: Vec::new(),
            combo_defs: Vec::new(),
            classes: BTreeMap::new(),
            custom_configs: BTreeMap::new(),
            hidden_class_ids: Vec::new(),
            active_class_id: None,
            toggle_hotkey: None,
            detection: DetectionSettings {
                enabled: true,
                interval_ms: 5000,
                icon_database_version: "builtin-empty-v1".to_string(),
            },
            settings: AppSettings::default(),
        }
    }

    fn valid_combo(id: &str, trigger_vk: u16) -> ComboDefinition {
        ComboDefinition {
            id: id.to_string(),
            name: "测试连招".to_string(),
            enabled: true,
            trigger_vk: Some(trigger_vk),
            actions: vec![ComboAction::Tap {
                id: format!("{id}-action"),
                label: String::new(),
                vk: Some(0x5A),
                hold_ms: 30,
                wait_after_ms: 100,
            }],
        }
    }

    fn valid_command_combo() -> ComboDefinition {
        ComboDefinition {
            id: "command-combo".to_string(),
            name: "手搓连招".to_string(),
            enabled: true,
            trigger_vk: Some(0x41),
            actions: vec![command_action(vec![0x26, 0x5A])],
        }
    }

    fn command_action(keys: Vec<u16>) -> ComboAction {
        ComboAction::Command {
            id: "command-action".to_string(),
            label: String::new(),
            keys,
            key_hold_ms: 30,
            key_gap_ms: 20,
            wait_after_ms: 100,
        }
    }
}
