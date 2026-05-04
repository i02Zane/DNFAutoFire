import { useCallback, useEffect, useRef, useState } from "react";
import { DEFAULT_CONFIG } from "../lib/config";
import { type AppConfig, tauriCommands } from "../lib/tauri";

export type ConfigUpdater = AppConfig | ((currentConfig: AppConfig) => AppConfig);

type StartupState = {
  config: AppConfig;
  running: boolean;
  detectionRunning: boolean;
};

type UseAppConfigOptions = {
  onSaveError: (message: string) => void;
  onStartupLoaded: (state: StartupState) => void;
};

export function useAppConfig({ onSaveError, onStartupLoaded }: UseAppConfigOptions) {
  const [config, setConfig] = useState<AppConfig>(DEFAULT_CONFIG);
  const [startupConfigLoaded, setStartupConfigLoaded] = useState(false);
  // ref 保存异步回调里的最新快照，避免保存完成后拿到过期闭包里的配置。
  const configRef = useRef(config);
  const lastSavedConfigRef = useRef(config);
  const configSaveIdRef = useRef(0);
  const configSaveQueueRef = useRef(Promise.resolve());

  const applyConfig = useCallback((nextConfig: AppConfig) => {
    configRef.current = nextConfig;
    setConfig(nextConfig);
  }, []);

  const updateConfig = useCallback(
    async (updater: ConfigUpdater): Promise<AppConfig | null> => {
      // 配置保存采用乐观更新；串行队列避免快速切换设置时旧请求后完成并覆盖新配置。
      const saveId = configSaveIdRef.current + 1;
      const previousConfig = configRef.current;
      const nextConfig = typeof updater === "function" ? updater(previousConfig) : updater;
      configSaveIdRef.current = saveId;
      applyConfig(nextConfig);

      // 保存请求按顺序串行执行，界面则先展示乐观更新后的结果。
      const saveTask = configSaveQueueRef.current.then(() =>
        tauriCommands.saveAppConfig(nextConfig),
      );
      configSaveQueueRef.current = saveTask.then(
        () => undefined,
        () => undefined,
      );

      try {
        const saved = await saveTask;
        lastSavedConfigRef.current = saved;
        if (saveId === configSaveIdRef.current) {
          applyConfig(saved);
        }
        return saved;
      } catch (reason) {
        if (saveId === configSaveIdRef.current) {
          // 保存失败时回到最后一次确认落盘的配置，避免停留在未持久化的界面状态。
          applyConfig(lastSavedConfigRef.current);
          onSaveError(reason instanceof Error ? reason.message : String(reason));
        }
        return null;
      }
    },
    [applyConfig, onSaveError],
  );

  useEffect(() => {
    // 首屏加载时同时恢复持久配置和后端运行状态，避免按钮状态与引擎状态短暂不一致。
    void Promise.all([
      tauriCommands.loadAppConfig(),
      tauriCommands.isAssistantRunning(),
      tauriCommands.isDetectionRunning(),
    ]).then(([nextConfig, isRunning, detectionRunning]) => {
        lastSavedConfigRef.current = nextConfig;
        applyConfig(nextConfig);
        onStartupLoaded({ config: nextConfig, running: isRunning, detectionRunning });
        setStartupConfigLoaded(true);
      });
  }, [applyConfig, onStartupLoaded]);

  const updateSettings = useCallback(
    (settings: Partial<AppConfig["settings"]>) => {
      void updateConfig((currentConfig) => ({
        ...currentConfig,
        settings: {
          ...currentConfig.settings,
          ...settings,
        },
      }));
    },
    [updateConfig],
  );

  return {
    config,
    configRef,
    startupConfigLoaded,
    updateConfig,
    updateSettings,
  };
}
