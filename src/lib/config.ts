// 前端配置辅助：表单选项、职业配置补全和显示名称。
import {
  ClassConfig,
  CustomConfig,
  DetectionNoMatchPolicy,
  ProfileDisplaySnapshot,
  ProfilesConfig,
  makeClassConfig,
  makeCustomConfig,
} from "../types/app-config";

export const MAX_COMBO_COMMAND_DIRECTION_KEYS = 4;
export const COMBO_COMMAND_DIRECTION_VKS = new Set([0x25, 0x26, 0x27, 0x28]);
export const COMBO_COMMAND_FINISH_VKS = new Set([0x5a, 0x58, 0x43, 0x20]);
export const COMBO_COMMAND_ALLOWED_VKS = new Set([
  ...COMBO_COMMAND_DIRECTION_VKS,
  ...COMBO_COMMAND_FINISH_VKS,
]);

export const DETECTION_INTERVAL_OPTIONS = [100, 200, 500, 1000] as const;
export const AUTO_RUN_PULSE_DELAY_OPTIONS = [
  { label: "短", value: 10 },
  { label: "中", value: 25 },
  { label: "长", value: 50 },
] as const;
export const DETECTION_NO_MATCH_POLICY_OPTIONS: {
  label: string;
  value: DetectionNoMatchPolicy;
}[] = [
  { label: "保持当前配置", value: "current" },
  { label: "切回全局配置", value: "global" },
];

export function isCustomConfigId(profiles: ProfilesConfig, configId: string): boolean {
  return Object.prototype.hasOwnProperty.call(profiles.customConfigs, configId);
}

export function getClassConfig(profiles: ProfilesConfig, classId: string): ClassConfig {
  // 旧配置或空职业只持久化差异字段，读取时补上完整默认结构。
  return { ...makeClassConfig(), ...profiles.classes[classId] };
}

export function getCustomConfig(profiles: ProfilesConfig, configId: string): CustomConfig {
  return { ...makeCustomConfig(), ...profiles.customConfigs[configId] };
}

export function getProfileConfig(
  profiles: ProfilesConfig,
  configId: string,
): ClassConfig | CustomConfig {
  return isCustomConfigId(profiles, configId)
    ? getCustomConfig(profiles, configId)
    : getClassConfig(profiles, configId);
}

export function getConfigDisplayName(
  configId: string | null,
  profileDisplay: ProfileDisplaySnapshot,
): string {
  if (!configId) return "全局配置";
  return profileDisplay.displayNames[configId] ?? "未知配置";
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
