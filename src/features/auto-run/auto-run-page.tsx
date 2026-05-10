import { Footprints } from "lucide-react";
import { useEffect, useState } from "react";
import { SettingsSwitch } from "../../components/app-ui";
import { browserKeyToVk } from "../../lib/browser-keys";
import { keyLabel, toU16Integer } from "../../lib/keys";
import type { AutoRunConfig } from "../../types/app-config";

const MIN_AUTO_RUN_PULSE_DELAY_MS = 10;
const MAX_AUTO_RUN_PULSE_DELAY_MS = 200;

export function AutoRunPage({
  autoRun,
  onAutoRunEnabledChange,
  onAutoRunLeftVkChange,
  onAutoRunPulseDelayChange,
  onAutoRunRightVkChange,
}: {
  autoRun: AutoRunConfig;
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
          <p>
            按住左右移动键时自动双击进行奔跑。会影响含有单数左右移动键的手搓，如→+空格、↑→→→+空格等。
          </p>
        </div>
      </section>
      <section className="max-w-[500px]">
        <div className="mt-6 overflow-hidden rounded border border-slate-200 bg-white shadow-sm">
          <SettingsSwitch
            checked={autoRun.enabled}
            description=""
            label="启用一键奔跑"
            onChange={onAutoRunEnabledChange}
          />

          <div className="flex min-h-[74px] items-center gap-4 border-b border-slate-100 px-5 py-4 last:border-b-0 transition hover:bg-slate-50">
            <span className="min-w-0 flex-1">
              <span className="block text-sm font-semibold text-slate-800">左移动键</span>
              <span className="mt-1 block text-xs leading-5 text-slate-500 hidden">
                点击后录入一个按键。
              </span>
            </span>
            <button
              className="h-9 min-w-[132px] rounded border border-slate-300 bg-slate-50 px-3 text-left text-sm font-medium text-slate-800 transition hover:border-blue-400 hover:bg-blue-50"
              type="button"
              onClick={() => setRecordingTarget("left")}
            >
              {recordingTarget === "left" ? "请按下按键..." : keyLabel(autoRun.leftVk)}
            </button>
          </div>

          <div className="flex min-h-[74px] items-center gap-4 border-b border-slate-100 px-5 py-4 last:border-b-0 transition hover:bg-slate-50">
            <span className="min-w-0 flex-1">
              <span className="block text-sm font-semibold text-slate-800">右移动键</span>
              <span className="mt-1 block text-xs leading-5 text-slate-500 hidden">
                点击后录入一个按键。
              </span>
            </span>
            <button
              className="h-9 min-w-[132px] rounded border border-slate-300 bg-slate-50 px-3 text-left text-sm font-medium text-slate-800 transition hover:border-blue-400 hover:bg-blue-50"
              type="button"
              onClick={() => setRecordingTarget("right")}
            >
              {recordingTarget === "right" ? "请按下按键..." : keyLabel(autoRun.rightVk)}
            </button>
          </div>

          <div className="flex min-h-[74px] items-center gap-4 px-5 py-4 transition hover:bg-slate-50">
            <span className="min-w-0 flex-1">
              <span className="block text-sm font-semibold text-slate-800">双击间隔</span>
              <span className="mt-1 block text-xs leading-5 text-slate-500">
                允许范围 {MIN_AUTO_RUN_PULSE_DELAY_MS}-{MAX_AUTO_RUN_PULSE_DELAY_MS}ms
              </span>
            </span>
            <AutoRunPulseDelayInput
              value={autoRun.pulseDelayMs}
              onChange={onAutoRunPulseDelayChange}
            />
          </div>
        </div>
      </section>
    </main>
  );
}

function AutoRunPulseDelayInput({
  value,
  onChange,
}: {
  value: number;
  onChange: (value: number) => void;
}) {
  const [draftState, setDraftState] = useState({ sourceValue: value, text: String(value) });
  const inputValue = draftState.sourceValue === value ? draftState.text : String(value);

  return (
    <div className="w-[132px]">
      <label className="grid h-9 grid-cols-[1fr_auto] items-center gap-2 rounded border border-slate-300 bg-slate-50 px-3 focus-within:border-blue-400 focus-within:bg-white focus-within:ring-1 focus-within:ring-blue-100">
        <input
          className="h-8 min-w-0 border-0 bg-transparent text-right text-sm font-semibold text-slate-900 outline-none"
          max={MAX_AUTO_RUN_PULSE_DELAY_MS}
          min={MIN_AUTO_RUN_PULSE_DELAY_MS}
          type="number"
          value={inputValue}
          onBlur={(event) => {
            const nextValue = Number(event.currentTarget.value);
            const normalizedValue = toU16Integer(nextValue, value, MIN_AUTO_RUN_PULSE_DELAY_MS);
            setDraftState({ sourceValue: normalizedValue, text: String(normalizedValue) });
            onChange(normalizedValue);
          }}
          onChange={(event) =>
            setDraftState({ sourceValue: value, text: event.currentTarget.value })
          }
          onKeyDown={(event) => {
            if (event.key === "Enter") {
              event.currentTarget.blur();
            }
          }}
        />
        <span className="text-xs font-semibold text-slate-500">ms</span>
      </label>
    </div>
  );
}
