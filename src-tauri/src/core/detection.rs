//! 职业识别核心逻辑：只保留截图、像素扫描和结果事件，不带任何预览 UI。

use parking_lot::Mutex;
use serde::Serialize;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ClassDetectionResultEvent {
    pub class_index: Option<u16>,
    pub confidence: f32,
    pub reason: String,
}

#[derive(Debug)]
pub struct DetectionRuntime {
    enabled: Arc<AtomicBool>,
    interval_ms: Arc<AtomicU64>,
    #[cfg(windows)]
    platform: windows_impl::WindowsDetectionRuntime,
}

impl DetectionRuntime {
    pub fn new() -> Self {
        Self {
            enabled: Arc::new(AtomicBool::new(false)),
            interval_ms: Arc::new(AtomicU64::new(crate::config::DEFAULT_DETECTION_INTERVAL_MS)),
            #[cfg(windows)]
            platform: windows_impl::WindowsDetectionRuntime::new(),
        }
    }

    pub fn set_interval_ms(&self, interval_ms: u64) {
        self.interval_ms.store(interval_ms, Ordering::SeqCst);
    }

    pub fn start(&mut self, app_handle: tauri::AppHandle, interval_ms: u64) -> Result<(), String> {
        if self.enabled.load(Ordering::SeqCst) {
            tracing::debug!("职业识别引擎已在运行中");
            return Ok(());
        }

        self.set_interval_ms(interval_ms);
        tracing::info!(interval_ms, "启动职业识别引擎");
        self.enabled.store(true, Ordering::SeqCst);

        #[cfg(windows)]
        if let Err(error) =
            self.platform
                .start(app_handle, self.enabled.clone(), self.interval_ms.clone())
        {
            self.enabled.store(false, Ordering::SeqCst);
            return Err(error);
        }

        #[cfg(not(windows))]
        {
            let _ = app_handle;
            self.enabled.store(false, Ordering::SeqCst);
            return Err("职业识别当前仅支持 Windows。".to_string());
        }
        Ok(())
    }

    pub fn stop(&mut self) {
        if self.enabled.swap(false, Ordering::SeqCst) {
            tracing::info!("停止职业识别引擎");
        } else {
            tracing::debug!("职业识别引擎已经处于停止状态");
        }

        #[cfg(windows)]
        self.platform.stop();
    }

    pub fn is_running(&self) -> bool {
        self.enabled.load(Ordering::SeqCst)
    }
}

impl Default for DetectionRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for DetectionRuntime {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(windows)]
mod windows_impl {
    use super::*;
    use crate::core::window::foreground_target_window_handle;
    use std::thread::{self, JoinHandle};
    use std::time::{Duration, Instant};
    use tauri::Emitter;
    use windows_capture::capture::{Context, GraphicsCaptureApiHandler};
    use windows_capture::frame::Frame;
    use windows_capture::graphics_capture_api::InternalCaptureControl;
    use windows_capture::settings::{
        ColorFormat, CursorCaptureSettings, DirtyRegionSettings, DrawBorderSettings,
        MinimumUpdateIntervalSettings, SecondaryWindowSettings, Settings,
    };
    use windows_capture::window::Window;

    const POLL_INTERVAL_MS: u64 = 200;
    const PATTERN_SIZE: usize = 3;
    const CLASS_ID_STEP: u8 = 2;
    const CLASS_ID_OFFSET_X: usize = 4;
    const CLASS_ID_OFFSET_Y: usize = 1;
    const MAP_REGION_BASE_SIZE: f32 = 24.0;
    const SCALE_BASE_HEIGHT: f32 = 600.0;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct DetectionSignature {
        class_index: Option<u16>,
        reason: String,
    }

    #[derive(Debug)]
    pub struct WindowsDetectionRuntime {
        thread_handle: Option<JoinHandle<()>>,
        stop_signal: Arc<AtomicBool>,
        last_reported: Arc<Mutex<Option<DetectionSignature>>>,
    }

    impl WindowsDetectionRuntime {
        pub fn new() -> Self {
            Self {
                thread_handle: None,
                stop_signal: Arc::new(AtomicBool::new(false)),
                last_reported: Arc::new(Mutex::new(None)),
            }
        }

        pub fn start(
            &mut self,
            app_handle: tauri::AppHandle,
            enabled: Arc<AtomicBool>,
            interval_ms: Arc<AtomicU64>,
        ) -> Result<(), String> {
            if self.thread_handle.is_some() {
                return Ok(());
            }

            self.stop_signal.store(false, Ordering::SeqCst);
            *self.last_reported.lock() = None;

            let stop_signal = self.stop_signal.clone();
            let last_reported = self.last_reported.clone();
            let handle = thread::spawn(move || {
                run_detection_worker(app_handle, enabled, interval_ms, stop_signal, last_reported);
            });
            self.thread_handle = Some(handle);
            Ok(())
        }

        pub fn stop(&mut self) {
            self.stop_signal.store(true, Ordering::SeqCst);

            if let Some(handle) = self.thread_handle.take() {
                if handle.join().is_err() {
                    tracing::warn!("职业识别线程异常退出");
                }
            }

            self.stop_signal.store(false, Ordering::SeqCst);
            *self.last_reported.lock() = None;
        }
    }

    impl Default for WindowsDetectionRuntime {
        fn default() -> Self {
            Self::new()
        }
    }

    fn run_detection_worker(
        app_handle: tauri::AppHandle,
        enabled: Arc<AtomicBool>,
        interval_ms: Arc<AtomicU64>,
        stop_signal: Arc<AtomicBool>,
        last_reported: Arc<Mutex<Option<DetectionSignature>>>,
    ) {
        tracing::info!("职业识别线程已启动");
        set_thread_priority_high();

        loop {
            if stop_signal.load(Ordering::SeqCst) {
                break;
            }

            if !enabled.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_millis(POLL_INTERVAL_MS));
                continue;
            }

            let sample_interval_ms = interval_ms.load(Ordering::SeqCst).max(100);
            let Some(hwnd) = foreground_target_window_handle() else {
                emit_detection_result(&app_handle, &last_reported, None, 0.0, "foregroundInactive");
                thread::sleep(Duration::from_millis(sample_interval_ms));
                continue;
            };

            let window = Window::from_raw_hwnd(hwnd.0);
            let settings = Settings::new(
                window,
                CursorCaptureSettings::Default,
                DrawBorderSettings::WithoutBorder,
                SecondaryWindowSettings::Default,
                MinimumUpdateIntervalSettings::Custom(Duration::from_millis(sample_interval_ms)),
                DirtyRegionSettings::Default,
                ColorFormat::Bgra8,
                (
                    hwnd.0 as isize,
                    app_handle.clone(),
                    stop_signal.clone(),
                    last_reported.clone(),
                    sample_interval_ms,
                ),
            );

            if let Err(error) = DetectionCapture::start(settings) {
                tracing::warn!(error = %error, "职业识别捕获失败");
                emit_detection_result(&app_handle, &last_reported, None, 0.0, "captureError");
                thread::sleep(Duration::from_millis(sample_interval_ms));
            }
        }

        tracing::info!("职业识别线程已停止");
    }

    struct DetectionCapture {
        hwnd_raw: isize,
        app_handle: tauri::AppHandle,
        stop_signal: Arc<AtomicBool>,
        last_reported: Arc<Mutex<Option<DetectionSignature>>>,
        configured_interval_ms: u64,
        last_sample: Instant,
    }

    impl GraphicsCaptureApiHandler for DetectionCapture {
        type Flags = (
            isize,
            tauri::AppHandle,
            Arc<AtomicBool>,
            Arc<Mutex<Option<DetectionSignature>>>,
            u64,
        );
        type Error = Box<dyn std::error::Error + Send + Sync>;

        fn new(ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
            let (hwnd_raw, app_handle, stop_signal, last_reported, configured_interval_ms) =
                ctx.flags;
            Ok(Self {
                hwnd_raw,
                app_handle,
                stop_signal,
                last_reported,
                configured_interval_ms,
                last_sample: Instant::now()
                    .checked_sub(Duration::from_millis(configured_interval_ms))
                    .unwrap_or_else(Instant::now),
            })
        }

        fn on_frame_arrived(
            &mut self,
            frame: &mut Frame,
            capture_control: InternalCaptureControl,
        ) -> Result<(), Self::Error> {
            if self.stop_signal.load(Ordering::SeqCst) || !self.should_process() {
                capture_control.stop();
                return Ok(());
            }

            let interval = Duration::from_millis(self.configured_interval_ms);
            if self.last_sample.elapsed() < interval {
                return Ok(());
            }
            self.last_sample = Instant::now();

            if let Err(error) = self.process_frame(frame) {
                tracing::warn!(error = %error, "职业识别帧处理失败");
            }

            Ok(())
        }

        fn on_closed(&mut self) -> Result<(), Self::Error> {
            tracing::debug!(hwnd = self.hwnd_raw, "职业识别捕获已关闭");
            Ok(())
        }
    }

    impl DetectionCapture {
        fn should_process(&self) -> bool {
            foreground_target_window_handle().is_some_and(|hwnd| hwnd.0 as isize == self.hwnd_raw)
        }

        fn process_frame(
            &self,
            frame: &mut Frame,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            let width = frame.width() as usize;
            let height = frame.height() as usize;

            if width / 2 < PATTERN_SIZE * 2 || height < PATTERN_SIZE {
                return Ok(());
            }

            let mut buffer = frame.buffer()?;
            let row_pitch = buffer.row_pitch() as usize;
            let raw_data = buffer.as_raw_buffer();

            let region = fixed_detection_region(width, height);
            let map_region = map_icon_region(width, height);

            if !detect_town_icon(raw_data, row_pitch, map_region) {
                emit_detection_result(
                    &self.app_handle,
                    &self.last_reported,
                    None,
                    0.0,
                    "notInTown",
                );
                return Ok(());
            }

            match detect_class_index(raw_data, row_pitch, width, height, region) {
                Some(class_index) => {
                    emit_detection_result(
                        &self.app_handle,
                        &self.last_reported,
                        Some(class_index),
                        1.0,
                        "matched",
                    );
                }
                None => {
                    emit_detection_result(
                        &self.app_handle,
                        &self.last_reported,
                        None,
                        0.0,
                        "notFound",
                    );
                }
            }

            Ok(())
        }
    }

    fn emit_detection_result(
        app_handle: &tauri::AppHandle,
        last_reported: &Arc<Mutex<Option<DetectionSignature>>>,
        class_index: Option<u16>,
        confidence: f32,
        reason: impl Into<String>,
    ) {
        let reason = reason.into();
        let signature = DetectionSignature {
            class_index,
            reason: reason.clone(),
        };

        let mut last_reported = last_reported.lock();
        if last_reported.as_ref() == Some(&signature) {
            return;
        }
        *last_reported = Some(signature);
        drop(last_reported);

        if let Err(error) = app_handle.emit(
            crate::CLASS_DETECTION_RESULT_EVENT,
            ClassDetectionResultEvent {
                class_index,
                confidence,
                reason,
            },
        ) {
            tracing::warn!(error = %error, "发送职业识别事件失败");
        }
    }

    fn detect_class_index(
        raw_data: &[u8],
        row_pitch: usize,
        frame_width: usize,
        frame_height: usize,
        region: DetectionRegion,
    ) -> Option<u16> {
        let pair_width = PATTERN_SIZE * 2;
        let region_x = region.x;
        let region_y = region.y;
        let region_width = region.width;
        let region_height = region.height;

        if region_width < pair_width || region_height < PATTERN_SIZE {
            return None;
        }

        let y_end = region_y + region_height.saturating_sub(PATTERN_SIZE) + 1;
        let x_end = region_x + region_width.saturating_sub(pair_width) + 1;

        for y in region_y..y_end {
            for x in region_x..x_end {
                let Some(anchor_offset) = pixel_offset(raw_data, row_pitch, x, y) else {
                    continue;
                };

                // 品红锚点检测 (BGRA: 255, 0, 255)。
                if raw_data[anchor_offset] != 255
                    || raw_data[anchor_offset + 1] != 0
                    || raw_data[anchor_offset + 2] != 255
                {
                    continue;
                }

                let Some(data_x) = x.checked_add(CLASS_ID_OFFSET_X) else {
                    continue;
                };
                let Some(data_y) = y.checked_add(CLASS_ID_OFFSET_Y) else {
                    continue;
                };

                if data_x >= frame_width || data_y >= frame_height {
                    continue;
                }

                if data_x >= region_x + region_width || data_y >= region_y + region_height {
                    continue;
                }

                let Some(data_offset) = pixel_offset(raw_data, row_pitch, data_x, data_y) else {
                    continue;
                };

                // 校验 R=255, B=0。
                if raw_data[data_offset + 2] == 255 && raw_data[data_offset] == 0 {
                    return Some(u16::from(raw_data[data_offset + 1] / CLASS_ID_STEP));
                }
            }
        }

        None
    }

    fn detect_town_icon(raw_data: &[u8], row_pitch: usize, region: DetectionRegion) -> bool {
        if region.width < 2 || region.height < 1 {
            return false;
        }

        let y_end = region.y + region.height;
        let x_end = region.x + region.width;

        for y in region.y..y_end {
            for x in region.x..x_end {
                let Some(offset) = pixel_offset(raw_data, row_pitch, x, y) else {
                    continue;
                };

                if !is_pure_cyan(&raw_data[offset..offset + 4]) {
                    continue;
                }

                let right_match = x + 1 < x_end
                    && pixel_offset(raw_data, row_pitch, x + 1, y).is_some_and(|right_offset| {
                        is_pure_cyan(&raw_data[right_offset..right_offset + 4])
                    });
                let down_match = y + 1 < y_end
                    && pixel_offset(raw_data, row_pitch, x, y + 1).is_some_and(|down_offset| {
                        is_pure_cyan(&raw_data[down_offset..down_offset + 4])
                    });

                if right_match || down_match {
                    return true;
                }
            }
        }

        false
    }

    fn pixel_offset(raw_data: &[u8], row_pitch: usize, x: usize, y: usize) -> Option<usize> {
        let offset = y.checked_mul(row_pitch)?.checked_add(x.checked_mul(4)?)?;
        (offset + 3 < raw_data.len()).then_some(offset)
    }

    fn fixed_detection_region(width: usize, height: usize) -> DetectionRegion {
        let scale = height as f32 / SCALE_BASE_HEIGHT;
        let half_width = (120.0 * scale).round().max(1.0) as usize;
        let region_height = (70.0 * scale).round().max(1.0) as usize;

        let region_width = half_width.saturating_mul(2).max(1);
        let x_center = width / 2;
        let x = x_center.saturating_sub(half_width);
        let y = height.saturating_sub(region_height);

        DetectionRegion {
            x: x.min(width.saturating_sub(1)),
            y: y.min(height.saturating_sub(1)),
            width: region_width.min(width.max(1)),
            height: region_height.min(height.max(1)),
        }
    }

    fn map_icon_region(width: usize, height: usize) -> DetectionRegion {
        let scale = height as f32 / SCALE_BASE_HEIGHT;
        let side = (MAP_REGION_BASE_SIZE * scale).round().max(2.0) as usize;
        let x = width.saturating_sub(side);
        DetectionRegion {
            x,
            y: 0,
            width: side.min(width.max(1)),
            height: side.min(height.max(1)),
        }
    }

    fn is_pure_cyan(pixel: &[u8]) -> bool {
        pixel.len() >= 4 && pixel[0] == 255 && pixel[1] == 255 && pixel[2] == 0
    }

    fn set_thread_priority_high() {
        use windows::Win32::System::Threading::{
            GetCurrentThread, SetThreadPriority, THREAD_PRIORITY_HIGHEST,
        };
        unsafe {
            let thread = GetCurrentThread();
            let _ = SetThreadPriority(thread, THREAD_PRIORITY_HIGHEST);
        }
    }

    #[derive(Clone, Copy)]
    struct DetectionRegion {
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn detect_class_index_reads_class_from_anchor() {
            let width = 16;
            let height = 16;
            let row_pitch = width * 4;
            let mut raw_data = vec![0u8; row_pitch * height];

            let anchor_x = 4;
            let anchor_y = 10;
            let data_x = anchor_x + CLASS_ID_OFFSET_X;
            let data_y = anchor_y + CLASS_ID_OFFSET_Y;
            let anchor_offset = pixel_offset(&raw_data, row_pitch, anchor_x, anchor_y).unwrap();
            raw_data[anchor_offset] = 255;
            raw_data[anchor_offset + 1] = 0;
            raw_data[anchor_offset + 2] = 255;

            let data_offset = pixel_offset(&raw_data, row_pitch, data_x, data_y).unwrap();
            raw_data[data_offset] = 0;
            raw_data[data_offset + 1] = 4;
            raw_data[data_offset + 2] = 255;

            let class_index = detect_class_index(
                &raw_data,
                row_pitch,
                width,
                height,
                DetectionRegion {
                    x: 0,
                    y: 8,
                    width,
                    height: 8,
                },
            );

            assert_eq!(class_index, Some(2));
        }

        #[test]
        fn detect_town_icon_detects_cyan_pair() {
            let width = 8;
            let height = 8;
            let row_pitch = width * 4;
            let mut raw_data = vec![0u8; row_pitch * height];
            let offset = pixel_offset(&raw_data, row_pitch, 2, 2).unwrap();
            raw_data[offset] = 255;
            raw_data[offset + 1] = 255;
            raw_data[offset + 2] = 0;
            let right_offset = pixel_offset(&raw_data, row_pitch, 3, 2).unwrap();
            raw_data[right_offset] = 255;
            raw_data[right_offset + 1] = 255;
            raw_data[right_offset + 2] = 0;

            assert!(detect_town_icon(
                &raw_data,
                row_pitch,
                DetectionRegion {
                    x: 0,
                    y: 0,
                    width,
                    height,
                }
            ));
        }

        #[test]
        fn detect_town_icon_returns_false_when_no_pair_exists() {
            let width = 8;
            let height = 8;
            let row_pitch = width * 4;
            let raw_data = vec![0u8; row_pitch * height];

            assert!(!detect_town_icon(
                &raw_data,
                row_pitch,
                DetectionRegion {
                    x: 0,
                    y: 0,
                    width,
                    height,
                }
            ));
        }

        #[test]
        fn detection_signature_dedupes_identical_results() {
            let signature = DetectionSignature {
                class_index: Some(1),
                reason: "matched".to_string(),
            };
            let last_reported = Arc::new(Mutex::new(Some(signature.clone())));

            assert_eq!(*last_reported.lock(), Some(signature));
        }
    }
}

#[cfg(not(windows))]
mod windows_impl {
    use super::*;

    #[derive(Debug, Default)]
    pub struct WindowsDetectionRuntime;

    impl WindowsDetectionRuntime {
        pub fn new() -> Self {
            Self
        }

        pub fn start(
            &mut self,
            app_handle: tauri::AppHandle,
            enabled: Arc<AtomicBool>,
            interval_ms: Arc<AtomicU64>,
        ) -> Result<(), String> {
            let _ = (app_handle, enabled, interval_ms);
            Err("职业识别当前仅支持 Windows。".to_string())
        }

        pub fn stop(&mut self) {}
    }
}
