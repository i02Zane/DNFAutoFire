// 浏览器预览模式的 Tauri mock，支持前端在没有桌面壳时调试主要交互。
import { AppConfig } from "../types/app-config";
import type { ClassCategory } from "../types/class-catalog";

const mockClassCategories: ClassCategory[] = [
  {
    name: "鬼剑士(男)",
    classes: [{ id: "male_slayer_blade_master", name: "剑魂", detectionIndex: 0 }],
  },
];

let mockConfig: AppConfig = {
  version: 11,
  globalKeys: [{ vk: 0x58, intervalMs: 20, mode: "hold" }],
  comboDefs: [],
  classes: {
    male_slayer_blade_master: {
      enabledKeys: [
        { vk: 0x41, intervalMs: 20, mode: "hold" },
        { vk: 0x53, intervalMs: 20, mode: "hold" },
        { vk: 0x44, intervalMs: 15, mode: "hold" },
        { vk: 0x46, intervalMs: 20, mode: "hold" },
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
      enabledKeys: [{ vk: 0x51, intervalMs: 30, mode: "hold" }],
      effectRule: "globalAndClass",
      comboDefs: [],
    },
  },
  hiddenClassIds: [],
  activeClassId: null,
  toggleHotkey: { ctrl: true, alt: false, shift: false, vk: 0x77 },
  detection: {
    enabled: false,
    intervalMs: 200,
    noMatchPolicy: "current",
    iconDatabaseVersion: "builtin-empty-v1",
  },
  settings: {
    launchAtStartup: false,
    startMinimized: false,
    minimizeToTray: false,
    openFloatingControlOnStart: false,
    autoRunEnabled: false,
    autoRunLeftVk: 0x25,
    autoRunRightVk: 0x27,
    autoRunPulseDelayMs: 25,
    logLevel: "debug",
  },
};

let mockRunning = false;

function isMockSelectableConfigId(config: AppConfig, configId: string): boolean {
  const customConfig = config.customConfigs[configId];
  if (customConfig) {
    return customConfig.enabledKeys.length > 0 || customConfig.comboDefs.length > 0;
  }

  if (!Object.prototype.hasOwnProperty.call(config.classes, configId)) {
    return false;
  }

  const classConfig = config.classes[configId];
  return (classConfig?.enabledKeys.length ?? 0) > 0 || (classConfig?.comboDefs.length ?? 0) > 0;
}

export async function mockInvoke<T>(name: string, args?: Record<string, unknown>): Promise<T> {
  // mock 只模拟前端需要的返回值，不触碰真实 Win32、托盘或文件系统。
  switch (name) {
    case "load_app_config":
      return structuredClone(mockConfig) as T;
    case "load_class_categories":
      return structuredClone(mockClassCategories) as T;
    case "save_app_config":
      mockConfig = structuredClone(args?.config as AppConfig);
      return structuredClone(mockConfig) as T;
    case "select_active_config": {
      const requestedActiveClassId = (args?.activeClassId as string | null | undefined) ?? null;
      const normalizedActiveClassId =
        requestedActiveClassId && isMockSelectableConfigId(mockConfig, requestedActiveClassId)
          ? requestedActiveClassId
          : null;
      mockConfig = {
        ...mockConfig,
        activeClassId: normalizedActiveClassId,
      };
      return structuredClone(mockConfig) as T;
    }
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
    case "start_auto_run":
      return true as T;
    case "stop_auto_run":
      return true as T;
    case "is_auto_run_running":
      return false as T;
    case "start_detection":
      return true as T;
    case "stop_detection":
      return true as T;
    case "is_detection_running":
      return false as T;
    case "is_running":
      return mockRunning as T;
    case "active_autofire_toggle_keys":
      return [] as T;
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
