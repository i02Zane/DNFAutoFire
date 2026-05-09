import { useCallback } from "react";
import { tauriCommands } from "../lib/tauri-commands";
import type { AppStoreContextValue } from "../store/app-store-context";

type UseActiveConfigActionsOptions = {
  mutateSnapshot: AppStoreContextValue["mutateSnapshot"];
};

export function useActiveConfigActions({ mutateSnapshot }: UseActiveConfigActionsOptions) {
  const onActiveConfigChange = useCallback(
    (classId: string | null) =>
      void mutateSnapshot(() => tauriCommands.selectActiveConfig(classId)),
    [mutateSnapshot],
  );

  return { onActiveConfigChange };
}
