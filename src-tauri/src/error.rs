//! 应用统一错误类型：后端内部用结构化错误，IPC 只暴露稳定 kind/message。

use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;
use thiserror::Error;
use ts_rs::TS;

pub(crate) type AppResult<T> = Result<T, AppError>;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, TS)]
#[ts(rename_all = "camelCase")]
pub(crate) enum AppErrorKind {
    Validation,
    Config,
    Io,
    Runtime,
    Engine,
    Platform,
    Window,
    Vision,
    Hotkey,
    Startup,
    Logging,
    System,
    Internal,
}

impl AppErrorKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Validation => "validation",
            Self::Config => "config",
            Self::Io => "io",
            Self::Runtime => "runtime",
            Self::Engine => "engine",
            Self::Platform => "platform",
            Self::Window => "window",
            Self::Vision => "vision",
            Self::Hotkey => "hotkey",
            Self::Startup => "startup",
            Self::Logging => "logging",
            Self::System => "system",
            Self::Internal => "internal",
        }
    }
}

impl Serialize for AppErrorKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[derive(Debug, Clone, Error, TS)]
#[error("{message}")]
pub(crate) struct AppError {
    pub kind: AppErrorKind,
    pub message: String,
}

#[allow(dead_code)]
impl AppError {
    pub(crate) fn new(kind: AppErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub(crate) fn validation(message: impl Into<String>) -> Self {
        Self::new(AppErrorKind::Validation, message)
    }

    pub(crate) fn config(message: impl Into<String>) -> Self {
        Self::new(AppErrorKind::Config, message)
    }

    pub(crate) fn io(message: impl Into<String>) -> Self {
        Self::new(AppErrorKind::Io, message)
    }

    pub(crate) fn runtime(message: impl Into<String>) -> Self {
        Self::new(AppErrorKind::Runtime, message)
    }

    pub(crate) fn engine(message: impl Into<String>) -> Self {
        Self::new(AppErrorKind::Engine, message)
    }

    pub(crate) fn platform(message: impl Into<String>) -> Self {
        Self::new(AppErrorKind::Platform, message)
    }

    pub(crate) fn window(message: impl Into<String>) -> Self {
        Self::new(AppErrorKind::Window, message)
    }

    pub(crate) fn vision(message: impl Into<String>) -> Self {
        Self::new(AppErrorKind::Vision, message)
    }

    pub(crate) fn hotkey(message: impl Into<String>) -> Self {
        Self::new(AppErrorKind::Hotkey, message)
    }

    pub(crate) fn startup(message: impl Into<String>) -> Self {
        Self::new(AppErrorKind::Startup, message)
    }

    pub(crate) fn logging(message: impl Into<String>) -> Self {
        Self::new(AppErrorKind::Logging, message)
    }

    pub(crate) fn system(message: impl Into<String>) -> Self {
        Self::new(AppErrorKind::System, message)
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("AppError", 2)?;
        state.serialize_field("kind", &self.kind)?;
        state.serialize_field("message", &self.message)?;
        state.end()
    }
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        Self::io(error.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(error: serde_json::Error) -> Self {
        Self::config(error.to_string())
    }
}
