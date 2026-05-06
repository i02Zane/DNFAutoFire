import { Footprints } from "lucide-react";
import { useEffect, useState } from "react";
import { RuleButton, SettingsSwitch } from "../components/app-ui";
import { AUTO_RUN_PULSE_DELAY_OPTIONS, normalizeAutoRunPulseDelayMs } from "../lib/config";
import { browserKeyToVk } from "../lib/browser-keys";
import { keyLabel } from "../lib/keys";

export function AutoRunPage({
  autoRunEnabled,
  autoRunLeftVk,
  autoRunPulseDelayMs,
  autoRunRightVk,
  onAutoRunEnabledChange,
  onAutoRunLeftVkChange,
  onAutoRunPulseDelayChange,
  onAutoRunRightVkChange,
}: {
  autoRunEnabled: boolean;
  autoRunLeftVk: number;
  autoRunPulseDelayMs: number;
  autoRunRightVk: number;
  onAutoRunEnabledChange: (checked: boolean) => void;
  onAutoRunLeftVkChange: (vk: number) => void;
  onAutoRunPulseDelayChange: (pulseDelayMs: number) => void;
  onAutoRunRightVkChange: (vk: number) => void;
}) {
  const [recordingTarget, setRecordingTarget] = useState<"left" | "right" | null>(null);

  useEffect(() => {
    if (!recordingTarget) return;

    const onKeyDown = (event: KeyboardEvent) => {
      const vk = browserKeyToVk(event);
      if (vk === null) return;

      event.preventDefault();
      event.stopPropagation();

      const normalizedVk = vk;
      if (recordingTarget === "left") {
        onAutoRunLeftVkChange(normalizedVk);
      } else {
        onAutoRunRightVkChange(normalizedVk);
      }
      setRecordingTarget(null);
    };

    window.addEventListener("keydown", onKeyDown, true);
    return () => window.removeEventListener("keydown", onKeyDown, true);
  }, [onAutoRunLeftVkChange, onAutoRunRightVkChange, recordingTarget]);

  return (
    <main className="h-full min-w-0 overflow-y-auto px-7 py-6">
      <section className="max-w-[760px]">
        <div className="flex items-center gap-2">
          <Footprints size={20} className="text-slate-700" />
          <h1 className="text-[22px] font-semibold tracking-tight">一键奔跑</h1>
        </div>
        <div className="mt-1 space-y-1 text-sm leading-6 text-slate-500">
          <p>按住左右移动键时自动补发奔跑脉冲。</p>
        </div>

        <div className="mt-6 overflow-hidden rounded border border-slate-200 bg-white shadow-sm">
          <SettingsSwitch
            checked={autoRunEnabled}
            description="仅在助手运行时启用这一键奔跑。"
            label="启用一键奔跑"
            onChange={onAutoRunEnabledChange}
          />

          <div className="flex min-h-[74px] items-center gap-4 border-b border-slate-100 px-5 py-4 last:border-b-0 transition hover:bg-slate-50">
            <span className="min-w-0 flex-1">
              <span className="block text-sm font-semibold text-slate-800">向左移动键</span>
              <span className="mt-1 block text-xs leading-5 text-slate-500">
                点击后录入一个按键。
              </span>
            </span>
            <button
              className="h-9 min-w-[132px] rounded border border-slate-300 bg-slate-50 px-3 text-left text-sm font-medium text-slate-800 transition hover:border-blue-400 hover:bg-blue-50"
              type="button"
              onClick={() => setRecordingTarget("left")}
            >
              {recordingTarget === "left" ? "请按下按键..." : keyLabel(autoRunLeftVk)}
            </button>
          </div>

          <div className="flex min-h-[74px] items-center gap-4 border-b border-slate-100 px-5 py-4 last:border-b-0 transition hover:bg-slate-50">
            <span className="min-w-0 flex-1">
              <span className="block text-sm font-semibold text-slate-800">向右移动键</span>
              <span className="mt-1 block text-xs leading-5 text-slate-500">
                点击后录入一个按键。
              </span>
            </span>
            <button
              className="h-9 min-w-[132px] rounded border border-slate-300 bg-slate-50 px-3 text-left text-sm font-medium text-slate-800 transition hover:border-blue-400 hover:bg-blue-50"
              type="button"
              onClick={() => setRecordingTarget("right")}
            >
              {recordingTarget === "right" ? "请按下按键..." : keyLabel(autoRunRightVk)}
            </button>
          </div>

          <div className="flex min-h-[74px] items-center gap-4 px-5 py-4 transition hover:bg-slate-50">
            <span className="min-w-0 flex-1">
              <span className="block text-sm font-semibold text-slate-800">脉冲间隔</span>
              <span className="mt-1 block text-xs leading-5 text-slate-500">
                决定补发脉冲的等待档位。
              </span>
            </span>
            <div className="grid min-w-[192px] grid-cols-3 rounded border border-slate-200 bg-slate-50 p-1">
              {AUTO_RUN_PULSE_DELAY_OPTIONS.map((option) => (
                <RuleButton
                  key={option.value}
                  active={normalizeAutoRunPulseDelayMs(autoRunPulseDelayMs) === option.value}
                  label={option.label}
                  onClick={() => onAutoRunPulseDelayChange(option.value)}
                />
              ))}
            </div>
          </div>
        </div>
      </section>
    </main>
  );
}
