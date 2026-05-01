//! Windows 键盘钩子线程生命周期封装。

#[cfg(windows)]
mod windows_impl {
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
    use std::sync::mpsc;
    use std::sync::Arc;
    use std::thread::{self, JoinHandle};
    use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::System::Threading::GetCurrentThreadId;
    use windows::Win32::UI::WindowsAndMessaging::{
        GetMessageW, PeekMessageW, PostThreadMessageW, SetWindowsHookExW, UnhookWindowsHookEx,
        KBDLLHOOKSTRUCT, LLKHF_INJECTED, LLKHF_LOWER_IL_INJECTED, MSG, PM_NOREMOVE, WH_KEYBOARD_LL,
        WM_QUIT,
    };

    pub type KeyboardHookProc = unsafe extern "system" fn(i32, WPARAM, LPARAM) -> LRESULT;

    pub(crate) const WM_KEYDOWN: u32 = 0x0100;
    pub(crate) const WM_KEYUP: u32 = 0x0101;
    pub(crate) const WM_SYSKEYDOWN: u32 = 0x0104;
    pub(crate) const WM_SYSKEYUP: u32 = 0x0105;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) struct KeyboardHookEvent {
        pub(crate) vk: u16,
        pub(crate) message: u32,
        pub(crate) is_keydown: bool,
        pub(crate) is_keyup: bool,
        pub(crate) is_injected: bool,
    }

    impl KeyboardHookEvent {
        pub(crate) unsafe fn from_raw(wparam: WPARAM, lparam: LPARAM) -> Self {
            let kb = *(lparam.0 as *const KBDLLHOOKSTRUCT);
            let message = wparam.0 as u32;
            Self {
                vk: kb.vkCode as u16,
                message,
                is_keydown: is_keydown_message(message),
                is_keyup: is_keyup_message(message),
                is_injected: is_injected_keyboard_flags(kb.flags.0),
            }
        }
    }

    pub(crate) fn is_keydown_message(message: u32) -> bool {
        matches!(message, WM_KEYDOWN | WM_SYSKEYDOWN)
    }

    pub(crate) fn is_keyup_message(message: u32) -> bool {
        matches!(message, WM_KEYUP | WM_SYSKEYUP)
    }

    pub(crate) fn is_injected_keyboard_flags(flags: u32) -> bool {
        flags & LLKHF_INJECTED.0 != 0 || flags & LLKHF_LOWER_IL_INJECTED.0 != 0
    }

    pub struct KeyboardHookRunner {
        label: &'static str,
        thread_handle: Option<JoinHandle<()>>,
        stop_signal: Arc<AtomicBool>,
        thread_id: Arc<AtomicU32>,
    }

    impl KeyboardHookRunner {
        pub fn new(label: &'static str) -> Self {
            Self {
                label,
                thread_handle: None,
                stop_signal: Arc::new(AtomicBool::new(false)),
                thread_id: Arc::new(AtomicU32::new(0)),
            }
        }

        pub fn is_running(&self) -> bool {
            self.thread_handle.is_some()
        }

        pub fn stop_signal(&self) -> Arc<AtomicBool> {
            self.stop_signal.clone()
        }

        pub fn start<Setup, Cleanup>(
            &mut self,
            install_error: &'static str,
            hook_proc: KeyboardHookProc,
            setup: Setup,
            cleanup: Cleanup,
        ) -> Result<(), String>
        where
            Setup: FnOnce() + Send + 'static,
            Cleanup: FnOnce() + Send + 'static,
        {
            if self.thread_handle.is_some() {
                return Ok(());
            }

            self.stop_signal.store(false, Ordering::SeqCst);

            let label = self.label;
            let stop_signal = self.stop_signal.clone();
            let thread_id = self.thread_id.clone();
            let (ready_tx, ready_rx) = mpsc::channel();

            let handle = thread::spawn(move || {
                unsafe {
                    thread_id.store(GetCurrentThreadId(), Ordering::SeqCst);
                    // 线程消息队列是惰性创建的；ready 前先触碰队列，保证 stop 能投递 WM_QUIT。
                    let mut bootstrap_msg = MSG::default();
                    let _ = PeekMessageW(&mut bootstrap_msg, None, 0, 0, PM_NOREMOVE);
                }

                setup();

                unsafe {
                    let hmod = GetModuleHandleW(None)
                        .ok()
                        .map(|h| windows::Win32::Foundation::HINSTANCE(h.0));
                    let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), hmod, 0);

                    if let Ok(hook) = hook {
                        let _ = ready_tx.send(Ok(()));
                        tracing::info!(hook = label, "键盘钩子已安装");
                        let mut msg = MSG::default();
                        while !stop_signal.load(Ordering::SeqCst) {
                            if !GetMessageW(&mut msg, None, 0, 0).as_bool() {
                                break;
                            }
                        }

                        let _ = UnhookWindowsHookEx(hook);
                        tracing::info!(hook = label, "键盘钩子已卸载");
                    } else {
                        let error = install_error.to_string();
                        let _ = ready_tx.send(Err(error.clone()));
                        tracing::error!(hook = label, error = %error, "键盘钩子安装失败");
                    }
                }

                cleanup();
                thread_id.store(0, Ordering::SeqCst);
            });

            self.thread_handle = Some(handle);
            if let Err(error) = ready_rx
                .recv()
                .map_err(|e| format!("等待{}键盘钩子启动失败: {e}", self.label))
                .and_then(|result| result)
            {
                tracing::error!(hook = self.label, error = %error, "键盘钩子启动失败");
                self.stop();
                return Err(error);
            }

            Ok(())
        }

        pub fn stop(&mut self) {
            self.stop_signal.store(true, Ordering::SeqCst);
            let thread_id = self.thread_id.load(Ordering::SeqCst);
            if thread_id != 0 {
                unsafe {
                    if let Err(error) = PostThreadMessageW(thread_id, WM_QUIT, WPARAM(0), LPARAM(0))
                    {
                        tracing::warn!(
                            hook = self.label,
                            error = %error,
                            "投递键盘钩子退出消息失败"
                        );
                    }
                }
            }

            if let Some(handle) = self.thread_handle.take() {
                if handle.join().is_err() {
                    tracing::warn!(hook = self.label, "键盘钩子线程异常退出");
                }
            }
        }
    }

    impl Drop for KeyboardHookRunner {
        fn drop(&mut self) {
            self.stop();
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn system_key_messages_count_as_key_events() {
            assert!(is_keydown_message(WM_SYSKEYDOWN));
            assert!(is_keyup_message(WM_SYSKEYUP));
        }

        #[test]
        fn injected_flags_include_lower_integrity_injected() {
            assert!(is_injected_keyboard_flags(LLKHF_INJECTED.0));
            assert!(is_injected_keyboard_flags(LLKHF_LOWER_IL_INJECTED.0));
            assert!(!is_injected_keyboard_flags(0));
        }
    }
}

#[cfg(windows)]
pub(crate) use windows_impl::KeyboardHookEvent;
#[cfg(windows)]
pub use windows_impl::KeyboardHookRunner;
