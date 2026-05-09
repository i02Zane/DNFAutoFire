//! 系统托盘：展示当前配置，并直接驱动后端运行态切换。

use crate::app::events::RUNTIME_STATE_CHANGED_EVENT;
use crate::app::state::AppState;
use crate::domain::classes::class_name_by_id;
use crate::APP_NAME;
use tauri::menu::{MenuBuilder, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Listener, Manager};

const TRAY_CURRENT_CONFIG_PREFIX: &str = "配置：";
const TRAY_CURRENT_CONFIG_EMPTY: &str = "-";
const TRAY_SHOW_MAIN_TEXT: &str = "打开主界面";
const TRAY_OPEN_FLOATING_CONTROL_TEXT: &str = "打开悬浮窗";
const TRAY_CLOSE_FLOATING_CONTROL_TEXT: &str = "关闭悬浮窗";
const TRAY_QUIT_TEXT: &str = "退出";

pub(crate) fn create_tray_icon(app: &tauri::App) -> tauri::Result<()> {
    let state = app.state::<AppState>();
    let current_config_item = MenuItem::with_id(
        app,
        "tray_current_config",
        tray_current_config_menu_text(&current_config_label(&state)),
        false,
        None::<&str>,
    )?;
    *state.tray_current_config_item.lock() = Some(current_config_item.clone());

    let control_item = MenuItem::with_id(
        app,
        "tray_toggle_control",
        floating_control_menu_text(state.runtime_supervisor.snapshot().floating_control_visible),
        true,
        None::<&str>,
    )?;
    let control_item_for_tray = control_item.clone();
    let control_item_for_runtime = control_item.clone();
    // 悬浮窗可见性由后端运行态统一驱动，托盘只根据运行态快照刷新菜单文字。
    app.listen(RUNTIME_STATE_CHANGED_EVENT, move |event| {
        if let Some(visible) = runtime_floating_control_visible_from_payload(event.payload()) {
            let _ = control_item_for_runtime.set_text(floating_control_menu_text(visible));
        } else {
            tracing::warn!(payload = %event.payload(), "解析运行态事件失败");
        }
    });
    let menu = MenuBuilder::new(app)
        .item(&current_config_item)
        .separator()
        .text("tray_show_main", TRAY_SHOW_MAIN_TEXT)
        .separator()
        .item(&control_item)
        .separator()
        .text("tray_quit", TRAY_QUIT_TEXT)
        .build()?;

    let mut tray_builder = TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip(APP_NAME)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "tray_show_main" => show_main_window(app),
            "tray_toggle_control" => request_toggle_floating_control(app, &control_item),
            "tray_quit" => {
                tracing::info!("用户通过托盘退出应用");
                app.exit(0)
            }
            _ => {}
        })
        .on_tray_icon_event(move |tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                tracing::debug!("点击托盘图标打开主界面");
                show_main_window(tray.app_handle());
            }
            update_floating_control_menu_item(tray.app_handle(), &control_item_for_tray);
        });

    if let Some(icon) = app.default_window_icon() {
        tray_builder = tray_builder.icon(icon.clone());
    }

    tray_builder.build(app)?;
    tracing::info!("托盘图标已创建");
    Ok(())
}

pub(crate) fn update_tray_current_config_item(app: &tauri::AppHandle) {
    let state = app.state::<AppState>();
    let label = current_config_label(&state);
    tracing::debug!(label = %label, "更新托盘当前配置");

    let menu_item = state.tray_current_config_item.lock().as_ref().cloned();
    if let Some(menu_item) = menu_item {
        let _ = menu_item.set_text(tray_current_config_menu_text(&label));
    }
}

fn tray_current_config_menu_text(label: &str) -> String {
    if label.is_empty() {
        return format!("{TRAY_CURRENT_CONFIG_PREFIX}{TRAY_CURRENT_CONFIG_EMPTY}");
    }
    format!("{TRAY_CURRENT_CONFIG_PREFIX}{label}")
}

fn request_toggle_floating_control(app: &tauri::AppHandle, menu_item: &MenuItem<tauri::Wry>) {
    tracing::info!("托盘请求切换悬浮窗");
    let state = app.state::<AppState>();
    if let Err(error) = state
        .runtime_supervisor
        .toggle_floating_control_visible(app)
    {
        tracing::warn!(error = %error, "切换悬浮窗失败");
        return;
    }
    update_floating_control_menu_item(app, menu_item);
}

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        tracing::info!("从托盘打开主界面");
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn update_floating_control_menu_item(app: &tauri::AppHandle, menu_item: &MenuItem<tauri::Wry>) {
    let state = app.state::<AppState>();
    let _ = menu_item.set_text(floating_control_menu_text(
        state.runtime_supervisor.snapshot().floating_control_visible,
    ));
}

fn floating_control_menu_text(visible: bool) -> &'static str {
    if visible {
        TRAY_CLOSE_FLOATING_CONTROL_TEXT
    } else {
        TRAY_OPEN_FLOATING_CONTROL_TEXT
    }
}

fn runtime_floating_control_visible_from_payload(payload: &str) -> Option<bool> {
    serde_json::from_str::<serde_json::Value>(payload)
        .ok()
        .and_then(|value| {
            value
                .get("floatingControlVisible")
                .and_then(|visible| visible.as_bool())
        })
}

fn current_config_label(state: &AppState) -> String {
    let profiles = state.config_store.profiles();
    let Some(active_class_id) = profiles.active_class_id.as_deref() else {
        return "全局配置".to_string();
    };

    if let Some(custom_config) = profiles.custom_configs.get(active_class_id) {
        let label = custom_config.name.trim();
        return if label.is_empty() {
            "未命名配置".to_string()
        } else {
            label.to_string()
        };
    }

    class_name_by_id(active_class_id)
        .map(str::to_string)
        .unwrap_or_else(|| "未知职业".to_string())
}
