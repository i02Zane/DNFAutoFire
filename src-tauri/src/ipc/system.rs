use crate::platform::notify::show_error_message_box;

#[tauri::command]
pub(crate) fn is_elevated() -> bool {
    #[cfg(windows)]
    {
        use windows::Win32::Foundation::{CloseHandle, HANDLE};
        use windows::Win32::Security::{
            GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
        };
        use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

        unsafe {
            let mut token_handle = HANDLE::default();
            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token_handle).is_err() {
                return false;
            }

            let mut elevation = TOKEN_ELEVATION::default();
            let mut size = std::mem::size_of::<TOKEN_ELEVATION>() as u32;
            let result = GetTokenInformation(
                token_handle,
                TokenElevation,
                Some(&mut elevation as *mut _ as *mut _),
                size,
                &mut size,
            );
            let _ = CloseHandle(token_handle);
            result.is_ok() && elevation.TokenIsElevated != 0
        }
    }

    #[cfg(not(windows))]
    {
        false
    }
}

#[tauri::command]
pub(crate) fn show_error_message(message: String) -> bool {
    show_error_message_box(&message)
}

#[tauri::command]
pub(crate) fn restart_as_admin() -> bool {
    tracing::info!("请求以管理员权限重启");
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use windows::core::PCWSTR;
        use windows::Win32::UI::Shell::ShellExecuteW;
        use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

        let exe_path = std::env::current_exe().unwrap_or_default();
        let exe_str: Vec<u16> = exe_path.as_os_str().encode_wide().chain(Some(0)).collect();
        let verb: Vec<u16> = "runas\0".encode_utf16().collect();

        unsafe {
            let result = ShellExecuteW(
                None,
                PCWSTR(verb.as_ptr()),
                PCWSTR(exe_str.as_ptr()),
                PCWSTR::null(),
                PCWSTR::null(),
                SW_SHOWNORMAL,
            );

            if result.0 as usize > 32 {
                tracing::info!("已发起管理员重启");
                std::process::exit(0);
            }
            tracing::warn!("管理员重启失败");
            false
        }
    }

    #[cfg(not(windows))]
    {
        false
    }
}
