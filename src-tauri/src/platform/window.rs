//! 窗口检测模块
//!
//! 检测目标窗口（DNF 游戏或调试用记事本）是否处于活动状态。

/// DNF 游戏窗口的类名列表
#[cfg(windows)]
const TARGET_WINDOW_CLASSES: &[&str] = &[
    "地下城与勇士", // DNF 国服
    #[cfg(debug_assertions)]
    "Notepad", // 记事本（测试用）
];

#[cfg(windows)]
pub(crate) fn is_target_window_class_name(class_name: &str) -> bool {
    !class_name.is_empty()
        && TARGET_WINDOW_CLASSES
            .iter()
            .any(|target| class_name.contains(target))
}

#[cfg(windows)]
pub(crate) fn is_foreground_target_window_active() -> bool {
    foreground_target_window_handle().is_some()
}

#[cfg(windows)]
pub(crate) fn foreground_target_window_handle() -> Option<windows::Win32::Foundation::HWND> {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd == HWND::default() {
            return None;
        }

        is_target_window_class_name(&get_window_class_name(hwnd)).then_some(hwnd)
    }
}

#[cfg(windows)]
pub(crate) fn get_foreground_window_class_name() -> String {
    use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

    get_window_class_name(unsafe { GetForegroundWindow() })
}

#[cfg(windows)]
fn get_window_class_name(hwnd: windows::Win32::Foundation::HWND) -> String {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::GetClassNameW;

    unsafe {
        if hwnd == HWND::default() {
            return String::new();
        }

        let mut class_name_buf = [0u16; 256];
        let len = GetClassNameW(hwnd, &mut class_name_buf);
        if len == 0 {
            return String::new();
        }

        String::from_utf16_lossy(&class_name_buf[..len as usize])
    }
}

/// 窗口检测器接口
pub trait WindowDetector: Send + Sync + std::fmt::Debug {
    /// 检测目标窗口是否处于活动状态
    fn is_target_active(&self) -> bool;

    /// 获取当前前台窗口的类名
    fn get_foreground_class_name(&self) -> String;
}

// ============================================================================
// Windows 实现
// ============================================================================

#[cfg(windows)]
mod windows_impl {
    use super::*;

    /// Windows 窗口检测器
    #[derive(Debug, Default)]
    pub struct WindowsWindowDetector;

    impl WindowsWindowDetector {
        pub fn new() -> Self {
            Self
        }
    }

    impl WindowDetector for WindowsWindowDetector {
        fn is_target_active(&self) -> bool {
            let class_name = self.get_foreground_class_name();
            if class_name.is_empty() {
                return false;
            }

            let is_match = is_target_window_class_name(&class_name);

            // 只在前台窗口类名变化时打印，避免连发循环刷屏。
            static LAST_CLASS: std::sync::Mutex<String> = std::sync::Mutex::new(String::new());
            if let Ok(mut last) = LAST_CLASS.lock() {
                if *last != class_name {
                    tracing::info!(
                        class_name = %class_name,
                        matched = is_match,
                        "前台窗口变化"
                    );
                    tracing::debug!(
                        targets = ?TARGET_WINDOW_CLASSES,
                        "目标窗口列表"
                    );
                    *last = class_name.clone();
                }
            }

            is_match
        }

        fn get_foreground_class_name(&self) -> String {
            get_foreground_window_class_name()
        }
    }
}

#[cfg(windows)]
pub use windows_impl::WindowsWindowDetector;

#[cfg(all(windows, debug_assertions))]
#[cfg(test)]
mod tests {
    use super::is_target_window_class_name;

    #[test]
    fn debug_notepad_class_counts_as_target_window() {
        assert!(is_target_window_class_name("Notepad"));
    }
}
