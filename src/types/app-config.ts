// 前后端共享 DTO 从 Rust ts-rs 生成文件收敛到这里；本文件只保留前端本地 helper。
import type { ClassConfig, ComboAction, CustomConfig } from "../generated/backend-types";

export type {
  ActiveConfigDiagnostics,
  AppStateSnapshot,
  AssistantEngineSnapshots,
  AssistantRuntimeSnapshot,
  AutoFireKeySnapshot,
  AutoFireSnapshot,
  AutoRunConfig,
  AutoRunDiagnostics,
  AutoRunSnapshot,
  BootstrapEffectiveProfile,
  BootstrapRuntimeState,
  BootstrapState,
  ClassCategory,
  ClassConfig,
  ClassDetectionResultEvent,
  ClassDisplayState,
  ClassInfo,
  ClassInfoView,
  ComboAction,
  ComboDefinition,
  ComboValidationField,
  ComboValidationIssue,
  ConfigOption,
  CustomConfig,
  CustomConfigDisplayState,
  DetectionDiagnostics,
  DetectionNoMatchPolicy,
  DetectionResultDiagnostics,
  DetectionSettings,
  EffectRule,
  FireKeyMode,
  ForegroundDiagnostics,
  Hotkey,
  KeyBinding,
  LogLevelSetting,
  ProfileDisplaySnapshot,
  ProfilesConfig,
  RuntimeDiagnostics,
  RuntimeEffectiveProfile,
  RuntimeStateSnapshot,
  RuntimeStatusSnapshot,
  SettingsConfig,
  WindowPosition,
} from "../generated/backend-types";

export type ImageRect = {
  x: number;
  y: number;
  width: number;
  height: number;
};

export type DetectionDebugSnapshot = {
  savedDir: string;
  fullImagePath: string;
  skillbarImagePath: string;
  windowRect: ImageRect;
  skillbarRect: ImageRect;
  geometry: {
    source: string;
    gameScale: number;
    anchorRect: ImageRect | null;
    anchorPixelScale: number | null;
    uiScalePercent: number | null;
  };
  slots: { index: number; rect: ImageRect }[];
  scores: {
    classId: string;
    confidence: number;
    coreConfidence: number;
    classConfidence: number;
    matchedIcons: number;
    requiredIcons: number;
  }[];
  reason: string;
};

export type ComboTapAction = Extract<ComboAction, { type: "tap" }>;
export type ComboCommandAction = Extract<ComboAction, { type: "command" }>;

export function makeClassConfig(): ClassConfig {
  // 新职业配置必须从这个工厂创建，避免漏掉后续新增字段。
  return {
    enabledKeys: [],
    effectRule: "globalAndClass",
    comboDefs: [],
  };
}

export function makeCustomConfig(name = "自定义配置"): CustomConfig {
  return {
    ...makeClassConfig(),
    name,
  };
}
