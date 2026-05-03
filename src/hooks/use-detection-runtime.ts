import { useEffect, type MutableRefObject } from "react";
import { getClassIdByDetectionIndex } from "../data/classes";
import { APP_EVENTS, listenAppEvent } from "../lib/app-events";
import { isMockMode, tauriCommands, type AppConfig } from "../lib/tauri";
import type { ConfigUpdater } from "./use-app-config";

type UseDetectionRuntimeOptions = {
  config: AppConfig;
  configRef: MutableRefObject<AppConfig>;
  showMessage: (message: string) => void;
  startupConfigLoaded: boolean;
  updateConfig: (updater: ConfigUpdater) => Promise<AppConfig | null>;
};

export function useDetectionRuntime({
  config,
  configRef,
  showMessage,
  startupConfigLoaded,
  updateConfig,
}: UseDetectionRuntimeOptions) {
  useEffect(() => {
    if (isMockMode() || !startupConfigLoaded || !config.detection.enabled) return;

    // 识别结果只在自动识别开启时写回 activeClassId，关闭后保留最后一次结果。
    let disposed = false;
    let unlisten: (() => void) | undefined;
    const listenDetectionResults = async () => {
      unlisten = await listenAppEvent(
        APP_EVENTS.classDetectionResult,
        ({ classIndex, reason }) => {
          const currentConfig = configRef.current;
          if (!currentConfig.detection.enabled) {
            return;
          }

          // 只在 DNF 仍处于前台时处理识别失败；切到别的软件只是暂停识别结果，不改 activeClassId。
          if (reason === "foregroundInactive") {
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
  }, [config.detection.enabled, startupConfigLoaded, updateConfig, configRef]);

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
