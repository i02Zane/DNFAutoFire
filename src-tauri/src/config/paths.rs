//! 配置路径：统一定位 legacy 配置和拆分后的 settings / profiles 文件。

use std::path::PathBuf;

pub(crate) const LEGACY_CONFIG_FILE_NAME: &str = "app-config.json";
pub(crate) const SETTINGS_CONFIG_FILE_NAME: &str = "settings.json";
pub(crate) const PROFILES_CONFIG_FILE_NAME: &str = "profiles.json";

pub(crate) fn config_path() -> PathBuf {
    // 配置跟随 exe 目录，方便把整个程序目录复制到其他机器继续使用。
    config_dir().join(LEGACY_CONFIG_FILE_NAME)
}

pub(crate) fn settings_config_path() -> PathBuf {
    config_dir().join(SETTINGS_CONFIG_FILE_NAME)
}

pub(crate) fn profiles_config_path() -> PathBuf {
    config_dir().join(PROFILES_CONFIG_FILE_NAME)
}

fn config_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("configs")
}
