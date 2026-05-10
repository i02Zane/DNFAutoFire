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
pub(crate) const AUTOFIRE_KEY_CANDIDATE_VKS: [u16; 95] = [
    0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4A, 0x4B, 0x4C, 0x4D, 0x4E, 0x4F, 0x50,
    0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35,
    0x36, 0x37, 0x38, 0x39, 0x09, 0x0D, 0x10, 0x11, 0x12, 0x13, 0x14, 0x1B, 0x20, 0x25, 0x26, 0x27,
    0x28, 0x2D, 0x2E, 0x60, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x6B, 0x6D,
    0x6E, 0x6F, 0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x7B, 0xA0, 0xA1,
    0xA2, 0xA3, 0xA4, 0xA5, 0xBA, 0xBB, 0xBC, 0xBD, 0xBE, 0xBF, 0xC0, 0xDB, 0xDC, 0xDD, 0xDE,
];
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
