import { useEffect, useRef, type MutableRefObject } from "react";
import { getClassIdByDetectionIndex } from "../data/classes";
import { APP_EVENTS, listenAppEvent } from "../lib/app-events";
import { isMockMode, tauriCommands, type AppConfig } from "../lib/tauri";
import type { ConfigUpdater } from "./use-app-config";

type UseDetectionRuntimeOptions = {
  config: AppConfig;
  configRef: MutableRefObject<AppConfig>;
  detectionRunning: boolean;
  setDetectionRunning: (running: boolean) => void;
  showMessage: (message: string) => void;
  startupConfigLoaded: boolean;
  updateConfig: (updater: ConfigUpdater) => Promise<AppConfig | null>;
};

export function useDetectionRuntime({
  config,
  configRef,
  detectionRunning,
  setDetectionRunning,
  showMessage,
  startupConfigLoaded,
  updateConfig,
}: UseDetectionRuntimeOptions) {
  const detectionRunningRef = useRef(detectionRunning);

  useEffect(() => {
    detectionRunningRef.current = detectionRunning;
  }, [detectionRunning]);

  useEffect(() => {
    if (isMockMode() || !startupConfigLoaded) return;

    // 识别结果只在自动识别开启时写回 activeClassId，关闭后保留最后一次结果。
    let disposed = false;
    let unlisten: (() => void) | undefined;
    const listenDetectionResults = async () => {
      unlisten = await listenAppEvent(
        APP_EVENTS.classDetectionResult,
        ({ classIndex, reason }) => {
          if (!detectionRunningRef.current) {
            return;
          }

          const currentConfig = configRef.current;

          // 切到别的软件、进入副本或采集失败时只是暂停识别结果，不改 activeClassId。
          if (
            reason === "foregroundInactive" ||
            reason === "notInTown" ||
            reason === "captureError"
          ) {
            return;
          }

          const nextClassId = classIndex === null ? null : getClassIdByDetectionIndex(classIndex);

          if (!nextClassId) {
            if (currentConfig.detection.noMatchPolicy !== "global") {
              return;
            }
            if (currentConfig.activeClassId === null) {
              return;
            }
            void updateConfig((nextConfig) => ({
              ...nextConfig,
              activeClassId: null,
            }));
            return;
          }

          if (currentConfig.activeClassId === nextClassId) {
            return;
          }

          void updateConfig((nextConfig) => ({
            ...nextConfig,
            activeClassId: nextClassId,
          }));
        },
      );
      if (disposed) {
        unlisten();
      }
    };

    void listenDetectionResults().catch(() => undefined);
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [startupConfigLoaded, updateConfig, configRef]);

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
