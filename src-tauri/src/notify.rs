//! 用户可见提示入口：统一封装系统级错误弹窗和非 Windows 回退行为。

use crate::APP_NAME;

pub(crate) fn show_error_message_box(message: &str) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use windows::core::PCWSTR;
        use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};

        let title: Vec<u16> = APP_NAME.encode_utf16().chain(Some(0)).collect();
        let body: Vec<u16> = std::ffi::OsStr::new(message.trim())
            .encode_wide()
            .chain(Some(0))
            .collect();
        unsafe {
            let _ = MessageBoxW(
                None,
                PCWSTR(body.as_ptr()),
                PCWSTR(title.as_ptr()),
                MB_OK | MB_ICONERROR,
            );
        }
        true
    }

    #[cfg(not(windows))]
    {
        tracing::error!(message = %message.trim(), "显示错误消息失败，当前平台不支持消息框");
        false
    }
}
