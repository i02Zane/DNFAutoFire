//! 连发核心逻辑：低级键盘钩子记录物理按键，独立循环按配置间隔补发按键。

use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use crate::config::FireKeyMode;
use crate::error::AppResult;
use crate::platform::logging::format_vk;
use serde::Serialize;
use ts_rs::TS;

#[cfg(windows)]
use crate::platform::keyboard::WindowsKeyboardDriver;
#[cfg(windows)]
use crate::platform::window::{WindowDetector, WindowsWindowDetector};

const VK_BITS_PER_SLOT: usize = 64;
const PRESSED_KEY_SLOT_COUNT: usize = 4;
type PressedKeyBits = [AtomicU64; PRESSED_KEY_SLOT_COUNT];

#[derive(Debug, Clone)]
pub struct FireKeyConfig {
    pub vk: u16,
    pub interval_ms: u16,
    pub mode: FireKeyMode,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct AutoFireKeySnapshot {
    pub vk: u16,
    #[ts(type = "number")]
    pub interval_ms: u64,
    pub mode: FireKeyMode,
    pub pressed: bool,
    pub toggle_active: bool,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct AutoFireSnapshot {
    pub running: bool,
    pub keys: Vec<AutoFireKeySnapshot>,
}

pub struct AutoFireEngine {
    enabled: Arc<AtomicBool>,
    configured_keys: Arc<RwLock<HashSet<u16>>>,
    key_intervals: Arc<RwLock<HashMap<u16, u64>>>,
    key_modes: Arc<RwLock<HashMap<u16, FireKeyMode>>>,
    // VK 码范围超过 64，拆成多个 AtomicU64 槽位可避免锁住键盘钩子回调。
    pressed_keys: Arc<PressedKeyBits>,
    toggle_active_keys: Arc<PressedKeyBits>,
    #[cfg(windows)]
    platform: windows_impl::WindowsAutoFire,
}

impl std::fmt::Debug for AutoFireEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AutoFireEngine")
            .field("enabled", &self.enabled.load(Ordering::SeqCst))
            .field("configured_keys", &*self.configured_keys.read())
            .field("key_intervals", &*self.key_intervals.read())
            .field("key_modes", &*self.key_modes.read())
            .finish()
    }
}

impl AutoFireEngine {
    pub fn new() -> Self {
        Self {
            enabled: Arc::new(AtomicBool::new(false)),
            configured_keys: Arc::new(RwLock::new(HashSet::new())),
            key_intervals: Arc::new(RwLock::new(HashMap::new())),
            key_modes: Arc::new(RwLock::new(HashMap::new())),
            pressed_keys: Arc::new(std::array::from_fn(|_| AtomicU64::new(0))),
            toggle_active_keys: Arc::new(std::array::from_fn(|_| AtomicU64::new(0))),
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
                mode: FireKeyMode::Hold,
            })
            .collect();
        self.set_key_configs(keys);
    }

    pub fn set_key_configs(&self, keys: Vec<FireKeyConfig>) {
        tracing::info!(key_count = keys.len(), "更新连发按键快照");
        let mut configured = self.configured_keys.write();
        let mut intervals = self.key_intervals.write();
        let mut modes = self.key_modes.write();
        let mut toggle_keys = HashSet::new();

        // 运行中修改配置时直接替换共享集合，连发线程下一轮循环即可看到新配置。
        configured.clear();
        intervals.clear();
        modes.clear();
        for key in keys {
            configured.insert(key.vk);
            intervals.insert(key.vk, key.interval_ms as u64);
            modes.insert(key.vk, key.mode);
            if key.mode == FireKeyMode::Toggle {
                toggle_keys.insert(key.vk);
            }
        }
        retain_pressed_keys(&self.toggle_active_keys, &toggle_keys);
    }

    pub fn start(&mut self) -> AppResult<()> {
        if self.enabled.load(Ordering::SeqCst) {
            tracing::debug!("连发引擎已在运行中");
            return Ok(());
        }

        tracing::info!(
            key_count = self.configured_keys.read().len(),
            "启动连发引擎"
        );
        clear_pressed_keys(&self.pressed_keys);
        clear_pressed_keys(&self.toggle_active_keys);

        #[cfg(windows)]
        self.platform.start(
            self.enabled.clone(),
            self.configured_keys.clone(),
            self.key_intervals.clone(),
            self.key_modes.clone(),
            self.pressed_keys.clone(),
            self.toggle_active_keys.clone(),
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
        clear_pressed_keys(&self.pressed_keys);
        clear_pressed_keys(&self.toggle_active_keys);
    }

    #[allow(dead_code)]
    pub fn is_running(&self) -> bool {
        self.enabled.load(Ordering::SeqCst)
    }

    pub fn active_toggle_keys(&self) -> Vec<u16> {
        if !self.enabled.load(Ordering::SeqCst) {
            return Vec::new();
        }

        let configured = self.configured_keys.read();
        let modes = self.key_modes.read();
        let toggle_snapshot = pressed_key_snapshot(&self.toggle_active_keys);
        let mut keys = configured
            .iter()
            .filter(|&&vk| modes.get(&vk).copied().unwrap_or_default() == FireKeyMode::Toggle)
            .filter_map(|&vk| {
                let (slot_index, key_bit) = vk_to_slot_bit(vk)?;
                pressed_key_snapshot_contains(&toggle_snapshot, slot_index, key_bit).then_some(vk)
            })
            .collect::<Vec<_>>();
        keys.sort_unstable();
        keys
    }

    pub fn snapshot(&self) -> AutoFireSnapshot {
        let configured = self.configured_keys.read();
        let intervals = self.key_intervals.read();
        let modes = self.key_modes.read();
        let pressed_snapshot = pressed_key_snapshot(&self.pressed_keys);
        let toggle_snapshot = pressed_key_snapshot(&self.toggle_active_keys);
        let mut keys: Vec<_> = configured
            .iter()
            .filter_map(|&vk| {
                let (slot_index, key_bit) = vk_to_slot_bit(vk)?;
                Some(AutoFireKeySnapshot {
                    vk,
                    interval_ms: intervals.get(&vk).copied().unwrap_or_default(),
                    mode: modes.get(&vk).copied().unwrap_or_default(),
                    pressed: pressed_key_snapshot_contains(&pressed_snapshot, slot_index, key_bit),
                    toggle_active: pressed_key_snapshot_contains(
                        &toggle_snapshot,
                        slot_index,
                        key_bit,
                    ),
                })
            })
            .collect();
        keys.sort_by_key(|key| key.vk);

        AutoFireSnapshot {
            running: self.is_running(),
            keys,
        }
    }
}

fn clear_pressed_keys(pressed_keys: &PressedKeyBits) {
    for slot in pressed_keys {
        slot.store(0, Ordering::SeqCst);
    }
}

fn retain_pressed_keys(pressed_keys: &PressedKeyBits, retained_vks: &HashSet<u16>) {
    let mut retained_masks = [0u64; PRESSED_KEY_SLOT_COUNT];
    for &vk in retained_vks {
        if let Some((slot_index, key_bit)) = vk_to_slot_bit(vk) {
            retained_masks[slot_index] |= key_bit;
        }
    }

    for (slot, retained_mask) in pressed_keys.iter().zip(retained_masks) {
        slot.fetch_and(retained_mask, Ordering::SeqCst);
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

fn fire_key_is_active(
    mode: FireKeyMode,
    pressed_snapshot: &[u64; PRESSED_KEY_SLOT_COUNT],
    toggle_snapshot: &[u64; PRESSED_KEY_SLOT_COUNT],
    slot_index: usize,
    key_bit: u64,
) -> bool {
    match mode {
        FireKeyMode::Hold => pressed_key_snapshot_contains(pressed_snapshot, slot_index, key_bit),
        FireKeyMode::Toggle => pressed_key_snapshot_contains(toggle_snapshot, slot_index, key_bit),
    }
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

    #[test]
    fn fire_key_active_state_uses_configured_mode() {
        let pressed_keys: PressedKeyBits = std::array::from_fn(|_| AtomicU64::new(0));
        let toggle_active_keys: PressedKeyBits = std::array::from_fn(|_| AtomicU64::new(0));
        let (slot_index, key_bit) = vk_to_slot_bit(0x58).unwrap();

        pressed_keys[slot_index].store(key_bit, Ordering::SeqCst);
        let pressed = pressed_key_snapshot(&pressed_keys);
        let toggle_active = pressed_key_snapshot(&toggle_active_keys);

        assert!(fire_key_is_active(
            FireKeyMode::Hold,
            &pressed,
            &toggle_active,
            slot_index,
            key_bit
        ));
        assert!(!fire_key_is_active(
            FireKeyMode::Toggle,
            &pressed,
            &toggle_active,
            slot_index,
            key_bit
        ));

        pressed_keys[slot_index].store(0, Ordering::SeqCst);
        toggle_active_keys[slot_index].store(key_bit, Ordering::SeqCst);
        let pressed = pressed_key_snapshot(&pressed_keys);
        let toggle_active = pressed_key_snapshot(&toggle_active_keys);

        assert!(!fire_key_is_active(
            FireKeyMode::Hold,
            &pressed,
            &toggle_active,
            slot_index,
            key_bit
        ));
        assert!(fire_key_is_active(
            FireKeyMode::Toggle,
            &pressed,
            &toggle_active,
            slot_index,
            key_bit
        ));
    }

    #[test]
    fn retain_pressed_keys_keeps_only_configured_toggle_keys() {
        let pressed_keys: PressedKeyBits = std::array::from_fn(|_| AtomicU64::new(0));
        let (x_slot, x_bit) = vk_to_slot_bit(0x58).unwrap();
        let (num8_slot, num8_bit) = vk_to_slot_bit(0x68).unwrap();
        pressed_keys[x_slot].fetch_or(x_bit, Ordering::SeqCst);
        pressed_keys[num8_slot].fetch_or(num8_bit, Ordering::SeqCst);

        retain_pressed_keys(&pressed_keys, &HashSet::from([0x68]));
        let snapshot = pressed_key_snapshot(&pressed_keys);

        assert!(!pressed_key_snapshot_contains(&snapshot, x_slot, x_bit));
        assert!(pressed_key_snapshot_contains(
            &snapshot, num8_slot, num8_bit
        ));
    }

    #[test]
    fn active_toggle_keys_reads_runtime_toggle_state_only() {
        let engine = AutoFireEngine::new();
        engine.set_key_configs(vec![
            FireKeyConfig {
                vk: 0x58,
                interval_ms: 20,
                mode: FireKeyMode::Toggle,
            },
            FireKeyConfig {
                vk: 0x5A,
                interval_ms: 20,
                mode: FireKeyMode::Hold,
            },
        ]);

        let (toggle_slot, toggle_bit) = vk_to_slot_bit(0x58).unwrap();
        let (hold_slot, hold_bit) = vk_to_slot_bit(0x5A).unwrap();
        engine.toggle_active_keys[toggle_slot].store(toggle_bit, Ordering::SeqCst);
        engine.toggle_active_keys[hold_slot].fetch_or(hold_bit, Ordering::SeqCst);

        assert!(engine.active_toggle_keys().is_empty());
        engine.enabled.store(true, Ordering::SeqCst);

        assert_eq!(engine.active_toggle_keys(), vec![0x58]);
    }
}

#[cfg(windows)]
mod windows_impl {
    use super::*;
    use std::thread::{self, JoinHandle};
    use std::time::{Duration, Instant};

    use crate::platform::hook::{KeyboardHookEvent, KeyboardHookRunner};
    use crate::platform::keyboard::KeyboardDriver;

    const MIN_KEY_HOLD_MS: u64 = 8;
    const MAX_KEY_HOLD_MS: u64 = 15;
    const SEND_STATS_LOG_INTERVAL: Duration = Duration::from_secs(1);

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
            key_modes: Arc<RwLock<HashMap<u16, FireKeyMode>>>,
            pressed_keys: Arc<PressedKeyBits>,
            toggle_active_keys: Arc<PressedKeyBits>,
        ) -> AppResult<()> {
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
                    let key_modes = key_modes.clone();
                    let pressed_keys = pressed_keys.clone();
                    let toggle_active_keys = toggle_active_keys.clone();
                    move || {
                        HOOK_ENABLED.store(true, Ordering::SeqCst);
                        *HOOK_PRESSED_KEYS.write() = Some(pressed_keys);
                        *HOOK_TOGGLE_ACTIVE_KEYS.write() = Some(toggle_active_keys);
                        *HOOK_ENABLED_FLAG.write() = Some(enabled);
                        *HOOK_CONFIGURED_KEYS_REF.write() = Some(configured_keys);
                        *HOOK_KEY_MODES_REF.write() = Some(key_modes);
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
                    key_modes,
                    pressed_keys,
                    toggle_active_keys,
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
    static HOOK_TOGGLE_ACTIVE_KEYS: once_cell::sync::Lazy<RwLock<Option<Arc<PressedKeyBits>>>> =
        once_cell::sync::Lazy::new(|| RwLock::new(None));
    static HOOK_ENABLED_FLAG: once_cell::sync::Lazy<RwLock<Option<Arc<AtomicBool>>>> =
        once_cell::sync::Lazy::new(|| RwLock::new(None));
    #[allow(clippy::type_complexity)]
    static HOOK_CONFIGURED_KEYS_REF: once_cell::sync::Lazy<
        RwLock<Option<Arc<RwLock<HashSet<u16>>>>>,
    > = once_cell::sync::Lazy::new(|| RwLock::new(None));
    #[allow(clippy::type_complexity)]
    static HOOK_KEY_MODES_REF: once_cell::sync::Lazy<
        RwLock<Option<Arc<RwLock<HashMap<u16, FireKeyMode>>>>>,
    > = once_cell::sync::Lazy::new(|| RwLock::new(None));
    static HOOK_WINDOW_DETECTOR: once_cell::sync::Lazy<WindowsWindowDetector> =
        once_cell::sync::Lazy::new(WindowsWindowDetector::new);

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

                            if key_mode(event.vk) == FireKeyMode::Toggle {
                                if !is_target_active_for_toggle() {
                                    tracing::debug!(
                                        vk = %format_vk(event.vk),
                                        "忽略非目标窗口的切换连发按键"
                                    );
                                } else {
                                    let toggle_keys = HOOK_TOGGLE_ACTIVE_KEYS.read();
                                    if let Some(ref toggle_keys) = *toggle_keys {
                                        let toggle_slot = &toggle_keys[slot_index];
                                        let old_toggle =
                                            toggle_slot.fetch_xor(key_bit, Ordering::SeqCst);
                                        tracing::debug!(
                                            vk = %format_vk(event.vk),
                                            active = old_toggle & key_bit == 0,
                                            "切换连发状态"
                                        );
                                    }
                                }
                            }
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

    fn key_mode(vk: u16) -> FireKeyMode {
        HOOK_KEY_MODES_REF
            .read()
            .as_ref()
            .and_then(|modes| modes.read().get(&vk).copied())
            .unwrap_or_default()
    }

    fn is_target_active_for_toggle() -> bool {
        HOOK_WINDOW_DETECTOR.is_target_active()
    }

    fn autofire_loop(
        enabled: Arc<AtomicBool>,
        configured_keys: Arc<RwLock<HashSet<u16>>>,
        key_intervals: Arc<RwLock<HashMap<u16, u64>>>,
        key_modes: Arc<RwLock<HashMap<u16, FireKeyMode>>>,
        pressed_keys: Arc<PressedKeyBits>,
        toggle_active_keys: Arc<PressedKeyBits>,
        stop_signal: Arc<AtomicBool>,
    ) {
        tracing::info!("连发线程已启动");
        set_thread_priority_high();

        let keyboard = WindowsKeyboardDriver::new();
        let window = WindowsWindowDetector::new();
        let mut next_fire_at: HashMap<u16, Instant> = HashMap::new();
        let mut pending_key_ups: HashMap<u16, (u16, Instant)> = HashMap::new();
        let mut send_counts: HashMap<u16, u64> = HashMap::new();
        let mut send_stats_started_at = Instant::now();
        let mut draining = false;

        loop {
            if stop_signal.load(Ordering::SeqCst) {
                draining = true;
            }

            let now = Instant::now();
            if now.duration_since(send_stats_started_at) >= SEND_STATS_LOG_INTERVAL {
                log_send_stats(send_stats_started_at, now, &send_counts);
                send_counts.clear();
                send_stats_started_at = now;
            }

            let due_up_keys: Vec<u16> = pending_key_ups
                .iter()
                .filter_map(|(&vk, &(_, due_at))| (due_at <= now).then_some(vk))
                .collect();
            for vk in due_up_keys {
                if let Some((sc, _)) = pending_key_ups.remove(&vk) {
                    keyboard.send_game_key_up(vk, sc);
                }
            }

            if stop_signal.load(Ordering::SeqCst) && pending_key_ups.is_empty() {
                break;
            }

            if !enabled.load(Ordering::SeqCst) {
                next_fire_at.clear();
                let sleep_for =
                    next_pending_delay(None, pending_key_ups.values().map(|&(_, due_at)| due_at));
                sleep_for_pending_ups(sleep_for, Duration::from_millis(10));
                continue;
            }

            if !window.is_target_active() {
                log_waiting_window(&window);
                next_fire_at.clear();
                let sleep_for =
                    next_pending_delay(None, pending_key_ups.values().map(|&(_, due_at)| due_at));
                sleep_for_pending_ups(sleep_for, Duration::from_millis(50));
                continue;
            }

            if draining {
                let sleep_for =
                    next_pending_delay(None, pending_key_ups.values().map(|&(_, due_at)| due_at));
                sleep_for_pending_ups(sleep_for, Duration::from_millis(1));
                continue;
            }

            let keys_with_sc: Vec<(u16, u16, usize, u64, u64, FireKeyMode)> = {
                let configured = configured_keys.read();
                let intervals = key_intervals.read();
                let modes = key_modes.read();
                configured
                    .iter()
                    .filter_map(|&vk| {
                        let sc = keyboard.vk_to_scan_code(vk);
                        let (slot_index, bit) = vk_to_slot_bit(vk)?;
                        let interval = intervals.get(&vk).copied().unwrap_or(33);
                        let mode = modes.get(&vk).copied().unwrap_or_default();
                        Some((vk, sc, slot_index, bit, interval, mode))
                    })
                    .collect()
            };

            let pressed = pressed_key_snapshot(&pressed_keys);
            let toggle_active = pressed_key_snapshot(&toggle_active_keys);
            if pressed_key_snapshot_has_any(&pressed)
                || pressed_key_snapshot_has_any(&toggle_active)
            {
                // 先抓快照再发送按键，避免发送过程中钩子状态被模拟输入反复改写。
                next_fire_at.retain(|vk, _| {
                    keys_with_sc
                        .iter()
                        .any(|(key_vk, _, slot_index, bit, _, mode)| {
                            key_vk == vk
                                && fire_key_is_active(
                                    *mode,
                                    &pressed,
                                    &toggle_active,
                                    *slot_index,
                                    *bit,
                                )
                        })
                });

                let mut has_active_key = false;
                let mut next_down_deadline: Option<Instant> = None;
                for &(vk, sc, slot_index, bit, interval_ms, mode) in &keys_with_sc {
                    if !fire_key_is_active(mode, &pressed, &toggle_active, slot_index, bit) {
                        continue;
                    }
                    has_active_key = true;
                    if let Some((_, due_at)) = pending_key_ups.get(&vk) {
                        next_down_deadline = combine_deadline(next_down_deadline, Some(*due_at));
                        continue;
                    }
                    if let Some(next_fire_at_at) = next_fire_at.get(&vk).copied() {
                        next_down_deadline =
                            combine_deadline(next_down_deadline, Some(next_fire_at_at));
                        if now < next_fire_at_at {
                            continue;
                        }
                    }

                    let previous_fire_at = next_fire_at.get(&vk).copied();
                    let fire_started_at = Instant::now();
                    keyboard.send_game_key_down(vk, sc);
                    *send_counts.entry(vk).or_default() += 1;
                    pending_key_ups.insert(
                        vk,
                        (
                            sc,
                            fire_started_at + Duration::from_millis(key_hold_ms(interval_ms)),
                        ),
                    );
                    next_fire_at.insert(
                        vk,
                        next_theoretical_fire_at(previous_fire_at, fire_started_at, interval_ms),
                    );
                    next_down_deadline =
                        combine_deadline(next_down_deadline, next_fire_at.get(&vk).copied());
                }
                if !has_active_key {
                    next_fire_at.clear();
                }
                let next_up_deadline = pending_key_ups.values().map(|&(_, due_at)| due_at);
                let sleep_for = next_pending_delay(next_down_deadline, next_up_deadline);
                sleep_for_pending_ups(sleep_for, Duration::from_millis(1));
            } else {
                next_fire_at.clear();
                let sleep_for =
                    next_pending_delay(None, pending_key_ups.values().map(|&(_, due_at)| due_at));
                sleep_for_pending_ups(sleep_for, Duration::from_millis(1));
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

    fn log_send_stats(started_at: Instant, ended_at: Instant, send_counts: &HashMap<u16, u64>) {
        if send_counts.is_empty() {
            return;
        }

        let total: u64 = send_counts.values().sum();
        tracing::debug!(
            total,
            elapsed_ms = ended_at.duration_since(started_at).as_millis(),
            counts = %format_send_counts(send_counts),
            "按键连发发送统计"
        );
    }

    fn format_send_counts(send_counts: &HashMap<u16, u64>) -> String {
        let mut counts = send_counts
            .iter()
            .map(|(&vk, &count)| (vk, count))
            .collect::<Vec<_>>();
        counts.sort_by_key(|(vk, _)| *vk);
        counts
            .into_iter()
            .map(|(vk, count)| format!("{}={count}", format_vk(vk)))
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn next_theoretical_fire_at(
        previous_fire_at: Option<Instant>,
        fire_started_at: Instant,
        interval_ms: u64,
    ) -> Instant {
        let interval = Duration::from_millis(interval_ms);
        let mut next_fire_at = previous_fire_at.unwrap_or(fire_started_at) + interval;
        while next_fire_at <= fire_started_at {
            next_fire_at += interval;
        }
        next_fire_at
    }

    fn key_hold_ms(interval_ms: u64) -> u64 {
        (interval_ms / 3).clamp(MIN_KEY_HOLD_MS, MAX_KEY_HOLD_MS)
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

    fn combine_deadline(current: Option<Instant>, next: Option<Instant>) -> Option<Instant> {
        match (current, next) {
            (Some(current), Some(next)) => Some(current.min(next)),
            (None, Some(next)) => Some(next),
            (current, None) => current,
        }
    }

    fn next_pending_delay(
        next_down_deadline: Option<Instant>,
        next_up_deadlines: impl Iterator<Item = Instant>,
    ) -> Option<Duration> {
        let mut deadline = next_down_deadline;
        for due_at in next_up_deadlines {
            deadline = combine_deadline(deadline, Some(due_at));
        }
        deadline.map(|due_at| due_at.saturating_duration_since(Instant::now()))
    }

    fn sleep_for_pending_ups(delay: Option<Duration>, fallback: Duration) {
        match delay {
            Some(delay) if delay.is_zero() => thread::yield_now(),
            Some(delay) => thread::sleep(delay.min(Duration::from_millis(1))),
            None => thread::sleep(fallback),
        }
    }
}
