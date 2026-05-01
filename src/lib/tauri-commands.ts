// 前端到 Tauri 后端的命令门面：真实桌面环境走 invoke，浏览器预览走 mock。
import {
  hideFloatingControlWindow,
  showFloatingControlWindow,
} from "../floating-control/floating-control-manager";
import {
  AppConfig,
  ComboDefinition,
  Hotkey,
  KeyBinding,
  LogLevelSetting,
} from "../types/app-config";
import { isTauriEnvironment } from "./tauri-env";
import { mockInvoke } from "./mock-tauri";

async function invokeCommand<T>(name: string, args?: Record<string, unknown>): Promise<T> {
  if (isTauriEnvironment()) {
    // 动态导入让 Vite 浏览器预览不必解析 Tauri 运行时模块。
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<T>(name, args);
  }

  return mockInvoke<T>(name, args);
}

export const tauriCommands = {
  loadAppConfig: () => invokeCommand<AppConfig>("load_app_config"),
  saveAppConfig: (config: AppConfig) => invokeCommand<AppConfig>("save_app_config", { config }),
  setRuntimeKeys: (keys: KeyBinding[]) => invokeCommand<boolean>("set_runtime_keys", { keys }),
  setRuntimeProfile: (keys: KeyBinding[], combos: ComboDefinition[]) =>
    invokeCommand<boolean>("set_runtime_profile", { keys, combos }),
  startAssistant: (keys: KeyBinding[], combos: ComboDefinition[]) =>
    invokeCommand<boolean>("start_assistant", { keys, combos }),
  stopAssistant: () => invokeCommand<boolean>("stop_assistant"),
  isAssistantRunning: () => invokeCommand<boolean>("is_assistant_running"),
  startAutofire: (keys: KeyBinding[]) => invokeCommand<boolean>("start_autofire", { keys }),
  stopAutofire: () => invokeCommand<boolean>("stop_autofire"),
  isRunning: () => invokeCommand<boolean>("is_running"),
  registerToggleHotkey: (hotkey: Hotkey | null) =>
    invokeCommand<boolean>("register_toggle_hotkey", { hotkey }),
  updateTrayCurrentConfig: (label: string) =>
    invokeCommand<boolean>("update_tray_current_config", { label }),
  setLogLevel: (logLevel: LogLevelSetting) => invokeCommand<boolean>("set_log_level", { logLevel }),
  isElevated: () => invokeCommand<boolean>("is_elevated"),
  showErrorMessage: (message: string) => invokeCommand<boolean>("show_error_message", { message }),
  restartAsAdmin: () => invokeCommand<boolean>("restart_as_admin"),
  setLaunchAtStartup: (enabled: boolean) =>
    invokeCommand<boolean>("set_launch_at_startup", { enabled }),
  showFloatingControlWindow,
  hideFloatingControlWindow,
};
