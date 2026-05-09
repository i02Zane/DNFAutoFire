import { useCallback } from "react";
import { tauriCommands } from "../lib/tauri-commands";
import { appErrorMessage } from "../types/app-error";

type UseAssistantRuntimeOptions = {
  running: boolean;
  showMessage: (message: string) => void;
};

export function useAssistantRuntime({ running, showMessage }: UseAssistantRuntimeOptions) {
  return useCallback(async () => {
    try {
      await tauriCommands.setAssistantRunning(!running);
    } catch (reason) {
      showMessage(appErrorMessage(reason));
    }
  }, [running, showMessage]);
}
