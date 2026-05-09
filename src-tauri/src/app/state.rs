//! 后端共享状态：Tauri 命令、托盘回调和全局快捷键线程都从这里取状态。
use crate::config::ConfigRepository;
use crate::engines::{AutoFireEngine, AutoRunEngine, ComboEngine};
use crate::platform::floating_control::FloatingControlRuntime;
use crate::runtime::{AssistantRuntime, RuntimeSupervisor};
use crate::vision::DetectionRuntime;
use parking_lot::Mutex;
use std::sync::Arc;
use tauri::menu::MenuItem;

pub struct AppState {
    /// 助手运行时，统一收拢当前生效快照、启动/停止和失败回滚。
    pub(crate) assistant_runtime: AssistantRuntime,
    /// 统一运行时协调器，负责把配置变更同步到检测、热键和助手引擎。
    pub(crate) runtime_supervisor: RuntimeSupervisor,
    /// 职业识别运行时，按开关懒加载并复用同一份后端线程。
    pub(crate) detection_runtime: Arc<Mutex<DetectionRuntime>>,
    /// 配置唯一入口，负责持久化路径、启动读取和内存缓存。
    pub(crate) config_store: Arc<ConfigRepository>,
    /// 托盘当前配置菜单项句柄，供后端在配置变化后直接刷新文案。
    pub(crate) tray_current_config_item: Arc<Mutex<Option<MenuItem<tauri::Wry>>>>,
}

impl AppState {
    pub(crate) fn new() -> Self {
        let engine = Arc::new(Mutex::new(AutoFireEngine::new()));
        let auto_run_runtime = Arc::new(Mutex::new(AutoRunEngine::new()));
        let combo_engine = Arc::new(Mutex::new(ComboEngine::new()));
        let config_store = Arc::new(ConfigRepository::new());
        let floating_control_runtime = Arc::new(Mutex::new(FloatingControlRuntime::new()));
        let assistant_runtime = AssistantRuntime::new(
            engine.clone(),
            combo_engine,
            auto_run_runtime.clone(),
            config_store.clone(),
        );
        let detection_runtime = Arc::new(Mutex::new(DetectionRuntime::new(config_store.clone())));
        let hotkey_registration = Arc::new(Mutex::new(None));
        let runtime_supervisor = RuntimeSupervisor::new(
            assistant_runtime.clone(),
            detection_runtime.clone(),
            floating_control_runtime.clone(),
            config_store.clone(),
            hotkey_registration.clone(),
        );

        Self {
            assistant_runtime,
            runtime_supervisor,
            detection_runtime,
            config_store,
            tray_current_config_item: Arc::new(Mutex::new(None)),
        }
    }
}
