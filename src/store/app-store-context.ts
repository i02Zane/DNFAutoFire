import { createContext, useContext, useEffect } from "react";
import type {
  AppStateSnapshot,
  BootstrapEffectiveProfile,
  BootstrapRuntimeState,
  ProfileDisplaySnapshot,
  ProfilesConfig,
  SettingsConfig,
  ClassCategory,
} from "../types/app-config";

export type SettingsUpdater =
  | SettingsConfig
  | ((currentSettings: SettingsConfig, currentProfiles: ProfilesConfig) => SettingsConfig);

export type SnapshotMutationOptions = {
  optimistic?: (currentSnapshot: AppStateSnapshot) => AppStateSnapshot;
};

export type AppStoreHandlers = {
  onSaveError: (message: string) => void;
};

export type AppStoreContextValue = {
  settings: SettingsConfig;
  profiles: ProfilesConfig;
  classCategories: ClassCategory[];
  profileDisplay: ProfileDisplaySnapshot;
  effectiveProfile: BootstrapEffectiveProfile;
  runtime: BootstrapRuntimeState;
  updateSettings: (updater: SettingsUpdater) => Promise<AppStateSnapshot | null>;
  mutateSnapshot: (
    mutation: () => Promise<AppStateSnapshot>,
    options?: SnapshotMutationOptions,
  ) => Promise<AppStateSnapshot | null>;
  registerHandlers: (handlers: AppStoreHandlers) => () => void;
};

export const AppStoreContext = createContext<AppStoreContextValue | null>(null);

export function useAppStore(handlers: AppStoreHandlers): AppStoreContextValue {
  const context = useContext(AppStoreContext);
  if (!context) {
    throw new Error("useAppStore must be used within AppProvider");
  }
  const { registerHandlers } = context;
  const { onSaveError } = handlers;

  useEffect(() => registerHandlers({ onSaveError }), [onSaveError, registerHandlers]);

  return context;
}
