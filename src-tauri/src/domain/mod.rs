//! 纯业务规则：职业目录、配置合并、当前配置归一化和运行时 profile 计算。

pub mod classes;

use crate::config::{
    ClassConfig, ComboDefinition, CustomConfig, EffectRule, KeyBinding, ProfilesConfig,
};
use crate::domain::classes::{class_categories, class_name_by_id};
use serde::Serialize;
use std::collections::BTreeMap;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub(crate) struct ConfigOption {
    pub id: Option<String>,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub(crate) struct ProfileDisplaySnapshot {
    pub config_options: Vec<ConfigOption>,
    pub visible_class_categories: Vec<ClassCategoryView>,
    #[ts(type = "Record<string, string>")]
    pub display_names: BTreeMap<String, String>,
    #[ts(type = "Record<string, ClassDisplayState>")]
    pub class_states: BTreeMap<String, ClassDisplayState>,
    #[ts(type = "Record<string, CustomConfigDisplayState>")]
    pub custom_config_states: BTreeMap<String, CustomConfigDisplayState>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub(crate) struct ClassCategoryView {
    pub name: String,
    pub classes: Vec<ClassInfoView>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub(crate) struct ClassInfoView {
    pub id: String,
    pub name: String,
    pub detection_index: u16,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub(crate) struct ClassDisplayState {
    pub configured: bool,
    pub has_keys: bool,
    pub has_combos: bool,
    pub hidden: bool,
    pub visible: bool,
    pub can_hide: bool,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub(crate) struct CustomConfigDisplayState {
    pub configured: bool,
    pub has_keys: bool,
    pub has_combos: bool,
}

#[derive(Debug, Clone)]
enum ProfileConfigRef<'a> {
    Class(&'a ClassConfig),
    Custom(&'a CustomConfig),
}

impl ProfileConfigRef<'_> {
    fn enabled_keys(&self) -> &[KeyBinding] {
        match self {
            Self::Class(config) => &config.enabled_keys,
            Self::Custom(config) => &config.enabled_keys,
        }
    }

    fn effect_rule(&self) -> &EffectRule {
        match self {
            Self::Class(config) => &config.effect_rule,
            Self::Custom(config) => &config.effect_rule,
        }
    }

    fn combo_defs(&self) -> &[ComboDefinition] {
        match self {
            Self::Class(config) => &config.combo_defs,
            Self::Custom(config) => &config.combo_defs,
        }
    }
}

pub(crate) fn compute_effective_keys(profiles: &ProfilesConfig) -> Vec<KeyBinding> {
    let Some(active_id) = profiles.active_class_id.as_deref() else {
        return dedupe_keys_prefer_last(&profiles.global_keys);
    };
    compute_effective_keys_for_profile(profiles, active_id)
}

pub(crate) fn compute_effective_keys_for_profile(
    profiles: &ProfilesConfig,
    profile_id: &str,
) -> Vec<KeyBinding> {
    let Some(profile_config) = profile_config(profiles, profile_id) else {
        return dedupe_keys_prefer_last(&profiles.global_keys);
    };

    if profile_config.enabled_keys().is_empty() {
        return dedupe_keys_prefer_last(&profiles.global_keys);
    }
    if *profile_config.effect_rule() == EffectRule::ClassOnly {
        return dedupe_keys_prefer_last(profile_config.enabled_keys());
    }

    let merged = profiles
        .global_keys
        .iter()
        .chain(profile_config.enabled_keys())
        .cloned()
        .collect::<Vec<_>>();
    dedupe_keys_prefer_last(&merged)
}

pub(crate) fn compute_effective_combos(profiles: &ProfilesConfig) -> Vec<ComboDefinition> {
    let Some(active_id) = profiles.active_class_id.as_deref() else {
        return Vec::new();
    };
    let Some(profile_config) = profile_config(profiles, active_id) else {
        return Vec::new();
    };

    profile_config
        .combo_defs()
        .iter()
        .filter(|combo| combo.enabled)
        .cloned()
        .collect()
}

pub(crate) fn profile_display_snapshot(profiles: &ProfilesConfig) -> ProfileDisplaySnapshot {
    let mut class_states = BTreeMap::new();
    let mut display_names = BTreeMap::new();
    let mut visible_class_categories = Vec::new();

    for category in class_categories() {
        let mut visible_classes = Vec::new();
        for class_info in category.classes {
            display_names.insert(class_info.id.to_string(), class_info.name.to_string());
            let state = class_display_state(profiles, class_info.id);
            if state.visible {
                visible_classes.push(ClassInfoView {
                    id: class_info.id.to_string(),
                    name: class_info.name.to_string(),
                    detection_index: class_info.detection_index,
                });
            }
            class_states.insert(class_info.id.to_string(), state);
        }
        if !visible_classes.is_empty() {
            visible_class_categories.push(ClassCategoryView {
                name: category.name.to_string(),
                classes: visible_classes,
            });
        }
    }

    let mut custom_config_states = BTreeMap::new();
    for (config_id, custom_config) in &profiles.custom_configs {
        display_names.insert(config_id.clone(), custom_config_label(custom_config));
        custom_config_states.insert(
            config_id.clone(),
            custom_config_display_state(custom_config),
        );
    }

    ProfileDisplaySnapshot {
        config_options: config_options(profiles),
        visible_class_categories,
        display_names,
        class_states,
        custom_config_states,
    }
}

fn config_options(profiles: &ProfilesConfig) -> Vec<ConfigOption> {
    let class_options = profiles
        .classes
        .iter()
        .filter(|(_, class_config)| has_class_config(class_config))
        .map(|(class_id, _)| ConfigOption {
            id: Some(class_id.clone()),
            label: class_name_by_id(class_id).unwrap_or("未知职业").to_string(),
        });
    let custom_options = profiles
        .custom_configs
        .iter()
        .filter(|(_, custom_config)| has_custom_config(custom_config))
        .map(|(config_id, custom_config)| ConfigOption {
            id: Some(config_id.clone()),
            label: custom_config_label(custom_config),
        });

    class_options.chain(custom_options).collect()
}

fn class_display_state(profiles: &ProfilesConfig, class_id: &str) -> ClassDisplayState {
    let config = profiles.classes.get(class_id);
    let has_keys = config.is_some_and(|config| !config.enabled_keys.is_empty());
    let has_combos = config.is_some_and(|config| !config.combo_defs.is_empty());
    let configured = has_keys || has_combos;
    let hidden = profiles.hidden_class_ids.iter().any(|id| id == class_id) && !configured;
    ClassDisplayState {
        configured,
        has_keys,
        has_combos,
        hidden,
        visible: configured || !hidden,
        can_hide: !configured,
    }
}

fn custom_config_display_state(config: &CustomConfig) -> CustomConfigDisplayState {
    let has_keys = !config.enabled_keys.is_empty();
    let has_combos = !config.combo_defs.is_empty();
    CustomConfigDisplayState {
        configured: has_keys || has_combos,
        has_keys,
        has_combos,
    }
}

fn custom_config_label(config: &CustomConfig) -> String {
    let label = config.name.trim();
    if label.is_empty() {
        "未命名配置".to_string()
    } else {
        label.to_string()
    }
}

fn has_class_config(config: &ClassConfig) -> bool {
    !config.enabled_keys.is_empty() || !config.combo_defs.is_empty()
}

fn has_custom_config(config: &CustomConfig) -> bool {
    !config.enabled_keys.is_empty() || !config.combo_defs.is_empty()
}

fn profile_config<'a>(
    profiles: &'a ProfilesConfig,
    profile_id: &str,
) -> Option<ProfileConfigRef<'a>> {
    if let Some(custom_config) = profiles.custom_configs.get(profile_id) {
        return Some(ProfileConfigRef::Custom(custom_config));
    }
    profiles
        .classes
        .get(profile_id)
        .map(ProfileConfigRef::Class)
}

fn dedupe_keys_prefer_last(keys: &[KeyBinding]) -> Vec<KeyBinding> {
    let mut deduped: Vec<KeyBinding> = Vec::new();
    for key in keys {
        if let Some(existing) = deduped.iter_mut().find(|candidate| candidate.vk == key.vk) {
            *existing = key.clone();
        } else {
            deduped.push(key.clone());
        }
    }
    deduped
}

#[cfg(test)]
mod tests {
    use super::{compute_effective_combos, compute_effective_keys};
    use crate::config::{
        ClassConfig, ComboAction, ComboDefinition, CustomConfig, EffectRule, FireKeyMode,
        KeyBinding, ProfilesConfig,
    };

    #[test]
    fn effective_keys_use_global_when_no_active_profile() {
        let profiles = minimal_profiles();

        assert_eq!(compute_effective_keys(&profiles)[0].vk, 0x58);
    }

    #[test]
    fn effective_keys_allow_profile_to_override_global_key() {
        let mut profiles = minimal_profiles();
        profiles.active_class_id = Some("class-a".to_string());
        profiles.classes.insert(
            "class-a".to_string(),
            ClassConfig {
                enabled_keys: vec![KeyBinding {
                    vk: 0x58,
                    interval_ms: 100,
                    mode: FireKeyMode::Hold,
                }],
                effect_rule: EffectRule::GlobalAndClass,
                combo_defs: Vec::new(),
            },
        );

        let keys = compute_effective_keys(&profiles);

        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].interval_ms, 100);
    }

    #[test]
    fn effective_combos_only_return_enabled_active_profile_combos() {
        let mut profiles = minimal_profiles();
        profiles.active_class_id = Some("custom-a".to_string());
        profiles.custom_configs.insert(
            "custom-a".to_string(),
            CustomConfig {
                name: "custom".to_string(),
                enabled_keys: Vec::new(),
                effect_rule: EffectRule::GlobalAndClass,
                combo_defs: vec![combo("enabled", true), combo("disabled", false)],
            },
        );

        let combos = compute_effective_combos(&profiles);

        assert_eq!(combos.len(), 1);
        assert_eq!(combos[0].id, "enabled");
    }

    fn minimal_profiles() -> ProfilesConfig {
        ProfilesConfig {
            global_keys: vec![KeyBinding {
                vk: 0x58,
                interval_ms: 20,
                mode: FireKeyMode::Hold,
            }],
            ..ProfilesConfig::default()
        }
    }

    fn combo(id: &str, enabled: bool) -> ComboDefinition {
        ComboDefinition {
            id: id.to_string(),
            name: id.to_string(),
            enabled,
            trigger_vk: Some(0x41),
            actions: vec![ComboAction::Tap {
                id: format!("{id}-action"),
                label: String::new(),
                vk: Some(0x5A),
                hold_ms: 30,
                wait_after_ms: 100,
            }],
        }
    }
}
