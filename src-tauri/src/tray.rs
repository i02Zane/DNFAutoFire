//! 系统托盘：展示当前配置，并把悬浮窗开关请求交回主窗口处理。

use crate::state::AppState;
use crate::{
    APP_NAME, FLOATING_CONTROL_TOGGLE_REQUEST_EVENT, FLOATING_CONTROL_VISIBILITY_EVENT,
    FLOATING_CONTROL_WINDOW_LABEL,
};
use tauri::menu::{MenuBuilder, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Emitter, Listener, Manager};

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
        tray_current_config_menu_text(&state.tray_current_config_label.lock()),
        false,
        None::<&str>,
    )?;
    *state.tray_current_config_item.lock() = Some(current_config_item.clone());

    let control_item = MenuItem::with_id(
        app,
        "tray_toggle_control",
        floating_control_menu_text(app.handle()),
        true,
        None::<&str>,
    )?;
    let control_item_for_menu = control_item.clone();
    let control_item_for_tray = control_item.clone();
    let control_item_for_visibility = control_item.clone();
    // 悬浮窗真实可见性由前端广播，托盘只根据广播刷新菜单文字。
    app.listen(FLOATING_CONTROL_VISIBILITY_EVENT, move |event| {
        match floating_control_visible_from_payload(event.payload()) {
            Some(true) => {
                tracing::info!(visible = true, "悬浮窗可见性已同步");
                let _ = control_item_for_visibility.set_text(TRAY_CLOSE_FLOATING_CONTROL_TEXT);
            }
            Some(false) => {
                tracing::info!(visible = false, "悬浮窗可见性已同步");
                let _ = control_item_for_visibility.set_text(TRAY_OPEN_FLOATING_CONTROL_TEXT);
            }
            None => {
                tracing::warn!(
                    payload = %event.payload(),
                    "解析悬浮窗可见性事件失败"
                );
            }
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
            "tray_toggle_control" => request_toggle_floating_control(app, &control_item_for_menu),
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

pub(crate) fn update_tray_current_config_item(state: &AppState) {
    if let Some(menu_item) = state.tray_current_config_item.lock().as_ref() {
        let label = state.tray_current_config_label.lock().clone();
        tracing::debug!(
            label = %label,
            "更新托盘当前配置"
        );
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
    update_floating_control_menu_item(app, menu_item);
    tracing::info!("托盘请求切换悬浮窗");
    // 不在 Rust 侧创建/关闭窗口，避免和前端唯一窗口链路分叉。
    let _ = app.emit(FLOATING_CONTROL_TOGGLE_REQUEST_EVENT, ());
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
    let _ = menu_item.set_text(floating_control_menu_text(app));
}

fn floating_control_menu_text(app: &tauri::AppHandle) -> &'static str {
    if floating_control_is_visible(app) {
        TRAY_CLOSE_FLOATING_CONTROL_TEXT
    } else {
        TRAY_OPEN_FLOATING_CONTROL_TEXT
    }
}

fn floating_control_is_visible(app: &tauri::AppHandle) -> bool {
    app.get_webview_window(FLOATING_CONTROL_WINDOW_LABEL)
        .and_then(|window| window.is_visible().ok())
        .unwrap_or(false)
}

fn floating_control_visible_from_payload(payload: &str) -> Option<bool> {
    serde_json::from_str::<serde_json::Value>(payload)
        .ok()
        .and_then(|value| value.get("visible").and_then(|visible| visible.as_bool()))
}
