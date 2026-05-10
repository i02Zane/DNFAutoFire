//! 配置系统：定义 schema、默认值、旧配置迁移、校验和原子写入。

mod defaults;
mod migration;
mod paths;
mod repository;
mod schema;
mod validation;

pub(crate) use defaults::*;
pub(crate) use migration::*;
pub(crate) use paths::*;
pub(crate) use repository::*;
pub(crate) use schema::*;
pub(crate) use validation::*;

#[cfg(test)]
mod tests {
    use super::{
        is_legacy_profile_candidate, load_config_from_path, read_log_level_setting,
        validate_legacy_config, validate_runtime_profile, AppSettings, ClassConfig, ComboAction,
        ComboDefinition, ConfigRepository, CustomConfig, DetectionNoMatchPolicy, DetectionSettings,
        EffectRule, FireKeyMode, KeyBinding, LegacyAppConfig, LogLevelSetting, ProfilesConfig,
        SettingsConfig, CONFIG_VERSION, DEFAULT_DETECTION_INTERVAL_MS, PROFILES_CONFIG_FILE_NAME,
        SETTINGS_CONFIG_FILE_NAME,
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
                "detection":{"enabled":true,"intervalMs":5000},
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
                "detection":{"enabled":true,"intervalMs":5000},
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
    fn load_config_normalizes_legacy_detection_settings() {
        let dir = unique_temp_dir("legacy-detection-settings");
        fs::write(
            dir.join("app-config.json"),
            r#"{
                "version":6,
                "globalKeys":[],
                "comboDefs":[],
                "classes":{},
                "activeClassId":null,
                "toggleHotkey":null,
                "detection":{"enabled":true,"intervalMs":5000},
                "settings":{}
            }"#,
        )
        .unwrap();

        let config = load_config_from_path(&dir.join("app-config.json"));

        assert_eq!(config.version, CONFIG_VERSION);
        assert!(!config.detection.enabled);
        assert_eq!(config.detection.interval_ms, DEFAULT_DETECTION_INTERVAL_MS);
        assert_eq!(
            config.detection.no_match_policy,
            DetectionNoMatchPolicy::Current
        );

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn load_config_defaults_missing_fire_key_mode_to_hold() {
        let dir = unique_temp_dir("missing-fire-key-mode");
        fs::write(
            dir.join("app-config.json"),
            r#"{
                "version":10,
                "globalKeys":[{"vk":88,"intervalMs":20}],
                "comboDefs":[],
                "classes":{
                    "male_slayer_blade_master":{
                        "enabledKeys":[{"vk":65,"intervalMs":30}],
                        "effectRule":"globalAndClass",
                        "comboDefs":[]
                    }
                },
                "customConfigs":{},
                "hiddenClassIds":[],
                "activeClassId":null,
                "toggleHotkey":null,
                "detection":{"enabled":false,"intervalMs":200,"noMatchPolicy":"current"},
                "settings":{}
            }"#,
        )
        .unwrap();

        let config = load_config_from_path(&dir.join("app-config.json"));

        assert_eq!(config.version, CONFIG_VERSION);
        assert_eq!(config.global_keys[0].mode, FireKeyMode::Hold);
        assert_eq!(
            config.classes["male_slayer_blade_master"].enabled_keys[0].mode,
            FireKeyMode::Hold
        );

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn validate_config_rejects_invalid_detection_interval() {
        let mut config = minimal_config();
        config.detection.interval_ms = 300;

        assert!(validate_legacy_config(&config)
            .unwrap_err()
            .message
            .contains("职业识别间隔只能是"));
    }

    #[test]
    fn validate_config_rejects_invalid_auto_run_pulse_delay() {
        let mut config = minimal_config();
        config.settings.auto_run_pulse_delay_ms = 201;

        assert!(validate_legacy_config(&config)
            .unwrap_err()
            .message
            .contains("一键奔跑双击间隔必须在"));
    }

    #[test]
    fn load_config_preserves_valid_auto_run_pulse_delay() {
        let dir = unique_temp_dir("auto-run-pulse-delay");
        fs::write(
            dir.join("app-config.json"),
            r#"{
                "version":10,
                "globalKeys":[],
                "comboDefs":[],
                "classes":{},
                "customConfigs":{},
                "hiddenClassIds":[],
                "activeClassId":null,
                "toggleHotkey":null,
                "detection":{"enabled":false,"intervalMs":200,"noMatchPolicy":"current"},
                "settings":{"autoRunPulseDelayMs":30}
            }"#,
        )
        .unwrap();

        let config = load_config_from_path(&dir.join("app-config.json"));

        assert_eq!(config.settings.auto_run_pulse_delay_ms, 30);

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
                "detection":{"enabled":true,"intervalMs":5000},
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
                "detection":{"enabled":true,"intervalMs":5000},
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
                "detection":{"enabled":true,"intervalMs":5000},
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
                "detection":{"enabled":true,"intervalMs":5000},
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
                "detection":{"enabled":true,"intervalMs":5000},
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
            .message
            .contains("同一职业的一键连招触发键不能重复"));
    }

    #[test]
    fn validate_runtime_profile_rejects_trigger_autofire_overlap() {
        let keys = vec![KeyBinding {
            vk: 0x41,
            interval_ms: 20,
            mode: FireKeyMode::Hold,
        }];
        let combos = vec![valid_combo("combo-1", 0x41)];

        let result = validate_runtime_profile(&keys, &combos);

        assert!(result
            .unwrap_err()
            .message
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
            .message
            .contains("手搓动作只能使用上下左右和 Z/X/C/空格"));
    }

    #[test]
    fn validate_runtime_profile_rejects_command_with_too_many_directions() {
        let mut combo = valid_command_combo();
        combo.actions = vec![command_action(vec![0x26, 0x28, 0x25, 0x27, 0x26, 0x5A])];

        let result = validate_runtime_profile(&[], &[combo]);

        assert!(result
            .unwrap_err()
            .message
            .contains("手搓动作最多只能包含 4 个方向键"));
    }

    #[test]
    fn validate_runtime_profile_rejects_command_without_finish_key() {
        let mut combo = valid_command_combo();
        combo.actions = vec![command_action(vec![0x26, 0x28])];

        let result = validate_runtime_profile(&[], &[combo]);

        assert!(result
            .unwrap_err()
            .message
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

        assert!(validate_legacy_config(&config).is_ok());
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
                        mode: FireKeyMode::Hold,
                    },
                    KeyBinding {
                        vk: 0x41,
                        interval_ms: 25,
                        mode: FireKeyMode::Hold,
                    },
                ],
                effect_rule: EffectRule::GlobalAndClass,
                combo_defs: Vec::new(),
            },
        );

        assert!(validate_legacy_config(&config)
            .unwrap_err()
            .message
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
                    mode: FireKeyMode::Hold,
                }],
                effect_rule: EffectRule::ClassOnly,
                combo_defs: vec![valid_combo("combo-1", 0x41)],
            },
        );

        assert!(validate_legacy_config(&config)
            .unwrap_err()
            .message
            .contains("一键连招触发键不能与当前生效连发键重复"));
    }

    #[test]
    fn config_repository_save_updates_cache_and_split_files() {
        let dir = unique_temp_dir("config-repository-save");
        let path = dir.join("app-config.json");
        let store = ConfigRepository::from_path(path.clone());

        let legacy_config = minimal_config();
        let mut settings = SettingsConfig::from(&legacy_config);
        settings.launch_at_startup = true;

        let mut profiles = ProfilesConfig::from(&legacy_config);
        profiles.global_keys.push(KeyBinding {
            vk: 0x4A,
            interval_ms: 25,
            mode: FireKeyMode::Hold,
        });

        store.save_settings(settings.clone()).unwrap();
        store.replace_profiles_for_import(profiles.clone()).unwrap();

        let cached_settings = store.settings();
        let cached_profiles = store.profiles();
        let file_profiles: ProfilesConfig =
            serde_json::from_str(&fs::read_to_string(dir.join(PROFILES_CONFIG_FILE_NAME)).unwrap())
                .unwrap();
        let file_settings: SettingsConfig =
            serde_json::from_str(&fs::read_to_string(dir.join(SETTINGS_CONFIG_FILE_NAME)).unwrap())
                .unwrap();

        assert!(cached_settings.launch_at_startup);
        assert_eq!(cached_settings.log_level, LogLevelSetting::default());
        assert_eq!(cached_profiles.global_keys[0].interval_ms, 25);
        assert_eq!(file_profiles.global_keys[0].interval_ms, 25);
        assert_eq!(file_profiles.auto_run.left_vk, 0x25);
        assert!(file_settings.launch_at_startup);
        assert_eq!(file_settings.log_level, LogLevelSetting::default());

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn config_repository_migrates_legacy_config_to_split_files() {
        let dir = unique_temp_dir("config-repository-legacy-migration");
        let path = dir.join("app-config.json");
        let mut legacy_config = minimal_config();
        legacy_config.global_keys.push(KeyBinding {
            vk: 0x58,
            interval_ms: 30,
            mode: FireKeyMode::Toggle,
        });
        legacy_config.hidden_class_ids = vec!["male_slayer_blade_master".to_string()];
        legacy_config.detection.enabled = true;
        legacy_config.detection.interval_ms = 500;
        legacy_config.settings.launch_at_startup = true;
        legacy_config.settings.auto_run_enabled = true;
        legacy_config.settings.auto_run_pulse_delay_ms = 50;
        fs::write(&path, serde_json::to_string_pretty(&legacy_config).unwrap()).unwrap();

        let store = ConfigRepository::from_path(path.clone());
        let migrated_settings = store.settings();
        let migrated_profiles = store.profiles();

        assert!(path.exists());
        assert!(dir.join(SETTINGS_CONFIG_FILE_NAME).exists());
        assert!(dir.join(PROFILES_CONFIG_FILE_NAME).exists());
        assert_eq!(migrated_profiles.global_keys[0].mode, FireKeyMode::Toggle);
        assert_eq!(migrated_settings.detection.interval_ms, 500);
        assert!(migrated_settings.launch_at_startup);
        assert!(migrated_profiles.auto_run.enabled);
        assert_eq!(migrated_profiles.auto_run.pulse_delay_ms, 50);
        let profiles_json: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(dir.join(PROFILES_CONFIG_FILE_NAME)).unwrap())
                .unwrap();
        let settings_json: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(dir.join(SETTINGS_CONFIG_FILE_NAME)).unwrap())
                .unwrap();
        assert_eq!(profiles_json["autoRun"]["enabled"], true);
        assert_eq!(profiles_json["autoRun"]["pulseDelayMs"], 50);
        assert!(profiles_json.get("autoRunEnabled").is_none());
        assert!(profiles_json.get("autoRunLeftVk").is_none());
        assert!(profiles_json.get("autoRunRightVk").is_none());
        assert!(profiles_json.get("autoRunPulseDelayMs").is_none());
        assert!(settings_json.get("activeClassId").is_none());
        assert!(settings_json.get("hiddenClassIds").is_none());
        assert_eq!(
            profiles_json["hiddenClassIds"],
            serde_json::json!(["male_slayer_blade_master"])
        );
        assert_eq!(
            migrated_profiles.hidden_class_ids,
            vec!["male_slayer_blade_master".to_string()]
        );

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn config_repository_falls_back_only_for_invalid_split_file() {
        let dir = unique_temp_dir("config-repository-split-fallback");
        let path = dir.join("app-config.json");
        fs::write(dir.join(SETTINGS_CONFIG_FILE_NAME), "{not-json").unwrap();
        let mut profiles = ProfilesConfig::default();
        profiles.global_keys = vec![KeyBinding {
            vk: 0x5A,
            interval_ms: 40,
            mode: FireKeyMode::Hold,
        }];
        fs::write(
            dir.join(PROFILES_CONFIG_FILE_NAME),
            serde_json::to_string_pretty(&profiles).unwrap(),
        )
        .unwrap();

        let store = ConfigRepository::from_path(path);
        let settings = store.settings();
        let profiles = store.profiles();

        assert_eq!(profiles.global_keys[0].vk, 0x5A);
        assert_eq!(profiles.global_keys[0].interval_ms, 40);
        assert!(!settings.detection.enabled);
        assert_eq!(settings.log_level, LogLevelSetting::default());

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn select_active_config_returns_false_when_selection_does_not_change() {
        let dir = unique_temp_dir("select-active-config-no-op");
        let path = dir.join("app-config.json");
        let store = ConfigRepository::from_path(path);
        let mut config = minimal_config();
        config.classes.insert(
            "male_slayer_blade_master".to_string(),
            ClassConfig {
                enabled_keys: vec![KeyBinding {
                    vk: 0x41,
                    interval_ms: 20,
                    mode: FireKeyMode::Hold,
                }],
                effect_rule: EffectRule::GlobalAndClass,
                combo_defs: Vec::new(),
            },
        );
        let profiles = ProfilesConfig::from(&config);
        store.replace_profiles_for_import(profiles).unwrap();

        let first = store
            .select_active_config(Some("male_slayer_blade_master".to_string()))
            .unwrap();
        let second = store
            .select_active_config(Some("male_slayer_blade_master".to_string()))
            .unwrap();

        assert!(first);
        assert!(!second);
        assert_eq!(
            store.profiles().active_class_id,
            Some("male_slayer_blade_master".to_string())
        );

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn select_active_config_falls_back_to_global_for_unknown_class_id() {
        let dir = unique_temp_dir("select-active-config-unknown");
        let path = dir.join("app-config.json");
        let store = ConfigRepository::from_path(path);
        let mut config = minimal_config();
        config.classes.insert(
            "male_slayer_blade_master".to_string(),
            ClassConfig {
                enabled_keys: vec![KeyBinding {
                    vk: 0x41,
                    interval_ms: 20,
                    mode: FireKeyMode::Hold,
                }],
                effect_rule: EffectRule::GlobalAndClass,
                combo_defs: Vec::new(),
            },
        );
        config.active_class_id = Some("male_slayer_blade_master".to_string());
        let profiles = ProfilesConfig::from(&config);
        store.replace_profiles_for_import(profiles).unwrap();

        let saved = store
            .select_active_config(Some("missing-class-id".to_string()))
            .unwrap();

        assert!(saved);
        assert_eq!(store.profiles().active_class_id, None);

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

    fn minimal_config() -> LegacyAppConfig {
        LegacyAppConfig {
            version: CONFIG_VERSION,
            global_keys: Vec::new(),
            combo_defs: Vec::new(),
            classes: BTreeMap::new(),
            custom_configs: BTreeMap::new(),
            hidden_class_ids: Vec::new(),
            active_class_id: None,
            toggle_hotkey: None,
            detection: DetectionSettings::default(),
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
