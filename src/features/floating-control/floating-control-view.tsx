// 悬浮控制视图：展示后端快照，并把轻量操作作为用户意图发回后端。
import { Play, Square } from "lucide-react";
import { useCallback, useEffect, useRef } from "react";
import { ConfigSelect, ToggleKeySummary } from "../../components/app-ui";
import { useActiveConfigActions } from "../../hooks/use-active-config-actions";
import { useAssistantRuntime } from "../../hooks/use-assistant-runtime";
import { getFloatingControlTextScale } from "../../lib/floating-control";
import { tauriCommands } from "../../lib/tauri-commands";
import { isMockMode } from "../../lib/tauri-env";
import { useAppStore } from "../../store/app-store-context";

export function FloatingControlView() {
  const showError = useCallback(
    (message: string) => void tauriCommands.showErrorMessage(message).catch(() => undefined),
    [],
  );
  const {
    effectiveProfile,
    mutateSnapshot,
    profileDisplay,
    profiles,
    runtime,
    settings,
    updateSettings,
  } = useAppStore({ onSaveError: showError });
  const panelRef = useRef<HTMLDivElement | null>(null);
  const positionSaveTimerRef = useRef<number | null>(null);

  const running = runtime.assistantRunning;
  const detectionRunning = runtime.detectionRunning;
  const activeToggleKeys = runtime.activeToggleKeys;
  const keys = effectiveProfile.keys;
  const combos = effectiveProfile.combos;
  const detectionModeLabel = settings.detection.enabled ? "A" : "M";
  const { onActiveConfigChange } = useActiveConfigActions({ mutateSnapshot });
  const toggleFloatingAutofire = useAssistantRuntime({ running, showMessage: showError });
  const toggleDetectionMode = useCallback(async () => {
    await updateSettings((currentSettings) => ({
      ...currentSettings,
      detection: {
        ...currentSettings.detection,
        enabled: !currentSettings.detection.enabled,
      },
    }));
  }, [updateSettings]);

  useEffect(() => {
    if (isMockMode()) return;

    const resizeWindow = async () => {
      await new Promise((resolve) => window.requestAnimationFrame(() => resolve(undefined)));
      const panel = panelRef.current;
      if (!panel) return;

      const rect = panel.getBoundingClientRect();
      const { LogicalSize, getCurrentWindow, currentMonitor, primaryMonitor } =
        await import("@tauri-apps/api/window");
      const monitor = (await currentMonitor()) ?? (await primaryMonitor());
      if (!monitor) return;

      const textScale = getFloatingControlTextScale(monitor.scaleFactor);
      const width = Math.ceil(rect.width * textScale);
      const height = Math.ceil(rect.height * textScale);
      await getCurrentWindow().setSize(new LogicalSize(width, height));
    };

    void resizeWindow().catch(() => undefined);
  }, [activeToggleKeys, combos, keys, running, profiles.activeClassId, profiles]);

  useEffect(() => {
    if (isMockMode()) return;

    let disposed = false;
    let unlisten: (() => void) | undefined;
    const savePosition = async () => {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      unlisten = await getCurrentWindow().onMoved(({ payload }) => {
        if (positionSaveTimerRef.current !== null) {
          window.clearTimeout(positionSaveTimerRef.current);
        }
        positionSaveTimerRef.current = window.setTimeout(() => {
          void mutateSnapshot(() =>
            tauriCommands.updateFloatingControlPosition(payload.x, payload.y),
          );
        }, 250);
      });
      if (disposed) unlisten?.();
    };

    void savePosition().catch(() => undefined);
    return () => {
      disposed = true;
      if (positionSaveTimerRef.current !== null) {
        window.clearTimeout(positionSaveTimerRef.current);
      }
      unlisten?.();
    };
  }, [mutateSnapshot]);

  return (
    <div
      ref={panelRef}
      className="inline-flex min-w-[188px] max-w-[340px] overflow-hidden rounded-none border border-slate-200 bg-white shadow-xl"
      data-tauri-drag-region
    >
      <main className="flex flex-col gap-1.5 px-2 py-1.5 text-slate-900" data-tauri-drag-region>
        <div className="flex items-center gap-2" data-tauri-drag-region>
          <div className="flex shrink-0 items-center whitespace-nowrap">
            <ConfigSelect
              key={detectionRunning ? "detection-locked" : "detection-unlocked"}
              activeClassId={profiles.activeClassId}
              disabled={detectionRunning}
              options={profileDisplay.configOptions}
              compact
              native
              onChange={onActiveConfigChange}
            />
          </div>
          <button
            aria-label={settings.detection.enabled ? "切换到手动选择" : "切换到自动识别"}
            className={`inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-md border text-xs font-semibold transition ${
              settings.detection.enabled
                ? "border-blue-200 bg-blue-50 text-blue-700 hover:bg-blue-100"
                : "border-slate-200 bg-slate-50 text-slate-700 hover:bg-slate-100"
            }`}
            title={settings.detection.enabled ? "自动识别" : "手动选择"}
            type="button"
            onClick={() => void toggleDetectionMode()}
          >
            {detectionModeLabel}
          </button>
          <button
            aria-label={running ? "停止连发" : "启动连发"}
            className={`inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-md border transition ${
              running
                ? "border-red-200 bg-red-50 text-red-600 hover:bg-red-100"
                : "border-blue-200 bg-blue-50 text-blue-600 hover:bg-blue-100"
            }`}
            type="button"
            onClick={() => void toggleFloatingAutofire()}
          >
            {running ? <Square size={14} /> : <Play size={14} />}
          </button>
        </div>
        <div className="flex min-w-0 items-center gap-1.5 overflow-hidden" data-tauri-drag-region>
          <span className="shrink-0 text-[10px] text-slate-500">切换激活</span>
          <div className="flex min-w-0 flex-nowrap gap-1 overflow-hidden">
            <ToggleKeySummary activeToggleKeys={activeToggleKeys} compact />
          </div>
        </div>
      </main>
    </div>
  );
}
