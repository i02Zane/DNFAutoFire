import { type ReactNode, useCallback, useEffect, useMemo, useReducer, useRef } from "react";
import { APP_EVENTS, listenAppEvent } from "../lib/app-events";
import { tauriCommands } from "../lib/tauri-commands";
import { isTauriEnvironment } from "../lib/tauri-env";
import type { AppStateSnapshot, ProfilesConfig, SettingsConfig } from "../types/app-config";
import { appErrorMessage } from "../types/app-error";
import { appActions, appReducer, appSelectors, initialAppStoreState } from "./app-store";
import { AppStoreContext } from "./app-store-context";
import type {
  AppStoreContextValue,
  AppStoreHandlers,
  SettingsUpdater,
  SnapshotMutationOptions,
} from "./app-store-context";

const noopHandlers: AppStoreHandlers = {
  onSaveError: () => undefined,
};

const initialSnapshot: AppStateSnapshot = {
  revision: initialAppStoreState.revision,
  settings: initialAppStoreState.settings,
  profiles: initialAppStoreState.profiles,
  profileDisplay: initialAppStoreState.profileDisplay,
  runtime: initialAppStoreState.runtime,
  effectiveProfile: initialAppStoreState.effectiveProfile,
};

export function AppProvider({ children }: { children: ReactNode }) {
  const [storeState, dispatch] = useReducer(appReducer, initialAppStoreState);
  const settingsRef = useRef<SettingsConfig>(storeState.settings);
  const profilesRef = useRef<ProfilesConfig>(storeState.profiles);
  const snapshotRef = useRef<AppStateSnapshot>(initialSnapshot);
  const lastSavedSnapshotRef = useRef<AppStateSnapshot>(initialSnapshot);
  const configSaveIdRef = useRef(0);
  const configSaveQueueRef = useRef(Promise.resolve());
  const handlersRef = useRef<AppStoreHandlers>(noopHandlers);

  const settings = appSelectors.settings(storeState);
  const profiles = appSelectors.profiles(storeState);
  const classCategories = appSelectors.classCategories(storeState);
  const profileDisplay = appSelectors.profileDisplay(storeState);
  const effectiveProfile = appSelectors.effectiveProfile(storeState);
  const runtime = appSelectors.runtime(storeState);

  const applySnapshot = useCallback((snapshot: AppStateSnapshot, force = false) => {
    if (!force && snapshot.revision < snapshotRef.current.revision) {
      return false;
    }
    settingsRef.current = snapshot.settings;
    profilesRef.current = snapshot.profiles;
    snapshotRef.current = snapshot;
    dispatch(appActions.stateSnapshotApplied(snapshot, force));
    return true;
  }, []);

  const mutateSnapshot = useCallback(
    async (
      mutation: () => Promise<AppStateSnapshot>,
      options: SnapshotMutationOptions = {},
    ): Promise<AppStateSnapshot | null> => {
      const saveId = configSaveIdRef.current + 1;
      const currentSnapshot = snapshotRef.current;
      configSaveIdRef.current = saveId;
      if (options.optimistic) {
        const optimisticSnapshot = options.optimistic(currentSnapshot);
        const optimisticRevision = Math.max(
          optimisticSnapshot.revision,
          currentSnapshot.revision + 1,
        );
        applySnapshot(
          {
            ...optimisticSnapshot,
            revision: optimisticRevision,
            runtime: {
              ...optimisticSnapshot.runtime,
              revision: optimisticRevision,
            },
          },
          true,
        );
      }

      const saveTask = configSaveQueueRef.current.then(async () => {
        return mutation();
      });
      configSaveQueueRef.current = saveTask.then(
        () => undefined,
        () => undefined,
      );

      try {
        const saved = await saveTask;
        if (saved.revision >= lastSavedSnapshotRef.current.revision) {
          lastSavedSnapshotRef.current = saved;
        }
        if (saveId === configSaveIdRef.current) {
          applySnapshot(saved);
        }
        return saved;
      } catch (reason) {
        if (saveId === configSaveIdRef.current) {
          applySnapshot(lastSavedSnapshotRef.current, true);
          handlersRef.current.onSaveError(appErrorMessage(reason));
        }
        return null;
      }
    },
    [applySnapshot],
  );

  const updateSettings = useCallback(
    async (updater: SettingsUpdater): Promise<AppStateSnapshot | null> => {
      const nextSettings =
        typeof updater === "function" ? updater(settingsRef.current, profilesRef.current) : updater;
      return mutateSnapshot(() => tauriCommands.saveSettings(nextSettings), {
        optimistic: (currentSnapshot) => ({
          ...currentSnapshot,
          settings: nextSettings,
        }),
      });
    },
    [mutateSnapshot],
  );

  const registerHandlers = useCallback((handlers: AppStoreHandlers) => {
    handlersRef.current = handlers;

    return () => {
      handlersRef.current = noopHandlers;
    };
  }, []);

  useEffect(() => {
    if (!isTauriEnvironment()) return;

    // 首屏加载时一次性恢复持久配置、职业目录和后端运行状态。
    void tauriCommands
      .loadBootstrap()
      .then((bootstrap) => {
        const snapshot: AppStateSnapshot = {
          revision: bootstrap.revision,
          settings: bootstrap.settings,
          profiles: bootstrap.profiles,
          profileDisplay: bootstrap.profileDisplay,
          runtime: bootstrap.runtime,
          effectiveProfile: bootstrap.effectiveProfile,
        };
        settingsRef.current = snapshot.settings;
        profilesRef.current = snapshot.profiles;
        snapshotRef.current = snapshot;
        lastSavedSnapshotRef.current = snapshot;
        dispatch(appActions.bootstrapLoaded(bootstrap));
      })
      .catch((reason) => {
        handlersRef.current.onSaveError(appErrorMessage(reason));
      });
  }, []);

  useEffect(() => {
    if (typeof window === "undefined") return;

    let disposed = false;
    let unlisten: (() => void) | undefined;
    const listenConfigChanged = async () => {
      unlisten = await listenAppEvent(APP_EVENTS.appConfigChanged, (snapshot) => {
        if (snapshot.revision < snapshotRef.current.revision) return;
        lastSavedSnapshotRef.current = snapshot;
        configSaveIdRef.current += 1;
        applySnapshot(snapshot);
      });
      if (disposed) unlisten?.();
    };

    void listenConfigChanged().catch(() => undefined);
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [applySnapshot]);

  useEffect(() => {
    if (typeof window === "undefined") return;

    let disposed = false;
    let unlistenRuntime: (() => void) | undefined;
    let unlistenRuntimeError: (() => void) | undefined;
    const listenRuntimeEvents = async () => {
      unlistenRuntime = await listenAppEvent(APP_EVENTS.runtimeStateChanged, (nextRuntime) => {
        if (nextRuntime.revision < snapshotRef.current.revision) return;
        snapshotRef.current = {
          ...snapshotRef.current,
          revision: nextRuntime.revision,
          runtime: {
            revision: nextRuntime.revision,
            assistantRunning: nextRuntime.assistantRunning,
            detectionRunning: nextRuntime.detectionRunning,
            floatingControlVisible: nextRuntime.floatingControlVisible,
            activeToggleKeys: nextRuntime.activeToggleKeys,
          },
          effectiveProfile: nextRuntime.effectiveProfile,
        };
        dispatch(appActions.runtimeStateApplied(nextRuntime));
      });
      unlistenRuntimeError = await listenAppEvent(APP_EVENTS.runtimeError, (error) => {
        handlersRef.current.onSaveError(error.message);
      });
      if (disposed) {
        unlistenRuntime?.();
        unlistenRuntimeError?.();
      }
    };

    void listenRuntimeEvents().catch(() => undefined);
    return () => {
      disposed = true;
      unlistenRuntime?.();
      unlistenRuntimeError?.();
    };
  }, []);

  const value = useMemo<AppStoreContextValue>(
    () => ({
      settings,
      profiles,
      classCategories,
      profileDisplay,
      effectiveProfile,
      registerHandlers,
      runtime,
      mutateSnapshot,
      updateSettings,
    }),
    [
      classCategories,
      effectiveProfile,
      profileDisplay,
      profiles,
      registerHandlers,
      runtime,
      settings,
      mutateSnapshot,
      updateSettings,
    ],
  );

  return <AppStoreContext.Provider value={value}>{children}</AppStoreContext.Provider>;
}
