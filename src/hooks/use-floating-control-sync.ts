import { useEffect } from "react";
import { APP_EVENTS, emitAppEvent, listenAppEvent } from "../lib/app-events";
import { type AppConfig, isMockMode, tauriCommands } from "../lib/tauri";
import type { ConfigUpdater } from "./use-app-config";

type UseFloatingControlSyncOptions = {
  config: AppConfig;
  floatingControlEnabled: boolean;
  running: boolean;
  setFloatingControlEnabled: (floatingControlEnabled: boolean) => void;
  showMessage: (message: string) => void;
  startupConfigLoaded: boolean;
  toggleFloatingControlEnabled: () => void;
  updateConfig: (updater: ConfigUpdater) => Promise<AppConfig | null>;
};

export function useFloatingControlSync({
  config,
  floatingControlEnabled,
  running,
  setFloatingControlEnabled,
  showMessage,
  startupConfigLoaded,
  toggleFloatingControlEnabled,
  updateConfig,
}: UseFloatingControlSyncOptions) {
  // 这条链路只负责“显示层”同步；真正可持久化的配置仍由主窗口统一写盘。
  useEffect(() => {
    if (!startupConfigLoaded) return;

    // 可见性变化既要体现在前端，也要广播给 Rust 侧去更新托盘状态。
    const syncFloatingControlWindow = async () => {
      if (floatingControlEnabled) {
        await tauriCommands.showFloatingControlWindow();
      } else {
        await tauriCommands.hideFloatingControlWindow();
      }

      await emitAppEvent(APP_EVENTS.floatingControlVisibilityChanged, {
        visible: floatingControlEnabled,
      });
    };

    void syncFloatingControlWindow().catch((reason) => {
      setFloatingControlEnabled(false);
      showMessage(reason instanceof Error ? reason.message : String(reason));
    });
  }, [floatingControlEnabled, setFloatingControlEnabled, showMessage, startupConfigLoaded]);

  useEffect(() => {
    if (isMockMode() || !floatingControlEnabled) return;

    // 悬浮窗只消费主窗口广播的快照，不自己维护第二份配置。
    const emitFloatingControlUpdate = async () => {
      await emitAppEvent(APP_EVENTS.floatingControlUpdate, {
        config,
        running,
      });
    };

    void emitFloatingControlUpdate().catch(() => undefined);
  }, [config, floatingControlEnabled, running]);

  useEffect(() => {
    if (isMockMode()) return;

    // 悬浮窗没有直接写配置权限，只通过事件把职业切换交回主窗口统一保存。
    let disposed = false;
    let unlisten: (() => void) | undefined;
    const listenClassChange = async () => {
      unlisten = await listenAppEvent(
        APP_EVENTS.floatingControlClassChanged,
        ({ activeClassId }) => {
          void updateConfig((currentConfig) => ({ ...currentConfig, activeClassId }));
        },
      );
      if (disposed) unlisten();
    };
    void listenClassChange().catch(() => undefined);
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [updateConfig]);

  useEffect(() => {
    if (isMockMode()) return;

    // 托盘只发出“切换请求”，真实窗口开关仍由主窗口状态驱动，保证入口一致。
    let disposed = false;
    let unlistenToggleRequest: (() => void) | undefined;
    let unlistenVisibilityChange: (() => void) | undefined;
    const listenFloatingControlEvents = async () => {
      unlistenToggleRequest = await listenAppEvent(APP_EVENTS.floatingControlToggleRequest, () => {
        toggleFloatingControlEnabled();
      });
      unlistenVisibilityChange = await listenAppEvent(
        APP_EVENTS.floatingControlVisibilityChanged,
        ({ visible }) => {
          setFloatingControlEnabled(visible);
        },
      );
      if (disposed) {
        unlistenToggleRequest();
        unlistenVisibilityChange();
      }
    };
    void listenFloatingControlEvents().catch(() => undefined);
    return () => {
      disposed = true;
      unlistenToggleRequest?.();
      unlistenVisibilityChange?.();
    };
  }, [setFloatingControlEnabled, toggleFloatingControlEnabled]);
}
