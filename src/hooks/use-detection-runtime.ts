import { useEffect } from "react";
import { APP_EVENTS, listenAppEvent } from "../lib/app-events";
import { isMockMode, tauriCommands, type AppConfig } from "../lib/tauri";

type UseDetectionRuntimeOptions = {
  config: AppConfig;
  setDetectionRunning: (running: boolean) => void;
  showMessage: (message: string) => void;
  startupConfigLoaded: boolean;
};

export function useDetectionRuntime({
  config,
  setDetectionRunning,
  showMessage,
  startupConfigLoaded,
}: UseDetectionRuntimeOptions) {
  useEffect(() => {
    if (isMockMode() || !startupConfigLoaded) return;

    let disposed = false;
    let unlisten: (() => void) | undefined;
    const listenDetectionRunning = async () => {
      unlisten = await listenAppEvent(APP_EVENTS.detectionRunningChanged, (running) => {
        setDetectionRunning(running);
      });
      if (disposed) {
        unlisten();
      }
    };

    void listenDetectionRunning().catch(() => undefined);
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [setDetectionRunning, startupConfigLoaded]);

  useEffect(() => {
    if (isMockMode() || !startupConfigLoaded) return;

    if (!config.detection.enabled) {
      void tauriCommands.stopDetection().catch(() => undefined);
      return;
    }

    void tauriCommands.startDetection(config.detection.intervalMs).catch((reason) => {
      showMessage(reason instanceof Error ? reason.message : String(reason));
    });

    return () => {
      void tauriCommands.stopDetection().catch(() => undefined);
    };
  }, [config.detection.enabled, config.detection.intervalMs, showMessage, startupConfigLoaded]);
}
