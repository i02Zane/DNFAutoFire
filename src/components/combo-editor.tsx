// 一键连招编辑器：按配置维护触发键和动作块，运行时只下发当前生效配置的启用连招。
import {
  ChevronDown,
  ChevronRight,
  CircleHelp,
  Keyboard,
  Plus,
  Search,
  Trash2,
  Wand2,
  X,
} from "lucide-react";
import { useCallback, useEffect, useMemo, useState } from "react";
import type { SVGProps } from "react";
import { classCategories } from "../data/classes";
import { browserKeyToVk } from "../lib/browser-keys";
import {
  computeEffectiveKeysForProfile,
  countComboCommandDirections,
  getConfigDisplayName,
  getProfileConfig,
  hasClassComboConfig,
  isClassVisible,
  isComboCommandDirectionVk,
  isComboCommandFinishVk,
  isComboCommandVk,
  MAX_COMBO_COMMAND_DIRECTION_KEYS,
  normalizeComboGapMs,
  normalizeComboHoldMs,
  normalizeComboWaitMs,
  validateClassComboDefs,
} from "../lib/config";
import { keyLabel } from "../lib/keys";
import {
  AppConfig,
  ComboAction,
  ComboCommandAction,
  ComboDefinition,
  ComboTapAction,
  ComboValidationIssue,
} from "../lib/tauri";

type RecordingTarget =
  | { type: "trigger"; comboId: string }
  | { type: "tap"; comboId: string; actionId: string }
  | { type: "commandSequence"; comboId: string; actionId: string };

const EMPTY_COMBOS: ComboDefinition[] = [];

export function ComboEditorPage({
  config,
  selectedConfigId,
  onCombosChange,
  onSelectedConfigIdChange,
}: {
  config: AppConfig;
  selectedConfigId: string | null;
  onCombosChange: (configId: string, combos: ComboDefinition[]) => void;
  onSelectedConfigIdChange: (configId: string | null) => void;
}) {
  const [recordingTarget, setRecordingTarget] = useState<RecordingTarget | null>(null);
  const [classSearch, setClassSearch] = useState("");
  const [collapsedComboIds, setCollapsedComboIds] = useState<Set<string>>(() => new Set());
  const [comboDrafts, setComboDrafts] = useState<Record<string, ComboDefinition[]>>({});
  const activeConfigId = selectedConfigId;
  const activeConfig = activeConfigId ? getProfileConfig(config, activeConfigId) : null;
  const savedCombos = activeConfig?.comboDefs ?? EMPTY_COMBOS;
  // 编辑时先落在本地草稿里，只有通过校验的连招才会回写主配置并保存。
  const combos = activeConfigId ? (comboDrafts[activeConfigId] ?? savedCombos) : EMPTY_COMBOS;
  const effectiveKeys = useMemo(
    () => (activeConfigId ? computeEffectiveKeysForProfile(config, activeConfigId) : []),
    [activeConfigId, config],
  );
  const validationIssues = useMemo(
    () => validateClassComboDefs(combos, effectiveKeys),
    [combos, effectiveKeys],
  );
  const normalizedClassSearch = classSearch.trim().toLowerCase();
  const visibleClassCategories = useMemo(
    () =>
      classCategories
        .map((category) => ({
          ...category,
          classes: category.classes.filter(
            (classInfo) =>
              isClassVisible(config, classInfo.id) &&
              (!normalizedClassSearch ||
                classInfo.name.toLowerCase().includes(normalizedClassSearch) ||
                classInfo.id.toLowerCase().includes(normalizedClassSearch)),
          ),
        }))
        .filter((category) => category.classes.length > 0),
    [config, normalizedClassSearch],
  );
  const visibleCustomConfigs = useMemo(
    () =>
      Object.entries(config.customConfigs).filter(([, customConfig]) => {
        const name = customConfig.name.trim() || "未命名配置";
        return !normalizedClassSearch || name.toLowerCase().includes(normalizedClassSearch);
      }),
    [config.customConfigs, normalizedClassSearch],
  );

  const updateCombos = useCallback(
    (nextCombos: ComboDefinition[]) => {
      if (!activeConfigId) return;
      setComboDrafts((current) => ({ ...current, [activeConfigId]: nextCombos }));
      // 无效草稿继续留在页面上供用户修正，避免把不可运行的连招写入配置。
      if (validateClassComboDefs(nextCombos, effectiveKeys).length === 0) {
        onCombosChange(activeConfigId, nextCombos);
      }
    },
    [activeConfigId, effectiveKeys, onCombosChange],
  );

  useEffect(() => {
    if (!recordingTarget || !activeConfigId) return;

    // 录制模式在 capture 阶段接管键盘输入，避免输入框或按钮先消费按键。
    const onKeyDown = (event: KeyboardEvent) => {
      event.preventDefault();
      event.stopPropagation();
      if (event.repeat) return;

      const vk = browserKeyToVk(event);
      if (!vk) return;
      if (recordingTarget.type === "commandSequence" && !isComboCommandVk(vk)) return;

      // 手搓动作只允许有限数量的方向键，结束键由 isComboCommandFinishVk 决定。
      if (recordingTarget.type === "commandSequence" && isComboCommandDirectionVk(vk)) {
        const combo = combos.find((item) => item.id === recordingTarget.comboId);
        const action = combo?.actions.find((item) => item.id === recordingTarget.actionId);
        if (
          action?.type === "command" &&
          countComboCommandDirections(action.keys) >= MAX_COMBO_COMMAND_DIRECTION_KEYS
        ) {
          return;
        }
      }

      updateCombos(
        combos.map((combo) => {
          if (combo.id !== recordingTarget.comboId) return combo;
          if (recordingTarget.type === "trigger") {
            return { ...combo, triggerVk: vk };
          }
          return {
            ...combo,
            actions: combo.actions.map((action) =>
              updateRecordedAction(action, recordingTarget, vk),
            ),
          };
        }),
      );
      if (
        recordingTarget.type !== "commandSequence" ||
        (recordingTarget.type === "commandSequence" && isComboCommandFinishVk(vk))
      ) {
        setRecordingTarget(null);
      }
    };

    window.addEventListener("keydown", onKeyDown, true);
    return () => window.removeEventListener("keydown", onKeyDown, true);
  }, [activeConfigId, combos, recordingTarget, updateCombos]);

  function addCombo() {
    updateCombos([...combos, createCombo()]);
  }

  function updateCombo(comboId: string, patch: Partial<ComboDefinition>) {
    updateCombos(combos.map((combo) => (combo.id === comboId ? { ...combo, ...patch } : combo)));
  }

  function deleteCombo(comboId: string) {
    updateCombos(combos.filter((combo) => combo.id !== comboId));
    setCollapsedComboIds((current) => {
      const next = new Set(current);
      next.delete(comboId);
      return next;
    });
  }

  function toggleComboCollapsed(comboId: string) {
    setCollapsedComboIds((current) => {
      const next = new Set(current);
      if (next.has(comboId)) {
        next.delete(comboId);
      } else {
        next.add(comboId);
      }
      return next;
    });
  }

  function addAction(comboId: string, type: ComboAction["type"]) {
    const action = type === "tap" ? createTapAction() : createCommandAction();
    updateCombos(
      combos.map((combo) =>
        combo.id === comboId ? { ...combo, actions: [...combo.actions, action] } : combo,
      ),
    );
  }

  function updateAction(comboId: string, actionId: string, patch: Partial<ComboAction>) {
    updateCombos(
      combos.map((combo) =>
        combo.id === comboId
          ? {
              ...combo,
              actions: combo.actions.map((action) =>
                action.id === actionId ? patchAction(action, patch) : action,
              ),
            }
          : combo,
      ),
    );
  }

  function deleteAction(comboId: string, actionId: string) {
    updateCombos(
      combos.map((combo) =>
        combo.id === comboId
          ? { ...combo, actions: combo.actions.filter((action) => action.id !== actionId) }
          : combo,
      ),
    );
  }

  return (
    <main className="relative h-full min-w-0 overflow-hidden">
      <section className="h-full min-w-0 overflow-y-auto px-7 py-6">
        <header className="mb-5 flex items-start justify-between gap-5">
          <div>
            <div className="flex items-center gap-2">
              <h1 className="text-[22px] font-semibold tracking-tight">一键连招</h1>
              <span className="rounded border border-amber-200 bg-amber-50 px-2 py-0.5 text-xs font-semibold text-amber-700">
                Beta
              </span>
            </div>
            <div className="mt-1 space-y-1 text-sm leading-6 text-slate-500">
              <p>选择职业或自定义配置后编辑触发键和动作块，功能仍在测试中。</p>
              <p>实现方式与按键连发略有不同，会影响输入功能：</p>
              <p>
                1.
                触发键会先在底层被拦截，所以建议触发键也是连招的第一个技能的快捷键，不然会影响打字
              </p>
              <p>2. 键盘的独立方向键的模拟方式与字母区不同，会影响打字功能，建议不用手搓连招</p>
            </div>
          </div>
          <label className="flex h-9 w-[260px] shrink-0 items-center gap-2 rounded border border-slate-200 bg-white px-2.5 text-slate-500 shadow-sm focus-within:border-blue-400 focus-within:ring-1 focus-within:ring-blue-100">
            <Search size={15} />
            <input
              className="min-w-0 flex-1 bg-transparent text-sm text-slate-800 outline-none placeholder:text-slate-400"
              placeholder="搜索职业/自定义配置"
              value={classSearch}
              onChange={(event) => setClassSearch(event.currentTarget.value)}
            />
          </label>
        </header>

        <div className="space-y-4 pb-6">
          {visibleCustomConfigs.length > 0 && (
            <div>
              <h2 className="mb-3 text-sm font-semibold text-slate-700">自定义配置</h2>
              <div className="flex flex-wrap gap-2">
                {visibleCustomConfigs.map(([configId, customConfig]) => {
                  const active = activeConfigId === configId;
                  const configured = hasClassComboConfig(customConfig);
                  return (
                    <button
                      key={configId}
                      className={classButtonClass(active)}
                      type="button"
                      onClick={() => onSelectedConfigIdChange(configId)}
                    >
                      <span>{customConfig.name || "未命名配置"}</span>
                      {configured && <span className={configuredDotClass(active)} />}
                    </button>
                  );
                })}
              </div>
            </div>
          )}

          <h2 className="text-sm font-semibold text-slate-700">职业配置</h2>
          {visibleClassCategories.map((category) => (
            <div key={category.name} className="grid grid-cols-[84px_1fr] items-center gap-3">
              <div className="text-sm font-medium text-slate-600">{category.name}</div>
              <div className="flex flex-wrap gap-2">
                {category.classes.map((classInfo) => {
                  const active = activeConfigId === classInfo.id;
                  const configured = hasClassComboConfig(config.classes[classInfo.id]);
                  return (
                    <button
                      key={classInfo.id}
                      className={classButtonClass(active)}
                      type="button"
                      onClick={() => onSelectedConfigIdChange(classInfo.id)}
                    >
                      <span>{classInfo.name}</span>
                      {configured && <span className={configuredDotClass(active)} />}
                    </button>
                  );
                })}
              </div>
            </div>
          ))}
          {visibleClassCategories.length === 0 && visibleCustomConfigs.length === 0 && (
            <div className="rounded border border-dashed border-slate-200 bg-white px-3 py-8 text-center text-sm text-slate-500">
              没有匹配的配置。
            </div>
          )}
        </div>
      </section>

      {activeConfigId && (
        <div
          className="absolute inset-0 z-20 flex justify-end bg-slate-950/10"
          onClick={() => onSelectedConfigIdChange(null)}
        >
          <aside
            className="grid h-full w-[580px] max-w-[calc(100vw-168px)] grid-rows-[auto_1fr] border-l border-slate-200 bg-slate-50 shadow-2xl"
            onClick={(event) => event.stopPropagation()}
          >
            <header className="border-b border-slate-200 bg-white px-6 py-5">
              <div className="flex items-start justify-between gap-4">
                <div className="min-w-0">
                  <div className="text-xs font-medium text-slate-500 hidden">当前职业</div>
                  <h2 className="mt-1 truncate text-xl font-semibold text-slate-900">
                    {getConfigDisplayName(config, activeConfigId)}
                  </h2>
                </div>
                <div className="flex items-center gap-2">
                  <button
                    className="inline-flex h-9 items-center gap-1.5 rounded border border-blue-200 bg-blue-50 px-3 text-sm font-medium text-blue-700 transition hover:bg-blue-100"
                    type="button"
                    onClick={addCombo}
                  >
                    <Plus size={16} />
                    添加连招
                  </button>
                  <button
                    aria-label="关闭连招编辑"
                    className="inline-flex h-9 w-9 items-center justify-center rounded border border-slate-300 text-slate-500 transition hover:border-slate-400 hover:bg-slate-50 hover:text-slate-800"
                    type="button"
                    onClick={() => onSelectedConfigIdChange(null)}
                  >
                    <X size={16} />
                  </button>
                </div>
              </div>
              <p className="mt-2 text-xs text-slate-500">配置会自动保存，时间单位均为毫秒。</p>
            </header>

            <div className="min-h-0 overflow-y-auto px-6 py-5">
              {combos.length === 0 ? (
                <div className="flex h-[280px] items-center justify-center rounded border border-dashed border-slate-300 bg-white text-sm text-slate-500">
                  还没有连招配置。
                </div>
              ) : (
                <div className="space-y-3 pb-6">
                  {combos.map((combo) => (
                    <ComboBlock
                      key={combo.id}
                      combo={combo}
                      issues={validationIssues.filter((issue) => issue.comboId === combo.id)}
                      collapsed={collapsedComboIds.has(combo.id)}
                      recordingTarget={recordingTarget}
                      onAddAction={(type) => addAction(combo.id, type)}
                      onCollapsedChange={() => toggleComboCollapsed(combo.id)}
                      onDelete={() => deleteCombo(combo.id)}
                      onDeleteAction={(actionId) => deleteAction(combo.id, actionId)}
                      onRecord={(target) => setRecordingTarget(target)}
                      onUpdate={(patch) => updateCombo(combo.id, patch)}
                      onUpdateAction={(actionId, patch) => updateAction(combo.id, actionId, patch)}
                    />
                  ))}
                </div>
              )}
            </div>
          </aside>
        </div>
      )}
    </main>
  );
}

function ComboBlock({
  combo,
  issues,
  collapsed,
  recordingTarget,
  onAddAction,
  onCollapsedChange,
  onDelete,
  onDeleteAction,
  onRecord,
  onUpdate,
  onUpdateAction,
}: {
  combo: ComboDefinition;
  issues: ComboValidationIssue[];
  collapsed: boolean;
  recordingTarget: RecordingTarget | null;
  onAddAction: (type: ComboAction["type"]) => void;
  onCollapsedChange: () => void;
  onDelete: () => void;
  onDeleteAction: (actionId: string) => void;
  onRecord: (target: RecordingTarget | null) => void;
  onUpdate: (patch: Partial<ComboDefinition>) => void;
  onUpdateAction: (actionId: string, patch: Partial<ComboAction>) => void;
}) {
  const recordingTrigger =
    recordingTarget?.type === "trigger" && recordingTarget.comboId === combo.id;
  const nameIssue = getComboIssue(issues, "name");
  const triggerIssue = getComboIssue(issues, "trigger");
  const actionsIssue = getComboIssue(issues, "actions");

  return (
    <article className="overflow-hidden rounded border border-slate-200 bg-white shadow-sm">
      <header
        className={`grid grid-cols-[36px_minmax(132px,1fr)_auto_auto_40px] items-start gap-3 bg-white px-4 py-3 ${
          collapsed ? "" : "border-b border-slate-100"
        }`}
      >
        <button
          aria-label={collapsed ? "展开连招" : "折叠连招"}
          className="inline-flex h-9 w-9 items-center justify-center rounded border border-slate-200 text-slate-500 transition hover:border-blue-200 hover:bg-blue-50 hover:text-blue-700"
          type="button"
          onClick={onCollapsedChange}
        >
          {collapsed ? <ChevronRight size={16} /> : <ChevronDown size={16} />}
        </button>
        <div className="min-w-0">
          <input
            className={`h-9 w-full rounded border bg-white px-3 text-sm font-semibold text-slate-900 shadow-sm ${
              nameIssue ? "border-red-300 bg-red-50/40" : "border-slate-300"
            }`}
            placeholder="连招名称"
            value={combo.name}
            onChange={(event) => onUpdate({ name: event.currentTarget.value })}
          />
          <FieldError message={nameIssue?.message} />
        </div>
        <div className="space-y-1">
          <div className="inline-flex h-9 items-center gap-2">
            <span className="whitespace-nowrap text-sm font-semibold text-slate-500">触发键</span>
            <button
              className={`h-9 w-[84px] rounded border px-3 text-left text-sm font-semibold shadow-sm transition ${
                recordingTrigger
                  ? "border-blue-400 bg-blue-50 text-blue-700"
                  : triggerIssue
                    ? "border-red-300 bg-red-50/40 text-red-700 hover:border-red-400"
                    : "border-slate-300 bg-white text-slate-800 hover:border-blue-300 hover:bg-blue-50"
              }`}
              type="button"
              onClick={() => onRecord({ type: "trigger", comboId: combo.id })}
            >
              {recordingTrigger
                ? "请按键..."
                : combo.triggerVk === null
                  ? "未设置"
                  : keyLabel(combo.triggerVk)}
            </button>
            <TriggerKeyHelpTooltip />
          </div>
          <FieldError message={triggerIssue?.message} />
        </div>
        <label className="inline-flex h-9 cursor-pointer items-center gap-2 whitespace-nowrap rounded border border-slate-200 bg-slate-50 px-3 text-sm font-semibold text-slate-600 transition hover:bg-slate-100">
          <input
            checked={combo.enabled}
            className="h-3.5 w-3.5 rounded border-slate-300 text-blue-600"
            type="checkbox"
            onChange={(event) => onUpdate({ enabled: event.currentTarget.checked })}
          />
          启用
        </label>
        <button
          className="inline-flex h-9 w-9 items-center justify-center rounded border border-slate-200 bg-white text-slate-500 transition hover:border-red-300 hover:bg-red-50 hover:text-red-600"
          type="button"
          onClick={onDelete}
        >
          <Trash2 size={16} />
        </button>
      </header>

      {collapsed ? null : (
        <div className="space-y-3 bg-white px-5 py-4">
          <FieldError message={actionsIssue?.message} />
          <div className="space-y-1.5">
            {combo.actions.map((action, index) => (
              <div key={action.id} className="space-y-2">
                <ActionBlock
                  action={action}
                  comboId={combo.id}
                  index={index}
                  issues={issues.filter((issue) => issue.actionId === action.id)}
                  recordingTarget={recordingTarget}
                  onDelete={() => onDeleteAction(action.id)}
                  onRecord={onRecord}
                  onUpdate={(patch) => onUpdateAction(action.id, patch)}
                />
                {index < combo.actions.length - 1 && (
                  <WaitBetweenActionsField
                    issue={getActionIssue(issues, action.id, "waitAfterMs")}
                    value={action.waitAfterMs}
                    onChange={(value) =>
                      onUpdateAction(action.id, { waitAfterMs: normalizeComboWaitMs(value) })
                    }
                  />
                )}
              </div>
            ))}
          </div>

          <div className="flex gap-2 pt-1">
            <button
              className="inline-flex h-9 items-center gap-1.5 rounded border border-slate-200 bg-white px-3 text-sm font-semibold text-slate-700 shadow-sm transition hover:border-blue-200 hover:bg-blue-50 hover:text-blue-700"
              type="button"
              onClick={() => onAddAction("tap")}
            >
              <Keyboard size={16} />
              快捷栏按键
            </button>
            <button
              className="inline-flex h-9 items-center gap-1.5 rounded border border-slate-200 bg-white px-3 text-sm font-semibold text-slate-700 shadow-sm transition hover:border-blue-200 hover:bg-blue-50 hover:text-blue-700"
              type="button"
              onClick={() => onAddAction("command")}
            >
              <Wand2 size={16} />
              手搓
            </button>
          </div>
        </div>
      )}
    </article>
  );
}

function TriggerKeyHelpTooltip() {
  return (
    <span className="group relative inline-flex">
      <CircleHelp
        aria-label="触发键说明"
        className="text-slate-400 transition group-hover:text-blue-600"
        size={15}
      />
      <span className="pointer-events-none absolute top-6 left-1/2 z-30 hidden w-[220px] -translate-x-1/2 rounded border border-slate-200 bg-white p-3 text-left text-xs leading-5 text-slate-600 shadow-xl group-hover:block">
        触发键会被拦截，不会继续传给游戏。
      </span>
    </span>
  );
}

function WaitBetweenActionsField({
  issue,
  value,
  onChange,
}: {
  issue: ComboValidationIssue | undefined;
  value: number;
  onChange: (value: number) => void;
}) {
  return (
    <div className="grid grid-cols-[40px_minmax(0,1fr)] gap-3 px-3 py-1">
      <div className="flex h-12 items-center justify-center text-slate-400">
        <DownActionArrow className="h-12 w-8" />
      </div>
      <div className="w-[132px]">
        <div className="rounded bg-slate-50">
          <NumberField
            issue={issue}
            label="等待"
            max={5000}
            min={0}
            value={value}
            onChange={onChange}
          />
        </div>
      </div>
    </div>
  );
}

function DownActionArrow(props: SVGProps<SVGSVGElement>) {
  return (
    <svg viewBox="0 0 64 96" fill="none" aria-hidden="true" {...props}>
      <path
        d="M32 8V66 M32 66L20 50 M32 66L44 50"
        stroke="currentColor"
        strokeWidth={3}
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}

function ActionBlock({
  action,
  comboId,
  index,
  issues,
  recordingTarget,
  onDelete,
  onRecord,
  onUpdate,
}: {
  action: ComboAction;
  comboId: string;
  index: number;
  issues: ComboValidationIssue[];
  recordingTarget: RecordingTarget | null;
  onDelete: () => void;
  onRecord: (target: RecordingTarget | null) => void;
  onUpdate: (patch: Partial<ComboAction>) => void;
}) {
  const title = action.type === "tap" ? "快捷栏按键" : "手搓";

  return (
    <div className="grid grid-cols-[40px_minmax(0,1fr)] gap-3 rounded border border-slate-200 bg-slate-50/70 p-3">
      <div className="flex flex-col items-center">
        <div className="flex h-9 w-9 items-center justify-center rounded border border-blue-200 bg-blue-50 text-sm font-semibold text-blue-700">
          {index + 1}
        </div>
      </div>
      <div className="min-w-0">
        <div className="mb-2 flex items-center justify-between gap-3">
          <div className="text-sm font-semibold text-slate-600">{title}</div>
          <button
            className="inline-flex h-8 w-8 items-center justify-center rounded border border-slate-200 bg-white text-slate-500 transition hover:border-red-300 hover:bg-red-50 hover:text-red-600"
            type="button"
            onClick={onDelete}
          >
            <Trash2 size={15} />
          </button>
        </div>
        {action.type === "tap" ? (
          <TapActionEditor
            action={action}
            comboId={comboId}
            issues={issues}
            recordingTarget={recordingTarget}
            onRecord={onRecord}
            onUpdate={onUpdate}
          />
        ) : (
          <CommandActionEditor
            action={action}
            comboId={comboId}
            issues={issues}
            recordingTarget={recordingTarget}
            onRecord={onRecord}
            onUpdate={onUpdate}
          />
        )}
      </div>
    </div>
  );
}

function TapActionEditor({
  action,
  comboId,
  issues,
  recordingTarget,
  onRecord,
  onUpdate,
}: {
  action: ComboTapAction;
  comboId: string;
  issues: ComboValidationIssue[];
  recordingTarget: RecordingTarget | null;
  onRecord: (target: RecordingTarget | null) => void;
  onUpdate: (patch: Partial<ComboTapAction>) => void;
}) {
  const recording =
    recordingTarget?.type === "tap" &&
    recordingTarget.comboId === comboId &&
    recordingTarget.actionId === action.id;
  const keyIssue = getComboIssue(issues, "tapKey");
  const holdIssue = getComboIssue(issues, "holdMs");

  return (
    <div className="flex min-w-0 flex-wrap items-start gap-2">
      <div>
        <button
          className={`inline-flex h-9 min-w-[96px] items-center justify-center rounded border px-4 text-[15px] font-semibold transition ${
            recording
              ? "border-blue-400 bg-blue-50 text-blue-700"
              : keyIssue
                ? "border-red-300 bg-red-50/40 text-red-700 hover:border-red-400"
                : "border-slate-300 bg-white text-slate-800 hover:border-blue-300 hover:bg-blue-50"
          }`}
          type="button"
          onClick={() => onRecord({ type: "tap", comboId, actionId: action.id })}
        >
          {recording ? "请按键..." : action.vk === null ? "未设置" : keyLabel(action.vk)}
        </button>
        <FieldError message={keyIssue?.message} />
      </div>
      <div className="w-[132px]">
        <NumberField
          issue={holdIssue}
          label="按下"
          max={1000}
          min={10}
          value={action.holdMs}
          onChange={(value) => onUpdate({ holdMs: normalizeComboHoldMs(value) })}
        />
      </div>
    </div>
  );
}

function CommandActionEditor({
  action,
  comboId,
  issues,
  recordingTarget,
  onRecord,
  onUpdate,
}: {
  action: ComboCommandAction;
  comboId: string;
  issues: ComboValidationIssue[];
  recordingTarget: RecordingTarget | null;
  onRecord: (target: RecordingTarget | null) => void;
  onUpdate: (patch: Partial<ComboCommandAction>) => void;
}) {
  const recordingSequence =
    recordingTarget?.type === "commandSequence" &&
    recordingTarget.comboId === comboId &&
    recordingTarget.actionId === action.id;
  const commandKeysIssue = getComboIssue(issues, "commandKeys");
  const directionCount = countComboCommandDirections(action.keys);

  return (
    <div className="space-y-2">
      <div className="flex flex-wrap gap-2">
        <div className="w-[132px]">
          <NumberField
            issue={getComboIssue(issues, "keyHoldMs")}
            label="按下"
            max={1000}
            min={10}
            value={action.keyHoldMs}
            onChange={(value) => onUpdate({ keyHoldMs: normalizeComboHoldMs(value) })}
          />
        </div>
        <div className="w-[148px]">
          <NumberField
            issue={getComboIssue(issues, "keyGapMs")}
            label="键间隔"
            max={1000}
            min={0}
            value={action.keyGapMs}
            onChange={(value) => onUpdate({ keyGapMs: normalizeComboGapMs(value) })}
          />
        </div>
      </div>
      <div className="flex flex-wrap gap-1.5">
        {action.keys.map((vk, keyIndex) => {
          return (
            <span
              key={`${vk}-${keyIndex}`}
              className="inline-flex h-9 min-w-10 items-center justify-center rounded border border-slate-200 bg-white px-3 text-[15px] font-semibold text-slate-800"
            >
              {keyLabel(vk)}
            </span>
          );
        })}
        <button
          className={`inline-flex h-9 items-center gap-1.5 rounded border px-3 text-[15px] font-semibold shadow-sm transition ${
            recordingSequence
              ? "border-blue-400 bg-blue-50 text-blue-700"
              : commandKeysIssue
                ? "border-red-300 bg-red-50/40 text-red-700 hover:border-red-400"
                : "border-blue-200 bg-blue-50 text-blue-700 hover:bg-blue-100"
          }`}
          type="button"
          onClick={() => {
            if (recordingSequence) {
              onRecord(null);
              return;
            }
            onUpdate({ keys: [] });
            onRecord({ type: "commandSequence", comboId, actionId: action.id });
          }}
        >
          <Plus size={15} />
          {recordingSequence ? "录入中..." : action.keys.length > 0 ? "重新录入" : "录入序列"}
        </button>
      </div>
      {recordingSequence && (
        <div className="text-xs font-medium text-blue-600">
          方向键最多 {MAX_COMBO_COMMAND_DIRECTION_KEYS} 个，按 Z/X/C/空格结束
          {directionCount > 0 && `（已录入 ${directionCount} 个方向键）`}
        </div>
      )}
      <FieldError message={commandKeysIssue?.message} />
    </div>
  );
}

function NumberField({
  issue,
  label,
  max,
  min,
  value,
  onChange,
}: {
  issue?: ComboValidationIssue;
  label: string;
  max: number;
  min: number;
  value: number;
  onChange: (value: number) => void;
}) {
  const [draftState, setDraftState] = useState({ sourceValue: value, text: String(value) });
  const inputValue = draftState.sourceValue === value ? draftState.text : String(value);

  return (
    <div>
      <label
        className={`grid h-9 grid-cols-[auto_1fr] items-center gap-2 rounded border bg-white px-3 ${
          issue ? "border-red-300 bg-red-50/40" : "border-slate-200"
        }`}
      >
        <span className="text-[15px] font-semibold text-slate-500">{label}</span>
        <input
          className="h-8 w-full min-w-0 border-0 bg-transparent text-center text-[15px] font-semibold text-slate-900 outline-none"
          max={max}
          min={min}
          type="number"
          value={inputValue}
          onBlur={(event) => {
            const nextValue = Number(event.currentTarget.value);
            if (Number.isFinite(nextValue)) {
              setDraftState({ sourceValue: nextValue, text: String(nextValue) });
              onChange(nextValue);
            } else {
              setDraftState({ sourceValue: value, text: String(value) });
            }
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
      </label>
      <FieldError message={issue?.message} />
    </div>
  );
}

function FieldError({ message }: { message: string | undefined }) {
  if (!message) return null;
  return <div className="mt-1 text-xs leading-4 text-red-600">{message}</div>;
}

function getComboIssue(
  issues: ComboValidationIssue[],
  field: ComboValidationIssue["field"],
): ComboValidationIssue | undefined {
  return issues.find((issue) => issue.field === field);
}

function getActionIssue(
  issues: ComboValidationIssue[],
  actionId: string,
  field: ComboValidationIssue["field"],
): ComboValidationIssue | undefined {
  return issues.find((issue) => issue.actionId === actionId && issue.field === field);
}

function updateRecordedAction(
  action: ComboAction,
  target: RecordingTarget,
  vk: number,
): ComboAction {
  // 录制目标决定更新哪个字段：触发键在外层处理，这里只处理动作块内部按键。
  if (target.type === "tap" && action.id === target.actionId && action.type === "tap") {
    return { ...action, vk };
  }
  if (
    target.type === "commandSequence" &&
    action.id === target.actionId &&
    action.type === "command"
  ) {
    return { ...action, keys: [...action.keys, vk] };
  }
  return action;
}

function patchAction(action: ComboAction, patch: Partial<ComboAction>): ComboAction {
  // ComboAction 是联合类型，按当前动作类型收窄后再合并，保留各自动作字段。
  if (action.type === "tap") {
    return { ...action, ...(patch as Partial<ComboTapAction>) };
  }
  return { ...action, ...(patch as Partial<ComboCommandAction>) };
}

function createCombo(): ComboDefinition {
  return {
    id: createId("combo"),
    name: "新连招",
    enabled: false,
    triggerVk: null,
    actions: [],
  };
}

function createTapAction(): ComboTapAction {
  return {
    id: createId("action"),
    type: "tap",
    label: "",
    vk: null,
    holdMs: 30,
    waitAfterMs: 100,
  };
}

function createCommandAction(): ComboCommandAction {
  return {
    id: createId("action"),
    type: "command",
    label: "",
    keys: [],
    keyHoldMs: 30,
    keyGapMs: 20,
    waitAfterMs: 100,
  };
}

function createId(prefix: string): string {
  return `${prefix}-${globalThis.crypto?.randomUUID?.() ?? Date.now().toString(36)}`;
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
