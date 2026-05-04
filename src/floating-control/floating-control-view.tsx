// 悬浮控制视图：只展示主窗口广播的配置快照，并把轻量操作回传给主窗口。
import { Play, Square } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { ConfigSelect } from "../components/app-ui";
import { APP_EVENTS, emitAppEvent, listenAppEvent } from "../lib/app-events";
import {
  computeEffectiveCombos,
  computeEffectiveKeys,
  configuredConfigOptions,
  DEFAULT_CONFIG,
} from "../lib/config";
import {
  FLOATING_CONTROL_POSITION_KEY,
  getFloatingControlTextScale,
} from "../lib/floating-control";
import { type AppConfig, isMockMode, tauriCommands } from "../lib/tauri";

export function FloatingControlView() {
  const [config, setConfig] = useState<AppConfig>(DEFAULT_CONFIG);
  const [running, setRunning] = useState(false);
  const [detectionRunning, setDetectionRunning] = useState(false);
  const panelRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    // 悬浮控制可能晚于主窗口创建，先主动拉一次真实状态，再接入事件广播。
    void Promise.all([
      tauriCommands.loadAppConfig(),
      tauriCommands.isAssistantRunning(),
      tauriCommands.isDetectionRunning(),
    ]).then(([nextConfig, isRunning, isDetectionRunning]) => {
      setConfig(nextConfig);
      setRunning(isRunning);
      setDetectionRunning(isDetectionRunning);
    });
  }, []);

  useEffect(() => {
    if (isMockMode()) return;

    // 悬浮窗展示主窗口广播的快照，避免自己维护另一份可保存配置。
    let disposed = false;
    let unlisten: (() => void) | undefined;
    const listenFloatingControlUpdate = async () => {
      unlisten = await listenAppEvent(
        APP_EVENTS.floatingControlUpdate,
        ({ config, detectionRunning, running }) => {
          setConfig(config);
          setDetectionRunning(detectionRunning);
          setRunning(running);
        },
      );
      if (disposed) unlisten();
    };

    void listenFloatingControlUpdate().catch(() => undefined);
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);

  const keys = computeEffectiveKeys(config);
  const combos = computeEffectiveCombos(config);
  const detectionModeLabel = config.detection.enabled ? "A" : "M";

  async function handleClassChange(classId: string | null) {
    setConfig((prev) => ({ ...prev, activeClassId: classId }));
    await emitAppEvent(APP_EVENTS.floatingControlClassChanged, { activeClassId: classId });
  }

  async function toggleDetectionMode() {
    const nextEnabled = !config.detection.enabled;
    setConfig((prev) => ({
      ...prev,
      detection: {
        ...prev.detection,
        enabled: nextEnabled,
      },
    }));
    await emitAppEvent(APP_EVENTS.floatingControlDetectionModeToggleRequest, undefined);
  }

  async function toggleFloatingAutofire() {
    try {
      if (running) {
        await tauriCommands.stopAssistant();
        setRunning(false);
        return;
      }

      await tauriCommands.startAssistant(keys, combos);
      setRunning(true);
    } catch (reason) {
      const message = reason instanceof Error ? reason.message : String(reason);
      setRunning(false);
      await tauriCommands.showErrorMessage(message).catch(() => undefined);
    }
  }

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
  }, [combos, keys, running, config.activeClassId, config.classes]);

  useEffect(() => {
    if (isMockMode()) return;

    let disposed = false;
    let unlisten: (() => void) | undefined;
    const savePosition = async () => {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      unlisten = await getCurrentWindow().onMoved(({ payload }) => {
        window.localStorage.setItem(
          FLOATING_CONTROL_POSITION_KEY,
          JSON.stringify({ x: payload.x, y: payload.y }),
        );
      });
      if (disposed) unlisten();
    };

    void savePosition().catch(() => undefined);
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);

  return (
    <div
      ref={panelRef}
      className="inline-flex min-w-[188px] max-w-[340px] overflow-hidden rounded-none border border-slate-200 bg-white shadow-xl"
      data-tauri-drag-region
    >
      <main className="flex items-center gap-2 px-2 py-1.5 text-slate-900" data-tauri-drag-region>
        <div className="flex shrink-0 items-center whitespace-nowrap">
          <ConfigSelect
            key={detectionRunning ? "detection-locked" : "detection-unlocked"}
            activeClassId={config.activeClassId}
            disabled={detectionRunning}
            options={configuredConfigOptions(config)}
            compact
            native
            onChange={(id) => void handleClassChange(id)}
          />
        </div>
        <button
          aria-label={config.detection.enabled ? "切换到手动选择" : "切换到自动识别"}
          className={`inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-md border text-xs font-semibold transition ${config.detection.enabled
            ? "border-blue-200 bg-blue-50 text-blue-700 hover:bg-blue-100"
            : "border-slate-200 bg-slate-50 text-slate-700 hover:bg-slate-100"
            }`}
          title={config.detection.enabled ? "自动识别" : "手动选择"}
          type="button"
          onClick={() => void toggleDetectionMode()}
        >
          {detectionModeLabel}
        </button>
        <button
          aria-label={running ? "停止连发" : "启动连发"}
          className={`inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-md border transition ${running
            ? "border-red-200 bg-red-50 text-red-600 hover:bg-red-100"
            : "border-blue-200 bg-blue-50 text-blue-600 hover:bg-blue-100"
            }`}
          type="button"
          onClick={() => void toggleFloatingAutofire()}
        >
          {running ? <Square size={14} /> : <Play size={14} />}
        </button>
      </main>
    </div>
  );
}
