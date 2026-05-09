// 主窗口入口：负责配置编辑、连发运行状态，以及托盘和悬浮窗的状态同步。
import { useCallback, useEffect, useState } from "react";
import { AppShell } from "./components/app-shell";
import { GlobalStatusBar } from "./components/global-status-bar";
import { AboutPage } from "./features/about/about-page";
import { AutoRunPage } from "./features/auto-run/auto-run-page";
import { AutofirePage } from "./features/autofire/autofire-page";
import { ComboEditorPage } from "./features/combo/combo-editor-page";
import { RuntimeDiagnosticsPage } from "./features/diagnostics/runtime-diagnostics-page";
import { FloatingControlView } from "./features/floating-control/floating-control-view";
import { ConfigManagementPage } from "./features/profile-management/config-management-page";
import { SettingsPage } from "./features/settings/settings-page";
import { useActiveConfigActions } from "./hooks/use-active-config-actions";
import { useAssistantRuntime } from "./hooks/use-assistant-runtime";
import { useHotkeyRecorder } from "./hooks/use-hotkey-recorder";
import { FLOATING_CONTROL_VIEW } from "./lib/floating-control";
import { tauriCommands } from "./lib/tauri-commands";
import { isMockMode } from "./lib/tauri-env";
import { getWebviewTextScale } from "./lib/window-scale";
import { AppProvider } from "./store/app-provider";
import { useAppStore } from "./store/app-store-context";
import type { EditTarget, Page } from "./types/ui";
import { useAutoRunActions } from "./features/auto-run/use-auto-run-actions";
import { useAutofireActions } from "./features/autofire/use-autofire-actions";
import { useComboActions } from "./features/combo/use-combo-actions";
import { useProfileManagementActions } from "./features/profile-management/use-profile-management-actions";
import { useSettingsActions } from "./features/settings/use-settings-actions";
type UiState = {
  page: Page;
  target: EditTarget | null;
  comboClassId: string | null;
  recordingHotkey: boolean;
  message: string | null;
};

function App() {
  // 同一个前端包同时承载主窗口和悬浮窗，通过 URL 参数选择具体视图。
  const view = new URLSearchParams(window.location.search).get("view");
  return (
    <AppProvider>
      {view === FLOATING_CONTROL_VIEW ? <FloatingControlView /> : <MainApp />}
    </AppProvider>
  );
}

function MainApp() {
  const [autofireClassSearch, setAutofireClassSearch] = useState("");
  const [uiState, setUiState] = useState<UiState>({
    page: "autofire",
    target: null,
    comboClassId: null,
    recordingHotkey: false,
    message: null,
  });
  // 状态分层：settings/profiles 是持久配置，runtime 是后端快照，uiState 只保存临时界面状态。
  const { comboClassId, message, page, recordingHotkey, target } = uiState;

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

  const showMessage = useCallback((nextMessage: string) => {
    setUiState((current) => ({ ...current, message: nextMessage }));
  }, []);

  const clearMessage = useCallback(() => {
    setUiState((current) => ({ ...current, message: null }));
  }, []);

  // 配置的加载、保存和回滚都封装在 hook 里；这里仅消费最新快照。
  const {
    settings,
    profiles,
    classCategories,
    profileDisplay,
    effectiveProfile,
    runtime,
    mutateSnapshot,
    updateSettings,
  } = useAppStore({
    onSaveError: showMessage,
  });
  const running = runtime.assistantRunning;
  const detectionRunning = runtime.detectionRunning;
  const activeToggleKeys = runtime.activeToggleKeys;
  const floatingControlVisible = runtime.floatingControlVisible;
  const { launchAtStartup, minimizeToTray, openFloatingControlOnStart, startMinimized, logLevel } =
    settings;
  const { autoRun } = profiles;
  const {
    enabled: detectionEnabled,
    intervalMs: detectionIntervalMs,
    noMatchPolicy,
  } = settings.detection;
  // 快捷键录制要短暂接管全局键盘输入，单独拆出去避免污染页面交互逻辑。
  useHotkeyRecorder({
    recordingHotkey,
    setRecordingHotkey,
    showMessage,
    updateSettings,
  });

  const effectiveKeys = effectiveProfile.keys;
  const effectiveCombos = effectiveProfile.combos;
  // 启动/停止动作只表达用户意图，实际运行态由后端事件回推。
  const toggleAssistant = useAssistantRuntime({
    running,
    showMessage,
  });
  const onFloatingControlVisibleChange = useCallback(
    (visible: boolean) => {
      void mutateSnapshot(() =>
        visible
          ? tauriCommands.showFloatingControlWindow()
          : tauriCommands.hideFloatingControlWindow(),
      );
    },
    [mutateSnapshot],
  );

  const {
    addKey,
    deleteKey,
    selectedKeys,
    selectedTitle,
    updateEffectRule: updateClassEffectRule,
    updateKey,
    visibleAutofireClassCategories,
    visibleCustomConfigs,
  } = useAutofireActions({
    autofireClassSearch,
    mutateSnapshot,
    profileDisplay,
    profiles,
    target,
  });
  const { updateProfileCombos, validateComboDefs } = useComboActions({
    mutateSnapshot,
  });
  const {
    updateDetectionEnabled,
    updateDetectionInterval,
    updateDetectionNoMatchPolicy,
    updateLaunchAtStartup,
    updateLogLevel,
    updateMinimizeToTray,
    updateOpenFloatingControlOnStart,
    updateStartMinimized,
  } = useSettingsActions({
    updateSettings,
  });
  const { addCustomConfig, deleteCustomConfig, toggleClassHidden } = useProfileManagementActions({
    comboClassId,
    mutateSnapshot,
    profileDisplay,
    profiles,
    setComboClassId,
    setTarget,
    showMessage,
    target,
  });
  const {
    onAutoRunEnabledChange,
    onAutoRunLeftVkChange,
    onAutoRunPulseDelayChange,
    onAutoRunRightVkChange,
  } = useAutoRunActions({ mutateSnapshot });
  const configOptions = profileDisplay.configOptions;
  const { onActiveConfigChange } = useActiveConfigActions({ mutateSnapshot });

  useEffect(() => {
    if (isMockMode()) return;

    const resizeMainWindow = async () => {
      const { PhysicalSize, currentMonitor, getCurrentWindow, primaryMonitor } =
        await import("@tauri-apps/api/window");
      const monitor = (await currentMonitor()) ?? (await primaryMonitor());
      if (!monitor) return;

      const textScale = getWebviewTextScale(monitor.scaleFactor);
      const baseSize = await getCurrentWindow().innerSize();
      await getCurrentWindow().setSize(
        new PhysicalSize(
          Math.ceil(baseSize.width * textScale),
          Math.ceil(baseSize.height * textScale),
        ),
      );
    };

    void resizeMainWindow().catch(() => undefined);
  }, []);
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
    <AppShell
      message={message}
      page={page}
      statusBar={
        <GlobalStatusBar
          activeToggleKeys={activeToggleKeys}
          configOptions={configOptions}
          detectionRunning={detectionRunning}
          effectiveCombos={effectiveCombos}
          effectiveKeys={effectiveKeys}
          floatingControlVisible={floatingControlVisible}
          recordingHotkey={recordingHotkey}
          running={running}
          onFloatingControlVisibleChange={onFloatingControlVisibleChange}
          setRecordingHotkey={setRecordingHotkey}
          profiles={profiles}
          settings={settings}
          onActiveConfigChange={onActiveConfigChange}
          toggleAssistant={toggleAssistant}
          updateSettings={updateSettings}
        />
      }
      onMessageClose={clearMessage}
      onPageChange={changePage}
    >
      {page === "autofire" ? (
        <AutofirePage
          autofireClassSearch={autofireClassSearch}
          closeTarget={closeTarget}
          profiles={profiles}
          openTarget={openTarget}
          profileDisplay={profileDisplay}
          selectedKeys={selectedKeys}
          selectedTitle={selectedTitle}
          setAutofireClassSearch={setAutofireClassSearch}
          target={target}
          visibleAutofireClassCategories={visibleAutofireClassCategories}
          visibleCustomConfigs={visibleCustomConfigs}
          onAddKey={addKey}
          onDeleteKey={deleteKey}
          onEffectRuleChange={updateClassEffectRule}
          onKeyUpdate={updateKey}
        />
      ) : page === "combo" ? (
        <ComboEditorPage
          profileDisplay={profileDisplay}
          profiles={profiles}
          selectedConfigId={comboClassId}
          onCombosChange={updateProfileCombos}
          onSelectedConfigIdChange={(configId) => setComboClassId(configId)}
          onValidateComboDefs={validateComboDefs}
        />
      ) : page === "auto-run" ? (
        <AutoRunPage
          autoRun={autoRun}
          onAutoRunEnabledChange={onAutoRunEnabledChange}
          onAutoRunLeftVkChange={onAutoRunLeftVkChange}
          onAutoRunPulseDelayChange={onAutoRunPulseDelayChange}
          onAutoRunRightVkChange={onAutoRunRightVkChange}
        />
      ) : page === "runtime-diagnostics" ? (
        <RuntimeDiagnosticsPage onError={showMessage} />
      ) : page === "config-management" ? (
        <ConfigManagementPage
          classCategories={classCategories}
          profileDisplay={profileDisplay}
          profiles={profiles}
          onAddCustomConfig={addCustomConfig}
          onDeleteCustomConfig={deleteCustomConfig}
          onToggleClassHidden={toggleClassHidden}
        />
      ) : page === "settings" ? (
        <SettingsPage
          detectionEnabled={detectionEnabled}
          detectionIntervalMs={detectionIntervalMs}
          detectionNoMatchPolicy={noMatchPolicy}
          launchAtStartup={launchAtStartup}
          logLevel={logLevel}
          minimizeToTray={minimizeToTray}
          openFloatingControlOnStart={openFloatingControlOnStart}
          startMinimized={startMinimized}
          onDetectionEnabledChange={updateDetectionEnabled}
          onDetectionIntervalChange={updateDetectionInterval}
          onDetectionNoMatchPolicyChange={updateDetectionNoMatchPolicy}
          onLaunchAtStartupChange={updateLaunchAtStartup}
          onLogLevelChange={updateLogLevel}
          onMinimizeToTrayChange={updateMinimizeToTray}
          onOpenFloatingControlOnStartChange={updateOpenFloatingControlOnStart}
          onStartMinimizedChange={updateStartMinimized}
        />
      ) : (
        <AboutPage />
      )}
    </AppShell>
  );
}

export default App;
