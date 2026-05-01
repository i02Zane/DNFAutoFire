//! Windows 全局启动/停止快捷键注册：独立消息线程监听 WM_HOTKEY。

use crate::assistant::AssistantRuntime;
use crate::config::Hotkey;
use crate::logging::format_hotkey;
use crate::notify::show_error_message_box;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;

pub(crate) struct HotkeyRegistration {
    thread_id: Arc<AtomicU32>,
    handle: Option<JoinHandle<()>>,
    summary: String,
}

impl Drop for HotkeyRegistration {
    fn drop(&mut self) {
        #[cfg(windows)]
        {
            // RegisterHotKey 绑定在线程消息循环上，注销前先投递 WM_QUIT 让线程退出。
            tracing::info!(hotkey = %self.summary, "注销全局启动/停止快捷键");
            let thread_id = self.thread_id.load(Ordering::SeqCst);
            if thread_id != 0 {
                unsafe {
                    if let Err(error) = windows::Win32::UI::WindowsAndMessaging::PostThreadMessageW(
                        thread_id,
                        windows::Win32::UI::WindowsAndMessaging::WM_QUIT,
                        windows::Win32::Foundation::WPARAM(0),
                        windows::Win32::Foundation::LPARAM(0),
                    ) {
                        tracing::warn!(
                            hotkey = %self.summary,
                            error = %error,
                            "投递全局快捷键退出消息失败"
                        );
                    }
                }
            }
            if let Some(handle) = self.handle.take() {
                if handle.join().is_err() {
                    tracing::warn!(hotkey = %self.summary, "全局快捷键线程异常退出");
                }
            }
        }
    }
}

pub(crate) fn validate_hotkey(hotkey: &Hotkey) -> Result<(), String> {
    if !(hotkey.ctrl || hotkey.alt || hotkey.shift) || is_modifier_vk(hotkey.vk) {
        return Err("启动/停止快捷键必须是组合键，例如 Ctrl + F8。".to_string());
    }
    Ok(())
}

fn is_modifier_vk(vk: u16) -> bool {
    vk == 0x10 || vk == 0x11 || vk == 0x12 || (0xA0..=0xA5).contains(&vk)
}

#[cfg(windows)]
pub(crate) fn register_windows_hotkey(
    hotkey: Hotkey,
    assistant_runtime: AssistantRuntime,
) -> Result<HotkeyRegistration, String> {
    use std::sync::mpsc;
    use std::thread;
    use windows::Win32::Foundation::WPARAM;
    use windows::Win32::System::Threading::GetCurrentThreadId;
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT,
        MOD_SHIFT,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, GetMessageW, TranslateMessage, MSG, WM_HOTKEY,
    };

    const TOGGLE_HOTKEY_ID: i32 = 1;

    let hotkey_summary = format_hotkey(hotkey.ctrl, hotkey.alt, hotkey.shift, hotkey.vk);
    let hotkey_summary_for_thread = hotkey_summary.clone();
    let thread_id = Arc::new(AtomicU32::new(0));
    let thread_id_for_thread = thread_id.clone();
    let (ready_tx, ready_rx) = mpsc::channel();

    let handle = thread::spawn(move || {
        let mut modifiers = HOT_KEY_MODIFIERS(0);
        if hotkey.ctrl {
            modifiers |= MOD_CONTROL;
        }
        if hotkey.alt {
            modifiers |= MOD_ALT;
        }
        if hotkey.shift {
            modifiers |= MOD_SHIFT;
        }
        modifiers |= MOD_NOREPEAT;

        unsafe {
            thread_id_for_thread.store(GetCurrentThreadId(), Ordering::SeqCst);
        }

        // 注册结果通过 channel 回到调用线程，避免 UI 以为快捷键已生效但实际失败。
        let registered =
            unsafe { RegisterHotKey(None, TOGGLE_HOTKEY_ID, modifiers, hotkey.vk as u32) };
        if let Err(error) = registered {
            tracing::error!(
                hotkey = %hotkey_summary_for_thread,
                error = %error,
                "注册全局启动/停止快捷键失败"
            );
            let _ = ready_tx.send(Err(format!("注册启动/停止快捷键失败: {error}")));
            return;
        }
        let _ = ready_tx.send(Ok(()));
        tracing::info!(
            hotkey = %hotkey_summary_for_thread,
            "全局启动/停止快捷键已注册"
        );

        let mut msg = MSG::default();
        while unsafe { GetMessageW(&mut msg, None, 0, 0) }.as_bool() {
            if msg.message == WM_HOTKEY && msg.wParam == WPARAM(TOGGLE_HOTKEY_ID as usize) {
                tracing::info!(
                    hotkey = %hotkey_summary_for_thread,
                    "全局启动/停止快捷键触发"
                );
                if let Err(error) = assistant_runtime.toggle_from_runtime_profile() {
                    tracing::warn!(error = %error, "全局快捷键切换助手状态失败");
                    show_error_message_box(&error);
                }
                continue;
            }
            unsafe {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }

        unsafe {
            if let Err(error) = UnregisterHotKey(None, TOGGLE_HOTKEY_ID) {
                tracing::warn!(
                    hotkey = %hotkey_summary_for_thread,
                    error = %error,
                    "注销全局启动/停止快捷键失败"
                );
            } else {
                tracing::info!(
                    hotkey = %hotkey_summary_for_thread,
                    "全局启动/停止快捷键已注销"
                );
            }
        }
    });

    ready_rx.recv().map_err(|e| {
        let message = format!("注册启动/停止快捷键失败: {e}");
        tracing::error!(
            hotkey = %hotkey_summary,
            error = %message,
            "全局快捷键线程启动失败"
        );
        message
    })??;

    Ok(HotkeyRegistration {
        thread_id,
        handle: Some(handle),
        summary: hotkey_summary,
    })
}
