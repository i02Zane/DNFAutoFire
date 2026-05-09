import { useCallback } from "react";
import { tauriCommands } from "../../lib/tauri-commands";
import type { AppStoreContextValue } from "../../store/app-store-context";
import type { ComboDefinition } from "../../types/app-config";

type UseComboActionsOptions = {
  mutateSnapshot: AppStoreContextValue["mutateSnapshot"];
};

export function useComboActions({ mutateSnapshot }: UseComboActionsOptions) {
  const validateComboDefs = useCallback(
    (configId: string, combos: ComboDefinition[]) =>
      tauriCommands.validateComboDefs(configId, combos),
    [],
  );

  const updateProfileCombos = useCallback(
    async (configId: string, combos: ComboDefinition[]) => {
      return (
        (await mutateSnapshot(() => tauriCommands.updateProfileCombos(configId, combos))) !== null
      );
    },
    [mutateSnapshot],
  );

  return {
    updateProfileCombos,
    validateComboDefs,
  };
}
