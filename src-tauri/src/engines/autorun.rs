//! 一键奔跑核心逻辑：监听左右移动键，在按住后补发一次奔跑脉冲。

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use parking_lot::RwLock;
use serde::Serialize;
use ts_rs::TS;

use crate::error::AppResult;
use crate::platform::logging::format_vk;

#[cfg(windows)]
use crate::platform::keyboard::WindowsKeyboardDriver;
#[cfg(windows)]
use crate::platform::window::{WindowDetector, WindowsWindowDetector};

const DEFAULT_AUTO_RUN_PULSE_DELAY_MS: u64 = 25;
const AUTO_RUN_KEY_HOLD_MS: u64 = 8;
const AUTO_RUN_LOOP_SLEEP_MS: u64 = 5;
const AUTO_RUN_WINDOW_RECHECK_SLEEP_MS: u64 = 50;

#[derive(Debug, Clone, Copy)]
struct AutoRunSettings {
    left_vk: u16,
    right_vk: u16,
    pulse_delay_ms: u64,
}

impl Default for AutoRunSettings {
    fn default() -> Self {
        Self {
            left_vk: 0x25,
            right_vk: 0x27,
            pulse_delay_ms: DEFAULT_AUTO_RUN_PULSE_DELAY_MS,
        }
    }
}

#[derive(Debug)]
struct AutoRunKeyState {
    pressed: AtomicBool,
    pulse_sent: AtomicBool,
    pressed_at: AtomicU64,
}

impl AutoRunKeyState {
    fn new() -> Self {
        Self {
            pressed: AtomicBool::new(false),
            pulse_sent: AtomicBool::new(false),
            pressed_at: AtomicU64::new(0),
        }
    }

    fn clear(&self) {
        self.pressed.store(false, Ordering::SeqCst);
        self.pulse_sent.store(false, Ordering::SeqCst);
        self.pressed_at.store(0, Ordering::SeqCst);
    }
}

#[derive(Debug)]
struct AutoRunState {
    left: AutoRunKeyState,
    right: AutoRunKeyState,
}

impl AutoRunState {
    fn new() -> Self {
        Self {
            left: AutoRunKeyState::new(),
            right: AutoRunKeyState::new(),
        }
    }

    fn clear(&self) {
        self.left.clear();
        self.right.clear();
    }
}

pub struct AutoRunEngine {
    enabled: Arc<AtomicBool>,
    settings: Arc<RwLock<AutoRunSettings>>,
    #[cfg(windows)]
    platform: windows_impl::WindowsAutoRun,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct AutoRunSnapshot {
    pub running: bool,
    pub left_vk: u16,
    pub right_vk: u16,
    #[ts(type = "number")]
    pub pulse_delay_ms: u64,
}

impl std::fmt::Debug for AutoRunEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AutoRunEngine")
            .field("enabled", &self.enabled.load(Ordering::SeqCst))
            .finish()
    }
}

impl AutoRunEngine {
    pub fn new() -> Self {
        Self {
            enabled: Arc::new(AtomicBool::new(false)),
            settings: Arc::new(RwLock::new(AutoRunSettings::default())),
            #[cfg(windows)]
            platform: windows_impl::WindowsAutoRun::new(),
        }
    }

    pub fn set_settings(&self, left_vk: u16, right_vk: u16, pulse_delay_ms: u64) {
        *self.settings.write() = AutoRunSettings {
            left_vk,
            right_vk,
            pulse_delay_ms,
        };
        tracing::info!(
            left_vk = %format_vk(left_vk),
            right_vk = %format_vk(right_vk),
            pulse_delay_ms,
            "更新一键奔跑设置"
        );
    }

    pub fn start(&mut self) -> AppResult<()> {
        if self.enabled.load(Ordering::SeqCst) {
            tracing::debug!("一键奔跑引擎已在运行中");
            return Ok(());
        }

        tracing::info!("启动一键奔跑引擎");
        #[cfg(windows)]
        self.platform
            .start(self.enabled.clone(), self.settings.clone())?;

        self.enabled.store(true, Ordering::SeqCst);
        Ok(())
    }

    pub fn stop(&mut self) {
        if self.enabled.swap(false, Ordering::SeqCst) {
            tracing::info!("停止一键奔跑引擎");
        } else {
            tracing::debug!("一键奔跑引擎已经处于停止状态");
        }

        #[cfg(windows)]
        self.platform.stop();
    }

    pub fn is_running(&self) -> bool {
        self.enabled.load(Ordering::SeqCst)
    }

    pub fn snapshot(&self) -> AutoRunSnapshot {
        let settings = *self.settings.read();
        AutoRunSnapshot {
            running: self.is_running(),
            left_vk: settings.left_vk,
            right_vk: settings.right_vk,
            pulse_delay_ms: settings.pulse_delay_ms,
        }
    }
}

impl Default for AutoRunEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for AutoRunEngine {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(windows)]
mod windows_impl {
    use super::*;
    use crate::platform::hook::{KeyboardHookEvent, KeyboardHookRunner};
    use crate::platform::keyboard::KeyboardDriver;
    use std::thread::{self, JoinHandle};

    pub struct WindowsAutoRun {
        hook_runner: KeyboardHookRunner,
        worker_thread_handle: Option<JoinHandle<()>>,
        state: Arc<AutoRunState>,
    }

    impl WindowsAutoRun {
        pub fn new() -> Self {
            Self {
                hook_runner: KeyboardHookRunner::new("AutoRun"),
                worker_thread_handle: None,
                state: Arc::new(AutoRunState::new()),
            }
        }

        pub fn start(
            &mut self,
            enabled: Arc<AtomicBool>,
            settings: Arc<RwLock<AutoRunSettings>>,
        ) -> AppResult<()> {
            if self.hook_runner.is_running() || self.worker_thread_handle.is_some() {
                return Ok(());
            }

            set_timer_resolution(true);

            if let Err(error) = self.hook_runner.start(
                "安装一键奔跑键盘钩子失败",
                keyboard_hook_proc,
                {
                    let enabled = enabled.clone();
                    let state = self.state.clone();
                    let settings = settings.clone();
                    move || {
                        HOOK_ENABLED.store(true, Ordering::SeqCst);
                        *HOOK_ENABLED_FLAG.write() = Some(enabled);
                        *HOOK_STATE.write() = Some(state);
                        *HOOK_SETTINGS.write() = Some(settings);
                    }
                },
                || {
                    HOOK_ENABLED.store(false, Ordering::SeqCst);
                    *HOOK_ENABLED_FLAG.write() = None;
                    *HOOK_STATE.write() = None;
                    *HOOK_SETTINGS.write() = None;
                },
            ) {
                set_timer_resolution(false);
                return Err(error);
            }

            let stop_signal = self.hook_runner.stop_signal();
            let enabled_for_worker = enabled.clone();
            let state_for_worker = self.state.clone();
            let settings_for_worker = settings.clone();
            let handle = thread::spawn(move || {
                autorun_loop(
                    enabled_for_worker,
                    state_for_worker,
                    settings_for_worker,
                    stop_signal,
                );
            });
            self.worker_thread_handle = Some(handle);
            Ok(())
        }

        pub fn stop(&mut self) {
            self.hook_runner.stop();

            if let Some(handle) = self.worker_thread_handle.take() {
                if handle.join().is_err() {
                    tracing::warn!("一键奔跑线程异常退出");
                }
            }

            self.state.clear();
            set_timer_resolution(false);
        }
    }

    static HOOK_ENABLED: AtomicBool = AtomicBool::new(false);
    static HOOK_ENABLED_FLAG: once_cell::sync::Lazy<RwLock<Option<Arc<AtomicBool>>>> =
        once_cell::sync::Lazy::new(|| RwLock::new(None));
    static HOOK_STATE: once_cell::sync::Lazy<RwLock<Option<Arc<AutoRunState>>>> =
        once_cell::sync::Lazy::new(|| RwLock::new(None));
    static HOOK_SETTINGS: once_cell::sync::Lazy<RwLock<Option<Arc<RwLock<AutoRunSettings>>>>> =
        once_cell::sync::Lazy::new(|| RwLock::new(None));

    unsafe extern "system" fn keyboard_hook_proc(
        code: i32,
        wparam: windows::Win32::Foundation::WPARAM,
        lparam: windows::Win32::Foundation::LPARAM,
    ) -> windows::Win32::Foundation::LRESULT {
        use windows::Win32::UI::WindowsAndMessaging::CallNextHookEx;

        if code >= 0 && HOOK_ENABLED.load(Ordering::SeqCst) {
            let event = KeyboardHookEvent::from_raw(wparam, lparam);

            if !event.is_injected && is_engine_enabled() {
                let settings = current_settings();
                if event.vk == settings.left_vk {
                    handle_left_event(event);
                } else if event.vk == settings.right_vk {
                    handle_right_event(event);
                }
            }
        }

        CallNextHookEx(None, code, wparam, lparam)
    }

    fn current_settings() -> AutoRunSettings {
        HOOK_SETTINGS
            .read()
            .as_ref()
            .map(|settings| *settings.read())
            .unwrap_or_default()
    }

    fn is_engine_enabled() -> bool {
        HOOK_ENABLED_FLAG
            .read()
            .as_ref()
            .map(|flag| flag.load(Ordering::SeqCst))
            .unwrap_or(false)
    }

    fn handle_left_event(event: KeyboardHookEvent) {
        let Some(state) = HOOK_STATE.read().as_ref().cloned() else {
            return;
        };

        if event.is_keydown {
            if !state.left.pressed.swap(true, Ordering::SeqCst) {
                state.left.pulse_sent.store(false, Ordering::SeqCst);
                state.left.pressed_at.store(now_millis(), Ordering::SeqCst);
                tracing::debug!(vk = %format_vk(event.vk), "记录一键奔跑左键按下");
            }
            return;
        }
        if !event.is_keyup {
            return;
        }

        state.left.pressed.store(false, Ordering::SeqCst);
        state.left.pulse_sent.store(false, Ordering::SeqCst);
        state.left.pressed_at.store(0, Ordering::SeqCst);
        tracing::debug!(vk = %format_vk(event.vk), "记录一键奔跑左键释放");
        let keyboard = WindowsKeyboardDriver::new();
        let scan_code = keyboard.vk_to_scan_code(event.vk);
        keyboard.send_game_key_up(event.vk, scan_code);
    }

    fn handle_right_event(event: KeyboardHookEvent) {
        let Some(state) = HOOK_STATE.read().as_ref().cloned() else {
            return;
        };

        if event.is_keydown {
            if !state.right.pressed.swap(true, Ordering::SeqCst) {
                state.right.pulse_sent.store(false, Ordering::SeqCst);
                state.right.pressed_at.store(now_millis(), Ordering::SeqCst);
                tracing::debug!(vk = %format_vk(event.vk), "记录一键奔跑右键按下");
            }
            return;
        }
        if !event.is_keyup {
            return;
        }

        state.right.pressed.store(false, Ordering::SeqCst);
        state.right.pulse_sent.store(false, Ordering::SeqCst);
        state.right.pressed_at.store(0, Ordering::SeqCst);
        tracing::debug!(vk = %format_vk(event.vk), "记录一键奔跑右键释放");
        let keyboard = WindowsKeyboardDriver::new();
        let scan_code = keyboard.vk_to_scan_code(event.vk);
        keyboard.send_game_key_up(event.vk, scan_code);
    }

    fn autorun_loop(
        enabled: Arc<AtomicBool>,
        state: Arc<AutoRunState>,
        settings: Arc<RwLock<AutoRunSettings>>,
        stop_signal: Arc<AtomicBool>,
    ) {
        tracing::info!("一键奔跑线程已启动");
        set_thread_priority_high();

        let keyboard = WindowsKeyboardDriver::new();
        let window = WindowsWindowDetector::new();

        loop {
            if stop_signal.load(Ordering::SeqCst) {
                break;
            }

            if !enabled.load(Ordering::SeqCst) {
                state.clear();
                thread::sleep(Duration::from_millis(AUTO_RUN_LOOP_SLEEP_MS));
                continue;
            }

            if !window.is_target_active() {
                state.clear();
                thread::sleep(Duration::from_millis(AUTO_RUN_WINDOW_RECHECK_SLEEP_MS));
                continue;
            }

            let now = now_millis();
            let settings = *settings.read();
            try_send_pulse(
                &keyboard,
                settings.left_vk,
                &state.left,
                settings.pulse_delay_ms,
                now,
            );
            try_send_pulse(
                &keyboard,
                settings.right_vk,
                &state.right,
                settings.pulse_delay_ms,
                now,
            );

            thread::sleep(Duration::from_millis(AUTO_RUN_LOOP_SLEEP_MS));
        }

        state.clear();
        tracing::info!("一键奔跑线程已停止");
    }

    fn try_send_pulse(
        keyboard: &WindowsKeyboardDriver,
        vk: u16,
        state: &AutoRunKeyState,
        pulse_delay_ms: u64,
        now: u64,
    ) {
        let pressed = state.pressed.load(Ordering::SeqCst);
        let pulse_sent = state.pulse_sent.load(Ordering::SeqCst);
        let pressed_at = state.pressed_at.load(Ordering::SeqCst);
        if !pressed || pulse_sent || pressed_at == 0 {
            return;
        }

        if now.saturating_sub(pressed_at) < pulse_delay_ms {
            return;
        }

        let scan_code = keyboard.vk_to_scan_code(vk);
        tracing::debug!(vk = %format_vk(vk), "发送一键奔跑脉冲");
        keyboard.send_game_key_up(vk, scan_code);
        thread::sleep(Duration::from_millis(pulse_delay_ms));
        if !state.pressed.load(Ordering::SeqCst) {
            state.pulse_sent.store(true, Ordering::SeqCst);
            return;
        }
        keyboard.send_game_key_down(vk, scan_code);
        thread::sleep(Duration::from_millis(AUTO_RUN_KEY_HOLD_MS));
        if !state.pressed.load(Ordering::SeqCst) {
            keyboard.send_game_key_up(vk, scan_code);
        }
        state.pulse_sent.store(true, Ordering::SeqCst);
    }

    fn now_millis() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    fn set_timer_resolution(high_precision: bool) {
        use windows::Win32::Media::{timeBeginPeriod, timeEndPeriod};
        unsafe {
            if high_precision {
                timeBeginPeriod(1);
            } else {
                timeEndPeriod(1);
            }
        }
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
}
