import type { RuntimeStateChangedPayload } from "../lib/app-events";
import type {
  AppStateSnapshot,
  BootstrapEffectiveProfile,
  BootstrapRuntimeState,
  BootstrapState,
  ClassCategory,
  ProfileDisplaySnapshot,
  ProfilesConfig,
  SettingsConfig,
} from "../types/app-config";

const DEFAULT_LOG_LEVEL = import.meta.env.DEV ? "debug" : "info";

export const DEFAULT_SETTINGS_CONFIG: SettingsConfig = {
  version: 1,
  launchAtStartup: false,
  startMinimized: false,
  minimizeToTray: false,
  openFloatingControlOnStart: false,
  logLevel: DEFAULT_LOG_LEVEL,
  toggleHotkey: { ctrl: true, alt: false, shift: false, vk: 0x77 },
  detection: {
    enabled: false,
    intervalMs: 200,
    noMatchPolicy: "current",
  },
  floatingControl: {
    position: null,
  },
};

export const DEFAULT_PROFILES_CONFIG: ProfilesConfig = {
  version: 1,
  globalKeys: [{ vk: 0x58, intervalMs: 20, mode: "hold" }],
  comboDefs: [],
  classes: {},
  customConfigs: {},
  hiddenClassIds: [],
  activeClassId: null,
  autoRun: {
    enabled: false,
    leftVk: 0x25,
    rightVk: 0x27,
    pulseDelayMs: 25,
  },
};

export type AppStoreState = {
  revision: number;
  settings: SettingsConfig;
  profiles: ProfilesConfig;
  classCategories: ClassCategory[];
  profileDisplay: ProfileDisplaySnapshot;
  effectiveProfile: BootstrapEffectiveProfile;
  runtime: BootstrapRuntimeState;
};

export type AppStoreAction =
  | {
      type: "bootstrapLoaded";
      bootstrap: BootstrapState;
    }
  | {
      type: "stateSnapshotApplied";
      snapshot: AppStateSnapshot;
      force?: boolean;
    }
  | {
      type: "runtimeStateApplied";
      runtime: RuntimeStateChangedPayload;
    };

const INITIAL_RUNTIME_STATE: BootstrapRuntimeState = {
  revision: 0,
  assistantRunning: false,
  detectionRunning: false,
  floatingControlVisible: false,
  activeToggleKeys: [],
};

const INITIAL_EFFECTIVE_PROFILE: BootstrapEffectiveProfile = {
  keys: [],
  combos: [],
};

const INITIAL_PROFILE_DISPLAY: ProfileDisplaySnapshot = {
  configOptions: [],
  visibleClassCategories: [],
  displayNames: {},
  classStates: {},
  customConfigStates: {},
};

export const initialAppStoreState: AppStoreState = {
  revision: 0,
  settings: DEFAULT_SETTINGS_CONFIG,
  profiles: DEFAULT_PROFILES_CONFIG,
  classCategories: [],
  profileDisplay: INITIAL_PROFILE_DISPLAY,
  effectiveProfile: INITIAL_EFFECTIVE_PROFILE,
  runtime: INITIAL_RUNTIME_STATE,
};

export const appActions = {
  bootstrapLoaded: (bootstrap: BootstrapState): AppStoreAction => ({
    type: "bootstrapLoaded",
    bootstrap,
  }),
  stateSnapshotApplied: (snapshot: AppStateSnapshot, force = false): AppStoreAction => ({
    type: "stateSnapshotApplied",
    snapshot,
    force,
  }),
  runtimeStateApplied: (runtime: RuntimeStateChangedPayload): AppStoreAction => ({
    type: "runtimeStateApplied",
    runtime,
  }),
};

export function appReducer(state: AppStoreState, action: AppStoreAction): AppStoreState {
  switch (action.type) {
    case "bootstrapLoaded":
      if (action.bootstrap.revision < state.revision) return state;
      return {
        ...state,
        revision: action.bootstrap.revision,
        settings: action.bootstrap.settings,
        profiles: action.bootstrap.profiles,
        classCategories: action.bootstrap.classCategories,
        profileDisplay: action.bootstrap.profileDisplay,
        effectiveProfile: action.bootstrap.effectiveProfile,
        runtime: action.bootstrap.runtime,
      };
    case "stateSnapshotApplied":
      if (!action.force && action.snapshot.revision < state.revision) return state;
      return {
        ...state,
        revision: action.snapshot.revision,
        settings: action.snapshot.settings,
        profiles: action.snapshot.profiles,
        profileDisplay: action.snapshot.profileDisplay,
        effectiveProfile: action.snapshot.effectiveProfile,
        runtime: action.snapshot.runtime,
      };
    case "runtimeStateApplied":
      if (action.runtime.revision < state.revision) return state;
      return {
        ...state,
        revision: action.runtime.revision,
        effectiveProfile: action.runtime.effectiveProfile,
        runtime: {
          revision: action.runtime.revision,
          assistantRunning: action.runtime.assistantRunning,
          detectionRunning: action.runtime.detectionRunning,
          floatingControlVisible: action.runtime.floatingControlVisible,
          activeToggleKeys: action.runtime.activeToggleKeys,
        },
      };
  }
}

export const appSelectors = {
  settings: (state: AppStoreState) => state.settings,
  revision: (state: AppStoreState) => state.revision,
  profiles: (state: AppStoreState) => state.profiles,
  classCategories: (state: AppStoreState) => state.classCategories,
  profileDisplay: (state: AppStoreState) => state.profileDisplay,
  effectiveProfile: (state: AppStoreState) => state.effectiveProfile,
  runtime: (state: AppStoreState) => state.runtime,
};
