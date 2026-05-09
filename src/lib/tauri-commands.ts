// 前端到 Tauri 后端的命令门面：桌面环境统一走 Tauri invoke。
import type {
  AppStateSnapshot,
  AutoRunConfig,
  BootstrapState,
  ComboDefinition,
  ComboValidationIssue,
  EffectRule,
  KeyBinding,
  RuntimeDiagnostics,
  SettingsConfig,
} from "../types/app-config";
import { normalizeAppError, TauriCommandError } from "../types/app-error";
import { isTauriEnvironment } from "./tauri-env";

async function invokeCommand<T>(name: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauriEnvironment()) {
    throw new TauriCommandError({
      kind: "transport",
      message: "当前能力只能在 Tauri 桌面环境中调用。",
    });
  }

  const { invoke } = await import("@tauri-apps/api/core");
  try {
    return await invoke<T>(name, args);
  } catch (reason) {
    throw normalizeAppError(reason);
  }
}

const setFloatingControlVisible = (visible: boolean) =>
  invokeCommand<AppStateSnapshot>("set_floating_control_visible", { visible });

export const tauriCommands = {
  loadBootstrap: () => invokeCommand<BootstrapState>("load_bootstrap"),
  loadRuntimeDiagnostics: () => invokeCommand<RuntimeDiagnostics>("load_runtime_diagnostics"),
  saveSettings: (settings: SettingsConfig) =>
    invokeCommand<AppStateSnapshot>("save_settings", { settings }),
  updateGlobalKeys: (keys: KeyBinding[]) =>
    invokeCommand<AppStateSnapshot>("update_global_keys", { keys }),
  updateProfileKeys: (configId: string, keys: KeyBinding[]) =>
    invokeCommand<AppStateSnapshot>("update_profile_keys", { configId, keys }),
  updateProfileEffectRule: (configId: string, effectRule: EffectRule) =>
    invokeCommand<AppStateSnapshot>("update_profile_effect_rule", { configId, effectRule }),
  updateProfileCombos: (configId: string, combos: ComboDefinition[]) =>
    invokeCommand<AppStateSnapshot>("update_profile_combos", { configId, combos }),
  updateAutoRun: (patch: Partial<AutoRunConfig>) =>
    invokeCommand<AppStateSnapshot>("update_auto_run", { patch }),
  addCustomConfig: (name: string) => invokeCommand<AppStateSnapshot>("add_custom_config", { name }),
  deleteCustomConfig: (configId: string) =>
    invokeCommand<AppStateSnapshot>("delete_custom_config", { configId }),
  setClassHidden: (classId: string, hidden: boolean) =>
    invokeCommand<AppStateSnapshot>("set_class_hidden", { classId, hidden }),
  selectActiveConfig: (activeClassId: string | null) =>
    invokeCommand<AppStateSnapshot>("select_active_config", { activeClassId }),
  setAssistantRunning: (running: boolean) =>
    invokeCommand<boolean>("set_assistant_running", { running }),
  validateComboDefs: (configId: string, combos: ComboDefinition[]) =>
    invokeCommand<ComboValidationIssue[]>("validate_combo_defs", { configId, combos }),
  isElevated: () => invokeCommand<boolean>("is_elevated"),
  showErrorMessage: (message: string) => invokeCommand<boolean>("show_error_message", { message }),
  restartAsAdmin: () => invokeCommand<boolean>("restart_as_admin"),
  setFloatingControlVisible,
  showFloatingControlWindow: () => setFloatingControlVisible(true),
  hideFloatingControlWindow: () => setFloatingControlVisible(false),
  updateFloatingControlPosition: (x: number, y: number) =>
    invokeCommand<AppStateSnapshot>("update_floating_control_position", { x, y }),
  minimizeMainWindow: () => invokeCommand<void>("minimize_main_window"),
  closeMainWindow: () => invokeCommand<void>("close_main_window"),
};
