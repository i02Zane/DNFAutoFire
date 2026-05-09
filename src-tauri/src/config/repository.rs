//! 配置仓库：统一管理 settings / profiles 的读取、缓存、保存和原子写入。

use super::migration::{
    load_config_from_path, load_profiles_config_from_path, load_settings_config_from_path,
    normalize_active_config_id, normalize_profiles_config, normalize_settings_config,
};
use super::paths::*;
use super::schema::*;
use super::validation::{validate_profiles_config, validate_settings_config};
use crate::error::{AppError, AppResult};
use parking_lot::Mutex;
use serde::Serialize;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub(crate) struct ConfigRepository {
    settings_path: PathBuf,
    profiles_path: PathBuf,
    settings: Mutex<SettingsConfig>,
    profiles: Mutex<ProfilesConfig>,
}

impl ConfigRepository {
    pub(crate) fn new() -> Self {
        Self::from_path(config_path())
    }

    pub(crate) fn from_path(path: PathBuf) -> Self {
        let base_dir = if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.eq_ignore_ascii_case(LEGACY_CONFIG_FILE_NAME))
        {
            path.parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from("."))
        } else if path.is_dir() {
            path
        } else {
            path.parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from("."))
        };
        let legacy_path = base_dir.join(LEGACY_CONFIG_FILE_NAME);
        let settings_path = base_dir.join(SETTINGS_CONFIG_FILE_NAME);
        let profiles_path = base_dir.join(PROFILES_CONFIG_FILE_NAME);
        if let Err(error) = fs::create_dir_all(&base_dir) {
            tracing::warn!(path = %base_dir.display(), error = %error, "创建配置目录失败");
        }

        tracing::debug!(
            legacy_path = %legacy_path.display(),
            settings_path = %settings_path.display(),
            profiles_path = %profiles_path.display(),
            "初始化配置存储"
        );

        let legacy_config = if legacy_path.exists() {
            Some(load_config_from_path(&legacy_path))
        } else {
            None
        };

        let settings_loaded =
            load_settings_config_from_path(&settings_path, legacy_config.as_ref());
        let profiles_loaded =
            load_profiles_config_from_path(&profiles_path, legacy_config.as_ref());

        if settings_loaded.should_persist() {
            if let Err(error) = persist_json(&settings_path, &settings_loaded.value) {
                tracing::warn!(path = %settings_path.display(), error = %error, "保存 settings.json 失败");
            }
        }
        if profiles_loaded.should_persist() {
            if let Err(error) = persist_json(&profiles_path, &profiles_loaded.value) {
                tracing::warn!(path = %profiles_path.display(), error = %error, "保存 profiles.json 失败");
            }
        }

        Self {
            settings_path,
            profiles_path,
            settings: Mutex::new(settings_loaded.value),
            profiles: Mutex::new(profiles_loaded.value),
        }
    }

    pub(crate) fn settings(&self) -> SettingsConfig {
        self.settings.lock().clone()
    }

    pub(crate) fn profiles(&self) -> ProfilesConfig {
        self.profiles.lock().clone()
    }

    pub(crate) fn save_settings(&self, settings: SettingsConfig) -> AppResult<()> {
        let mut settings = settings;
        normalize_settings_config(&mut settings);
        if let Err(error) = validate_settings_config(&settings) {
            tracing::warn!(error = %error, "保存 settings.json 失败，配置校验失败");
            return Err(error);
        }

        self.persist_settings(settings)?;
        Ok(())
    }

    pub(crate) fn replace_profiles_for_import(&self, profiles: ProfilesConfig) -> AppResult<()> {
        self.persist_valid_profiles(profiles)
    }

    pub(crate) fn update_profiles<F>(&self, apply: F) -> AppResult<()>
    where
        F: FnOnce(&mut ProfilesConfig) -> AppResult<()>,
    {
        let mut profiles = self.profiles();
        apply(&mut profiles)?;
        self.persist_valid_profiles(profiles)
    }

    pub(crate) fn select_active_config(&self, active_class_id: Option<String>) -> AppResult<bool> {
        let profiles = self.profiles();
        let next_active_class_id = normalize_active_config_id(&profiles, active_class_id);
        if profiles.active_class_id == next_active_class_id {
            return Ok(false);
        }

        self.update_profiles(|profiles| {
            profiles.active_class_id = next_active_class_id;
            Ok(())
        })?;
        Ok(true)
    }

    fn persist_valid_profiles(&self, profiles: ProfilesConfig) -> AppResult<()> {
        let mut profiles = profiles;
        normalize_profiles_config(&mut profiles);
        if let Err(error) = validate_profiles_config(&profiles) {
            tracing::warn!(error = %error, "保存 profiles.json 失败，配置校验失败");
            return Err(error);
        }

        self.persist_profiles(profiles)?;
        Ok(())
    }

    fn persist_settings(&self, settings: SettingsConfig) -> AppResult<()> {
        self.ensure_parent_dir(&self.settings_path)?;
        persist_json(&self.settings_path, &settings)?;
        *self.settings.lock() = settings.clone();
        tracing::info!(
            path = %self.settings_path.display(),
            version = settings.version,
            "settings.json 已保存"
        );
        Ok(())
    }

    fn persist_profiles(&self, profiles: ProfilesConfig) -> AppResult<()> {
        self.ensure_parent_dir(&self.profiles_path)?;
        persist_json(&self.profiles_path, &profiles)?;
        *self.profiles.lock() = profiles.clone();
        tracing::info!(
            path = %self.profiles_path.display(),
            version = profiles.version,
            global_key_count = profiles.global_keys.len(),
            class_count = profiles.classes.len(),
            custom_config_count = profiles.custom_configs.len(),
            active_class_id = profiles.active_class_id.as_deref().unwrap_or("-"),
            "profiles.json 已保存"
        );
        Ok(())
    }

    fn ensure_parent_dir(&self, path: &Path) -> AppResult<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                let message = format!("创建配置目录失败: {e}");
                tracing::error!(path = %path.display(), error = %message, "创建配置目录失败");
                AppError::io(message)
            })?;
        }
        Ok(())
    }
}

fn persist_json<T: Serialize>(path: &Path, value: &T) -> AppResult<()> {
    let content = serde_json::to_string_pretty(value).map_err(|e| {
        let message = format!("序列化配置失败: {e}");
        tracing::error!(path = %path.display(), error = %message, "序列化配置失败");
        AppError::config(message)
    })?;
    write_config_atomically(path, content.as_bytes()).map_err(|e| {
        let message = format!("写入配置失败: {e}");
        tracing::error!(path = %path.display(), error = %message, "写入配置失败");
        AppError::io(message)
    })?;
    Ok(())
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
