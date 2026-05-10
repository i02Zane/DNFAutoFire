import { useCallback } from "react";
import type { SettingsUpdater } from "../../store/app-store-context";
import type {
  AppStateSnapshot,
  DetectionNoMatchPolicy,
  LogLevelSetting,
} from "../../types/app-config";

type UseSettingsActionsOptions = {
  updateSettings: (updater: SettingsUpdater) => Promise<AppStateSnapshot | null>;
};

export function useSettingsActions({ updateSettings }: UseSettingsActionsOptions) {
  const updateOpenFloatingControlOnStart = useCallback(
    (checked: boolean) => {
      void updateSettings((currentSettings) => ({
        ...currentSettings,
        openFloatingControlOnStart: checked,
      }));
    },
    [updateSettings],
  );

  const updateDetectionEnabled = useCallback(
    (checked: boolean) => {
      void updateSettings((currentSettings) => ({
        ...currentSettings,
        detection: {
          ...currentSettings.detection,
          enabled: checked,
        },
      }));
    },
    [updateSettings],
  );

  const updateDetectionInterval = useCallback(
    (intervalMs: number) => {
      void updateSettings((currentSettings) => ({
        ...currentSettings,
        detection: {
          ...currentSettings.detection,
          intervalMs,
        },
      }));
    },
    [updateSettings],
  );

  const updateDetectionNoMatchPolicy = useCallback(
    (policy: DetectionNoMatchPolicy) => {
      void updateSettings((currentSettings) => ({
        ...currentSettings,
        detection: {
          ...currentSettings.detection,
          noMatchPolicy: policy,
        },
      }));
    },
    [updateSettings],
  );

  const updateStartMinimized = useCallback(
    (checked: boolean) => {
      void updateSettings((currentSettings) => ({ ...currentSettings, startMinimized: checked }));
    },
    [updateSettings],
  );

  const updateMinimizeToTray = useCallback(
    (checked: boolean) => {
      void updateSettings((currentSettings) => ({ ...currentSettings, minimizeToTray: checked }));
    },
    [updateSettings],
  );

  const updateCloseButtonMinimizes = useCallback(
    (checked: boolean) => {
      void updateSettings((currentSettings) => ({
        ...currentSettings,
        closeButtonMinimizes: checked,
      }));
    },
    [updateSettings],
  );

  const updateLogLevel = useCallback(
    (level: LogLevelSetting) => {
      void updateSettings((currentSettings) => ({ ...currentSettings, logLevel: level }));
    },
    [updateSettings],
  );

  const updateLaunchAtStartup = useCallback(
    (checked: boolean) => {
      void updateSettings((currentSettings) => ({
        ...currentSettings,
        launchAtStartup: checked,
      }));
    },
    [updateSettings],
  );

  return {
    updateCloseButtonMinimizes,
    updateDetectionEnabled,
    updateDetectionInterval,
    updateDetectionNoMatchPolicy,
    updateLaunchAtStartup,
    updateLogLevel,
    updateMinimizeToTray,
    updateOpenFloatingControlOnStart,
    updateStartMinimized,
  };
}
