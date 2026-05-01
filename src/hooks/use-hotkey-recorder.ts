import { useEffect } from "react";
import { browserKeyToVk, isModifierVk, isValidComboHotkey } from "../lib/browser-keys";
import { type AppConfig, type Hotkey } from "../lib/tauri";
import type { ConfigUpdater } from "./use-app-config";

type UseHotkeyRecorderOptions = {
  recordingHotkey: boolean;
  setRecordingHotkey: (recordingHotkey: boolean) => void;
  showMessage: (message: string) => void;
  updateConfig: (updater: ConfigUpdater) => Promise<AppConfig | null>;
};

export function useHotkeyRecorder({
  recordingHotkey,
  setRecordingHotkey,
  showMessage,
  updateConfig,
}: UseHotkeyRecorderOptions) {
  useEffect(() => {
    if (!recordingHotkey) return;

    // 录制快捷键时拦截全局 keydown，保存统一的 VK 码而不是浏览器 key 字符串。
    const onKeyDown = (event: KeyboardEvent) => {
      event.preventDefault();
      const vk = browserKeyToVk(event);
      if (!vk) return;
      if (isModifierVk(vk)) return;
      if (!isValidComboHotkey(event, vk)) {
        showMessage("启动/停止快捷键必须是组合键，例如 Ctrl + F8。");
        return;
      }
      const nextHotkey: Hotkey = {
        ctrl: event.ctrlKey,
        alt: event.altKey,
        shift: event.shiftKey,
        vk,
      };
      setRecordingHotkey(false);
      void updateConfig((currentConfig) => ({ ...currentConfig, toggleHotkey: nextHotkey }));
    };

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [recordingHotkey, setRecordingHotkey, showMessage, updateConfig]);
}
