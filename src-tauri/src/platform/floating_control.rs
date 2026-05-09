//! 悬浮窗运行时：后端统一负责创建、显示、隐藏和运行态快照。

use crate::app::events::APP_NAME;
use crate::config::WindowPosition;
use crate::error::{AppError, AppResult};
use crate::FLOATING_CONTROL_WINDOW_LABEL;
use std::sync::mpsc;
use tauri::{AppHandle, Manager, PhysicalPosition, WebviewUrl, WebviewWindowBuilder};

const FLOATING_CONTROL_VIEW_PATH: &str = "index.html?view=floating-control";
const FLOATING_CONTROL_TITLE_SUFFIX: &str = "悬浮窗";
const FLOATING_CONTROL_INITIAL_WIDTH: f64 = 260.0;
const FLOATING_CONTROL_INITIAL_HEIGHT: f64 = 58.0;
const FLOATING_CONTROL_MARGIN: f64 = 18.0;

#[derive(Debug, Default)]
pub(crate) struct FloatingControlRuntime {
    visible: bool,
}

impl FloatingControlRuntime {
    pub(crate) fn new() -> Self {
        Self { visible: false }
    }

    pub(crate) fn is_visible(&self) -> bool {
        self.visible
    }

    pub(crate) fn set_visible(
        &mut self,
        app: &AppHandle,
        visible: bool,
        position: Option<WindowPosition>,
    ) -> AppResult<()> {
        if visible {
            self.show(app, position)
        } else {
            self.hide(app)
        }
    }

    pub(crate) fn toggle(
        &mut self,
        app: &AppHandle,
        position: Option<WindowPosition>,
    ) -> AppResult<()> {
        let next_visible = !self.visible;
        self.set_visible(app, next_visible, position)
    }

    fn show(&mut self, app: &AppHandle, position: Option<WindowPosition>) -> AppResult<()> {
        if let Some(window) = app.get_webview_window(FLOATING_CONTROL_WINDOW_LABEL) {
            if let Some(position) = position {
                window
                    .set_position(PhysicalPosition::new(position.x, position.y))
                    .map_err(map_window_error)?;
            }
            window.show().map_err(map_window_error)?;
            window.set_focus().map_err(map_window_error)?;
            self.visible = true;
            return Ok(());
        }

        create_floating_control_window(app, position)?;
        self.visible = true;
        Ok(())
    }

    fn hide(&mut self, app: &AppHandle) -> AppResult<()> {
        if let Some(window) = app.get_webview_window(FLOATING_CONTROL_WINDOW_LABEL) {
            window.hide().map_err(map_window_error)?;
        }
        self.visible = false;
        Ok(())
    }
}

fn create_floating_control_window(
    app: &AppHandle,
    position: Option<WindowPosition>,
) -> AppResult<()> {
    let app_handle = app.clone();
    let (sender, receiver) = mpsc::channel();

    std::thread::spawn(move || {
        let result = (|| -> AppResult<()> {
            let monitor = app_handle
                .primary_monitor()
                .map_err(map_window_error)?
                .ok_or_else(|| AppError::window("获取主显示器失败"))?;
            let scale_factor = monitor.scale_factor();
            let width = FLOATING_CONTROL_INITIAL_WIDTH;
            let height = FLOATING_CONTROL_INITIAL_HEIGHT;
            let work_area = monitor.work_area();
            let (x, y) = position
                .map(|position| {
                    (
                        position.x as f64 / scale_factor,
                        position.y as f64 / scale_factor,
                    )
                })
                .unwrap_or_else(|| {
                    (
                        (work_area.position.x as f64 / scale_factor)
                            + (work_area.size.width as f64 / scale_factor)
                            - width
                            - (FLOATING_CONTROL_MARGIN / scale_factor),
                        (work_area.position.y as f64 / scale_factor)
                            + (work_area.size.height as f64 / scale_factor)
                            - height
                            - (FLOATING_CONTROL_MARGIN / scale_factor),
                    )
                });

            let window = WebviewWindowBuilder::new(
                &app_handle,
                FLOATING_CONTROL_WINDOW_LABEL,
                WebviewUrl::App(FLOATING_CONTROL_VIEW_PATH.into()),
            )
            .title(format!("{APP_NAME}{FLOATING_CONTROL_TITLE_SUFFIX}"))
            .position(x.max(0.0), y.max(0.0))
            .inner_size(width, height)
            .resizable(false)
            .decorations(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .visible(false)
            .build()
            .map_err(map_window_error)?;

            window.show().map_err(map_window_error)?;
            window.set_focus().map_err(map_window_error)?;
            Ok(())
        })();

        let _ = sender.send(result);
    });

    receiver
        .recv()
        .map_err(|_| AppError::window("创建悬浮窗线程失败"))?
}

fn map_window_error(error: impl std::fmt::Display) -> AppError {
    AppError::window(error.to_string())
}
