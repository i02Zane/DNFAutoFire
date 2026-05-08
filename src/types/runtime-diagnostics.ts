import type { FireKeyMode } from "./app-config";
import type { ClassDetectionResult } from "./app-config";

export type RuntimeDiagnostics = {
  assistant: {
    running: boolean;
    profileKeyCount: number;
    profileComboCount: number;
  };
  foreground: {
    targetActive: boolean;
    className: string;
  };
  activeConfig: {
    activeClassId: string | null;
    detectionEnabled: boolean;
    detectionIntervalMs: number;
    autoRunEnabled: boolean;
  };
  autofire: {
    running: boolean;
    keys: {
      vk: number;
      intervalMs: number;
      mode: FireKeyMode;
      pressed: boolean;
      toggleActive: boolean;
    }[];
  };
  combo: {
    running: boolean;
    comboCount: number;
    enabledComboCount: number;
    triggerVks: number[];
    executing: boolean;
  };
  autoRun: {
    running: boolean;
    leftVk: number;
    rightVk: number;
    pulseDelayMs: number;
  };
  detection: {
    running: boolean;
    intervalMs: number;
    lastResult: ClassDetectionResult | null;
    townActive: boolean | null;
  };
};
