import { useCallback, useEffect, useRef, useState } from "react";
import { browserKeyToVk } from "../../lib/browser-keys";
import {
  countComboCommandDirections,
  getProfileConfig,
  isComboCommandDirectionVk,
  isComboCommandFinishVk,
  isComboCommandVk,
  MAX_COMBO_COMMAND_DIRECTION_KEYS,
} from "../../lib/config";
import type {
  ComboAction,
  ComboCommandAction,
  ComboDefinition,
  ComboTapAction,
  ComboValidationIssue,
  ProfilesConfig,
} from "../../types/app-config";

export type RecordingTarget =
  | { type: "trigger"; comboId: string }
  | { type: "tap"; comboId: string; actionId: string }
  | { type: "commandSequence"; comboId: string; actionId: string };

type UseComboDraftOptions = {
  profiles: ProfilesConfig;
  selectedConfigId: string | null;
  onCombosChange: (configId: string, combos: ComboDefinition[]) => Promise<boolean>;
  onValidateComboDefs: (
    configId: string,
    combos: ComboDefinition[],
  ) => Promise<ComboValidationIssue[]>;
};

const EMPTY_COMBOS: ComboDefinition[] = [];

export function useComboDraft({
  profiles,
  selectedConfigId,
  onCombosChange,
  onValidateComboDefs,
}: UseComboDraftOptions) {
  const [recordingTarget, setRecordingTarget] = useState<RecordingTarget | null>(null);
  const [comboDrafts, setComboDrafts] = useState<Record<string, ComboDefinition[]>>({});
  const [validationResult, setValidationResult] = useState<{
    configId: string | null;
    issues: ComboValidationIssue[];
  }>({ configId: null, issues: [] });
  const validationRequestIdRef = useRef(0);
  const pendingDraftValidationRef = useRef<{
    configId: string;
    combos: ComboDefinition[];
  } | null>(null);
  const activeConfigId = selectedConfigId;
  const activeConfig = activeConfigId ? getProfileConfig(profiles, activeConfigId) : null;
  const savedCombos = activeConfig?.comboDefs ?? EMPTY_COMBOS;
  const combos = activeConfigId ? (comboDrafts[activeConfigId] ?? savedCombos) : EMPTY_COMBOS;
  const validationIssues =
    activeConfigId && validationResult.configId === activeConfigId ? validationResult.issues : [];

  const validateCombos = useCallback(
    async (configId: string, nextCombos: ComboDefinition[], commitWhenValid: boolean) => {
      const requestId = validationRequestIdRef.current + 1;
      validationRequestIdRef.current = requestId;
      const issues = await onValidateComboDefs(configId, nextCombos);
      if (requestId !== validationRequestIdRef.current) return issues;

      setValidationResult({ configId, issues });
      if (commitWhenValid && issues.length === 0) {
        const saved = await onCombosChange(configId, nextCombos);
        if (saved && requestId === validationRequestIdRef.current) {
          setComboDrafts((current) => {
            if (!current[configId]) return current;
            const next = { ...current };
            delete next[configId];
            return next;
          });
        }
      }
      return issues;
    },
    [onCombosChange, onValidateComboDefs],
  );

  const updateCombos = useCallback(
    (nextCombos: ComboDefinition[]) => {
      if (!activeConfigId) return;
      pendingDraftValidationRef.current = { configId: activeConfigId, combos: nextCombos };
      setComboDrafts((current) => ({ ...current, [activeConfigId]: nextCombos }));
      // 无效草稿继续留在页面上供用户修正，避免把不可运行的连招写入配置。
      void validateCombos(activeConfigId, nextCombos, true);
    },
    [activeConfigId, validateCombos],
  );

  useEffect(() => {
    if (!activeConfigId) {
      validationRequestIdRef.current += 1;
      pendingDraftValidationRef.current = null;
      return;
    }

    const pendingDraftValidation = pendingDraftValidationRef.current;
    if (
      pendingDraftValidation?.configId === activeConfigId &&
      pendingDraftValidation.combos === combos
    ) {
      pendingDraftValidationRef.current = null;
      return;
    }

    void validateCombos(activeConfigId, combos, false);
  }, [activeConfigId, combos, validateCombos]);

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

  const addCombo = useCallback(() => {
    updateCombos([...combos, createCombo()]);
  }, [combos, updateCombos]);

  const updateCombo = useCallback(
    (comboId: string, patch: Partial<ComboDefinition>) => {
      updateCombos(combos.map((combo) => (combo.id === comboId ? { ...combo, ...patch } : combo)));
    },
    [combos, updateCombos],
  );

  const deleteCombo = useCallback(
    (comboId: string) => {
      updateCombos(combos.filter((combo) => combo.id !== comboId));
    },
    [combos, updateCombos],
  );

  const addAction = useCallback(
    (comboId: string, type: ComboAction["type"]) => {
      const action = type === "tap" ? createTapAction() : createCommandAction();
      updateCombos(
        combos.map((combo) =>
          combo.id === comboId ? { ...combo, actions: [...combo.actions, action] } : combo,
        ),
      );
    },
    [combos, updateCombos],
  );

  const updateAction = useCallback(
    (comboId: string, actionId: string, patch: Partial<ComboAction>) => {
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
    },
    [combos, updateCombos],
  );

  const deleteAction = useCallback(
    (comboId: string, actionId: string) => {
      updateCombos(
        combos.map((combo) =>
          combo.id === comboId
            ? { ...combo, actions: combo.actions.filter((action) => action.id !== actionId) }
            : combo,
        ),
      );
    },
    [combos, updateCombos],
  );

  const moveAction = useCallback(
    (
      comboId: string,
      sourceActionId: string,
      targetActionId: string,
      placement: ActionPlacement,
    ) => {
      if (sourceActionId === targetActionId) return;
      updateCombos(
        combos.map((combo) =>
          combo.id === comboId
            ? {
                ...combo,
                actions: moveActionInList(combo.actions, sourceActionId, targetActionId, placement),
              }
            : combo,
        ),
      );
    },
    [combos, updateCombos],
  );

  return {
    activeConfigId,
    addAction,
    addCombo,
    combos,
    deleteAction,
    deleteCombo,
    moveAction,
    recordingTarget,
    setRecordingTarget,
    updateAction,
    updateCombo,
    validationIssues,
  };
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

type ActionPlacement = "before" | "after";

function moveActionInList(
  actions: ComboAction[],
  sourceActionId: string,
  targetActionId: string,
  placement: ActionPlacement,
): ComboAction[] {
  const sourceIndex = actions.findIndex((action) => action.id === sourceActionId);
  const targetIndex = actions.findIndex((action) => action.id === targetActionId);
  if (sourceIndex < 0 || targetIndex < 0) return actions;

  const nextActions = [...actions];
  const [sourceAction] = nextActions.splice(sourceIndex, 1);
  if (!sourceAction) return actions;

  let insertIndex = targetIndex + (placement === "after" ? 1 : 0);
  if (sourceIndex < insertIndex) {
    insertIndex -= 1;
  }
  nextActions.splice(insertIndex, 0, sourceAction);
  return nextActions;
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
