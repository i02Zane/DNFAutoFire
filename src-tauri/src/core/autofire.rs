//! 连发核心逻辑：低级键盘钩子记录物理按键，独立循环按配置间隔补发按键。

use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use crate::logging::format_vk;

#[cfg(windows)]
use super::keyboard::WindowsKeyboardDriver;
#[cfg(windows)]
use super::window::{WindowDetector, WindowsWindowDetector};

const VK_BITS_PER_SLOT: usize = 64;
const PRESSED_KEY_SLOT_COUNT: usize = 4;
type PressedKeyBits = [AtomicU64; PRESSED_KEY_SLOT_COUNT];

#[derive(Debug, Clone)]
pub struct FireKeyConfig {
    pub vk: u16,
    pub interval_ms: u16,
}

pub struct AutoFireEngine {
    enabled: Arc<AtomicBool>,
    configured_keys: Arc<RwLock<HashSet<u16>>>,
    key_intervals: Arc<RwLock<HashMap<u16, u64>>>,
    // VK 码范围超过 64，拆成多个 AtomicU64 槽位可避免锁住键盘钩子回调。
    pressed_keys: Arc<PressedKeyBits>,
    #[cfg(windows)]
    platform: windows_impl::WindowsAutoFire,
}

impl std::fmt::Debug for AutoFireEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AutoFireEngine")
            .field("enabled", &self.enabled.load(Ordering::SeqCst))
            .field("configured_keys", &*self.configured_keys.read())
            .field("key_intervals", &*self.key_intervals.read())
            .finish()
    }
}

impl AutoFireEngine {
    pub fn new() -> Self {
        Self {
            enabled: Arc::new(AtomicBool::new(false)),
            configured_keys: Arc::new(RwLock::new(HashSet::new())),
            key_intervals: Arc::new(RwLock::new(HashMap::new())),
            pressed_keys: Arc::new(std::array::from_fn(|_| AtomicU64::new(0))),
            #[cfg(windows)]
            platform: windows_impl::WindowsAutoFire::new(),
        }
    }

    #[allow(dead_code)]
    pub fn set_keys(&self, keys: Vec<u16>) {
        let keys = keys
            .into_iter()
            .map(|vk| FireKeyConfig {
                vk,
                interval_ms: 33,
            })
            .collect();
        self.set_key_configs(keys);
    }

    pub fn set_key_configs(&self, keys: Vec<FireKeyConfig>) {
        tracing::info!(key_count = keys.len(), "更新连发按键快照");
        let mut configured = self.configured_keys.write();
        let mut intervals = self.key_intervals.write();

        // 运行中修改配置时直接替换共享集合，连发线程下一轮循环即可看到新配置。
        configured.clear();
        intervals.clear();
        for key in keys {
            configured.insert(key.vk);
            intervals.insert(key.vk, key.interval_ms as u64);
        }
    }

    pub fn start(&mut self) -> Result<(), String> {
        if self.enabled.load(Ordering::SeqCst) {
            tracing::debug!("连发引擎已在运行中");
            return Ok(());
        }

        tracing::info!(
            key_count = self.configured_keys.read().len(),
            "启动连发引擎"
        );
        clear_pressed_keys(&self.pressed_keys);

        #[cfg(windows)]
        self.platform.start(
            self.enabled.clone(),
            self.configured_keys.clone(),
            self.key_intervals.clone(),
            self.pressed_keys.clone(),
        )?;
        self.enabled.store(true, Ordering::SeqCst);
        Ok(())
    }

    pub fn stop(&mut self) {
        if self.enabled.swap(false, Ordering::SeqCst) {
            tracing::info!("停止连发引擎");
        } else {
            tracing::debug!("连发引擎已经处于停止状态");
        }
        #[cfg(windows)]
        self.platform.stop();
    }

    #[allow(dead_code)]
    pub fn is_running(&self) -> bool {
        self.enabled.load(Ordering::SeqCst)
    }
}

fn clear_pressed_keys(pressed_keys: &PressedKeyBits) {
    for slot in pressed_keys {
        slot.store(0, Ordering::SeqCst);
    }
}

fn pressed_key_snapshot(pressed_keys: &PressedKeyBits) -> [u64; PRESSED_KEY_SLOT_COUNT] {
    std::array::from_fn(|index| pressed_keys[index].load(Ordering::SeqCst))
}

fn pressed_key_snapshot_has_any(snapshot: &[u64; PRESSED_KEY_SLOT_COUNT]) -> bool {
    snapshot.iter().any(|slot| *slot != 0)
}

fn pressed_key_snapshot_contains(
    snapshot: &[u64; PRESSED_KEY_SLOT_COUNT],
    slot_index: usize,
    key_bit: u64,
) -> bool {
    snapshot
        .get(slot_index)
        .is_some_and(|slot| slot & key_bit != 0)
}

fn vk_to_slot_bit(vk: u16) -> Option<(usize, u64)> {
    // VK 码按 64 个一组映射，避免 LCtrl(0xA2) 和 NUM2(0x62) 这类同 bit 冲突。
    let vk = usize::from(vk);
    let slot_index = vk / VK_BITS_PER_SLOT;
    if slot_index >= PRESSED_KEY_SLOT_COUNT {
        return None;
    }

    Some((slot_index, 1u64 << (vk % VK_BITS_PER_SLOT)))
}

impl Default for AutoFireEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for AutoFireEngine {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vk_slot_bit_distinguishes_lctrl_and_num2() {
        let lctrl = vk_to_slot_bit(0xA2).unwrap();
        let num2 = vk_to_slot_bit(0x62).unwrap();

        assert_ne!(lctrl, num2);
        assert_ne!(lctrl.0, num2.0);
        assert_eq!(lctrl.1, num2.1);
    }

    #[test]
    fn pressed_key_snapshot_keeps_same_bit_in_different_slots_separate() {
        let pressed_keys: PressedKeyBits = std::array::from_fn(|_| AtomicU64::new(0));
        let (lctrl_slot, lctrl_bit) = vk_to_slot_bit(0xA2).unwrap();
        let (num2_slot, num2_bit) = vk_to_slot_bit(0x62).unwrap();

        pressed_keys[lctrl_slot].store(lctrl_bit, Ordering::SeqCst);
        let snapshot = pressed_key_snapshot(&pressed_keys);

        assert!(pressed_key_snapshot_contains(
            &snapshot, lctrl_slot, lctrl_bit
        ));
        assert!(!pressed_key_snapshot_contains(
            &snapshot, num2_slot, num2_bit
        ));
    }
}

#[cfg(windows)]
mod windows_impl {
    use super::*;
    use std::thread::{self, JoinHandle};
    use std::time::{Duration, Instant};

    use super::super::hook::{KeyboardHookEvent, KeyboardHookRunner};
    use super::super::keyboard::KeyboardDriver;

    const KEY_HOLD_MS: u64 = 8;
    const LOOP_SLEEP_MS: u64 = 1;

    pub struct WindowsAutoFire {
        thread_handle: Option<JoinHandle<()>>,
        hook_runner: KeyboardHookRunner,
    }

    impl WindowsAutoFire {
        pub fn new() -> Self {
            Self {
                thread_handle: None,
                hook_runner: KeyboardHookRunner::new("AutoFire"),
            }
        }

        pub fn start(
            &mut self,
            enabled: Arc<AtomicBool>,
            configured_keys: Arc<RwLock<HashSet<u16>>>,
            key_intervals: Arc<RwLock<HashMap<u16, u64>>>,
            pressed_keys: Arc<PressedKeyBits>,
        ) -> Result<(), String> {
            if self.thread_handle.is_some() {
                return Ok(());
            }

            // 钩子线程只记录真实按键状态，连发线程负责按间隔发送模拟输入。
            set_timer_resolution(true);

            if let Err(error) = self.hook_runner.start(
                "安装按键连发键盘钩子失败",
                keyboard_hook_proc,
                {
                    let enabled = enabled.clone();
                    let configured_keys = configured_keys.clone();
                    let pressed_keys = pressed_keys.clone();
                    move || {
                        HOOK_ENABLED.store(true, Ordering::SeqCst);
                        *HOOK_PRESSED_KEYS.write() = Some(pressed_keys);
                        *HOOK_ENABLED_FLAG.write() = Some(enabled);
                        *HOOK_CONFIGURED_KEYS_REF.write() = Some(configured_keys);
                    }
                },
                || {
                    HOOK_ENABLED.store(false, Ordering::SeqCst);
                },
            ) {
                set_timer_resolution(false);
                return Err(error);
            }

            let fire_stop_signal = self.hook_runner.stop_signal();
            let handle = thread::spawn(move || {
                autofire_loop(
                    enabled,
                    configured_keys,
                    key_intervals,
                    pressed_keys,
                    fire_stop_signal,
                );
            });
            self.thread_handle = Some(handle);
            Ok(())
        }

        pub fn stop(&mut self) {
            self.hook_runner.stop();

            if let Some(handle) = self.thread_handle.take() {
                if handle.join().is_err() {
                    tracing::warn!("连发线程异常退出");
                }
            }

            set_timer_resolution(false);
        }
    }

    static HOOK_ENABLED: AtomicBool = AtomicBool::new(false);
    static HOOK_PRESSED_KEYS: once_cell::sync::Lazy<RwLock<Option<Arc<PressedKeyBits>>>> =
        once_cell::sync::Lazy::new(|| RwLock::new(None));
    static HOOK_ENABLED_FLAG: once_cell::sync::Lazy<RwLock<Option<Arc<AtomicBool>>>> =
        once_cell::sync::Lazy::new(|| RwLock::new(None));
    #[allow(clippy::type_complexity)]
    static HOOK_CONFIGURED_KEYS_REF: once_cell::sync::Lazy<
        RwLock<Option<Arc<RwLock<HashSet<u16>>>>>,
    > = once_cell::sync::Lazy::new(|| RwLock::new(None));

    unsafe extern "system" fn keyboard_hook_proc(
        code: i32,
        wparam: windows::Win32::Foundation::WPARAM,
        lparam: windows::Win32::Foundation::LPARAM,
    ) -> windows::Win32::Foundation::LRESULT {
        use windows::Win32::UI::WindowsAndMessaging::CallNextHookEx;

        if code >= 0 && HOOK_ENABLED.load(Ordering::SeqCst) {
            let event = KeyboardHookEvent::from_raw(wparam, lparam);

            if !event.is_injected && is_engine_enabled() && is_configured_key(event.vk) {
                let pressed_keys = HOOK_PRESSED_KEYS.read();
                if let (Some(ref keys), Some((slot_index, key_bit))) =
                    (&*pressed_keys, vk_to_slot_bit(event.vk))
                {
                    let slot = &keys[slot_index];

                    if event.is_keydown {
                        let old = slot.fetch_or(key_bit, Ordering::SeqCst);
                        if old & key_bit == 0 {
                            tracing::debug!(
                                vk = %format_vk(event.vk),
                                "记录连发物理按键按下"
                            );
                        }
                    }

                    if event.is_keyup {
                        tracing::debug!(
                            vk = %format_vk(event.vk),
                            "记录连发物理按键释放"
                        );
                        slot.fetch_and(!key_bit, Ordering::SeqCst);
                    }
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

    fn is_configured_key(vk: u16) -> bool {
        HOOK_CONFIGURED_KEYS_REF
            .read()
            .as_ref()
            .map(|keys| keys.read().contains(&vk))
            .unwrap_or(false)
    }

    fn autofire_loop(
        enabled: Arc<AtomicBool>,
        configured_keys: Arc<RwLock<HashSet<u16>>>,
        key_intervals: Arc<RwLock<HashMap<u16, u64>>>,
        pressed_keys: Arc<PressedKeyBits>,
        stop_signal: Arc<AtomicBool>,
    ) {
        tracing::info!("连发线程已启动");
        set_thread_priority_high();

        let keyboard = WindowsKeyboardDriver::new();
        let window = WindowsWindowDetector::new();
        let mut next_fire_at: HashMap<u16, Instant> = HashMap::new();

        loop {
            if stop_signal.load(Ordering::SeqCst) {
                break;
            }

            if !enabled.load(Ordering::SeqCst) {
                next_fire_at.clear();
                thread::sleep(Duration::from_millis(10));
                continue;
            }

            if !window.is_target_active() {
                log_waiting_window(&window);
                next_fire_at.clear();
                thread::sleep(Duration::from_millis(50));
                continue;
            }

            let keys_with_sc: Vec<(u16, u16, usize, u64, u64)> = {
                let configured = configured_keys.read();
                let intervals = key_intervals.read();
                configured
                    .iter()
                    .filter_map(|&vk| {
                        let sc = keyboard.vk_to_scan_code(vk);
                        let (slot_index, bit) = vk_to_slot_bit(vk)?;
                        let interval = intervals.get(&vk).copied().unwrap_or(33);
                        Some((vk, sc, slot_index, bit, interval))
                    })
                    .collect()
            };

            let pressed = pressed_key_snapshot(&pressed_keys);
            if pressed_key_snapshot_has_any(&pressed) {
                // 先抓快照再发送按键，避免发送过程中钩子状态被模拟输入反复改写。
                let now = Instant::now();
                next_fire_at.retain(|vk, _| {
                    keys_with_sc.iter().any(|(key_vk, _, slot_index, bit, _)| {
                        key_vk == vk && pressed_key_snapshot_contains(&pressed, *slot_index, *bit)
                    })
                });

                for &(vk, sc, slot_index, bit, interval_ms) in &keys_with_sc {
                    if !pressed_key_snapshot_contains(&pressed, slot_index, bit) {
                        continue;
                    }
                    if stop_signal.load(Ordering::SeqCst) {
                        break;
                    }
                    if next_fire_at
                        .get(&vk)
                        .is_some_and(|next_fire| now < *next_fire)
                    {
                        continue;
                    }

                    let fire_started_at = Instant::now();
                    keyboard.send_game_key_down(vk, sc);
                    thread::sleep(Duration::from_millis(KEY_HOLD_MS));
                    keyboard.send_game_key_up(vk, sc);
                    next_fire_at.insert(vk, fire_started_at + Duration::from_millis(interval_ms));
                }
                thread::sleep(Duration::from_millis(LOOP_SLEEP_MS));
            } else {
                next_fire_at.clear();
                thread::sleep(Duration::from_millis(LOOP_SLEEP_MS));
            }
        }

        tracing::info!("连发线程已停止");
    }

    fn log_waiting_window(window: &WindowsWindowDetector) {
        static LAST_LOG: AtomicU64 = AtomicU64::new(0);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let last = LAST_LOG.load(Ordering::Relaxed);
        if now - last >= 5 {
            LAST_LOG.store(now, Ordering::Relaxed);
            tracing::debug!(
                foreground_class = %window.get_foreground_class_name(),
                "等待目标窗口激活"
            );
        }
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
