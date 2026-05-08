import { useCallback, useEffect } from "react";
import {
  type ComboDefinition,
  type Hotkey,
  type KeyBinding,
  isMockMode,
  tauriCommands,
} from "../lib/tauri";

type UseAssistantRuntimeOptions = {
  autoRunEnabled: boolean;
  autoRunLeftVk: number;
  autoRunPulseDelayMs: number;
  autoRunRightVk: number;
  currentConfigLabel: string;
  effectiveCombos: ComboDefinition[];
  effectiveKeys: KeyBinding[];
  setActiveToggleKeys: (activeToggleKeys: number[]) => void;
  running: boolean;
  setRunning: (running: boolean) => void;
  showMessage: (message: string) => void;
  startupConfigLoaded: boolean;
  toggleHotkey: Hotkey | null;
};

export function useAssistantRuntime({
  autoRunEnabled,
  autoRunLeftVk,
  autoRunPulseDelayMs,
  autoRunRightVk,
  currentConfigLabel,
  effectiveCombos,
  effectiveKeys,
  setActiveToggleKeys,
  running,
  setRunning,
  showMessage,
  startupConfigLoaded,
  toggleHotkey,
}: UseAssistantRuntimeOptions) {
  const refreshRuntimeState = useCallback(async () => {
    const [isRunning, activeToggleKeys] = await Promise.all([
      tauriCommands.isAssistantRunning(),
      tauriCommands.activeAutofireToggleKeys(),
    ]);
    setRunning(isRunning);
    setActiveToggleKeys(isRunning ? activeToggleKeys : []);
  }, [setActiveToggleKeys, setRunning]);

  // 后端也可能通过全局快捷键改变运行态，因此主窗口不能只相信自己的按钮点击。
  useEffect(() => {
    if (isMockMode() || !startupConfigLoaded) return;

    void refreshRuntimeState().catch(() => undefined);

    // 全局快捷键也能改变后端状态，主窗口用轻量轮询同步显示状态。
    const timer = window.setInterval(() => {
      void refreshRuntimeState().catch(() => undefined);
    }, 500);
    return () => window.clearInterval(timer);
  }, [refreshRuntimeState, startupConfigLoaded]);

  useEffect(() => {
    if (isMockMode() || !startupConfigLoaded) return;

    // 托盘文案是后端菜单状态，随主窗口当前生效配置同步即可。
    void tauriCommands.updateTrayCurrentConfig(currentConfigLabel).catch(() => undefined);
  }, [currentConfigLabel, startupConfigLoaded]);

  useEffect(() => {
    if (!startupConfigLoaded || !running) return;
    // 运行中刷新配置时，后端会更新连发键和当前职业连招快照。
    void tauriCommands
      .startAssistant(effectiveKeys, effectiveCombos)
      .then(refreshRuntimeState)
      .catch((reason) => {
        setRunning(false);
        setActiveToggleKeys([]);
        showMessage(reason instanceof Error ? reason.message : String(reason));
      });
  }, [
    autoRunEnabled,
    autoRunLeftVk,
    autoRunPulseDelayMs,
    autoRunRightVk,
    effectiveCombos,
    effectiveKeys,
    refreshRuntimeState,
    running,
    setActiveToggleKeys,
    setRunning,
    showMessage,
    startupConfigLoaded,
  ]);

  useEffect(() => {
    if (!startupConfigLoaded) return;

    // 后端快捷键启动时只读运行时快照，因此配置变更后要同步一份当前配置。
    void tauriCommands.setRuntimeProfile(effectiveKeys, effectiveCombos).catch((reason) => {
      showMessage(reason instanceof Error ? reason.message : String(reason));
    });
  }, [
    autoRunEnabled,
    autoRunLeftVk,
    autoRunPulseDelayMs,
    autoRunRightVk,
    effectiveCombos,
    effectiveKeys,
    showMessage,
    startupConfigLoaded,
  ]);

  useEffect(() => {
    if (!startupConfigLoaded) return;

    // 注册 null 等价于清除后端当前快捷键，保持前后端设置一致。
    void tauriCommands.registerToggleHotkey(toggleHotkey).catch((reason) => {
      showMessage(reason instanceof Error ? reason.message : String(reason));
    });
  }, [showMessage, startupConfigLoaded, toggleHotkey]);

  return useCallback(async () => {
    try {
      if (running) {
        await tauriCommands.stopAssistant();
      } else {
        await tauriCommands.startAssistant(effectiveKeys, effectiveCombos);
      }
      await refreshRuntimeState();
    } catch (reason) {
      setActiveToggleKeys([]);
      showMessage(reason instanceof Error ? reason.message : String(reason));
    }
  }, [
    effectiveCombos,
    effectiveKeys,
    refreshRuntimeState,
    running,
    setActiveToggleKeys,
    showMessage,
  ]);
}
