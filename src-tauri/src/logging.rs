//! 统一日志初始化：按会话写入安装目录下的 logs，并在启动时清理过期文件。

use chrono::Local;
use once_cell::sync::OnceCell;
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, prelude::*, reload, EnvFilter};

use crate::config::{config_path, read_log_level_setting, LogLevelSetting};

const LOG_DIR_NAME: &str = "logs";
const LOG_RETENTION_DAYS: u64 = 7;
const SESSION_TIME_FORMAT: &str = "%Y%m%d-%H%M%S";

static WORKER_GUARD: OnceCell<WorkerGuard> = OnceCell::new();
static PANIC_HOOK_INSTALLED: OnceCell<()> = OnceCell::new();
static FILTER_HANDLE: OnceCell<reload::Handle<EnvFilter, tracing_subscriber::Registry>> =
    OnceCell::new();

#[derive(Debug, Clone)]
pub(crate) struct LoggingState {
    pub(crate) log_dir: PathBuf,
    pub(crate) log_file: Option<PathBuf>,
    pub(crate) build_mode: &'static str,
    pub(crate) log_level: LogLevelSetting,
}

pub(crate) fn initialize() -> LoggingState {
    let build_mode = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    let log_level = read_log_level_setting(&config_path());
    let install_dir = application_dir();
    let log_dir = install_dir.join(LOG_DIR_NAME);

    if let Err(error) = fs::create_dir_all(&log_dir) {
        let reason = format!("创建日志目录失败: {error}");
        install_stderr_subscriber(log_level);
        tracing::warn!(
            log_dir = %log_dir.display(),
            error = %reason,
            "文件日志初始化失败，已切换为标准错误输出"
        );
        install_panic_hook();
        return LoggingState {
            log_dir,
            log_file: None,
            build_mode,
            log_level,
        };
    }

    clean_expired_logs(
        &log_dir,
        Duration::from_secs(LOG_RETENTION_DAYS * 24 * 60 * 60),
    );

    let session_file = session_log_file_name(build_mode);
    let log_file_path = log_dir.join(session_file);
    match initialize_file_subscriber(&log_file_path, log_level) {
        Ok(()) => {
            install_panic_hook();
            LoggingState {
                log_dir,
                log_file: Some(log_file_path),
                build_mode,
                log_level,
            }
        }
        Err(error) => {
            let reason = format!("文件日志初始化失败: {error}");
            install_stderr_subscriber(log_level);
            tracing::warn!(
                log_dir = %log_dir.display(),
                log_file = %log_file_path.display(),
                error = %reason,
                "文件日志初始化失败，已切换为标准错误输出"
            );
            install_panic_hook();
            LoggingState {
                log_dir,
                log_file: None,
                build_mode,
                log_level,
            }
        }
    }
}

pub(crate) fn update_log_level(log_level: LogLevelSetting) -> Result<(), String> {
    let filter = build_filter_for_level(log_level);
    let handle = FILTER_HANDLE
        .get()
        .ok_or_else(|| "日志系统尚未初始化".to_string())?;
    handle
        .modify(|current| {
            *current = filter;
        })
        .map_err(|error| format!("更新日志等级失败: {error}"))
}

pub(crate) fn format_hotkey(ctrl: bool, alt: bool, shift: bool, vk: u16) -> String {
    let mut parts = Vec::new();
    if ctrl {
        parts.push("Ctrl".to_string());
    }
    if alt {
        parts.push("Alt".to_string());
    }
    if shift {
        parts.push("Shift".to_string());
    }
    parts.push(format_vk(vk));
    parts.join(" + ")
}

pub(crate) fn format_vk(vk: u16) -> String {
    format!("VK 0x{vk:02X}")
}

fn application_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn session_log_file_name(build_mode: &str) -> String {
    let timestamp = Local::now().format(SESSION_TIME_FORMAT).to_string();
    format!(
        "{}-{build_mode}-{timestamp}-pid{}.log",
        env!("CARGO_PKG_NAME"),
        std::process::id()
    )
}

fn initialize_file_subscriber(
    log_file_path: &Path,
    log_level: LogLevelSetting,
) -> Result<(), String> {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file_path)
        .map_err(|error| format!("打开日志文件失败: {error}"))?;
    let (non_blocking, worker_guard) = tracing_appender::non_blocking(file);
    let (filter, filter_handle) = reload::Layer::new(build_env_filter(log_level));
    let include_source_metadata = cfg!(debug_assertions);
    let formatter = fmt::layer()
        .compact()
        .with_ansi(false)
        .with_timer(fmt::time::ChronoLocal::new(
            "%Y-%m-%d %H:%M:%S%.3f".to_string(),
        ))
        .with_file(include_source_metadata)
        .with_line_number(include_source_metadata)
        .with_target(include_source_metadata)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_writer(non_blocking);

    tracing_subscriber::registry()
        .with(filter)
        .with(formatter)
        .try_init()
        .map_err(|error| format!("注册日志订阅器失败: {error}"))?;

    let _ = WORKER_GUARD.set(worker_guard);
    let _ = FILTER_HANDLE.set(filter_handle);
    Ok(())
}

fn install_stderr_subscriber(log_level: LogLevelSetting) {
    let (filter, filter_handle) = reload::Layer::new(build_env_filter(log_level));
    let include_source_metadata = cfg!(debug_assertions);
    let formatter = fmt::layer()
        .compact()
        .with_ansi(false)
        .with_timer(fmt::time::ChronoLocal::new(
            "%Y-%m-%d %H:%M:%S%.3f".to_string(),
        ))
        .with_file(include_source_metadata)
        .with_line_number(include_source_metadata)
        .with_target(include_source_metadata)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_writer(std::io::stderr);

    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(formatter)
        .try_init();
    let _ = FILTER_HANDLE.set(filter_handle);
}

fn build_env_filter(log_level: LogLevelSetting) -> EnvFilter {
    for key in ["DNF_AUTO_FIRE_LOG", "RUST_LOG"] {
        if let Some(filter) = read_env_filter(key) {
            return filter;
        }
    }

    build_filter_for_level(log_level)
}

fn build_filter_for_level(log_level: LogLevelSetting) -> EnvFilter {
    EnvFilter::new(log_level.to_string())
}

fn read_env_filter(key: &str) -> Option<EnvFilter> {
    let raw = std::env::var(key).ok()?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    EnvFilter::try_new(trimmed).ok()
}

fn clean_expired_logs(log_dir: &Path, retention: Duration) {
    let now = SystemTime::now();
    let Ok(entries) = fs::read_dir(log_dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("log") {
            continue;
        }

        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        let Ok(modified) = metadata.modified() else {
            continue;
        };

        let Ok(age) = now.duration_since(modified) else {
            continue;
        };

        if age > retention {
            let _ = fs::remove_file(path);
        }
    }
}

fn install_panic_hook() {
    if PANIC_HOOK_INSTALLED.set(()).is_err() {
        return;
    }

    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let location = panic_info
            .location()
            .map(|location| format!("{}:{}", location.file(), location.line()))
            .unwrap_or_else(|| "unknown".to_string());
        let payload = panic_payload_message(panic_info);
        tracing::error!(
            target: "panic",
            location = %location,
            payload = %payload,
            "程序发生未捕获 panic"
        );
        previous_hook(panic_info);
    }));
}

fn panic_payload_message(panic_info: &std::panic::PanicHookInfo<'_>) -> String {
    if let Some(message) = panic_info.payload().downcast_ref::<&str>() {
        return (*message).to_string();
    }
    if let Some(message) = panic_info.payload().downcast_ref::<String>() {
        return message.clone();
    }

    "未知 panic payload".to_string()
}

#[cfg(test)]
mod tests {
    use super::{build_filter_for_level, clean_expired_logs, session_log_file_name};
    use crate::config::LogLevelSetting;
    use std::fs;
    use std::path::PathBuf;
    use std::time::Duration;

    #[test]
    fn session_log_file_name_includes_build_mode_and_pid() {
        let file_name = session_log_file_name("debug");

        assert!(file_name.starts_with(env!("CARGO_PKG_NAME")));
        assert!(file_name.contains("-debug-"));
        assert!(file_name.contains("-pid"));
        assert!(file_name.ends_with(".log"));
    }

    #[test]
    fn clean_expired_logs_removes_old_log_files() {
        let dir = unique_temp_dir("log-retention");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("old.log"), "old").unwrap();
        std::thread::sleep(Duration::from_millis(1200));
        fs::write(dir.join("fresh.log"), "fresh").unwrap();

        clean_expired_logs(&dir, Duration::from_millis(500));

        assert!(!dir.join("old.log").exists());
        assert!(dir.join("fresh.log").exists());

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn clean_expired_logs_keeps_non_log_files() {
        let dir = unique_temp_dir("log-retention-non-log");
        fs::create_dir_all(&dir).unwrap();
        let file_path = dir.join("notes.txt");
        fs::write(&file_path, "keep").unwrap();
        std::thread::sleep(Duration::from_millis(1200));

        clean_expired_logs(&dir, Duration::from_millis(500));

        assert!(file_path.exists());

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn build_filter_for_level_uses_selected_level() {
        assert_eq!(
            build_filter_for_level(LogLevelSetting::Off).to_string(),
            "off"
        );
        assert_eq!(
            build_filter_for_level(LogLevelSetting::Debug).to_string(),
            "debug"
        );
    }

    fn unique_temp_dir(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        let unique_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!(
            "dnfautofire-{name}-{}-{unique_id}",
            std::process::id()
        ));
        path
    }
}
