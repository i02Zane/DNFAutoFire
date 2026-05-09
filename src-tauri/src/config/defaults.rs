//! 默认值常量和默认函数。

pub(crate) const CONFIG_VERSION: u32 = 11;
pub(crate) const SETTINGS_CONFIG_VERSION: u32 = 1;
pub(crate) const PROFILES_CONFIG_VERSION: u32 = 1;
pub(crate) const DEFAULT_INTERVAL_MS: u16 = 20;
pub(crate) const DEFAULT_DETECTION_INTERVAL_MS: u64 = 200;
pub(crate) const DEFAULT_AUTO_RUN_PULSE_DELAY_MS: u64 = 25;
pub(crate) const MIN_INTERVAL_MS: u16 = 10;
pub(crate) const MAX_INTERVAL_MS: u16 = 1000;
pub(crate) const DETECTION_INTERVAL_OPTIONS: [u64; 4] = [100, 200, 500, 1000];
pub(crate) const AUTO_RUN_PULSE_DELAY_OPTIONS: [u64; 3] = [10, 25, 50];
pub(crate) const MIN_COMBO_HOLD_MS: u16 = 10;
pub(crate) const MAX_COMBO_HOLD_MS: u16 = 1000;
pub(crate) const MAX_COMBO_GAP_MS: u16 = 1000;
pub(crate) const MAX_COMBO_WAIT_MS: u16 = 5000;
pub(crate) const MAX_COMBO_COMMAND_DIRECTION_KEYS: usize = 4;
pub(crate) const COMBO_COMMAND_DIRECTION_VKS: [u16; 4] = [0x25, 0x26, 0x27, 0x28];
pub(crate) const COMBO_COMMAND_FINISH_VKS: [u16; 4] = [0x5A, 0x58, 0x43, 0x20];

pub(crate) fn default_combo_hold_ms() -> u16 {
    30
}

pub(crate) fn default_combo_gap_ms() -> u16 {
    20
}

pub(crate) fn default_combo_wait_after_ms() -> u16 {
    100
}

pub(crate) fn default_settings_config_version() -> u32 {
    SETTINGS_CONFIG_VERSION
}

pub(crate) fn default_profiles_config_version() -> u32 {
    PROFILES_CONFIG_VERSION
}

pub(crate) fn default_detection_enabled() -> bool {
    false
}

pub(crate) fn default_detection_interval_ms() -> u64 {
    DEFAULT_DETECTION_INTERVAL_MS
}

pub(crate) fn default_auto_run_left_vk() -> u16 {
    0x25
}

pub(crate) fn default_auto_run_right_vk() -> u16 {
    0x27
}

pub(crate) fn default_auto_run_pulse_delay_ms() -> u64 {
    DEFAULT_AUTO_RUN_PULSE_DELAY_MS
}
