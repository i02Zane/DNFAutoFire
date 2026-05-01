//! 一键连招核心逻辑：监听当前职业触发键，并按动作块顺序发送输入。

use crate::config::ComboDefinition;
use crate::logging::format_vk;
use parking_lot::RwLock;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Clone)]
pub struct ComboEngine {
    enabled: Arc<AtomicBool>,
    combos: Arc<RwLock<Vec<ComboDefinition>>>,
    trigger_keys: Arc<RwLock<HashSet<u16>>>,
    abort_generation: Arc<AtomicU64>,
    executing: Arc<AtomicBool>,
    #[cfg(windows)]
    platform: windows_impl::WindowsComboEngine,
}

impl std::fmt::Debug for ComboEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComboEngine")
            .field("enabled", &self.enabled.load(Ordering::SeqCst))
            .field("trigger_keys", &*self.trigger_keys.read())
            .field("executing", &self.executing.load(Ordering::SeqCst))
            .finish()
    }
}

impl ComboEngine {
    pub fn new() -> Self {
        Self {
            enabled: Arc::new(AtomicBool::new(false)),
            combos: Arc::new(RwLock::new(Vec::new())),
            trigger_keys: Arc::new(RwLock::new(HashSet::new())),
            abort_generation: Arc::new(AtomicU64::new(0)),
            executing: Arc::new(AtomicBool::new(false)),
            #[cfg(windows)]
            platform: windows_impl::WindowsComboEngine::new(),
        }
    }

    pub fn set_combo_configs(&self, combos: Vec<ComboDefinition>) {
        let trigger_keys: HashSet<u16> = combos
            .iter()
            .filter(|combo| combo.enabled)
            .filter_map(|combo| combo.trigger_vk)
            .collect();

        tracing::info!(
            combo_count = combos.len(),
            trigger_count = trigger_keys.len(),
            "更新一键连招快照"
        );
        *self.combos.write() = combos;
        *self.trigger_keys.write() = trigger_keys;
        // 运行中刷新职业或连招快照时，中止当前动作，下一次触发使用新快照。
        self.abort_generation.fetch_add(1, Ordering::SeqCst);
    }

    pub fn start(&mut self) -> Result<(), String> {
        if self.enabled.load(Ordering::SeqCst) {
            tracing::debug!("一键连招引擎已在运行中");
            return Ok(());
        }

        tracing::info!(combo_count = self.combos.read().len(), "启动一键连招引擎");
        self.abort_generation.fetch_add(1, Ordering::SeqCst);
        self.executing.store(false, Ordering::SeqCst);

        #[cfg(windows)]
        self.platform.start(
            self.enabled.clone(),
            self.combos.clone(),
            self.trigger_keys.clone(),
            self.abort_generation.clone(),
            self.executing.clone(),
        )?;

        self.enabled.store(true, Ordering::SeqCst);
        Ok(())
    }

    pub fn stop(&mut self) {
        if self.enabled.swap(false, Ordering::SeqCst) {
            tracing::info!("停止一键连招引擎");
        } else {
            tracing::debug!("一键连招引擎已经处于停止状态");
        }
        self.abort_generation.fetch_add(1, Ordering::SeqCst);
        self.executing.store(false, Ordering::SeqCst);

        #[cfg(windows)]
        self.platform.stop();
    }

    pub fn is_running(&self) -> bool {
        self.enabled.load(Ordering::SeqCst)
    }
}

impl Default for ComboEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ComboEngine {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(windows)]
mod windows_impl {
    use super::*;
    use crate::config::ComboAction;
    use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
    use std::thread::{self, JoinHandle};
    use std::time::{Duration, Instant};

    use super::super::hook::{KeyboardHookEvent, KeyboardHookRunner};
    use super::super::keyboard::{KeyboardDriver, WindowsKeyboardDriver};
    use super::super::window::{
        is_foreground_target_window_active, WindowDetector, WindowsWindowDetector,
    };

    const LOOP_WAIT_MS: u64 = 10;
    const INTERRUPT_SLEEP_MS: u64 = 2;

    pub struct WindowsComboEngine {
        hook_runner: KeyboardHookRunner,
        worker_thread_handle: Option<JoinHandle<()>>,
    }

    impl Clone for WindowsComboEngine {
        fn clone(&self) -> Self {
            Self::new()
        }
    }

    impl WindowsComboEngine {
        pub fn new() -> Self {
            Self {
                hook_runner: KeyboardHookRunner::new("Combo"),
                worker_thread_handle: None,
            }
        }

        pub fn start(
            &mut self,
            enabled: Arc<AtomicBool>,
            combos: Arc<RwLock<Vec<ComboDefinition>>>,
            trigger_keys: Arc<RwLock<HashSet<u16>>>,
            abort_generation: Arc<AtomicU64>,
            executing: Arc<AtomicBool>,
        ) -> Result<(), String> {
            if self.hook_runner.is_running() || self.worker_thread_handle.is_some() {
                return Ok(());
            }

            set_timer_resolution(true);

            let (trigger_tx, trigger_rx) = mpsc::channel();
            if let Err(error) = self.hook_runner.start(
                "安装一键连招键盘钩子失败",
                keyboard_hook_proc,
                {
                    let hook_enabled = enabled.clone();
                    let hook_executing = executing.clone();
                    move || {
                        HOOK_ENABLED.store(true, Ordering::SeqCst);
                        *HOOK_ENABLED_FLAG.write() = Some(hook_enabled);
                        *HOOK_TRIGGER_KEYS_REF.write() = Some(trigger_keys);
                        *HOOK_EXECUTING_FLAG.write() = Some(hook_executing);
                        *HOOK_TRIGGER_TX.write() = Some(trigger_tx);
                    }
                },
                || {
                    HOOK_ENABLED.store(false, Ordering::SeqCst);
                    *HOOK_ENABLED_FLAG.write() = None;
                    *HOOK_TRIGGER_KEYS_REF.write() = None;
                    *HOOK_EXECUTING_FLAG.write() = None;
                    *HOOK_TRIGGER_TX.write() = None;
                },
            ) {
                set_timer_resolution(false);
                return Err(error);
            }

            let worker_stop_signal = self.hook_runner.stop_signal();
            let worker_enabled = enabled.clone();
            let worker_executing = executing.clone();
            let worker_handle = thread::spawn(move || {
                run_combo_worker(
                    trigger_rx,
                    worker_enabled,
                    combos,
                    abort_generation,
                    worker_executing,
                    worker_stop_signal,
                );
            });
            self.worker_thread_handle = Some(worker_handle);
            Ok(())
        }

        pub fn stop(&mut self) {
            self.hook_runner.stop();

            if let Some(handle) = self.worker_thread_handle.take() {
                if handle.join().is_err() {
                    tracing::warn!("连招线程异常退出");
                }
            }

            set_timer_resolution(false);
        }
    }

    static HOOK_ENABLED: AtomicBool = AtomicBool::new(false);
    type TriggerKeySet = Arc<RwLock<HashSet<u16>>>;
    static HOOK_ENABLED_FLAG: once_cell::sync::Lazy<RwLock<Option<Arc<AtomicBool>>>> =
        once_cell::sync::Lazy::new(|| RwLock::new(None));
    static HOOK_TRIGGER_KEYS_REF: once_cell::sync::Lazy<RwLock<Option<TriggerKeySet>>> =
        once_cell::sync::Lazy::new(|| RwLock::new(None));
    static HOOK_EXECUTING_FLAG: once_cell::sync::Lazy<RwLock<Option<Arc<AtomicBool>>>> =
        once_cell::sync::Lazy::new(|| RwLock::new(None));
    static HOOK_TRIGGER_TX: once_cell::sync::Lazy<RwLock<Option<Sender<u16>>>> =
        once_cell::sync::Lazy::new(|| RwLock::new(None));

    unsafe extern "system" fn keyboard_hook_proc(
        code: i32,
        wparam: windows::Win32::Foundation::WPARAM,
        lparam: windows::Win32::Foundation::LPARAM,
    ) -> windows::Win32::Foundation::LRESULT {
        use windows::Win32::UI::WindowsAndMessaging::CallNextHookEx;

        if code >= 0 && HOOK_ENABLED.load(Ordering::SeqCst) {
            let event = KeyboardHookEvent::from_raw(wparam, lparam);

            if !event.is_injected
                && is_engine_enabled()
                && is_trigger_key(event.vk)
                && is_target_active_now()
            {
                if event.is_keydown && !is_executing() {
                    tracing::debug!(trigger_vk = %format_vk(event.vk), "收到连招触发键");
                    send_trigger(event.vk);
                } else if event.is_keydown {
                    tracing::debug!(
                        trigger_vk = %format_vk(event.vk),
                        "连招正在执行，忽略重复触发"
                    );
                }
                if event.is_keydown || event.is_keyup {
                    return windows::Win32::Foundation::LRESULT(1);
                }
            }
        }

        CallNextHookEx(None, code, wparam, lparam)
    }

    fn is_engine_enabled() -> bool {
        HOOK_ENABLED_FLAG
            .read()
            .as_ref()
            .map(|flag| flag.load(Ordering::SeqCst))
            .unwrap_or(false)
    }

    fn is_trigger_key(vk: u16) -> bool {
        HOOK_TRIGGER_KEYS_REF
            .read()
            .as_ref()
            .map(|keys| keys.read().contains(&vk))
            .unwrap_or(false)
    }

    fn is_executing() -> bool {
        HOOK_EXECUTING_FLAG
            .read()
            .as_ref()
            .map(|flag| flag.load(Ordering::SeqCst))
            .unwrap_or(false)
    }

    fn send_trigger(vk: u16) {
        if let Some(sender) = HOOK_TRIGGER_TX.read().as_ref() {
            if let Err(error) = sender.send(vk) {
                tracing::warn!(
                    trigger_vk = %format_vk(vk),
                    error = %error,
                    "投递连招触发键失败"
                );
            }
        } else {
            tracing::warn!(
                trigger_vk = %format_vk(vk),
                "连招触发通道尚未就绪"
            );
        }
    }

    fn is_target_active_now() -> bool {
        is_foreground_target_window_active()
    }

    fn run_combo_worker(
        trigger_rx: Receiver<u16>,
        enabled: Arc<AtomicBool>,
        combos: Arc<RwLock<Vec<ComboDefinition>>>,
        abort_generation: Arc<AtomicU64>,
        executing: Arc<AtomicBool>,
        stop_signal: Arc<AtomicBool>,
    ) {
        tracing::info!("连招线程已启动");
        set_thread_priority_high();

        let keyboard = WindowsKeyboardDriver::new();
        let window = WindowsWindowDetector::new();

        loop {
            if stop_signal.load(Ordering::SeqCst) {
                break;
            }

            match trigger_rx.recv_timeout(Duration::from_millis(LOOP_WAIT_MS)) {
                Ok(trigger_vk) => {
                    if !enabled.load(Ordering::SeqCst) {
                        continue;
                    }
                    if executing.swap(true, Ordering::SeqCst) {
                        continue;
                    }

                    let generation = abort_generation.load(Ordering::SeqCst);
                    if let Some(combo) = find_combo(&combos, trigger_vk) {
                        tracing::info!(
                            combo = %combo.name,
                            trigger_vk = %format_vk(trigger_vk),
                            action_count = combo.actions.len(),
                            "开始执行连招"
                        );
                        execute_combo(
                            &combo,
                            trigger_vk,
                            &keyboard,
                            &window,
                            &abort_generation,
                            generation,
                            &stop_signal,
                        );
                    } else {
                        tracing::debug!(
                            trigger_vk = %format_vk(trigger_vk),
                            "未找到匹配的连招"
                        );
                    }
                    while trigger_rx.try_recv().is_ok() {}
                    executing.store(false, Ordering::SeqCst);
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }

        executing.store(false, Ordering::SeqCst);
        tracing::info!("连招线程已停止");
    }

    fn find_combo(
        combos: &Arc<RwLock<Vec<ComboDefinition>>>,
        trigger_vk: u16,
    ) -> Option<ComboDefinition> {
        combos
            .read()
            .iter()
            .find(|combo| combo.enabled && combo.trigger_vk == Some(trigger_vk))
            .cloned()
    }

    fn execute_combo(
        combo: &ComboDefinition,
        trigger_vk: u16,
        keyboard: &WindowsKeyboardDriver,
        window: &WindowsWindowDetector,
        abort_generation: &Arc<AtomicU64>,
        generation: u64,
        stop_signal: &Arc<AtomicBool>,
    ) {
        if !should_continue(stop_signal, abort_generation, generation, window) {
            return;
        }

        for action in &combo.actions {
            if !should_continue(stop_signal, abort_generation, generation, window) {
                tracing::debug!(combo = %combo.name, "连招执行被中断");
                return;
            }

            let should_continue = match action {
                ComboAction::Tap {
                    vk: Some(vk),
                    hold_ms,
                    wait_after_ms,
                    ..
                } => send_tap(
                    *vk,
                    *hold_ms,
                    *wait_after_ms,
                    keyboard,
                    window,
                    abort_generation,
                    generation,
                    stop_signal,
                    *vk == trigger_vk,
                ),
                ComboAction::Command {
                    keys,
                    key_hold_ms,
                    key_gap_ms,
                    wait_after_ms,
                    ..
                } => send_command(
                    keys,
                    *key_hold_ms,
                    *key_gap_ms,
                    *wait_after_ms,
                    keyboard,
                    window,
                    abort_generation,
                    generation,
                    stop_signal,
                    trigger_vk,
                ),
                ComboAction::Tap { vk: None, .. } => false,
            };

            if !should_continue {
                tracing::debug!(combo = %combo.name, "连招动作执行被中断");
                return;
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn send_command(
        keys: &[u16],
        key_hold_ms: u16,
        key_gap_ms: u16,
        wait_after_ms: u16,
        keyboard: &WindowsKeyboardDriver,
        window: &WindowsWindowDetector,
        abort_generation: &Arc<AtomicU64>,
        generation: u64,
        stop_signal: &Arc<AtomicBool>,
        trigger_vk: u16,
    ) -> bool {
        for (index, vk) in keys.iter().enumerate() {
            if !send_tap(
                *vk,
                key_hold_ms,
                0,
                keyboard,
                window,
                abort_generation,
                generation,
                stop_signal,
                *vk == trigger_vk,
            ) {
                return false;
            }

            if index + 1 < keys.len()
                && !sleep_interruptible(
                    u64::from(key_gap_ms),
                    stop_signal,
                    abort_generation,
                    generation,
                    window,
                )
            {
                return false;
            }
        }

        sleep_interruptible(
            u64::from(wait_after_ms),
            stop_signal,
            abort_generation,
            generation,
            window,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn send_tap(
        vk: u16,
        hold_ms: u16,
        wait_after_ms: u16,
        keyboard: &WindowsKeyboardDriver,
        window: &WindowsWindowDetector,
        abort_generation: &Arc<AtomicU64>,
        generation: u64,
        stop_signal: &Arc<AtomicBool>,
        use_real_key: bool,
    ) -> bool {
        if !should_continue(stop_signal, abort_generation, generation, window) {
            return false;
        }

        tracing::debug!(
            vk = %format_vk(vk),
            real_key = use_real_key,
            hold_ms = hold_ms,
            wait_after_ms = wait_after_ms,
            "发送连招按键"
        );
        let scan_code = keyboard.vk_to_scan_code(vk);
        if use_real_key {
            // 物理触发键已被钩子拦截；当连招动作再次使用同一键时，
            // 需要按真实 VK 发送，保证 Space -> E 这类先 buff 再技能的顺序。
            keyboard.send_key_down(vk, scan_code);
        } else {
            keyboard.send_game_key_down(vk, scan_code);
        }

        let hold_completed = sleep_interruptible(
            u64::from(hold_ms),
            stop_signal,
            abort_generation,
            generation,
            window,
        );

        if use_real_key {
            keyboard.send_key_up(vk, scan_code);
        } else {
            keyboard.send_game_key_up(vk, scan_code);
        }

        hold_completed
            && sleep_interruptible(
                u64::from(wait_after_ms),
                stop_signal,
                abort_generation,
                generation,
                window,
            )
    }

    fn should_continue(
        stop_signal: &Arc<AtomicBool>,
        abort_generation: &Arc<AtomicU64>,
        generation: u64,
        window: &WindowsWindowDetector,
    ) -> bool {
        !stop_signal.load(Ordering::SeqCst)
            && abort_generation.load(Ordering::SeqCst) == generation
            && window.is_target_active()
    }

    fn sleep_interruptible(
        duration_ms: u64,
        stop_signal: &Arc<AtomicBool>,
        abort_generation: &Arc<AtomicU64>,
        generation: u64,
        window: &WindowsWindowDetector,
    ) -> bool {
        let deadline = Instant::now() + Duration::from_millis(duration_ms);
        while Instant::now() < deadline {
            if !should_continue(stop_signal, abort_generation, generation, window) {
                return false;
            }
            thread::sleep(Duration::from_millis(INTERRUPT_SLEEP_MS));
        }
        true
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
