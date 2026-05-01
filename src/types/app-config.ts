// 前后端共享的配置形状，字段名保持 camelCase 以匹配持久化 JSON。
export type KeyBinding = {
  vk: number;
  intervalMs: number;
};

export type ComboTapAction = {
  id: string;
  type: "tap";
  label: string;
  vk: number | null;
  holdMs: number;
  waitAfterMs: number;
};

export type ComboCommandAction = {
  id: string;
  type: "command";
  label: string;
  keys: number[];
  keyHoldMs: number;
  keyGapMs: number;
  waitAfterMs: number;
};

export type ComboAction = ComboTapAction | ComboCommandAction;

export type ComboDefinition = {
  id: string;
  name: string;
  enabled: boolean;
  triggerVk: number | null;
  actions: ComboAction[];
};

export type ComboValidationField =
  | "name"
  | "trigger"
  | "actions"
  | "tapKey"
  | "commandKeys"
  | "holdMs"
  | "keyHoldMs"
  | "keyGapMs"
  | "waitAfterMs";

export type ComboValidationIssue = {
  comboId: string;
  actionId?: string;
  field: ComboValidationField;
  message: string;
};

// 职业配置与全局配置的合并策略。
export type EffectRule = "globalAndClass" | "classOnly";

// 职业配置是预设 id/name 且不可删除的配置，功能形状与自定义配置保持一致。
export type ClassConfig = {
  enabledKeys: KeyBinding[];
  effectRule: EffectRule;
  comboDefs: ComboDefinition[];
};

// 用户自定义配置在 ClassConfig 的基础上额外持久化用户命名。
export type CustomConfig = ClassConfig & {
  name: string;
};

export type DetectionSettings = {
  // 职业识别结构目前保留，后续恢复识别能力时继续沿用这组字段。
  enabled: boolean;
  intervalMs: number;
  iconDatabaseVersion: string;
};

export type LogLevelSetting = "trace" | "debug" | "info" | "warn" | "error" | "off";

export type AppSettings = {
  launchAtStartup: boolean;
  startMinimized: boolean;
  minimizeToTray: boolean;
  openFloatingControlOnStart: boolean;
  logLevel: LogLevelSetting;
};

export type Hotkey = {
  ctrl: boolean;
  alt: boolean;
  shift: boolean;
  vk: number;
};

export type ClassDetectionResult = {
  classId: string | null;
  confidence: number;
  reason: string;
};

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

export type AppConfig = {
  // schema 版本由后端迁移和校验，前端只随保存结果展示。
  version: number;
  globalKeys: KeyBinding[];
  // 根级连招字段只做兼容保留，v1 不编辑、不合并、不下发。
  comboDefs: ComboDefinition[];
  classes: Record<string, ClassConfig>;
  customConfigs: Record<string, CustomConfig>;
  hiddenClassIds: string[];
  activeClassId: string | null;
  toggleHotkey: Hotkey | null;
  detection: DetectionSettings;
  settings: AppSettings;
};

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
