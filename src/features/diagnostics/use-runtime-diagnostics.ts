import { useCallback, useEffect, useState } from "react";
import { tauriCommands } from "../../lib/tauri-commands";
import { appErrorMessage } from "../../types/app-error";
import type { RuntimeDiagnostics } from "../../types/app-config";

type UseRuntimeDiagnosticsOptions = {
  onError: (message: string) => void;
};

export function useRuntimeDiagnostics({ onError }: UseRuntimeDiagnosticsOptions) {
  const [diagnostics, setDiagnostics] = useState<RuntimeDiagnostics | null>(null);
  const [lastUpdatedAt, setLastUpdatedAt] = useState<Date | null>(null);

  const refreshDiagnostics = useCallback(async () => {
    try {
      const nextDiagnostics = await tauriCommands.loadRuntimeDiagnostics();
      setDiagnostics(nextDiagnostics);
      setLastUpdatedAt(new Date());
    } catch (reason) {
      onError(appErrorMessage(reason));
    }
  }, [onError]);

  useEffect(() => {
    const firstRefresh = window.setTimeout(() => {
      void refreshDiagnostics();
    }, 0);
    const timer = window.setInterval(() => {
      void refreshDiagnostics();
    }, 1000);
    return () => {
      window.clearTimeout(firstRefresh);
      window.clearInterval(timer);
    };
  }, [refreshDiagnostics]);

  return { diagnostics, lastUpdatedAt };
}
