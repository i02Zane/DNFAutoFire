//! 后端共享状态：Tauri 命令、托盘回调和全局快捷键线程都从这里取状态。
use crate::assistant::AssistantRuntime;
use crate::config::AppConfigStore;
use crate::core::{AutoFireEngine, ComboEngine, DetectionRuntime};
use crate::hotkey::HotkeyRegistration;
use parking_lot::Mutex;
use std::sync::Arc;
use tauri::menu::MenuItem;

pub struct AppState {
    /// 连发引擎实例，所有启动/停止入口最终都操作这一份状态。
    pub(crate) engine: Arc<Mutex<AutoFireEngine>>,
    /// 助手运行时，统一收拢当前生效快照、启动/停止和失败回滚。
    pub(crate) assistant_runtime: AssistantRuntime,
    /// 职业识别运行时，按开关懒加载并复用同一份后端线程。
    pub(crate) detection_runtime: Arc<Mutex<DetectionRuntime>>,
    /// 配置唯一入口，负责持久化路径、启动读取和内存缓存。
    pub(crate) config_store: AppConfigStore,
    /// 当前注册的 Windows 全局快捷键，替换时 drop 会注销旧注册。
    pub(crate) hotkey_registration: Arc<Mutex<Option<HotkeyRegistration>>>,
    /// 托盘“当前配置”菜单项句柄，用于前端切换配置后更新文案。
    pub(crate) tray_current_config_item: Arc<Mutex<Option<MenuItem<tauri::Wry>>>>,
    pub(crate) tray_current_config_label: Arc<Mutex<String>>,
}

impl AppState {
    pub(crate) fn new() -> Self {
        let engine = Arc::new(Mutex::new(AutoFireEngine::new()));
        let combo_engine = Arc::new(Mutex::new(ComboEngine::new()));
        let assistant_runtime = AssistantRuntime::new(engine.clone(), combo_engine);
        let detection_runtime = Arc::new(Mutex::new(DetectionRuntime::new()));

        Self {
            engine,
            assistant_runtime,
            detection_runtime,
            config_store: AppConfigStore::new(),
            hotkey_registration: Arc::new(Mutex::new(None)),
            tray_current_config_item: Arc::new(Mutex::new(None)),
            tray_current_config_label: Arc::new(Mutex::new(String::new())),
        }
    }
}
