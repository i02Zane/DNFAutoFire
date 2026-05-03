// 兼容层：集中转出配置类型、Tauri 命令和少量 UI 展示工具。
import { keyOptions } from "./keys";

export {
  type AppConfig,
  type AppSettings,
  type ComboAction,
  type ComboCommandAction,
  type ComboDefinition,
  type ComboTapAction,
  type ComboValidationIssue,
  type ClassConfig,
  type ClassDetectionResult,
  type CustomConfig,
  type DetectionDebugSnapshot,
  type DetectionNoMatchPolicy,
  type DetectionSettings,
  type EffectRule,
  type LogLevelSetting,
  type Hotkey,
  type ImageRect,
  type KeyBinding,
  makeClassConfig,
  makeCustomConfig,
} from "../types/app-config";
export { isMockMode, isTauriEnvironment } from "./tauri-env";
export { tauriCommands } from "./tauri-commands";

import { Hotkey } from "../types/app-config";

export function hotkeyDisplay(hotkey: Hotkey | null): string {
  if (!hotkey) return "未设置";
  const parts: string[] = [];
  if (hotkey.ctrl) parts.push("Ctrl");
  if (hotkey.alt) parts.push("Alt");
  if (hotkey.shift) parts.push("Shift");
  parts.push(keyOptions.find((option) => option.vk === hotkey.vk)?.label ?? `VK ${hotkey.vk}`);
  return parts.join(" + ");
}
