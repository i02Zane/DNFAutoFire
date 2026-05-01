// 浏览器预览模式的 Tauri mock，支持前端在没有桌面壳时调试主要交互。
import { AppConfig } from "../types/app-config";

let mockConfig: AppConfig = {
  version: 6,
  globalKeys: [{ vk: 0x58, intervalMs: 20 }],
  comboDefs: [],
  classes: {
    male_slayer_blade_master: {
      enabledKeys: [
        { vk: 0x41, intervalMs: 20 },
        { vk: 0x53, intervalMs: 20 },
        { vk: 0x44, intervalMs: 15 },
        { vk: 0x46, intervalMs: 20 },
      ],
      effectRule: "globalAndClass",
      comboDefs: [
        {
          id: "mock-blade-master-combo-1",
          name: "示例连招",
          enabled: false,
          triggerVk: null,
          actions: [],
        },
      ],
    },
  },
  customConfigs: {
    "custom-mock-light-sword": {
      name: "剑魂光剑套",
      enabledKeys: [{ vk: 0x51, intervalMs: 30 }],
      effectRule: "globalAndClass",
      comboDefs: [],
    },
  },
  hiddenClassIds: [],
  activeClassId: null,
  toggleHotkey: { ctrl: true, alt: false, shift: false, vk: 0x77 },
  detection: {
    enabled: true,
    intervalMs: 5000,
    iconDatabaseVersion: "builtin-empty-v1",
  },
  settings: {
    launchAtStartup: false,
    startMinimized: false,
    minimizeToTray: false,
    openFloatingControlOnStart: false,
    logLevel: "debug",
  },
};

let mockRunning = false;

export async function mockInvoke<T>(name: string, args?: Record<string, unknown>): Promise<T> {
  // mock 只模拟前端需要的返回值，不触碰真实 Win32、托盘或文件系统。
  switch (name) {
    case "load_app_config":
      return structuredClone(mockConfig) as T;
    case "save_app_config":
      mockConfig = structuredClone(args?.config as AppConfig);
      return structuredClone(mockConfig) as T;
    case "set_runtime_keys":
      return true as T;
    case "set_runtime_profile":
      return true as T;
    case "start_assistant":
      mockRunning = true;
      return true as T;
    case "stop_assistant":
      mockRunning = false;
      return true as T;
    case "is_assistant_running":
      return mockRunning as T;
    case "start_autofire":
      mockRunning = true;
      return true as T;
    case "stop_autofire":
      mockRunning = false;
      return true as T;
    case "is_running":
      return mockRunning as T;
    case "is_elevated":
      return true as T;
    case "show_error_message":
      return true as T;
    case "restart_as_admin":
      return false as T;
    case "register_toggle_hotkey":
      return true as T;
    case "update_tray_current_config":
      return true as T;
    case "set_log_level":
      return true as T;
    case "set_launch_at_startup":
      return true as T;
    default:
      throw new Error(`Unknown mock command: ${name}`);
  }
}
