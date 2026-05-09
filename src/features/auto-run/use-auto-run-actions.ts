import { useCallback } from "react";
import { tauriCommands } from "../../lib/tauri-commands";
import type { AppStoreContextValue } from "../../store/app-store-context";
import type { AutoRunConfig } from "../../types/app-config";

type UseAutoRunActionsOptions = {
  mutateSnapshot: AppStoreContextValue["mutateSnapshot"];
};

export function useAutoRunActions({ mutateSnapshot }: UseAutoRunActionsOptions) {
  const saveAutoRun = useCallback(
    (patch: Partial<AutoRunConfig>) => {
      void mutateSnapshot(() => tauriCommands.updateAutoRun(patch));
    },
    [mutateSnapshot],
  );

  const onAutoRunEnabledChange = useCallback(
    (checked: boolean) => saveAutoRun({ enabled: checked }),
    [saveAutoRun],
  );

  const onAutoRunLeftVkChange = useCallback(
    (vk: number) => saveAutoRun({ leftVk: vk }),
    [saveAutoRun],
  );

  const onAutoRunPulseDelayChange = useCallback(
    (pulseDelayMs: number) => saveAutoRun({ pulseDelayMs }),
    [saveAutoRun],
  );

  const onAutoRunRightVkChange = useCallback(
    (vk: number) => saveAutoRun({ rightVk: vk }),
    [saveAutoRun],
  );

  return {
    onAutoRunEnabledChange,
    onAutoRunLeftVkChange,
    onAutoRunPulseDelayChange,
    onAutoRunRightVkChange,
  };
}
