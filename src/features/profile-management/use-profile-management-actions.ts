import { useCallback } from "react";
import { tauriCommands } from "../../lib/tauri-commands";
import type { AppStoreContextValue } from "../../store/app-store-context";
import type { ProfileDisplaySnapshot, ProfilesConfig } from "../../types/app-config";

type UseProfileManagementActionsOptions = {
  comboClassId: string | null;
  mutateSnapshot: AppStoreContextValue["mutateSnapshot"];
  profileDisplay: ProfileDisplaySnapshot;
  profiles: ProfilesConfig;
  setComboClassId: (configId: string | null) => void;
  setTarget: (target: { type: "global" } | { type: "profile"; configId: string } | null) => void;
  showMessage: (message: string) => void;
  target: { type: "global" } | { type: "profile"; configId: string } | null;
};

export function useProfileManagementActions({
  comboClassId,
  mutateSnapshot,
  profileDisplay,
  profiles,
  setComboClassId,
  setTarget,
  showMessage,
  target,
}: UseProfileManagementActionsOptions) {
  const addCustomConfig = useCallback(
    (name: string) => {
      const trimmedName = name.trim();
      if (!trimmedName) {
        showMessage("自定义配置名称不能为空");
        return false;
      }
      void mutateSnapshot(() => tauriCommands.addCustomConfig(trimmedName));
      return true;
    },
    [mutateSnapshot, showMessage],
  );

  const deleteCustomConfig = useCallback(
    (configId: string) => {
      const customConfig = profiles.customConfigs[configId];
      if (!customConfig) return;

      void mutateSnapshot(() => tauriCommands.deleteCustomConfig(configId));
      if (target?.type === "profile" && target.configId === configId) {
        setTarget(null);
      }
      if (comboClassId === configId) {
        setComboClassId(null);
      }
    },
    [comboClassId, mutateSnapshot, profiles.customConfigs, setComboClassId, setTarget, target],
  );

  const toggleClassHidden = useCallback(
    (classId: string, hidden: boolean) => {
      if (!(profileDisplay.classStates[classId]?.canHide ?? true)) return;
      void mutateSnapshot(() => tauriCommands.setClassHidden(classId, hidden));
    },
    [mutateSnapshot, profileDisplay.classStates],
  );

  return {
    addCustomConfig,
    deleteCustomConfig,
    toggleClassHidden,
  };
}
