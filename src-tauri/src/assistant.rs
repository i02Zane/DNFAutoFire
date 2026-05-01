//! 助手运行时：统一管理当前生效快照、启动/停止、热键切换和失败回滚。

use crate::config::{ComboDefinition, KeyBinding};
use crate::core::{AutoFireEngine, ComboEngine, FireKeyConfig};
use parking_lot::Mutex;
use std::sync::Arc;

pub(crate) const EMPTY_ASSISTANT_PROFILE_ERROR: &str = "请至少配置一个连发按键或一键连招";

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

#[derive(Clone)]
pub(crate) struct AssistantRuntime {
    engine: Arc<Mutex<AutoFireEngine>>,
    combo_engine: Arc<Mutex<ComboEngine>>,
    profile: Arc<Mutex<AssistantProfile>>,
}

impl AssistantRuntime {
    pub(crate) fn new(
        engine: Arc<Mutex<AutoFireEngine>>,
        combo_engine: Arc<Mutex<ComboEngine>>,
    ) -> Self {
        Self {
            engine,
            combo_engine,
            profile: Arc::new(Mutex::new(AssistantProfile::default())),
        }
    }

    pub(crate) fn set_runtime_keys(&self, keys: Vec<KeyBinding>) {
        tracing::debug!(key_count = keys.len(), "已更新运行时连发按键快照");
        self.profile.lock().keys = keys;
    }

    pub(crate) fn set_runtime_profile(&self, profile: AssistantProfile) -> Result<(), String> {
        tracing::debug!(
            key_count = profile.keys.len(),
            combo_count = profile.combos.len(),
            "已更新运行时快照"
        );
        self.store_runtime_profile(profile.clone());

        if self.is_running() {
            tracing::info!(
                key_count = profile.keys.len(),
                combo_count = profile.combos.len(),
                "运行中刷新运行时快照"
            );
            self.apply_profile(profile)?;
        }
        Ok(())
    }

    pub(crate) fn start_with_profile(&self, profile: AssistantProfile) -> Result<(), String> {
        if profile.is_empty() {
            tracing::warn!("尝试启动助手，但当前运行时快照为空");
            self.stop();
            return Err(EMPTY_ASSISTANT_PROFILE_ERROR.to_string());
        }

        tracing::info!(
            key_count = profile.keys.len(),
            combo_count = profile.combos.len(),
            "启动助手"
        );
        self.store_runtime_profile(profile.clone());
        self.apply_profile(profile)
    }

    pub(crate) fn toggle_from_runtime_profile(&self) -> Result<(), String> {
        if self.is_running() {
            tracing::info!("收到全局快捷键，准备停止助手");
            self.stop();
            return Ok(());
        }

        let profile = self.profile.lock().clone();
        tracing::info!(
            key_count = profile.keys.len(),
            combo_count = profile.combos.len(),
            "收到全局快捷键，准备启动助手"
        );
        self.start_with_profile(profile)
    }

    pub(crate) fn stop(&self) {
        let was_running = self.is_running();
        if was_running {
            tracing::info!("停止助手");
        } else {
            tracing::debug!("助手已经处于停止状态");
        }
        self.engine.lock().stop();
        self.combo_engine.lock().stop();
    }

    pub(crate) fn is_running(&self) -> bool {
        self.engine.lock().is_running() || self.combo_engine.lock().is_running()
    }

    fn store_runtime_profile(&self, profile: AssistantProfile) {
        *self.profile.lock() = profile;
    }

    fn apply_profile(&self, profile: AssistantProfile) -> Result<(), String> {
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
}

fn to_fire_key_configs(keys: Vec<KeyBinding>) -> Vec<FireKeyConfig> {
    keys.into_iter()
        .map(|key| FireKeyConfig {
            vk: key.vk,
            interval_ms: key.interval_ms,
        })
        .collect()
}
