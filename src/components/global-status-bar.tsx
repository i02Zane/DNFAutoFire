import { Play, Square, Trash2 } from "lucide-react";

import { ConfigSelect, ToggleKeySummary } from "./app-ui";
import type { SettingsUpdater } from "../store/app-store-context";
import type {
  AppStateSnapshot,
  ComboDefinition,
  ConfigOption,
  KeyBinding,
  ProfilesConfig,
  SettingsConfig,
} from "../types/app-config";
import { hotkeyDisplay } from "../lib/keys";

type GlobalStatusBarProps = {
  activeToggleKeys: number[];
  configOptions: ConfigOption[];
  detectionRunning: boolean;
  effectiveCombos: ComboDefinition[];
  effectiveKeys: KeyBinding[];
  floatingControlVisible: boolean;
  recordingHotkey: boolean;
  running: boolean;
  onFloatingControlVisibleChange: (enabled: boolean) => void;
  setRecordingHotkey: (recording: boolean) => void;
  profiles: ProfilesConfig;
  settings: SettingsConfig;
  onActiveConfigChange: (classId: string | null) => void;
  toggleAssistant: () => Promise<void> | void;
  updateSettings: (updater: SettingsUpdater) => Promise<AppStateSnapshot | null> | void;
};

export function GlobalStatusBar({
  activeToggleKeys,
  configOptions,
  detectionRunning,
  effectiveCombos,
  effectiveKeys,
  floatingControlVisible,
  recordingHotkey,
  running,
  onFloatingControlVisibleChange,
  setRecordingHotkey,
  profiles,
  settings,
  onActiveConfigChange,
  toggleAssistant,
  updateSettings,
}: GlobalStatusBarProps) {
  return (
    <footer className="grid min-w-0 grid-cols-[minmax(0,1fr)_auto] items-center gap-5 border-t border-slate-200 bg-white px-7">
      <div className="min-w-0">
        <div className="flex items-center gap-2">
          <span className="h-2 w-2 rounded-full bg-amber-400" />
          <span className="text-sm font-medium text-slate-700">配置</span>
          <ConfigSelect
            key={detectionRunning ? "detection-locked" : "detection-unlocked"}
            activeClassId={profiles.activeClassId}
            disabled={detectionRunning}
            options={configOptions}
            placement="top"
            onChange={onActiveConfigChange}
          />
          <label className="ml-3 inline-flex cursor-pointer items-center gap-2 rounded border border-slate-200 bg-slate-50 px-2.5 py-1 text-xs font-medium text-slate-600 transition hover:border-blue-200 hover:bg-blue-50 hover:text-blue-700">
            <input
              checked={floatingControlVisible}
              className="h-3.5 w-3.5 rounded border-slate-300 text-blue-600"
              type="checkbox"
              onChange={(event) => onFloatingControlVisibleChange(event.currentTarget.checked)}
            />
            显示悬浮窗
          </label>
        </div>
        <div className="mt-1 flex min-w-0 items-center gap-1.5 overflow-hidden">
          <span className="shrink-0 text-xs text-slate-500">当前生效</span>
          <div className="flex min-w-0 flex-nowrap gap-1.5 overflow-hidden">
            <span className="shrink-0 rounded border border-blue-200 bg-blue-50 px-2 py-1 text-xs font-medium text-blue-700 shadow-sm">
              {effectiveKeys.length} 个连发
            </span>
            <span className="shrink-0 rounded border border-emerald-200 bg-emerald-50 px-2 py-1 text-xs font-medium text-emerald-700 shadow-sm">
              {effectiveCombos.length} 个连招
            </span>
          </div>
          <span className="ml-2 shrink-0 text-xs text-slate-500">切换激活</span>
          <div className="flex min-w-0 flex-nowrap gap-1.5 overflow-hidden">
            <ToggleKeySummary activeToggleKeys={activeToggleKeys} />
          </div>
        </div>
      </div>
      <div className="flex min-w-0 shrink-0 items-center gap-4">
        <div className="grid min-w-0 grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-2">
          <span className="text-sm font-medium text-slate-700">启动/停止快捷键</span>
          <button
            className="h-9 min-w-[132px] max-w-[132px] rounded border border-slate-300 bg-slate-50 px-3 text-left text-sm font-medium text-slate-800 transition hover:border-blue-400 hover:bg-blue-50"
            type="button"
            onClick={() => setRecordingHotkey(true)}
          >
            {recordingHotkey ? "请按下快捷键..." : hotkeyDisplay(settings.toggleHotkey)}
          </button>
          <button
            className="inline-flex h-9 w-9 items-center justify-center rounded border border-slate-300 text-slate-500 transition hover:border-red-300 hover:bg-red-50 hover:text-red-600"
            type="button"
            onClick={() =>
              void updateSettings((currentSettings) => ({
                ...currentSettings,
                toggleHotkey: null,
              }))
            }
          >
            <Trash2 size={16} />
          </button>
        </div>

        <div className="flex shrink-0 items-center justify-end">
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
  );
}
