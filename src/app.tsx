// 主窗口入口：负责配置编辑、连发运行状态，以及托盘和悬浮窗的状态同步。
import {
  CircleHelp,
  Keyboard,
  ListChecks,
  Play,
  Search,
  Settings,
  Square,
  Trash2,
  Wand2,
} from "lucide-react";
import { useCallback, useMemo, useState } from "react";
import {
  AppTitleBar,
  ConfigSelect,
  KeySummary,
  KeyTable,
  MessageDialog,
  NavButton,
  RuleButton,
  RuleHelpTooltip,
} from "./components/app-ui";
import { ComboEditorPage } from "./components/combo-editor";
import { classCategories } from "./data/classes";
import { FloatingControlView } from "./floating-control/floating-control-view";
import { useAppConfig } from "./hooks/use-app-config";
import { useAssistantRuntime } from "./hooks/use-assistant-runtime";
import { useFloatingControlSync } from "./hooks/use-floating-control-sync";
import { useHotkeyRecorder } from "./hooks/use-hotkey-recorder";
import {
  computeEffectiveCombos,
  computeEffectiveKeys,
  computeEffectiveKeysForProfile,
  configuredConfigOptions,
  getClassConfig,
  getConfigDisplayName,
  getCustomConfig,
  getProfileConfig,
  hasClassConfig,
  hasClassKeyConfig,
  hasDuplicateKeys,
  isClassVisible,
  isCustomConfigId,
  validateClassComboDefs,
} from "./lib/config";
import { FLOATING_CONTROL_VIEW } from "./lib/floating-control";
import { keyOptions, normalizeInterval } from "./lib/keys";
import {
  type AppConfig,
  ComboDefinition,
  EffectRule,
  KeyBinding,
  LogLevelSetting,
  hotkeyDisplay,
  isMockMode,
  makeCustomConfig,
  tauriCommands,
} from "./lib/tauri";
import { AboutPage } from "./pages/about-page";
import { ConfigManagementPage } from "./pages/config-management-page";
import { SettingsPage } from "./pages/settings-page";
import type { EditTarget, Page } from "./types/ui";

type RuntimeState = {
  running: boolean;
};
type UiState = {
  page: Page;
  target: EditTarget | null;
  comboClassId: string | null;
  recordingHotkey: boolean;
  floatingControlEnabled: boolean;
  message: string | null;
};

function App() {
  // 同一个前端包同时承载主窗口和悬浮窗，通过 URL 参数选择具体视图。
  const view = new URLSearchParams(window.location.search).get("view");
  if (view === FLOATING_CONTROL_VIEW) {
    return <FloatingControlView />;
  }

  return <MainApp />;
}

function MainApp() {
  const [runtimeState, setRuntimeState] = useState<RuntimeState>({ running: false });
  const [autofireClassSearch, setAutofireClassSearch] = useState("");
  const [uiState, setUiState] = useState<UiState>({
    page: "autofire",
    target: null,
    comboClassId: null,
    recordingHotkey: false,
    floatingControlEnabled: false,
    message: null,
  });
  // 状态分层：config 是持久配置，runtimeState 是运行状态，uiState 只保存临时界面状态。
  const running = runtimeState.running;
  const { comboClassId, floatingControlEnabled, message, page, recordingHotkey, target } = uiState;

  const setRunning = useCallback((nextRunning: boolean) => {
    setRuntimeState({ running: nextRunning });
  }, []);

  const setPage = useCallback((nextPage: Page) => {
    setUiState((current) => ({ ...current, page: nextPage }));
  }, []);

  const setTarget = useCallback((nextTarget: EditTarget | null) => {
    setUiState((current) => ({ ...current, target: nextTarget }));
  }, []);

  const setComboClassId = useCallback((nextComboClassId: string | null) => {
    setUiState((current) => ({ ...current, comboClassId: nextComboClassId }));
  }, []);

  const setRecordingHotkey = useCallback((nextRecordingHotkey: boolean) => {
    setUiState((current) => ({ ...current, recordingHotkey: nextRecordingHotkey }));
  }, []);

  const setFloatingControlEnabled = useCallback((nextFloatingControlEnabled: boolean) => {
    setUiState((current) => ({
      ...current,
      floatingControlEnabled: nextFloatingControlEnabled,
    }));
  }, []);

  const toggleFloatingControlEnabled = useCallback(() => {
    setUiState((current) => ({
      ...current,
      floatingControlEnabled: !current.floatingControlEnabled,
    }));
  }, []);

  const showMessage = useCallback((nextMessage: string) => {
    setUiState((current) => ({ ...current, message: nextMessage }));
  }, []);

  const clearMessage = useCallback(() => {
    setUiState((current) => ({ ...current, message: null }));
  }, []);

  const handleStartupLoaded = useCallback(
    ({ config: nextConfig, running: isRunning }: { config: AppConfig; running: boolean }) => {
      setRunning(isRunning);
      setFloatingControlEnabled(nextConfig.settings.openFloatingControlOnStart);
      // 卸载保留用户数据后，重装首次启动要按配置重新同步 Windows Run 项。
      void tauriCommands.setLaunchAtStartup(nextConfig.settings.launchAtStartup).catch((reason) => {
        showMessage(reason instanceof Error ? reason.message : String(reason));
      });
    },
    [setFloatingControlEnabled, setRunning, showMessage],
  );

  // 配置的加载、保存和回滚都封装在 hook 里；这里仅消费最新快照。
  const { config, configRef, startupConfigLoaded, updateConfig, updateSettings } = useAppConfig({
    onSaveError: showMessage,
    onStartupLoaded: handleStartupLoaded,
  });
  const { launchAtStartup, minimizeToTray, openFloatingControlOnStart, startMinimized } =
    config.settings;
  const { logLevel } = config.settings;

  // 快捷键录制要短暂接管全局键盘输入，单独拆出去避免污染页面交互逻辑。
  useHotkeyRecorder({
    recordingHotkey,
    setRecordingHotkey,
    showMessage,
    updateConfig,
  });

  const selectedKeys =
    target?.type === "global"
      ? config.globalKeys
      : target?.type === "profile"
        ? getProfileConfig(config, target.configId).enabledKeys
        : [];
  const selectedTitle =
    target?.type === "global"
      ? "全局配置"
      : target?.type === "profile"
        ? getConfigDisplayName(config, target.configId)
        : "";
  const configOptions = useMemo(() => configuredConfigOptions(config), [config]);
  const visibleCustomConfigs = useMemo(() => {
    const normalizedSearch = autofireClassSearch.trim().toLowerCase();
    return Object.entries(config.customConfigs).filter(([, customConfig]) => {
      const name = customConfig.name.trim() || "未命名配置";
      return !normalizedSearch || name.toLowerCase().includes(normalizedSearch);
    });
  }, [autofireClassSearch, config.customConfigs]);

  const effectiveKeys = useMemo(() => computeEffectiveKeys(config), [config]);
  const effectiveCombos = useMemo(() => computeEffectiveCombos(config), [config]);
  const normalizedAutofireClassSearch = autofireClassSearch.trim().toLowerCase();
  const visibleAutofireClassCategories = useMemo(
    () =>
      classCategories
        .map((category) => ({
          ...category,
          classes: category.classes.filter(
            (classInfo) =>
              isClassVisible(config, classInfo.id) &&
              (!normalizedAutofireClassSearch ||
                classInfo.name.toLowerCase().includes(normalizedAutofireClassSearch) ||
                classInfo.id.toLowerCase().includes(normalizedAutofireClassSearch)),
          ),
        }))
        .filter((category) => category.classes.length > 0),
    [config, normalizedAutofireClassSearch],
  );
  const currentConfigLabel = useMemo(
    () => getConfigDisplayName(config, config.activeClassId),
    [config],
  );

  // 运行态同步、托盘文案和热键注册共用同一条状态链路，集中放进 hook。
  const toggleAssistant = useAssistantRuntime({
    currentConfigLabel,
    effectiveCombos,
    effectiveKeys,
    running,
    setRunning,
    showMessage,
    startupConfigLoaded,
    toggleHotkey: config.toggleHotkey,
  });

  // 悬浮控制与主窗口双向同步，但真正的配置落盘仍只发生在主窗口。
  useFloatingControlSync({
    config,
    floatingControlEnabled,
    running,
    setFloatingControlEnabled,
    showMessage,
    startupConfigLoaded,
    toggleFloatingControlEnabled,
    updateConfig,
  });

  function updateSelectedKeys(keys: KeyBinding[]) {
    if (!target) return;
    if (hasDuplicateKeys(keys)) {
      showMessage("同一配置中不能重复添加相同按键");
      return;
    }
    if (target.type === "global") {
      void updateConfig((currentConfig) => ({ ...currentConfig, globalKeys: keys }));
      return;
    }

    void updateConfig((currentConfig) => {
      if (isCustomConfigId(currentConfig, target.configId)) {
        const customConfig = getCustomConfig(currentConfig, target.configId);
        const nextCustomConfig = { ...customConfig, enabledKeys: keys };
        return {
          ...currentConfig,
          activeClassId:
            currentConfig.activeClassId === target.configId && !hasClassConfig(nextCustomConfig)
              ? null
              : currentConfig.activeClassId,
          customConfigs: {
            ...currentConfig.customConfigs,
            [target.configId]: nextCustomConfig,
          },
        };
      }

      const classConfig = getClassConfig(currentConfig, target.configId);
      const nextClassConfig = { ...classConfig, enabledKeys: keys };
      // 职业配置没有连发键和连招时才移除，避免误删 combo-only 职业。
      if (!hasClassConfig(nextClassConfig)) {
        const nextClasses = { ...currentConfig.classes };
        delete nextClasses[target.configId];
        return {
          ...currentConfig,
          activeClassId:
            currentConfig.activeClassId === target.configId ? null : currentConfig.activeClassId,
          classes: nextClasses,
        };
      }

      return {
        ...currentConfig,
        classes: {
          ...currentConfig.classes,
          [target.configId]: nextClassConfig,
        },
      };
    });
  }

  function updateProfileCombos(configId: string, combos: ComboDefinition[]) {
    const issues = validateClassComboDefs(
      combos,
      computeEffectiveKeysForProfile(configRef.current, configId),
    );
    if (issues.length > 0) {
      showMessage(issues[0].message);
      return;
    }

    void updateConfig((currentConfig) => {
      if (isCustomConfigId(currentConfig, configId)) {
        const customConfig = getCustomConfig(currentConfig, configId);
        const nextCustomConfig = { ...customConfig, comboDefs: combos };
        return {
          ...currentConfig,
          activeClassId:
            currentConfig.activeClassId === configId && !hasClassConfig(nextCustomConfig)
              ? null
              : currentConfig.activeClassId,
          customConfigs: {
            ...currentConfig.customConfigs,
            [configId]: nextCustomConfig,
          },
        };
      }

      const classConfig = getClassConfig(currentConfig, configId);
      const nextClassConfig = { ...classConfig, comboDefs: combos };
      if (!hasClassConfig(nextClassConfig)) {
        const nextClasses = { ...currentConfig.classes };
        delete nextClasses[configId];
        return {
          ...currentConfig,
          activeClassId:
            currentConfig.activeClassId === configId ? null : currentConfig.activeClassId,
          classes: nextClasses,
        };
      }

      return {
        ...currentConfig,
        classes: {
          ...currentConfig.classes,
          [configId]: nextClassConfig,
        },
      };
    });
  }

  function updateOpenFloatingControlOnStart(checked: boolean) {
    updateSettings({ openFloatingControlOnStart: checked });
  }

  function updateStartMinimized(checked: boolean) {
    updateSettings({ startMinimized: checked });
  }

  function updateMinimizeToTray(checked: boolean) {
    updateSettings({ minimizeToTray: checked });
  }

  function updateLogLevel(level: LogLevelSetting) {
    updateSettings({ logLevel: level });
    void tauriCommands.setLogLevel(level).catch((reason) => {
      showMessage(reason instanceof Error ? reason.message : String(reason));
    });
  }

  async function updateLaunchAtStartup(checked: boolean) {
    try {
      await tauriCommands.setLaunchAtStartup(checked);
      updateSettings({ launchAtStartup: checked });
    } catch (reason) {
      showMessage(reason instanceof Error ? reason.message : String(reason));
    }
  }

  function updateClassEffectRule(effectRule: EffectRule) {
    if (target?.type !== "profile") return;
    void updateConfig((currentConfig) => {
      if (isCustomConfigId(currentConfig, target.configId)) {
        const customConfig = getCustomConfig(currentConfig, target.configId);
        return {
          ...currentConfig,
          customConfigs: {
            ...currentConfig.customConfigs,
            [target.configId]: { ...customConfig, effectRule },
          },
        };
      }

      const classConfig = getClassConfig(currentConfig, target.configId);
      return {
        ...currentConfig,
        classes: {
          ...currentConfig.classes,
          [target.configId]: { ...classConfig, effectRule },
        },
      };
    });
  }

  function addCustomConfig(name: string) {
    const trimmedName = name.trim();
    if (!trimmedName) {
      showMessage("自定义配置名称不能为空");
      return;
    }
    const id = `custom-${globalThis.crypto?.randomUUID?.() ?? Date.now().toString(36)}`;
    void updateConfig((currentConfig) => ({
      ...currentConfig,
      customConfigs: {
        ...currentConfig.customConfigs,
        [id]: makeCustomConfig(trimmedName),
      },
    }));
  }

  function deleteCustomConfig(configId: string) {
    const customConfig = config.customConfigs[configId];
    if (!customConfig) return;

    void updateConfig((currentConfig) => {
      const nextCustomConfigs = { ...currentConfig.customConfigs };
      delete nextCustomConfigs[configId];
      return {
        ...currentConfig,
        activeClassId:
          currentConfig.activeClassId === configId ? null : currentConfig.activeClassId,
        customConfigs: nextCustomConfigs,
      };
    });
    if (target?.type === "profile" && target.configId === configId) {
      setTarget(null);
    }
    if (comboClassId === configId) {
      setComboClassId(null);
    }
  }

  function toggleClassHidden(classId: string, hidden: boolean) {
    if (hasClassConfig(config.classes[classId])) return;
    void updateConfig((currentConfig) => {
      const hiddenClassIds = new Set(currentConfig.hiddenClassIds);
      if (hidden) {
        hiddenClassIds.add(classId);
      } else {
        hiddenClassIds.delete(classId);
      }
      return { ...currentConfig, hiddenClassIds: Array.from(hiddenClassIds) };
    });
  }

  function addKey() {
    const nextKey = keyOptions.find((option) => !selectedKeys.some((key) => key.vk === option.vk));
    if (!nextKey) return;
    updateSelectedKeys([...selectedKeys, { vk: nextKey.vk, intervalMs: 20 }]);
  }

  function updateKey(index: number, patch: Partial<KeyBinding>) {
    updateSelectedKeys(
      selectedKeys.map((key, keyIndex) =>
        keyIndex === index
          ? { ...key, ...patch, intervalMs: normalizeInterval(patch.intervalMs ?? key.intervalMs) }
          : key,
      ),
    );
  }

  function deleteKey(index: number) {
    updateSelectedKeys(selectedKeys.filter((_, keyIndex) => keyIndex !== index));
  }

  function openTarget(nextTarget: EditTarget) {
    setTarget(nextTarget);
  }

  function closeTarget() {
    setTarget(null);
  }

  function changePage(nextPage: Page) {
    setTarget(null);
    setPage(nextPage);
  }

  return (
    <div className="flex h-screen w-screen flex-col overflow-hidden bg-[#eef3f8] text-slate-950">
      <AppTitleBar minimizeToTray={minimizeToTray} />
      <div className="flex min-h-0 flex-1">
        <aside className="flex w-[188px] shrink-0 flex-col border-r border-slate-200 bg-[#111827] px-4 py-5 text-slate-200">
          <nav className="space-y-2">
            <NavButton
              active={page === "autofire"}
              icon={<Keyboard size={18} />}
              label="按键连发"
              onClick={() => changePage("autofire")}
            />
            <NavButton
              active={page === "combo"}
              icon={<Wand2 size={18} />}
              label="一键连招(Beta)"
              onClick={() => changePage("combo")}
            />
          </nav>

          <div className="mt-auto space-y-2">
            <NavButton
              active={page === "config-management"}
              icon={<ListChecks size={18} />}
              label="配置管理"
              onClick={() => changePage("config-management")}
            />
            <NavButton
              active={page === "settings"}
              icon={<Settings size={18} />}
              label="设置"
              onClick={() => changePage("settings")}
            />
            <NavButton
              active={page === "about"}
              icon={<CircleHelp size={18} />}
              label="关于"
              onClick={() => changePage("about")}
            />
          </div>
        </aside>

        <main className="grid min-w-0 flex-1 grid-rows-[1fr_76px]">
          <div className="min-h-0 overflow-hidden">
            {page === "autofire" ? (
              <div className="relative h-full min-w-0">
                <section className="h-full min-w-0 overflow-y-auto px-7 py-6" onClick={closeTarget}>
                  <header className="mb-5 flex items-start justify-between gap-5">
                    <div>
                      <div className="flex items-center gap-2">
                        <h1 className="text-[22px] font-semibold tracking-tight">按键连发</h1>
                      </div>
                      <div className="mt-1 space-y-1 text-sm leading-6 text-slate-500">
                        <p>选择全局、职业或自定义配置，在右侧编辑键位和连发间隔。</p>
                      </div>
                    </div>
                    <div className="flex shrink-0 items-center gap-3">
                      {isMockMode() && (
                        <span className="rounded bg-amber-100 px-2 py-1 text-xs text-amber-800">
                          浏览器预览
                        </span>
                      )}
                      <label className="flex h-9 w-[260px] items-center gap-2 rounded border border-slate-200 bg-white px-2.5 text-slate-500 shadow-sm focus-within:border-blue-400 focus-within:ring-1 focus-within:ring-blue-100">
                        <Search size={15} />
                        <input
                          className="min-w-0 flex-1 bg-transparent text-sm text-slate-800 outline-none placeholder:text-slate-400"
                          placeholder="搜索职业/自定义配置"
                          value={autofireClassSearch}
                          onChange={(event) => setAutofireClassSearch(event.currentTarget.value)}
                        />
                      </label>
                    </div>
                  </header>

                  <button
                    className={cardClass(target?.type === "global")}
                    type="button"
                    onClick={(event) => {
                      event.stopPropagation();
                      openTarget({ type: "global" });
                    }}
                  >
                    <div>
                      <div className="text-sm font-semibold">全局配置</div>
                      <div className="mt-2 flex flex-wrap gap-1.5">
                        <KeySummary active={target?.type === "global"} keys={config.globalKeys} />
                      </div>
                    </div>
                  </button>

                  {visibleCustomConfigs.length > 0 && (
                    <div className="mt-6">
                      <h2 className="mb-3 text-sm font-semibold text-slate-700">自定义配置</h2>
                      <div className="flex flex-wrap gap-2">
                        {visibleCustomConfigs.map(([configId, customConfig]) => {
                          const active = target?.type === "profile" && target.configId === configId;
                          const configured = hasClassKeyConfig(customConfig);
                          return (
                            <button
                              key={configId}
                              className={classButtonClass(active)}
                              type="button"
                              onClick={(event) => {
                                event.stopPropagation();
                                openTarget({ type: "profile", configId });
                              }}
                            >
                              <span>{customConfig.name || "未命名配置"}</span>
                              {configured && <span className={configuredDotClass(active)} />}
                            </button>
                          );
                        })}
                      </div>
                    </div>
                  )}

                  <div className="mt-6 pb-6">
                    <h2 className="mb-3 text-sm font-semibold text-slate-700">职业配置</h2>
                    <div className="space-y-4">
                      {visibleAutofireClassCategories.map((category) => (
                        <div
                          key={category.name}
                          className="grid grid-cols-[84px_1fr] items-center gap-3"
                        >
                          <div className="text-sm font-medium text-slate-600">{category.name}</div>
                          <div className="flex flex-wrap gap-2">
                            {category.classes.map((classInfo) => {
                              const active =
                                target?.type === "profile" && target.configId === classInfo.id;
                              const configured = hasClassKeyConfig(config.classes[classInfo.id]);
                              return (
                                <span key={classInfo.id} className="group relative inline-flex">
                                  <button
                                    className={classButtonClass(active)}
                                    type="button"
                                    onClick={(event) => {
                                      event.stopPropagation();
                                      openTarget({ type: "profile", configId: classInfo.id });
                                    }}
                                  >
                                    <span>{classInfo.name}</span>
                                    {configured && <span className={configuredDotClass(active)} />}
                                  </button>
                                </span>
                              );
                            })}
                          </div>
                        </div>
                      ))}
                      {visibleAutofireClassCategories.length === 0 && (
                        <div className="rounded border border-dashed border-slate-200 bg-white px-3 py-8 text-center text-sm text-slate-500">
                          没有匹配的职业。
                        </div>
                      )}
                    </div>
                  </div>
                </section>

                {target && (
                  <div
                    className="absolute inset-0 z-20 flex justify-end bg-slate-950/10"
                    onClick={closeTarget}
                  >
                    <aside
                      className="h-full w-[380px] border-l border-slate-200 bg-white px-5 py-6 shadow-2xl"
                      onClick={(event) => event.stopPropagation()}
                    >
                      <div className="mb-5">
                        <h2 className="text-lg font-semibold">{selectedTitle}</h2>
                        <p className="mt-1 text-xs text-slate-500">按键与间隔会自动保存。</p>
                      </div>

                      {target.type === "profile" && (
                        <>
                          <div className="mb-5">
                            <div className="mb-2 flex items-center gap-1.5 text-xs font-medium text-slate-500">
                              <span>生效规则</span>
                              <RuleHelpTooltip />
                            </div>
                            <div className="grid grid-cols-2 rounded border border-slate-200 bg-slate-50 p-1">
                              <RuleButton
                                active={
                                  getProfileConfig(config, target.configId).effectRule ===
                                  "globalAndClass"
                                }
                                label="全局 + 当前配置"
                                onClick={() => updateClassEffectRule("globalAndClass")}
                              />
                              <RuleButton
                                active={
                                  getProfileConfig(config, target.configId).effectRule ===
                                  "classOnly"
                                }
                                label="仅当前配置"
                                onClick={() => updateClassEffectRule("classOnly")}
                              />
                            </div>
                          </div>
                        </>
                      )}

                      <KeyTable
                        keys={selectedKeys}
                        onAdd={addKey}
                        onDelete={deleteKey}
                        onUpdate={updateKey}
                      />
                    </aside>
                  </div>
                )}
              </div>
            ) : page === "combo" ? (
              <ComboEditorPage
                config={config}
                selectedConfigId={comboClassId}
                onCombosChange={updateProfileCombos}
                onSelectedConfigIdChange={(configId) => setComboClassId(configId)}
              />
            ) : page === "config-management" ? (
              <ConfigManagementPage
                config={config}
                onAddCustomConfig={addCustomConfig}
                onDeleteCustomConfig={deleteCustomConfig}
                onToggleClassHidden={toggleClassHidden}
              />
            ) : page === "settings" ? (
              <SettingsPage
                launchAtStartup={launchAtStartup}
                logLevel={logLevel}
                minimizeToTray={minimizeToTray}
                openFloatingControlOnStart={openFloatingControlOnStart}
                startMinimized={startMinimized}
                onLaunchAtStartupChange={(checked) => void updateLaunchAtStartup(checked)}
                onLogLevelChange={updateLogLevel}
                onMinimizeToTrayChange={updateMinimizeToTray}
                onOpenFloatingControlOnStartChange={updateOpenFloatingControlOnStart}
                onStartMinimizedChange={updateStartMinimized}
              />
            ) : (
              <AboutPage />
            )}
          </div>

          <footer className="flex items-center gap-5 border-t border-slate-200 bg-white px-7">
            <div className="min-w-0 flex-1">
              <div className="flex items-center gap-2">
                <span className="h-2 w-2 rounded-full bg-amber-400" />
                <span className="text-sm font-medium text-slate-700">配置</span>
                <ConfigSelect
                  activeClassId={config.activeClassId}
                  options={configOptions}
                  placement="top"
                  onChange={(id) =>
                    void updateConfig((currentConfig) => ({
                      ...currentConfig,
                      activeClassId: id,
                    }))
                  }
                />
                <label className="ml-3 inline-flex cursor-pointer items-center gap-2 rounded border border-slate-200 bg-slate-50 px-2.5 py-1 text-xs font-medium text-slate-600 transition hover:border-blue-200 hover:bg-blue-50 hover:text-blue-700">
                  <input
                    checked={floatingControlEnabled}
                    className="h-3.5 w-3.5 rounded border-slate-300 text-blue-600"
                    type="checkbox"
                    onChange={(event) => setFloatingControlEnabled(event.currentTarget.checked)}
                  />
                  显示悬浮窗
                </label>
              </div>
              <div className="mt-1 flex min-w-0 items-center gap-1.5 overflow-hidden">
                <span className="shrink-0 text-xs text-slate-500">当前生效</span>
                <div className="flex min-w-0 gap-1.5 overflow-hidden">
                  <KeySummary active={false} keys={effectiveKeys} />
                  {effectiveCombos.length > 0 && (
                    <span className="rounded border border-emerald-200 bg-emerald-50 px-2 py-1 text-xs font-medium text-emerald-700 shadow-sm">
                      {effectiveCombos.length} 个连招
                    </span>
                  )}
                </div>
              </div>
            </div>
            <div className="flex shrink-0 items-center gap-6">
              <div className="grid grid-cols-[auto_1fr_auto] items-center gap-2">
                <span className="text-sm font-medium text-slate-700">启动/停止快捷键</span>
                <button
                  className="h-9 min-w-[176px] rounded border border-slate-300 bg-slate-50 px-3 text-left text-sm font-medium text-slate-800 transition hover:border-blue-400 hover:bg-blue-50"
                  type="button"
                  onClick={() => setRecordingHotkey(true)}
                >
                  {recordingHotkey ? "请按下快捷键..." : hotkeyDisplay(config.toggleHotkey)}
                </button>
                <button
                  className="inline-flex h-9 w-9 items-center justify-center rounded border border-slate-300 text-slate-500 transition hover:border-red-300 hover:bg-red-50 hover:text-red-600"
                  type="button"
                  onClick={() =>
                    void updateConfig((currentConfig) => ({
                      ...currentConfig,
                      toggleHotkey: null,
                    }))
                  }
                >
                  <Trash2 size={16} />
                </button>
              </div>

              <div className="flex items-center justify-end">
                <button
                  className={`inline-flex h-10 items-center gap-2 rounded px-5 text-sm font-semibold text-white shadow-sm transition ${
                    running ? "bg-red-600 hover:bg-red-700" : "bg-blue-600 hover:bg-blue-700"
                  }`}
                  type="button"
                  onClick={() => void toggleAssistant()}
                >
                  {running ? <Square size={16} /> : <Play size={16} />}
                  {running ? "停止助手" : "启动助手"}
                </button>
              </div>
            </div>
          </footer>
        </main>
      </div>
      {message && <MessageDialog message={message} onClose={clearMessage} />}
    </div>
  );
}

function cardClass(active: boolean): string {
  return `w-full rounded border px-4 py-3 text-left transition ${
    active
      ? "border-blue-300 bg-blue-50 shadow-sm"
      : "border-slate-200 bg-white shadow-sm hover:border-blue-200 hover:bg-blue-50/40"
  }`;
}

function classButtonClass(active: boolean): string {
  return `relative inline-flex h-9 w-[120px] items-center justify-center rounded border px-2 text-sm transition ${
    active
      ? "border-blue-400 bg-blue-600 text-white shadow-sm"
      : "border-slate-200 bg-white text-slate-700 hover:border-blue-300 hover:bg-blue-50"
  }`;
}

function configuredDotClass(active: boolean): string {
  return `absolute top-1.5 right-1.5 h-1.5 w-1.5 rounded-full ${
    active ? "bg-white" : "bg-blue-500"
  }`;
}

export default App;
