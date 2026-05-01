// 主界面共享 UI 组件：这里保持无业务状态，只通过 props 读写配置和用户操作。
import { ChevronDown, CircleHelp, Minus, Plus, Trash2, X } from "lucide-react";
import { ReactNode, useEffect, useRef, useState } from "react";
import { APP_DISPLAY_NAME } from "../lib/app-meta";
import { browserKeyToVk } from "../lib/browser-keys";
import { type ConfigOption } from "../lib/config";
import { keyLabel, normalizeInterval } from "../lib/keys";
import { isMockMode, KeyBinding, tauriCommands } from "../lib/tauri";

export function AppTitleBar({ minimizeToTray }: { minimizeToTray: boolean }) {
  async function minimizeWindow() {
    if (isMockMode()) return;
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    const window = getCurrentWindow();
    if (minimizeToTray) {
      await window.hide();
      return;
    }

    await window.minimize();
  }

  async function closeWindow() {
    if (isMockMode()) return;
    // 主窗口关闭前先隐藏悬浮窗，避免留下孤立的 always-on-top 子窗口。
    await tauriCommands.hideFloatingControlWindow();
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    await getCurrentWindow().close();
  }

  return (
    <header
      className="flex h-10 shrink-0 items-center justify-between border-b border-slate-200 bg-white/95 pl-4 shadow-sm"
      data-tauri-drag-region
    >
      <div className="flex items-center gap-2" data-tauri-drag-region>
        <span className="h-2.5 w-2.5 rounded-full bg-blue-600" data-tauri-drag-region />
        <span className="text-sm font-semibold text-slate-800" data-tauri-drag-region>
          {APP_DISPLAY_NAME}
        </span>
      </div>
      <div className="flex h-full items-center">
        <button
          aria-label="最小化"
          className="inline-flex h-10 w-12 items-center justify-center text-slate-500 transition hover:bg-slate-100 hover:text-slate-800"
          type="button"
          onClick={() => void minimizeWindow()}
        >
          <Minus size={16} />
        </button>
        <button
          aria-label="关闭"
          className="inline-flex h-10 w-12 items-center justify-center text-slate-500 transition hover:bg-red-500 hover:text-white"
          type="button"
          onClick={() => void closeWindow()}
        >
          <X size={16} />
        </button>
      </div>
    </header>
  );
}

export function NavButton({
  active,
  icon,
  label,
  onClick,
}: {
  active: boolean;
  icon: ReactNode;
  label: string;
  onClick: () => void;
}) {
  return (
    <button
      className={`flex h-10 w-full items-center gap-2 rounded px-3 text-sm font-medium transition ${
        active ? "bg-blue-600 text-white" : "text-slate-300 hover:bg-slate-800 hover:text-white"
      }`}
      type="button"
      onClick={onClick}
    >
      {icon}
      {label}
    </button>
  );
}

export function MessageDialog({ message, onClose }: { message: string; onClose: () => void }) {
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape" || event.key === "Enter") {
        event.preventDefault();
        onClose();
      }
    };

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [onClose]);

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/20 px-4"
      role="presentation"
      onMouseDown={onClose}
    >
      <section
        aria-modal="true"
        className="w-[360px] rounded-lg border border-slate-200 bg-white p-5 shadow-2xl"
        role="dialog"
        onMouseDown={(event) => event.stopPropagation()}
      >
        <h2 className="text-base font-semibold text-slate-900">提示</h2>
        <p className="mt-3 text-sm leading-6 text-slate-600">{message}</p>
        <div className="mt-5 flex justify-end">
          <button
            className="inline-flex h-9 items-center rounded bg-blue-600 px-4 text-sm font-semibold text-white shadow-sm transition hover:bg-blue-700"
            type="button"
            onClick={onClose}
          >
            确定
          </button>
        </div>
      </section>
    </div>
  );
}

export function ConfirmDialog({
  cancelText = "取消",
  confirmText = "确定",
  description,
  title,
  onCancel,
  onConfirm,
}: {
  cancelText?: string;
  confirmText?: string;
  description: string;
  title: string;
  onCancel: () => void;
  onConfirm: () => void;
}) {
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        onCancel();
      }
      if (event.key === "Enter") {
        event.preventDefault();
        onConfirm();
      }
    };

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [onCancel, onConfirm]);

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/20 px-4"
      role="presentation"
      onMouseDown={onCancel}
    >
      <section
        aria-modal="true"
        className="w-[380px] rounded-lg border border-slate-200 bg-white p-5 shadow-2xl"
        role="dialog"
        onMouseDown={(event) => event.stopPropagation()}
      >
        <h2 className="text-base font-semibold text-slate-900">{title}</h2>
        <p className="mt-3 text-sm leading-6 text-slate-600">{description}</p>
        <div className="mt-5 flex justify-end gap-2">
          <button
            className="inline-flex h-9 items-center rounded border border-slate-300 bg-white px-4 text-sm font-semibold text-slate-700 shadow-sm transition hover:bg-slate-50"
            type="button"
            onClick={onCancel}
          >
            {cancelText}
          </button>
          <button
            className="inline-flex h-9 items-center rounded bg-red-600 px-4 text-sm font-semibold text-white shadow-sm transition hover:bg-red-700"
            type="button"
            onClick={onConfirm}
          >
            {confirmText}
          </button>
        </div>
      </section>
    </div>
  );
}

export function ConfigSelect({
  activeClassId,
  options: configOptions,
  compact = false,
  placement = "bottom",
  native = false,
  onChange,
}: {
  activeClassId: string | null;
  options: ConfigOption[];
  compact?: boolean;
  placement?: "top" | "bottom";
  native?: boolean;
  onChange: (classId: string | null) => void;
}) {
  // 悬浮窗空间很窄时使用原生 select，主界面则使用自绘菜单以支持向上弹出。
  const [open, setOpen] = useState(false);
  const rootRef = useRef<HTMLSpanElement | null>(null);
  const options: ConfigOption[] = [{ id: null, label: "全局配置" }, ...configOptions];
  const activeLabel = options.find((option) => option.id === activeClassId)?.label ?? "全局配置";
  const buttonClass = compact
    ? "h-6 min-w-[88px] max-w-[116px] rounded px-1.5 pr-5 text-[10px] font-medium"
    : "h-7 min-w-[112px] rounded px-2 pr-1 text-sm font-medium";
  const nativeSelectClass = compact
    ? "h-8 min-w-[122px] max-w-[146px] rounded-md py-0 pl-2.5 pr-7 text-sm font-medium shadow-sm"
    : "h-7 min-w-[114px] rounded py-0 pl-2 pr-6 text-[11px] font-medium";
  const menuClass = compact ? "min-w-[104px] text-[10px]" : "min-w-[112px] text-sm";
  const menuPositionClass = placement === "top" ? "bottom-full mb-1" : "top-full mt-1";
  const hasChoices = options.length > 1;

  function setMenuOpen(nextOpen: boolean) {
    // 只有全局配置时不展开菜单，减少无意义的弹层状态。
    const actualOpen = hasChoices && nextOpen;
    setOpen(actualOpen);
  }

  useEffect(() => {
    if (!open) return;

    const closeOnOutsideClick = (event: PointerEvent) => {
      if (!rootRef.current?.contains(event.target as Node)) {
        setOpen(false);
      }
    };

    window.addEventListener("pointerdown", closeOnOutsideClick, true);
    return () => window.removeEventListener("pointerdown", closeOnOutsideClick, true);
  }, [open]);

  function selectOption(classId: string | null) {
    onChange(classId);
    setOpen(false);
  }

  if (native) {
    return (
      <span className="relative inline-flex shrink-0 items-center">
        <select
          aria-label="选择配置"
          className={`${nativeSelectClass} appearance-none border border-slate-200 bg-slate-50/80 text-slate-700 outline-none focus:border-blue-400 focus:bg-white focus:ring-1 focus:ring-blue-100`}
          value={activeClassId ?? ""}
          onChange={(event) => onChange(event.target.value || null)}
        >
          {options.map((option) => (
            <option key={option.id ?? "global"} value={option.id ?? ""}>
              {option.label}
            </option>
          ))}
        </select>
        {hasChoices && (
          <ChevronDown
            aria-hidden="true"
            className={`pointer-events-none absolute shrink-0 text-slate-400 ${
              compact ? "right-2.5" : "right-1.5"
            } ${compact ? "size-3" : "size-3"}`}
          />
        )}
      </span>
    );
  }

  return (
    <span ref={rootRef} className="relative inline-flex shrink-0 items-center">
      <button
        aria-label="选择配置"
        aria-expanded={open}
        aria-haspopup="listbox"
        className={`${buttonClass} inline-flex items-center border border-slate-200 bg-white text-left text-slate-700 outline-none transition hover:border-slate-300 focus:border-blue-400 focus:ring-1 focus:ring-blue-100`}
        type="button"
        onClick={() => setMenuOpen(!open)}
      >
        <span className="min-w-0 flex-1 truncate pr-1.5">{activeLabel}</span>
        {hasChoices && (
          <span className="flex h-full w-5 shrink-0 items-center justify-center">
            <ChevronDown aria-hidden="true" className="size-3 text-slate-400" />
          </span>
        )}
      </button>

      {open && (
        <div
          className={`${menuClass} absolute left-0 z-50 max-h-[196px] overflow-y-auto rounded border border-slate-200 bg-white py-1 text-slate-700 shadow-lg ${menuPositionClass}`}
          role="listbox"
        >
          {options.map((option) => {
            const active = option.id === activeClassId;
            return (
              <button
                key={option.id ?? "global"}
                className={`block h-7 w-full truncate px-2 text-left transition ${
                  active
                    ? "bg-blue-50 text-blue-700"
                    : "text-slate-700 hover:bg-slate-50 hover:text-slate-900"
                }`}
                role="option"
                type="button"
                aria-selected={active}
                onClick={() => selectOption(option.id)}
              >
                {option.label}
              </button>
            );
          })}
        </div>
      )}
    </span>
  );
}

export function KeyTable({
  keys,
  onAdd,
  onDelete,
  onUpdate,
}: {
  keys: KeyBinding[];
  onAdd: () => void;
  onDelete: (index: number) => void;
  onUpdate: (index: number, patch: Partial<KeyBinding>) => void;
}) {
  const [recordingIndex, setRecordingIndex] = useState<number | null>(null);
  const [draftValues, setDraftValues] = useState<Record<string, string>>({});

  useEffect(() => {
    if (recordingIndex === null) return;

    // 录制单个连发键时用捕获阶段截断事件，避免同时触发快捷键录制或页面按钮。
    const onKeyDown = (event: KeyboardEvent) => {
      event.preventDefault();
      event.stopPropagation();
      const vk = browserKeyToVk(event);
      if (!vk) return;
      onUpdate(recordingIndex, { vk });
      setRecordingIndex(null);
    };

    window.addEventListener("keydown", onKeyDown, true);
    return () => window.removeEventListener("keydown", onKeyDown, true);
  }, [onUpdate, recordingIndex]);

  return (
    <div>
      <div className="grid grid-cols-[92px_1fr_42px] gap-2 border-b border-slate-200 pb-2 text-xs font-medium text-slate-500">
        <div>按键</div>
        <div>连发间隔(毫秒)</div>
        <div>删除</div>
      </div>
      <div className="mt-2 space-y-2">
        {keys.map((key, index) => (
          <div key={`${key.vk}-${index}`} className="grid grid-cols-[92px_1fr_42px] gap-2">
            <button
              className={`h-9 rounded border px-2 text-left text-sm font-medium transition ${
                recordingIndex === index
                  ? "border-blue-400 bg-blue-50 text-blue-700"
                  : "border-slate-300 bg-white text-slate-800 hover:border-blue-300 hover:bg-blue-50"
              }`}
              type="button"
              onClick={() => setRecordingIndex(index)}
            >
              {recordingIndex === index ? "请按键..." : keyLabel(key.vk)}
            </button>
            <input
              className="h-9 rounded border border-slate-300 px-2 text-sm"
              max={1000}
              min={10}
              type="number"
              value={draftValues[`${key.vk}-${index}`] ?? String(key.intervalMs)}
              onBlur={(event) => {
                const rawValue = event.currentTarget.value;
                const nextValue = normalizeInterval(Number(rawValue));
                setDraftValues((current) => {
                  const next = { ...current };
                  delete next[`${key.vk}-${index}`];
                  return next;
                });
                onUpdate(index, { intervalMs: nextValue });
              }}
              onChange={(event) => {
                const rawValue = event.currentTarget.value;
                setDraftValues((current) => ({
                  ...current,
                  [`${key.vk}-${index}`]: rawValue,
                }));
              }}
              onKeyDown={(event) => {
                if (event.key === "Enter") {
                  event.currentTarget.blur();
                }
              }}
            />
            <button
              className="inline-flex h-9 items-center justify-center rounded border border-slate-300 text-slate-500 transition hover:border-red-300 hover:bg-red-50 hover:text-red-600"
              type="button"
              onClick={() => onDelete(index)}
            >
              <Trash2 size={16} />
            </button>
          </div>
        ))}
      </div>
      <button
        className="mt-3 inline-flex h-9 items-center gap-1.5 rounded border border-blue-200 bg-blue-50 px-3 text-sm font-medium text-blue-700 transition hover:bg-blue-100"
        type="button"
        onClick={onAdd}
      >
        <Plus size={16} />
        添加按键
      </button>
    </div>
  );
}

export function RuleButton({
  active,
  label,
  onClick,
}: {
  active: boolean;
  label: string;
  onClick: () => void;
}) {
  return (
    <button
      className={`h-8 rounded text-xs font-medium transition ${
        active ? "bg-white text-blue-700 shadow-sm" : "text-slate-500 hover:text-slate-800"
      }`}
      type="button"
      onClick={onClick}
    >
      {label}
    </button>
  );
}

export function RuleHelpTooltip() {
  return (
    <span className="group relative inline-flex">
      <CircleHelp
        aria-label="生效规则说明"
        className="text-slate-400 transition group-hover:text-blue-600"
        size={14}
      />
      <span className="pointer-events-none absolute top-5 left-1/2 z-30 hidden w-[260px] -translate-x-1/2 rounded border border-slate-200 bg-white p-3 text-left text-xs leading-5 text-slate-600 shadow-xl group-hover:block">
        <span className="block font-semibold text-slate-800">全局配置 + 职业配置</span>
        <span className="block">
          识别到该职业时，同时使用全局键位和该职业键位；重复按键优先使用当前职业的配置。
        </span>
        <span className="mt-2 block font-semibold text-slate-800">仅职业配置</span>
        <span className="block">识别到该职业时只使用该职业的键位，不带入全局配置。</span>
      </span>
    </span>
  );
}

export function SettingsSwitch({
  checked,
  description,
  label,
  onChange,
}: {
  checked: boolean;
  description: string;
  label: string;
  onChange: (checked: boolean) => void;
}) {
  return (
    <label className="flex min-h-[74px] cursor-pointer items-center gap-4 border-b border-slate-100 px-5 py-4 last:border-b-0 transition hover:bg-slate-50">
      <span className="min-w-0 flex-1">
        <span className="block text-sm font-semibold text-slate-800">{label}</span>
        <span className="mt-1 block text-xs leading-5 text-slate-500">{description}</span>
      </span>
      <input
        checked={checked}
        className="peer sr-only"
        type="checkbox"
        onChange={(event) => onChange(event.currentTarget.checked)}
      />
      <span
        aria-hidden="true"
        className="relative h-6 w-11 shrink-0 rounded-full bg-slate-300 transition peer-checked:bg-blue-600 peer-focus-visible:outline peer-focus-visible:outline-2 peer-focus-visible:outline-offset-2 peer-focus-visible:outline-blue-500"
      >
        <span
          className={`absolute left-0.5 top-0.5 h-5 w-5 rounded-full bg-white shadow-sm transition ${
            checked ? "translate-x-5" : ""
          }`}
        />
      </span>
    </label>
  );
}

export function SettingsSelect({
  description,
  label,
  options,
  value,
  onChange,
}: {
  description: string;
  label: string;
  options: { label: string; value: string }[];
  value: string;
  onChange: (value: string) => void;
}) {
  return (
    <label className="flex min-h-[74px] items-center gap-4 border-b border-slate-100 px-5 py-4 last:border-b-0 transition hover:bg-slate-50">
      <span className="min-w-0 flex-1">
        <span className="block text-sm font-semibold text-slate-800">{label}</span>
        <span className="mt-1 block text-xs leading-5 text-slate-500">{description}</span>
      </span>
      <select
        className="h-9 min-w-[128px] rounded border border-slate-300 bg-white px-2 text-sm text-slate-800 outline-none transition focus:border-blue-400 focus:ring-1 focus:ring-blue-100"
        value={value}
        onChange={(event) => onChange(event.currentTarget.value)}
      >
        {options.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
    </label>
  );
}

export function KeySummary({ active, keys }: { active: boolean; keys: KeyBinding[] }) {
  // 配置卡片和底部状态栏都复用这个摘要，空配置要显式显示“未设置”。
  if (keys.length === 0) {
    return (
      <span className={active ? "text-xs text-blue-700" : "text-xs text-slate-400"}>未设置</span>
    );
  }
  return (
    <>
      {keys.map((key) => (
        <span
          key={key.vk}
          className={`rounded border px-2 py-1 text-xs font-medium ${
            active
              ? "border-blue-200 bg-white text-blue-800 shadow-sm"
              : "border-slate-200 bg-white text-slate-700 shadow-sm"
          }`}
        >
          {keyLabel(key.vk)} {key.intervalMs}ms
        </span>
      ))}
    </>
  );
}
