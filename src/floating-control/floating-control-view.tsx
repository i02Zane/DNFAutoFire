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
import { FLOATING_CONTROL_POSITION_KEY } from "../lib/floating-control";
import { type AppConfig, isMockMode, tauriCommands } from "../lib/tauri";

export function FloatingControlView() {
  const [config, setConfig] = useState<AppConfig>(DEFAULT_CONFIG);
  const [running, setRunning] = useState(false);
  const panelRef = useRef<HTMLElement | null>(null);

  useEffect(() => {
    // 悬浮控制可能晚于主窗口创建，先主动拉一次真实状态，再接入事件广播。
    void Promise.all([tauriCommands.loadAppConfig(), tauriCommands.isAssistantRunning()]).then(
      ([nextConfig, isRunning]) => {
        setConfig(nextConfig);
        setRunning(isRunning);
      },
    );
  }, []);

  useEffect(() => {
    if (isMockMode()) return;

    // 悬浮窗展示主窗口广播的快照，避免自己维护另一份可保存配置。
    let disposed = false;
    let unlisten: (() => void) | undefined;
    const listenFloatingControlUpdate = async () => {
      unlisten = await listenAppEvent(APP_EVENTS.floatingControlUpdate, ({ config, running }) => {
        setConfig(config);
        setRunning(running);
      });
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

  async function handleClassChange(classId: string | null) {
    setConfig((prev) => ({ ...prev, activeClassId: classId }));
    // 本地先反馈选择，真正的持久化由主窗口统一完成，避免双窗口同时写配置。
    await emitAppEvent(APP_EVENTS.floatingControlClassChanged, { activeClassId: classId });
  }

  async function toggleFloatingAutofire() {
    try {
      if (running) {
        await tauriCommands.stopAssistant();
        setRunning(false);
        return;
      }

      // 启动时直接使用当前快照，主窗口会在配置变化时继续广播新的快照。
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
      await new Promise((resolve) => window.requestAnimationFrame(resolve));
      const panel = panelRef.current;
      if (!panel) return;

      // 让窗口尺寸跟内容走：先等 React 完成布局，再测量实际尺寸。
      const rect = panel.getBoundingClientRect();
      const width = Math.ceil(Math.min(Math.max(rect.width, 188), 340));
      const height = Math.ceil(Math.min(Math.max(rect.height, 44), 96));
      const { LogicalSize, getCurrentWindow } = await import("@tauri-apps/api/window");
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
      // 记录用户拖动后的位置，下次创建窗口时由 manager 优先恢复。
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
    <main
      ref={panelRef}
      className="inline-flex min-w-[188px] max-w-[340px] items-center gap-2 rounded-lg border border-slate-200 bg-white px-2 py-1.5 text-slate-900 shadow-xl"
      data-tauri-drag-region
    >
      <div className="flex shrink-0 items-center whitespace-nowrap">
        <ConfigSelect
          activeClassId={config.activeClassId}
          options={configuredConfigOptions(config)}
          compact
          native
          onChange={(id) => void handleClassChange(id)}
        />
      </div>
      <div className="min-w-0 flex-1 self-stretch" data-tauri-drag-region />
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
    </main>
  );
}
