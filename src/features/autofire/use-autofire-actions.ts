import { useCallback, useMemo } from "react";
import { getConfigDisplayName, getProfileConfig } from "../../lib/config";
import { keyOptions } from "../../lib/keys";
import { tauriCommands } from "../../lib/tauri-commands";
import type { AppStoreContextValue } from "../../store/app-store-context";
import type {
  EffectRule,
  KeyBinding,
  ProfileDisplaySnapshot,
  ProfilesConfig,
} from "../../types/app-config";
import type { EditTarget } from "../../types/ui";

type UseAutofireActionsOptions = {
  autofireClassSearch: string;
  mutateSnapshot: AppStoreContextValue["mutateSnapshot"];
  profileDisplay: ProfileDisplaySnapshot;
  profiles: ProfilesConfig;
  target: EditTarget | null;
};

export function useAutofireActions({
  autofireClassSearch,
  mutateSnapshot,
  profileDisplay,
  profiles,
  target,
}: UseAutofireActionsOptions) {
  const selectedKeys = useMemo(() => {
    if (target?.type === "global") {
      return profiles.globalKeys;
    }
    if (target?.type === "profile") {
      return getProfileConfig(profiles, target.configId).enabledKeys;
    }
    return [];
  }, [profiles, target]);
  const selectedTitle = useMemo(() => {
    if (target?.type === "global") {
      return "全局配置";
    }
    if (target?.type === "profile") {
      return getConfigDisplayName(target.configId, profileDisplay);
    }
    return "";
  }, [profileDisplay, target]);

  const visibleCustomConfigs = useMemo(() => {
    const normalizedSearch = autofireClassSearch.trim().toLowerCase();
    return Object.entries(profiles.customConfigs).filter(([, customConfig]) => {
      const name = customConfig.name.trim() || "未命名配置";
      return !normalizedSearch || name.toLowerCase().includes(normalizedSearch);
    });
  }, [autofireClassSearch, profiles.customConfigs]);

  const normalizedAutofireClassSearch = autofireClassSearch.trim().toLowerCase();
  const visibleAutofireClassCategories = useMemo(
    () =>
      profileDisplay.visibleClassCategories
        .map((category) => ({
          ...category,
          classes: category.classes.filter(
            (classInfo) =>
              !normalizedAutofireClassSearch ||
              classInfo.name.toLowerCase().includes(normalizedAutofireClassSearch) ||
              classInfo.id.toLowerCase().includes(normalizedAutofireClassSearch),
          ),
        }))
        .filter((category) => category.classes.length > 0),
    [normalizedAutofireClassSearch, profileDisplay.visibleClassCategories],
  );

  const updateSelectedKeys = useCallback(
    (keys: KeyBinding[]) => {
      if (!target) return;
      if (target.type === "global") {
        void mutateSnapshot(() => tauriCommands.updateGlobalKeys(keys), {
          optimistic: (currentSnapshot) => ({
            ...currentSnapshot,
            profiles: updateProfilesSelectedKeys(currentSnapshot.profiles, target, keys),
          }),
        });
        return;
      }

      void mutateSnapshot(() => tauriCommands.updateProfileKeys(target.configId, keys), {
        optimistic: (currentSnapshot) => ({
          ...currentSnapshot,
          profiles: updateProfilesSelectedKeys(currentSnapshot.profiles, target, keys),
        }),
      });
    },
    [mutateSnapshot, target],
  );

  const addKey = useCallback(() => {
    const nextKey = keyOptions.find((option) => !selectedKeys.some((key) => key.vk === option.vk));
    if (!nextKey) return;
    updateSelectedKeys([...selectedKeys, { vk: nextKey.vk, intervalMs: 20, mode: "hold" }]);
  }, [selectedKeys, updateSelectedKeys]);

  const updateKey = useCallback(
    (index: number, patch: Partial<KeyBinding>) => {
      updateSelectedKeys(
        selectedKeys.map((key, keyIndex) =>
          keyIndex === index
            ? {
                ...key,
                ...patch,
              }
            : key,
        ),
      );
    },
    [selectedKeys, updateSelectedKeys],
  );

  const deleteKey = useCallback(
    (index: number) => {
      updateSelectedKeys(selectedKeys.filter((_, keyIndex) => keyIndex !== index));
    },
    [selectedKeys, updateSelectedKeys],
  );

  const updateEffectRule = useCallback(
    (effectRule: EffectRule) => {
      if (target?.type !== "profile") return;
      void mutateSnapshot(() => tauriCommands.updateProfileEffectRule(target.configId, effectRule));
    },
    [mutateSnapshot, target],
  );

  return {
    addKey,
    deleteKey,
    selectedKeys,
    selectedTitle,
    updateEffectRule,
    updateKey,
    visibleAutofireClassCategories,
    visibleCustomConfigs,
  };
}

function updateProfilesSelectedKeys(
  profiles: ProfilesConfig,
  target: EditTarget,
  keys: KeyBinding[],
): ProfilesConfig {
  if (target.type === "global") {
    return { ...profiles, globalKeys: keys };
  }

  if (Object.prototype.hasOwnProperty.call(profiles.customConfigs, target.configId)) {
    return {
      ...profiles,
      customConfigs: {
        ...profiles.customConfigs,
        [target.configId]: {
          ...getProfileConfig(profiles, target.configId),
          name: profiles.customConfigs[target.configId]?.name ?? "自定义配置",
          enabledKeys: keys,
        },
      },
    };
  }

  return {
    ...profiles,
    classes: {
      ...profiles.classes,
      [target.configId]: {
        ...getProfileConfig(profiles, target.configId),
        enabledKeys: keys,
      },
    },
  };
}
