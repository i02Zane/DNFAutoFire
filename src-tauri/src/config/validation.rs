//! 配置校验：只负责数据合法性和有效配置计算。

use super::defaults::*;
use super::schema::*;
use crate::domain::compute_effective_keys_for_profile;
use crate::error::{AppError, AppResult};
use std::collections::{BTreeMap, HashSet};

pub(crate) fn validate_settings_config(settings: &SettingsConfig) -> AppResult<()> {
    validate_detection_settings(&settings.detection)
}

pub(crate) fn validate_profiles_config(profiles: &ProfilesConfig) -> AppResult<()> {
    validate_auto_run_pulse_delay_ms(profiles.auto_run.pulse_delay_ms)?;
    validate_keys(&profiles.global_keys)?;
    for class_config in profiles.classes.values() {
        validate_keys(&class_config.enabled_keys)?;
        let effective_keys = effective_keys_for_profile(
            &profiles.global_keys,
            &class_config.enabled_keys,
            &class_config.effect_rule,
        );
        validate_combo_defs(&class_config.combo_defs, &effective_keys)?;
    }
    for custom_config in profiles.custom_configs.values() {
        if custom_config.name.trim().is_empty() {
            return Err(AppError::validation("自定义配置名称不能为空"));
        }
        validate_keys(&custom_config.enabled_keys)?;
        let effective_keys = effective_keys_for_profile(
            &profiles.global_keys,
            &custom_config.enabled_keys,
            &custom_config.effect_rule,
        );
        validate_combo_defs(&custom_config.combo_defs, &effective_keys)?;
    }
    Ok(())
}

pub(crate) fn validate_legacy_config(config: &LegacyAppConfig) -> AppResult<()> {
    validate_keys(&config.global_keys)?;
    validate_detection_settings(&config.detection)?;
    validate_app_settings(&config.settings)?;
    for class_config in config.classes.values() {
        validate_keys(&class_config.enabled_keys)?;
        let effective_keys = effective_keys_for_profile(
            &config.global_keys,
            &class_config.enabled_keys,
            &class_config.effect_rule,
        );
        validate_combo_defs(&class_config.combo_defs, &effective_keys)?;
    }
    for custom_config in config.custom_configs.values() {
        if custom_config.name.trim().is_empty() {
            return Err(AppError::validation("自定义配置名称不能为空"));
        }
        validate_keys(&custom_config.enabled_keys)?;
        let effective_keys = effective_keys_for_profile(
            &config.global_keys,
            &custom_config.enabled_keys,
            &custom_config.effect_rule,
        );
        validate_combo_defs(&custom_config.combo_defs, &effective_keys)?;
    }
    Ok(())
}

pub(crate) fn validate_app_settings(settings: &AppSettings) -> AppResult<()> {
    validate_auto_run_pulse_delay_ms(settings.auto_run_pulse_delay_ms)
}

pub(crate) fn validate_detection_settings(settings: &DetectionSettings) -> AppResult<()> {
    validate_detection_interval_ms(settings.interval_ms)
}

pub(crate) fn validate_detection_interval_ms(interval_ms: u64) -> AppResult<()> {
    if is_supported_detection_interval(interval_ms) {
        return Ok(());
    }

    Err(AppError::validation(format!(
        "职业识别间隔只能是 {} / {} / {} / {} 毫秒",
        DETECTION_INTERVAL_OPTIONS[0],
        DETECTION_INTERVAL_OPTIONS[1],
        DETECTION_INTERVAL_OPTIONS[2],
        DETECTION_INTERVAL_OPTIONS[3]
    )))
}

pub(crate) fn normalize_detection_interval_ms(value: u64) -> u64 {
    if is_supported_detection_interval(value) {
        value
    } else {
        DEFAULT_DETECTION_INTERVAL_MS
    }
}

fn is_supported_detection_interval(value: u64) -> bool {
    DETECTION_INTERVAL_OPTIONS.contains(&value)
}

pub(crate) fn validate_auto_run_pulse_delay_ms(pulse_delay_ms: u64) -> AppResult<()> {
    if is_supported_auto_run_pulse_delay(pulse_delay_ms) {
        return Ok(());
    }

    Err(AppError::validation(format!(
        "一键奔跑脉冲间隔只能是 {} / {} / {} 毫秒",
        AUTO_RUN_PULSE_DELAY_OPTIONS[0],
        AUTO_RUN_PULSE_DELAY_OPTIONS[1],
        AUTO_RUN_PULSE_DELAY_OPTIONS[2]
    )))
}

pub(crate) fn normalize_auto_run_pulse_delay_ms(value: u64) -> u64 {
    if is_supported_auto_run_pulse_delay(value) {
        value
    } else {
        DEFAULT_AUTO_RUN_PULSE_DELAY_MS
    }
}

fn is_supported_auto_run_pulse_delay(value: u64) -> bool {
    AUTO_RUN_PULSE_DELAY_OPTIONS.contains(&value)
}

pub(crate) fn validate_keys(keys: &[KeyBinding]) -> AppResult<()> {
    let mut seen = HashSet::new();
    for key in keys {
        if !(MIN_INTERVAL_MS..=MAX_INTERVAL_MS).contains(&key.interval_ms) {
            return Err(AppError::validation(format!(
                "连发间隔必须在 {MIN_INTERVAL_MS}-{MAX_INTERVAL_MS} 毫秒之间"
            )));
        }
        if !seen.insert(key.vk) {
            return Err(AppError::validation("同一配置中不能重复添加相同按键"));
        }
    }
    Ok(())
}

#[cfg(test)]
pub(crate) fn validate_runtime_profile(
    keys: &[KeyBinding],
    combos: &[ComboDefinition],
) -> AppResult<()> {
    validate_keys(keys)?;
    let effective_keys = keys.iter().map(|key| key.vk).collect();
    validate_combo_defs(combos, &effective_keys)
}

pub(crate) fn validate_combo_defs(
    combos: &[ComboDefinition],
    effective_key_vks: &HashSet<u16>,
) -> AppResult<()> {
    let mut seen_triggers = HashSet::new();
    for combo in combos {
        for action in &combo.actions {
            validate_combo_action_timing(action)?;
        }

        if !combo.enabled {
            continue;
        }

        if combo.name.trim().is_empty() {
            return Err(AppError::validation("启用的一键连招必须填写名称。"));
        }

        let trigger_vk = combo
            .trigger_vk
            .ok_or_else(|| AppError::validation("启用的一键连招必须设置触发键。"))?;
        if !seen_triggers.insert(trigger_vk) {
            return Err(AppError::validation("同一职业的一键连招触发键不能重复。"));
        }
        if effective_key_vks.contains(&trigger_vk) {
            return Err(AppError::validation(
                "一键连招触发键不能与当前生效连发键重复。",
            ));
        }
        if combo.actions.is_empty() {
            return Err(AppError::validation("启用的一键连招至少需要一个动作。"));
        }

        for action in &combo.actions {
            validate_enabled_combo_action(action)?;
        }
    }
    Ok(())
}

pub(crate) fn validate_combo_defs_for_profile(
    profiles: &ProfilesConfig,
    profile_id: &str,
    combos: &[ComboDefinition],
) -> Vec<ComboValidationIssue> {
    let effective_key_vks = compute_effective_keys_for_profile(profiles, profile_id)
        .into_iter()
        .map(|key| key.vk)
        .collect::<HashSet<_>>();
    validate_combo_defs_detailed(combos, &effective_key_vks)
}

fn validate_combo_defs_detailed(
    combos: &[ComboDefinition],
    effective_key_vks: &HashSet<u16>,
) -> Vec<ComboValidationIssue> {
    let mut issues = Vec::new();
    let mut trigger_owners: BTreeMap<u16, Vec<&ComboDefinition>> = BTreeMap::new();

    for combo in combos {
        for action in &combo.actions {
            issues.extend(validate_combo_action_timing_detailed(&combo.id, action));
        }

        if !combo.enabled {
            continue;
        }

        if combo.name.trim().is_empty() {
            issues.push(combo_issue(
                combo,
                ComboValidationField::Name,
                "启用的连招必须填写名称。",
            ));
        }

        match combo.trigger_vk {
            Some(trigger_vk) => {
                trigger_owners.entry(trigger_vk).or_default().push(combo);
                if effective_key_vks.contains(&trigger_vk) {
                    issues.push(combo_issue(
                        combo,
                        ComboValidationField::Trigger,
                        "触发键不能与当前生效连发键重复。",
                    ));
                }
            }
            None => issues.push(combo_issue(
                combo,
                ComboValidationField::Trigger,
                "启用的连招必须设置触发键。",
            )),
        }

        if combo.actions.is_empty() {
            issues.push(combo_issue(
                combo,
                ComboValidationField::Actions,
                "启用的连招至少需要一个动作。",
            ));
        }

        for action in &combo.actions {
            issues.extend(validate_enabled_combo_action_detailed(&combo.id, action));
        }
    }

    for owners in trigger_owners.values() {
        if owners.len() < 2 {
            continue;
        }
        for combo in owners {
            issues.push(combo_issue(
                combo,
                ComboValidationField::Trigger,
                "同一职业的连招触发键不能重复。",
            ));
        }
    }

    issues
}

fn combo_issue(
    combo: &ComboDefinition,
    field: ComboValidationField,
    message: &str,
) -> ComboValidationIssue {
    ComboValidationIssue {
        combo_id: combo.id.clone(),
        action_id: None,
        field,
        message: message.to_string(),
    }
}

fn combo_action_issue(
    combo_id: &str,
    action_id: &str,
    field: ComboValidationField,
    message: String,
) -> ComboValidationIssue {
    ComboValidationIssue {
        combo_id: combo_id.to_string(),
        action_id: Some(action_id.to_string()),
        field,
        message,
    }
}

fn validate_enabled_combo_action(action: &ComboAction) -> AppResult<()> {
    match action {
        ComboAction::Tap { vk, .. } => {
            if vk.is_none() {
                return Err(AppError::validation("快捷栏动作必须设置按键。"));
            }
        }
        ComboAction::Command { keys, .. } => {
            if keys.is_empty() {
                return Err(AppError::validation("手搓动作至少需要一个按键。"));
            }
            if !keys.iter().all(|vk| is_combo_command_vk(*vk)) {
                return Err(AppError::validation(
                    "手搓动作只能使用上下左右和 Z/X/C/空格。",
                ));
            }
            if keys
                .iter()
                .filter(|vk| COMBO_COMMAND_DIRECTION_VKS.contains(vk))
                .count()
                > MAX_COMBO_COMMAND_DIRECTION_KEYS
            {
                return Err(AppError::validation("手搓动作最多只能包含 4 个方向键。"));
            }
            if !keys
                .last()
                .is_some_and(|vk| COMBO_COMMAND_FINISH_VKS.contains(vk))
            {
                return Err(AppError::validation("手搓动作必须以 Z/X/C/空格结束。"));
            }
        }
    }
    Ok(())
}

fn is_combo_command_vk(vk: u16) -> bool {
    COMBO_COMMAND_DIRECTION_VKS.contains(&vk) || COMBO_COMMAND_FINISH_VKS.contains(&vk)
}

fn validate_combo_action_timing(action: &ComboAction) -> AppResult<()> {
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
                return Err(AppError::validation(format!(
                    "手搓按键间隔不能超过 {MAX_COMBO_GAP_MS} 毫秒"
                )));
            }
            validate_combo_wait(*wait_after_ms)?;
        }
    }
    Ok(())
}

fn validate_combo_action_timing_detailed(
    combo_id: &str,
    action: &ComboAction,
) -> Vec<ComboValidationIssue> {
    let mut issues = Vec::new();
    match action {
        ComboAction::Tap {
            id,
            hold_ms,
            wait_after_ms,
            ..
        } => {
            if !is_in_combo_range(*hold_ms, MIN_COMBO_HOLD_MS, MAX_COMBO_HOLD_MS) {
                issues.push(combo_action_issue(
                    combo_id,
                    id,
                    ComboValidationField::HoldMs,
                    format!("按下时长必须在 {MIN_COMBO_HOLD_MS}-{MAX_COMBO_HOLD_MS} 毫秒之间。"),
                ));
            }
            if !is_in_combo_range(*wait_after_ms, 0, MAX_COMBO_WAIT_MS) {
                issues.push(combo_action_issue(
                    combo_id,
                    id,
                    ComboValidationField::WaitAfterMs,
                    format!("动作后等待不能超过 {MAX_COMBO_WAIT_MS} 毫秒。"),
                ));
            }
        }
        ComboAction::Command {
            id,
            key_hold_ms,
            key_gap_ms,
            wait_after_ms,
            ..
        } => {
            if !is_in_combo_range(*key_hold_ms, MIN_COMBO_HOLD_MS, MAX_COMBO_HOLD_MS) {
                issues.push(combo_action_issue(
                    combo_id,
                    id,
                    ComboValidationField::KeyHoldMs,
                    format!("按下时长必须在 {MIN_COMBO_HOLD_MS}-{MAX_COMBO_HOLD_MS} 毫秒之间。"),
                ));
            }
            if *key_gap_ms > MAX_COMBO_GAP_MS {
                issues.push(combo_action_issue(
                    combo_id,
                    id,
                    ComboValidationField::KeyGapMs,
                    format!("按键间隔不能超过 {MAX_COMBO_GAP_MS} 毫秒。"),
                ));
            }
            if !is_in_combo_range(*wait_after_ms, 0, MAX_COMBO_WAIT_MS) {
                issues.push(combo_action_issue(
                    combo_id,
                    id,
                    ComboValidationField::WaitAfterMs,
                    format!("动作后等待不能超过 {MAX_COMBO_WAIT_MS} 毫秒。"),
                ));
            }
        }
    }

    issues
}

fn validate_enabled_combo_action_detailed(
    combo_id: &str,
    action: &ComboAction,
) -> Vec<ComboValidationIssue> {
    match action {
        ComboAction::Tap { id, vk, .. } => {
            if vk.is_none() {
                return vec![combo_action_issue(
                    combo_id,
                    id,
                    ComboValidationField::TapKey,
                    "快捷栏动作必须设置按键。".to_string(),
                )];
            }
            Vec::new()
        }
        ComboAction::Command { id, keys, .. } => {
            if keys.is_empty() {
                return vec![combo_action_issue(
                    combo_id,
                    id,
                    ComboValidationField::CommandKeys,
                    "手搓动作至少需要一个按键。".to_string(),
                )];
            }
            if !keys.iter().all(|vk| is_combo_command_vk(*vk)) {
                return vec![combo_action_issue(
                    combo_id,
                    id,
                    ComboValidationField::CommandKeys,
                    "手搓动作只能使用上下左右和 Z/X/C/空格。".to_string(),
                )];
            }
            if keys
                .iter()
                .filter(|vk| COMBO_COMMAND_DIRECTION_VKS.contains(vk))
                .count()
                > MAX_COMBO_COMMAND_DIRECTION_KEYS
            {
                return vec![combo_action_issue(
                    combo_id,
                    id,
                    ComboValidationField::CommandKeys,
                    "手搓动作最多只能包含 4 个方向键。".to_string(),
                )];
            }
            if !keys
                .last()
                .is_some_and(|vk| COMBO_COMMAND_FINISH_VKS.contains(vk))
            {
                return vec![combo_action_issue(
                    combo_id,
                    id,
                    ComboValidationField::CommandKeys,
                    "手搓动作必须以 Z/X/C/空格结束。".to_string(),
                )];
            }
            Vec::new()
        }
    }
}

fn is_in_combo_range(value: u16, min: u16, max: u16) -> bool {
    (min..=max).contains(&value)
}

fn validate_combo_hold(value: u16) -> AppResult<()> {
    if !(MIN_COMBO_HOLD_MS..=MAX_COMBO_HOLD_MS).contains(&value) {
        return Err(AppError::validation(format!(
            "连招按下时长必须在 {MIN_COMBO_HOLD_MS}-{MAX_COMBO_HOLD_MS} 毫秒之间"
        )));
    }
    Ok(())
}

fn validate_combo_wait(value: u16) -> AppResult<()> {
    if value > MAX_COMBO_WAIT_MS {
        return Err(AppError::validation(format!(
            "动作后等待不能超过 {MAX_COMBO_WAIT_MS} 毫秒"
        )));
    }
    Ok(())
}

fn effective_keys_for_profile(
    global_keys: &[KeyBinding],
    enabled_keys: &[KeyBinding],
    effect_rule: &EffectRule,
) -> HashSet<u16> {
    let mut effective_keys = HashSet::new();
    if *effect_rule == EffectRule::GlobalAndClass {
        effective_keys.extend(global_keys.iter().map(|key| key.vk));
    }
    effective_keys.extend(enabled_keys.iter().map(|key| key.vk));
    effective_keys
}
