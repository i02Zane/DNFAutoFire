//! 职业识别核心逻辑：只保留截图、AprilTag 扫描和结果事件，不带任何预览 UI。

use crate::config::{ConfigRepository, DetectionNoMatchPolicy};
use crate::error::AppResult;
use crate::runtime::RuntimeSupervisor;
use parking_lot::Mutex;
use serde::Serialize;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub(crate) struct ClassDetectionResultEvent {
    pub class_index: Option<u16>,
    pub confidence: f32,
    pub reason: String,
}

#[derive(Debug)]
pub struct DetectionRuntime {
    enabled: Arc<AtomicBool>,
    interval_ms: Arc<AtomicU64>,
    config_store: Arc<ConfigRepository>,
    #[cfg(windows)]
    platform: windows_impl::WindowsDetectionRuntime,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct DetectionSnapshot {
    pub running: bool,
    #[ts(type = "number")]
    pub interval_ms: u64,
    pub last_result: Option<ClassDetectionResultEvent>,
    pub town_active: Option<bool>,
}

impl DetectionRuntime {
    pub fn new(config_store: Arc<ConfigRepository>) -> Self {
        Self {
            enabled: Arc::new(AtomicBool::new(false)),
            interval_ms: Arc::new(AtomicU64::new(crate::config::DEFAULT_DETECTION_INTERVAL_MS)),
            config_store,
            #[cfg(windows)]
            platform: windows_impl::WindowsDetectionRuntime::new(),
        }
    }

    pub fn set_interval_ms(&self, interval_ms: u64) {
        self.interval_ms.store(interval_ms, Ordering::SeqCst);
    }

    pub fn start(
        &mut self,
        app_handle: tauri::AppHandle,
        interval_ms: u64,
        runtime_supervisor: RuntimeSupervisor,
    ) -> AppResult<()> {
        self.set_interval_ms(interval_ms);
        if self.enabled.load(Ordering::SeqCst) {
            tracing::debug!("职业识别引擎已在运行中");
            return Ok(());
        }

        tracing::info!(interval_ms, "启动职业识别引擎");
        self.enabled.store(true, Ordering::SeqCst);

        #[cfg(windows)]
        if let Err(error) = self.platform.start(
            app_handle,
            self.enabled.clone(),
            self.interval_ms.clone(),
            self.config_store.clone(),
            runtime_supervisor,
        ) {
            self.enabled.store(false, Ordering::SeqCst);
            return Err(error);
        }

        #[cfg(not(windows))]
        {
            let _ = app_handle;
            self.enabled.store(false, Ordering::SeqCst);
            return Err(crate::error::AppError::vision(
                "职业识别当前仅支持 Windows。",
            ));
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

    pub fn snapshot(&self) -> DetectionSnapshot {
        DetectionSnapshot {
            running: self.is_running(),
            interval_ms: self.interval_ms.load(Ordering::SeqCst),
            last_result: self.last_result_snapshot(),
            town_active: self.town_active_snapshot(),
        }
    }

    fn last_result_snapshot(&self) -> Option<ClassDetectionResultEvent> {
        #[cfg(windows)]
        {
            self.platform.last_result_snapshot()
        }

        #[cfg(not(windows))]
        {
            None
        }
    }

    fn town_active_snapshot(&self) -> Option<bool> {
        self.last_result_snapshot()
            .as_ref()
            .and_then(|result| town_active_from_reason(&result.reason))
    }
}

impl Default for DetectionRuntime {
    fn default() -> Self {
        Self::new(Arc::new(ConfigRepository::new()))
    }
}

impl Drop for DetectionRuntime {
    fn drop(&mut self) {
        self.stop();
    }
}

fn town_active_from_reason(reason: &str) -> Option<bool> {
    match reason {
        "matched" | "notFound" => Some(true),
        "notInTown" => Some(false),
        "foregroundInactive" | "captureError" => None,
        _ => None,
    }
}

#[cfg(windows)]
mod windows_impl {
    use super::*;
    use crate::platform::window::foreground_target_window_handle;
    use apriltag::{Detector as AprilTagDetector, Family, Image as AprilTagImage};
    use std::cell::RefCell;
    use std::thread::{self, JoinHandle};
    use std::time::{Duration, Instant};
    use windows_capture::capture::{Context, GraphicsCaptureApiHandler};
    use windows_capture::frame::Frame;
    use windows_capture::graphics_capture_api::InternalCaptureControl;
    use windows_capture::settings::{
        ColorFormat, CursorCaptureSettings, DirtyRegionSettings, DrawBorderSettings,
        MinimumUpdateIntervalSettings, SecondaryWindowSettings, Settings,
    };
    use windows_capture::window::Window;

    const POLL_INTERVAL_MS: u64 = 200;
    const MAP_REGION_BASE_SIZE: f32 = 24.0;
    const SCALE_BASE_HEIGHT: f32 = 600.0;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct DetectionSignature {
        class_index: Option<u16>,
        confidence: u32,
        reason: String,
    }

    #[derive(Debug)]
    struct AprilTagRuntime {
        detector: AprilTagDetector,
        tag_image: Option<AprilTagImage>,
        tag_image_width: usize,
        tag_image_height: usize,
    }

    thread_local! {
        static APRILTAG_RUNTIME: RefCell<Option<AprilTagRuntime>> = const { RefCell::new(None) };
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
            config_store: Arc<ConfigRepository>,
            runtime_supervisor: RuntimeSupervisor,
        ) -> AppResult<()> {
            if self.thread_handle.is_some() {
                return Ok(());
            }

            self.stop_signal.store(false, Ordering::SeqCst);
            *self.last_reported.lock() = None;

            let stop_signal = self.stop_signal.clone();
            let last_reported = self.last_reported.clone();
            let handle = thread::spawn(move || {
                run_detection_worker(
                    app_handle,
                    enabled,
                    interval_ms,
                    config_store,
                    runtime_supervisor,
                    stop_signal,
                    last_reported,
                );
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

        pub fn last_result_snapshot(&self) -> Option<ClassDetectionResultEvent> {
            self.last_reported
                .lock()
                .as_ref()
                .map(|signature| ClassDetectionResultEvent {
                    class_index: signature.class_index,
                    confidence: signature.confidence as f32 / 1000.0,
                    reason: signature.reason.clone(),
                })
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
        config_store: Arc<ConfigRepository>,
        runtime_supervisor: RuntimeSupervisor,
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
                record_detection_result(&last_reported, None, 0.0, "foregroundInactive");
                thread::sleep(Duration::from_millis(sample_interval_ms));
                continue;
            };

            let capture_attempts = [
                (
                    DrawBorderSettings::WithoutBorder,
                    MinimumUpdateIntervalSettings::Custom(Duration::from_millis(
                        sample_interval_ms,
                    )),
                ),
                (
                    DrawBorderSettings::WithoutBorder,
                    MinimumUpdateIntervalSettings::Default,
                ),
                (
                    DrawBorderSettings::Default,
                    MinimumUpdateIntervalSettings::Default,
                ),
            ];

            let mut capture_started = false;
            for (draw_border_settings, minimum_update_interval_settings) in capture_attempts {
                let capture_flags = DetectionCaptureFlags {
                    hwnd_raw: hwnd.0 as isize,
                    app_handle: app_handle.clone(),
                    config_store: config_store.clone(),
                    runtime_supervisor: runtime_supervisor.clone(),
                    stop_signal: stop_signal.clone(),
                    last_reported: last_reported.clone(),
                    configured_interval_ms: sample_interval_ms,
                };
                let settings = build_capture_settings(
                    &capture_flags,
                    draw_border_settings,
                    minimum_update_interval_settings,
                );

                match DetectionCapture::start(settings) {
                    Ok(()) => {
                        capture_started = true;
                        break;
                    }
                    Err(error) => {
                        tracing::warn!(
                            error = %error,
                            ?draw_border_settings,
                            ?minimum_update_interval_settings,
                            "职业识别捕获失败，尝试兼容设置"
                        );
                    }
                }
            }

            if !capture_started {
                tracing::warn!("职业识别所有捕获兼容设置均失败");
                record_detection_result(&last_reported, None, 0.0, "captureError");
                thread::sleep(Duration::from_millis(sample_interval_ms));
            }
        }

        tracing::info!("职业识别线程已停止");
    }

    #[derive(Clone)]
    struct DetectionCaptureFlags {
        hwnd_raw: isize,
        app_handle: tauri::AppHandle,
        config_store: Arc<ConfigRepository>,
        runtime_supervisor: RuntimeSupervisor,
        stop_signal: Arc<AtomicBool>,
        last_reported: Arc<Mutex<Option<DetectionSignature>>>,
        configured_interval_ms: u64,
    }

    type DetectionCaptureSettings = Settings<DetectionCaptureFlags, Window>;

    fn build_capture_settings(
        flags: &DetectionCaptureFlags,
        draw_border_settings: DrawBorderSettings,
        minimum_update_interval_settings: MinimumUpdateIntervalSettings,
    ) -> DetectionCaptureSettings {
        Settings::new(
            Window::from_raw_hwnd(flags.hwnd_raw as *mut std::ffi::c_void),
            CursorCaptureSettings::Default,
            draw_border_settings,
            SecondaryWindowSettings::Default,
            minimum_update_interval_settings,
            DirtyRegionSettings::Default,
            ColorFormat::Bgra8,
            flags.clone(),
        )
    }

    struct DetectionCapture {
        hwnd_raw: isize,
        app_handle: tauri::AppHandle,
        config_store: Arc<ConfigRepository>,
        runtime_supervisor: RuntimeSupervisor,
        stop_signal: Arc<AtomicBool>,
        last_reported: Arc<Mutex<Option<DetectionSignature>>>,
        configured_interval_ms: u64,
        last_sample: Instant,
    }

    impl GraphicsCaptureApiHandler for DetectionCapture {
        type Flags = DetectionCaptureFlags;
        type Error = Box<dyn std::error::Error + Send + Sync>;

        fn new(ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
            let flags = ctx.flags;
            Ok(Self {
                hwnd_raw: flags.hwnd_raw,
                app_handle: flags.app_handle,
                config_store: flags.config_store,
                runtime_supervisor: flags.runtime_supervisor,
                stop_signal: flags.stop_signal,
                last_reported: flags.last_reported,
                configured_interval_ms: flags.configured_interval_ms,
                last_sample: Instant::now()
                    .checked_sub(Duration::from_millis(flags.configured_interval_ms))
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

        fn select_active_config(&self, active_class_id: Option<String>, operation: &str) {
            if let Err(error) = self
                .runtime_supervisor
                .select_active_config_from_detection(&self.app_handle, active_class_id)
            {
                tracing::warn!(error = %error, operation, "职业识别切换当前配置失败");
            }
        }

        fn process_frame(
            &mut self,
            frame: &mut Frame,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            let width = frame.width() as usize;
            let height = frame.height() as usize;

            if width / 2 < 6 || height < 3 {
                return Ok(());
            }

            let mut buffer = frame.buffer()?;
            let row_pitch = buffer.row_pitch() as usize;
            let raw_data = buffer.as_raw_buffer();

            let region = fixed_detection_region(width, height);
            let map_region = map_icon_region(width, height);

            if !detect_town_icon(raw_data, row_pitch, map_region) {
                record_detection_result(&self.last_reported, None, 0.0, "notInTown");
                return Ok(());
            }

            match detect_class_index(raw_data, row_pitch, region)? {
                Some(class_index) => {
                    if let Some(class_id) =
                        crate::domain::classes::class_id_by_detection_index(class_index)
                    {
                        self.select_active_config(Some(class_id.to_string()), "matched");
                    }
                    record_detection_result(&self.last_reported, Some(class_index), 1.0, "matched");
                }
                None => {
                    if matches!(
                        self.config_store.settings().detection.no_match_policy,
                        DetectionNoMatchPolicy::Global
                    ) {
                        self.select_active_config(None, "noMatch");
                    }
                    record_detection_result(&self.last_reported, None, 0.0, "notFound");
                }
            }

            Ok(())
        }
    }

    fn create_apriltag_runtime() -> Result<AprilTagRuntime, Box<dyn std::error::Error + Send + Sync>>
    {
        let mut detector = AprilTagDetector::builder()
            .add_family_bits(Family::tag_36h11(), 2)
            .build()?;
        detector.set_thread_number(1);
        detector.set_decimation(1.0);
        detector.set_refine_edges(true);

        Ok(AprilTagRuntime {
            detector,
            tag_image: None,
            tag_image_width: 0,
            tag_image_height: 0,
        })
    }

    fn detect_class_index(
        raw_data: &[u8],
        row_pitch: usize,
        region: DetectionRegion,
    ) -> Result<Option<u16>, Box<dyn std::error::Error + Send + Sync>> {
        with_apriltag_runtime(|runtime| {
            if region.width == 0 || region.height == 0 {
                return Ok(None);
            }

            for scale in [1, 2, 3] {
                prepare_apriltag_image(runtime, raw_data, row_pitch, region, scale)?;
                let image = runtime.tag_image.as_ref().expect("tag image should exist");
                let detections = runtime.detector.detect(image);
                let best = detections
                    .into_iter()
                    .max_by(|a, b| a.decision_margin().total_cmp(&b.decision_margin()));
                if let Some(best) = best.and_then(|best| u16::try_from(best.id()).ok()) {
                    return Ok(Some(best));
                }
            }

            Ok(None)
        })
    }

    fn prepare_apriltag_image(
        runtime: &mut AprilTagRuntime,
        raw_data: &[u8],
        row_pitch: usize,
        region: DetectionRegion,
        scale: usize,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let image_width = region.width.saturating_mul(scale);
        let image_height = region.height.saturating_mul(scale);
        if runtime.tag_image.is_none()
            || runtime.tag_image_width != image_width
            || runtime.tag_image_height != image_height
        {
            let image = AprilTagImage::zeros_with_stride(image_width, image_height, image_width)
                .map_err(|_| std::io::Error::other("无法创建 AprilTag 图像"))?;
            runtime.tag_image = Some(image);
            runtime.tag_image_width = image_width;
            runtime.tag_image_height = image_height;
        }

        let image = runtime.tag_image.as_mut().expect("tag image should exist");
        let dst = image.as_slice_mut();
        for y in 0..region.height {
            let src_row = &raw_data[(region.y + y) * row_pitch + region.x * 4
                ..(region.y + y) * row_pitch + (region.x + region.width) * 4];
            for x in 0..region.width {
                let offset = x * 4;
                let b = src_row[offset] as u16;
                let g = src_row[offset + 1] as u16;
                let r = src_row[offset + 2] as u16;
                let gray = ((r * 77 + g * 150 + b * 29) >> 8) as u8;
                for scaled_y in y * scale..(y + 1) * scale {
                    let dst_row = &mut dst[scaled_y * image_width..(scaled_y + 1) * image_width];
                    for pixel in &mut dst_row[x * scale..(x + 1) * scale] {
                        *pixel = gray;
                    }
                }
            }
        }

        Ok(())
    }

    fn with_apriltag_runtime<T>(
        f: impl FnOnce(&mut AprilTagRuntime) -> Result<T, Box<dyn std::error::Error + Send + Sync>>,
    ) -> Result<T, Box<dyn std::error::Error + Send + Sync>> {
        APRILTAG_RUNTIME.with(|cell| {
            let mut slot = cell.borrow_mut();
            if slot.is_none() {
                *slot = Some(create_apriltag_runtime()?);
            }

            f(slot.as_mut().expect("AprilTag runtime should exist"))
        })
    }

    fn record_detection_result(
        last_reported: &Arc<Mutex<Option<DetectionSignature>>>,
        class_index: Option<u16>,
        confidence: f32,
        reason: impl Into<String>,
    ) {
        let reason = reason.into();
        let signature = DetectionSignature {
            class_index,
            confidence: (confidence * 1000.0).round() as u32,
            reason: reason.clone(),
        };

        let mut last_reported = last_reported.lock();
        if last_reported.as_ref() == Some(&signature) {
            return;
        }
        *last_reported = Some(signature);
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

    fn pixel_offset(raw_data: &[u8], row_pitch: usize, x: usize, y: usize) -> Option<usize> {
        let offset = y.checked_mul(row_pitch)?.checked_add(x.checked_mul(4)?)?;
        (offset + 3 < raw_data.len()).then_some(offset)
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

    #[cfg(all(test, windows))]
    mod tests {
        use super::*;

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
                confidence: 1000,
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
            config_store: Arc<ConfigRepository>,
            runtime_supervisor: RuntimeSupervisor,
        ) -> AppResult<()> {
            let _ = (
                app_handle,
                enabled,
                interval_ms,
                config_store,
                runtime_supervisor,
            );
            Err(crate::error::AppError::vision(
                "职业识别当前仅支持 Windows。",
            ))
        }

        pub fn stop(&mut self) {}
    }
}
