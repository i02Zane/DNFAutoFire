// 前端配置辅助：默认值、职业配置补全，以及当前实际生效按键和连招计算。
import {
  AppConfig,
  ClassConfig,
  ComboAction,
  ComboDefinition,
  ComboValidationIssue,
  CustomConfig,
  DetectionNoMatchPolicy,
  KeyBinding,
  LogLevelSetting,
  makeClassConfig,
  makeCustomConfig,
} from "./tauri";
import { getClassName } from "../data/classes";

const MIN_COMBO_HOLD_MS = 10;
const MAX_COMBO_HOLD_MS = 1000;
const MAX_COMBO_GAP_MS = 1000;
const MAX_COMBO_WAIT_MS = 5000;
export const MAX_COMBO_COMMAND_DIRECTION_KEYS = 4;
export const COMBO_COMMAND_DIRECTION_VKS = new Set([0x25, 0x26, 0x27, 0x28]);
export const COMBO_COMMAND_FINISH_VKS = new Set([0x5a, 0x58, 0x43, 0x20]);
export const COMBO_COMMAND_ALLOWED_VKS = new Set([
  ...COMBO_COMMAND_DIRECTION_VKS,
  ...COMBO_COMMAND_FINISH_VKS,
]);

const DEFAULT_LOG_LEVEL: LogLevelSetting = import.meta.env.DEV ? "debug" : "info";
export const DEFAULT_DETECTION_INTERVAL_MS = 200;
export const DETECTION_INTERVAL_OPTIONS = [100, 200, 500, 1000] as const;
export const DETECTION_NO_MATCH_POLICY_OPTIONS: {
  label: string;
  value: DetectionNoMatchPolicy;
}[] = [
  { label: "保持当前配置", value: "current" },
  { label: "切回全局配置", value: "global" },
];

export const DEFAULT_CONFIG: AppConfig = {
  version: 7,
  globalKeys: [{ vk: 0x58, intervalMs: 20 }],
  comboDefs: [],
  classes: {},
  customConfigs: {},
  hiddenClassIds: [],
  activeClassId: null,
  toggleHotkey: { ctrl: true, alt: false, shift: false, vk: 0x77 },
  detection: {
    enabled: false,
    intervalMs: DEFAULT_DETECTION_INTERVAL_MS,
    noMatchPolicy: "current",
    iconDatabaseVersion: "builtin-empty-v1",
  },
  settings: {
    launchAtStartup: false,
    startMinimized: false,
    minimizeToTray: false,
    openFloatingControlOnStart: false,
    logLevel: DEFAULT_LOG_LEVEL,
  },
};

export function normalizeDetectionIntervalMs(intervalMs: number): number {
  return DETECTION_INTERVAL_OPTIONS.some((option) => option === intervalMs)
    ? intervalMs
    : DEFAULT_DETECTION_INTERVAL_MS;
}

export type ConfigOption = {
  id: string | null;
  label: string;
};

export function isCustomConfigId(config: AppConfig, configId: string): boolean {
  return Object.prototype.hasOwnProperty.call(config.customConfigs, configId);
}

export function getClassConfig(config: AppConfig, classId: string): ClassConfig {
  // 旧配置或空职业只持久化差异字段，读取时补上完整默认结构。
  return { ...makeClassConfig(), ...config.classes[classId] };
}

export function getCustomConfig(config: AppConfig, configId: string): CustomConfig {
  return { ...makeCustomConfig(), ...config.customConfigs[configId] };
}

export function getProfileConfig(config: AppConfig, configId: string): ClassConfig | CustomConfig {
  return isCustomConfigId(config, configId)
    ? getCustomConfig(config, configId)
    : getClassConfig(config, configId);
}

export function getConfigDisplayName(config: AppConfig, configId: string | null): string {
  if (!configId) return "全局配置";
  if (isCustomConfigId(config, configId)) {
    return getCustomConfig(config, configId).name.trim() || "未命名配置";
  }
  return getClassName(configId);
}

export function hasDuplicateKeys(keys: KeyBinding[]): boolean {
  return new Set(keys.map((key) => key.vk)).size !== keys.length;
}

export function hasClassKeyConfig(classConfig: ClassConfig | undefined): boolean {
  return (classConfig?.enabledKeys.length ?? 0) > 0;
}

export function hasClassComboConfig(classConfig: ClassConfig | undefined): boolean {
  return (classConfig?.comboDefs.length ?? 0) > 0;
}

export function hasClassConfig(classConfig: ClassConfig | undefined): boolean {
  return hasClassKeyConfig(classConfig) || hasClassComboConfig(classConfig);
}

export function configuredConfigOptions(config: AppConfig): ConfigOption[] {
  // 只有连招、没有连发键的配置也要出现在“当前配置”选择器中。
  const classOptions = Object.entries(config.classes)
    .filter(([, classConfig]) => hasClassConfig(classConfig))
    .map(([classId]) => ({ id: classId, label: getClassName(classId) }));
  const customOptions = Object.entries(config.customConfigs)
    .filter(([, customConfig]) => hasClassConfig(customConfig))
    .map(([id, customConfig]) => ({
      id,
      label: customConfig.name.trim() || "未命名配置",
    }));

  return [...classOptions, ...customOptions];
}

export function isClassVisible(config: AppConfig, classId: string): boolean {
  return hasClassConfig(config.classes[classId]) || !config.hiddenClassIds.includes(classId);
}

function dedupeKeysPreferLast(keys: KeyBinding[]): KeyBinding[] {
  return Array.from(new Map(keys.map((key) => [key.vk, key])).values());
}

// 职业配置与全局配置合并时，同一个 VK 以后出现的配置为准，便于职业覆盖全局间隔。
export function computeEffectiveKeys(config: AppConfig): KeyBinding[] {
  if (!config.activeClassId) {
    return dedupeKeysPreferLast(config.globalKeys);
  }
  return computeEffectiveKeysForProfile(config, config.activeClassId);
}

export function computeEffectiveKeysForClass(config: AppConfig, classId: string): KeyBinding[] {
  return computeEffectiveKeysForProfileConfig(config, config.classes[classId]);
}

export function computeEffectiveKeysForProfile(config: AppConfig, configId: string): KeyBinding[] {
  const profileConfig = isCustomConfigId(config, configId)
    ? config.customConfigs[configId]
    : config.classes[configId];
  return computeEffectiveKeysForProfileConfig(config, profileConfig);
}

function computeEffectiveKeysForProfileConfig(
  config: AppConfig,
  profileConfig: ClassConfig | CustomConfig | undefined,
): KeyBinding[] {
  if (!profileConfig || !hasClassKeyConfig(profileConfig)) {
    return dedupeKeysPreferLast(config.globalKeys);
  }
  if (profileConfig.effectRule === "classOnly") {
    return dedupeKeysPreferLast(profileConfig.enabledKeys);
  }
  return dedupeKeysPreferLast([...config.globalKeys, ...profileConfig.enabledKeys]);
}

export function computeEffectiveCombos(config: AppConfig): ComboDefinition[] {
  if (!config.activeClassId) {
    return [];
  }

  return getProfileConfig(config, config.activeClassId).comboDefs.filter((combo) => combo.enabled);
}

export function validateClassComboDefs(
  combos: ComboDefinition[],
  effectiveKeys: KeyBinding[],
): ComboValidationIssue[] {
  const issues: ComboValidationIssue[] = [];
  const effectiveKeyVks = new Set(effectiveKeys.map((key) => key.vk));
  const triggerOwners = new Map<number, ComboDefinition[]>();

  for (const combo of combos) {
    for (const action of combo.actions) {
      issues.push(...validateComboActionTiming(combo.id, action));
    }

    if (!combo.enabled) continue;

    if (combo.name.trim().length === 0) {
      issues.push({
        comboId: combo.id,
        field: "name",
        message: "启用的连招必须填写名称。",
      });
    }

    if (combo.triggerVk === null) {
      issues.push({
        comboId: combo.id,
        field: "trigger",
        message: "启用的连招必须设置触发键。",
      });
    } else {
      const owners = triggerOwners.get(combo.triggerVk) ?? [];
      owners.push(combo);
      triggerOwners.set(combo.triggerVk, owners);
      if (effectiveKeyVks.has(combo.triggerVk)) {
        issues.push({
          comboId: combo.id,
          field: "trigger",
          message: "触发键不能与当前生效连发键重复。",
        });
      }
    }

    if (combo.actions.length === 0) {
      issues.push({
        comboId: combo.id,
        field: "actions",
        message: "启用的连招至少需要一个动作。",
      });
    }

    for (const action of combo.actions) {
      issues.push(...validateEnabledComboAction(combo.id, action));
    }
  }

  for (const owners of triggerOwners.values()) {
    if (owners.length < 2) continue;
    for (const combo of owners) {
      issues.push({
        comboId: combo.id,
        field: "trigger",
        message: "同一职业的连招触发键不能重复。",
      });
    }
  }

  return issues;
}

export function normalizeComboHoldMs(value: number): number {
  return normalizeNumber(value, MIN_COMBO_HOLD_MS, MAX_COMBO_HOLD_MS, 30);
}

export function normalizeComboGapMs(value: number): number {
  return normalizeNumber(value, 0, MAX_COMBO_GAP_MS, 20);
}

export function normalizeComboWaitMs(value: number): number {
  return normalizeNumber(value, 0, MAX_COMBO_WAIT_MS, 100);
}

export function isComboCommandVk(vk: number): boolean {
  return COMBO_COMMAND_ALLOWED_VKS.has(vk);
}

export function isComboCommandDirectionVk(vk: number): boolean {
  return COMBO_COMMAND_DIRECTION_VKS.has(vk);
}

export function isComboCommandFinishVk(vk: number): boolean {
  return COMBO_COMMAND_FINISH_VKS.has(vk);
}

export function countComboCommandDirections(keys: number[]): number {
  return keys.filter(isComboCommandDirectionVk).length;
}

function normalizeNumber(value: number, min: number, max: number, fallback: number): number {
  if (!Number.isFinite(value)) return fallback;
  return Math.max(min, Math.min(max, Math.trunc(value)));
}

function validateEnabledComboAction(comboId: string, action: ComboAction): ComboValidationIssue[] {
  if (action.type === "tap") {
    if (action.vk === null) {
      return [
        {
          comboId,
          actionId: action.id,
          field: "tapKey",
          message: "快捷栏动作必须设置按键。",
        },
      ];
    }
    return [];
  }

  if (action.keys.length === 0) {
    return [
      {
        comboId,
        actionId: action.id,
        field: "commandKeys",
        message: "手搓动作至少需要一个按键。",
      },
    ];
  }
  if (!action.keys.every(isComboCommandVk)) {
    return [
      {
        comboId,
        actionId: action.id,
        field: "commandKeys",
        message: "手搓动作只能使用上下左右和 Z/X/C/空格。",
      },
    ];
  }
  if (countComboCommandDirections(action.keys) > MAX_COMBO_COMMAND_DIRECTION_KEYS) {
    return [
      {
        comboId,
        actionId: action.id,
        field: "commandKeys",
        message: "手搓动作最多只能包含 4 个方向键。",
      },
    ];
  }
  if (!isComboCommandFinishVk(action.keys[action.keys.length - 1] ?? 0)) {
    return [
      {
        comboId,
        actionId: action.id,
        field: "commandKeys",
        message: "手搓动作必须以 Z/X/C/空格结束。",
      },
    ];
  }
  return [];
}

function validateComboActionTiming(comboId: string, action: ComboAction): ComboValidationIssue[] {
  const issues: ComboValidationIssue[] = [];
  if (action.type === "tap") {
    if (!isInRange(action.holdMs, MIN_COMBO_HOLD_MS, MAX_COMBO_HOLD_MS)) {
      issues.push({
        comboId,
        actionId: action.id,
        field: "holdMs",
        message: `按下时长必须在 ${MIN_COMBO_HOLD_MS}-${MAX_COMBO_HOLD_MS} 毫秒之间。`,
      });
    }
  } else {
    if (!isInRange(action.keyHoldMs, MIN_COMBO_HOLD_MS, MAX_COMBO_HOLD_MS)) {
      issues.push({
        comboId,
        actionId: action.id,
        field: "keyHoldMs",
        message: `按下时长必须在 ${MIN_COMBO_HOLD_MS}-${MAX_COMBO_HOLD_MS} 毫秒之间。`,
      });
    }
    if (!isInRange(action.keyGapMs, 0, MAX_COMBO_GAP_MS)) {
      issues.push({
        comboId,
        actionId: action.id,
        field: "keyGapMs",
        message: `按键间隔不能超过 ${MAX_COMBO_GAP_MS} 毫秒。`,
      });
    }
  }

  if (!isInRange(action.waitAfterMs, 0, MAX_COMBO_WAIT_MS)) {
    issues.push({
      comboId,
      actionId: action.id,
      field: "waitAfterMs",
      message: `动作后等待不能超过 ${MAX_COMBO_WAIT_MS} 毫秒。`,
    });
  }

  return issues;
}

function isInRange(value: number, min: number, max: number): boolean {
  return Number.isFinite(value) && Number.isInteger(value) && value >= min && value <= max;
}
