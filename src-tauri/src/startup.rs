//! Windows 开机自启动：通过当前用户 Run 注册表项写入或移除应用路径。

use crate::APP_NAME;

#[cfg(windows)]
pub(crate) fn set_windows_launch_at_startup(enabled: bool) -> Result<(), String> {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x08000000;
    const RUN_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";

    tracing::info!(enabled, "更新开机启动设置");
    if enabled {
        // 写入带引号的 exe 路径，防止安装目录包含空格时启动失败。
        let exe_path = std::env::current_exe().map_err(|e| {
            let message = format!("获取当前程序路径失败: {e}");
            tracing::error!(error = %message, "获取当前程序路径失败");
            message
        })?;
        let launch_command = format!("\"{}\"", exe_path.to_string_lossy());
        let status = std::process::Command::new("reg")
            .args([
                "add",
                RUN_KEY,
                "/v",
                APP_NAME,
                "/t",
                "REG_SZ",
                "/d",
                &launch_command,
                "/f",
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .status()
            .map_err(|e| {
                let message = format!("更新开机启动项失败: {e}");
                tracing::error!(error = %message, "更新开机启动项失败");
                message
            })?;
        if !status.success() {
            let message = format!("更新开机启动项失败，reg.exe 退出码: {:?}", status.code());
            tracing::error!(error = %message, "更新开机启动项失败");
            return Err(message);
        }

        tracing::info!(path = %exe_path.display(), "已启用开机启动");
        return Ok(());
    }

    delete_windows_startup_value(RUN_KEY, APP_NAME, CREATE_NO_WINDOW)?;
    tracing::info!("已关闭开机启动");
    Ok(())
}

#[cfg(windows)]
fn delete_windows_startup_value(
    run_key: &str,
    app_name: &str,
    creation_flags: u32,
) -> Result<(), String> {
    use std::os::windows::process::CommandExt;

    let query_status = std::process::Command::new("reg")
        .args(["query", run_key, "/v", app_name])
        .creation_flags(creation_flags)
        .status()
        .map_err(|e| {
            let message = format!("查询开机启动项失败: {e}");
            tracing::error!(error = %message, "查询开机启动项失败");
            message
        })?;
    if !query_status.success() {
        // 没有旧值时视为删除成功，便于设置开关重复关闭。
        tracing::debug!("未找到旧的开机启动项");
        return Ok(());
    }

    let status = std::process::Command::new("reg")
        .args(["delete", run_key, "/v", app_name, "/f"])
        .creation_flags(creation_flags)
        .status()
        .map_err(|e| {
            let message = format!("更新开机启动项失败: {e}");
            tracing::error!(error = %message, "更新开机启动项失败");
            message
        })?;
    if status.success() {
        tracing::info!("已移除开机启动项");
        return Ok(());
    }

    let message = format!("更新开机启动项失败，reg.exe 退出码: {:?}", status.code());
    tracing::error!(error = %message, "更新开机启动项失败");
    Err(message)
}
