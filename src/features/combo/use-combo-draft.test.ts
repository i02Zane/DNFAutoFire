import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { Mock } from "vitest";
import type {
  ComboCommandAction,
  ComboDefinition,
  ComboValidationIssue,
  ProfilesConfig,
} from "../../types/app-config";
import { useComboDraft } from "./use-combo-draft";

describe("useComboDraft", () => {
  afterEach(() => {
    cleanup();
    vi.restoreAllMocks();
  });

  it("提交有效草稿并丢弃过期校验结果", async () => {
    const staleValidation = createDeferred<ComboValidationIssue[]>();
    const onCombosChange = vi.fn<(configId: string, combos: ComboDefinition[]) => Promise<boolean>>(
      async () => true,
    );
    const onValidateComboDefs = vi.fn(async (configId: string, combos: ComboDefinition[]) => {
      void configId;
      if (combos[0]?.name === "") return staleValidation.promise;
      return [];
    });
    const profiles = createProfiles();
    const { result } = renderHook(() =>
      useComboDraft({
        profiles,
        selectedConfigId: "class-a",
        onCombosChange,
        onValidateComboDefs,
      }),
    );

    await waitFor(() => expect(onValidateComboDefs).toHaveBeenCalled());

    act(() => result.current.updateCombo("combo-1", { name: "" }));
    await waitFor(() => expect(result.current.combos[0]?.name).toBe(""));

    act(() => result.current.updateCombo("combo-1", { name: "新连招" }));
    await waitFor(() => expect(lastSavedCombos(onCombosChange)[0]?.name).toBe("新连招"));

    staleValidation.resolve([nameIssue()]);
    await Promise.resolve();

    expect(result.current.validationIssues).toEqual([]);
    expect(onCombosChange.mock.calls.some((call) => call[1][0]?.name === "")).toBe(false);
  });

  it("录入触发键后保存草稿并退出录制状态", async () => {
    const onCombosChange = vi.fn<(configId: string, combos: ComboDefinition[]) => Promise<boolean>>(
      async () => true,
    );
    const onValidateComboDefs = vi.fn(async () => []);
    const profiles = createProfiles();
    const { result } = renderHook(() =>
      useComboDraft({
        profiles,
        selectedConfigId: "class-a",
        onCombosChange,
        onValidateComboDefs,
      }),
    );
    await waitFor(() => expect(onValidateComboDefs).toHaveBeenCalled());

    act(() => result.current.setRecordingTarget({ type: "trigger", comboId: "combo-1" }));
    await waitFor(() => expect(result.current.recordingTarget?.type).toBe("trigger"));
    await flushEffects();

    const event = pressKey("KeyX");

    expect(event.defaultPrevented).toBe(true);
    await waitFor(() => expect(lastSavedCombos(onCombosChange)[0]?.triggerVk).toBe(0x58));
    expect(result.current.recordingTarget).toBeNull();
  });

  it("录入手搓序列时只接受允许键并在结束键后退出", async () => {
    let profiles = createProfiles();
    const onValidateComboDefs = vi.fn(async () => []);
    const onCombosChange = vi.fn(async (configId: string, combos: ComboDefinition[]) => {
      void configId;
      profiles = createProfiles(combos);
      return true;
    });
    const { rerender, result } = renderHook(
      ({ nextProfiles }: { nextProfiles: ProfilesConfig }) =>
        useComboDraft({
          profiles: nextProfiles,
          selectedConfigId: "class-a",
          onCombosChange,
          onValidateComboDefs,
        }),
      { initialProps: { nextProfiles: profiles } },
    );

    act(() =>
      result.current.setRecordingTarget({
        type: "commandSequence",
        comboId: "combo-1",
        actionId: "command-1",
      }),
    );
    await waitFor(() => expect(result.current.recordingTarget?.type).toBe("commandSequence"));
    await flushEffects();

    pressKey("KeyA");
    expect(onCombosChange).not.toHaveBeenCalled();
    expect(result.current.recordingTarget?.type).toBe("commandSequence");

    pressKey("ArrowLeft");
    await waitFor(() =>
      expect(commandAction(lastSavedCombos(onCombosChange)).keys).toEqual([0x25]),
    );
    rerender({ nextProfiles: profiles });
    await flushEffects();

    pressKey("KeyX");
    await waitFor(() =>
      expect(commandAction(lastSavedCombos(onCombosChange)).keys).toEqual([0x25, 0x58]),
    );
    expect(result.current.recordingTarget).toBeNull();
  });

  it("调整动作块顺序时连同等待时间一起保存", async () => {
    const onCombosChange = vi.fn<(configId: string, combos: ComboDefinition[]) => Promise<boolean>>(
      async () => true,
    );
    const onValidateComboDefs = vi.fn(async () => []);
    const profiles = createProfiles();
    const { result } = renderHook(() =>
      useComboDraft({
        profiles,
        selectedConfigId: "class-a",
        onCombosChange,
        onValidateComboDefs,
      }),
    );
    await waitFor(() => expect(onValidateComboDefs).toHaveBeenCalled());

    act(() => result.current.moveAction("combo-1", "command-1", "tap-1", "before"));

    await waitFor(() =>
      expect(lastSavedCombos(onCombosChange)[0]?.actions.map((action) => action.id)).toEqual([
        "command-1",
        "tap-1",
      ]),
    );
    expect(lastSavedCombos(onCombosChange)[0]?.actions[0]?.waitAfterMs).toBe(222);
  });
});

function createProfiles(combos: ComboDefinition[] = [createCombo()]): ProfilesConfig {
  return {
    version: 1,
    globalKeys: [],
    comboDefs: [],
    classes: {
      "class-a": {
        enabledKeys: [],
        effectRule: "globalAndClass",
        comboDefs: combos,
      },
    },
    customConfigs: {},
    hiddenClassIds: [],
    activeClassId: "class-a",
    autoRun: {
      enabled: false,
      leftVk: 0x25,
      rightVk: 0x27,
      pulseDelayMs: 25,
    },
  };
}

function createCombo(): ComboDefinition {
  return {
    id: "combo-1",
    name: "原连招",
    enabled: true,
    triggerVk: null,
    actions: [
      {
        id: "tap-1",
        type: "tap",
        label: "",
        vk: null,
        holdMs: 30,
        waitAfterMs: 111,
      },
      {
        id: "command-1",
        type: "command",
        label: "",
        keys: [],
        keyHoldMs: 30,
        keyGapMs: 20,
        waitAfterMs: 222,
      },
    ],
  };
}

function nameIssue(): ComboValidationIssue {
  return {
    comboId: "combo-1",
    actionId: null,
    field: "name",
    message: "连招名称不能为空",
  };
}

function pressKey(code: string): KeyboardEvent {
  const event = new KeyboardEvent("keydown", {
    bubbles: true,
    cancelable: true,
  });
  Object.defineProperty(event, "code", { value: code });
  act(() => {
    window.dispatchEvent(event);
  });
  return event;
}

function commandAction(combos: ComboDefinition[]): ComboCommandAction {
  const action = combos[0]?.actions.find((item) => item.id === "command-1");
  if (action?.type !== "command") {
    throw new Error("测试数据缺少手搓动作");
  }
  return action;
}

function lastSavedCombos(
  onCombosChange: Mock<(configId: string, combos: ComboDefinition[]) => Promise<boolean>>,
): ComboDefinition[] {
  const calls = onCombosChange.mock.calls;
  const lastCall = calls[calls.length - 1];
  if (!lastCall) {
    throw new Error("没有保存过连招草稿");
  }
  return lastCall[1];
}

function createDeferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((resolveValue) => {
    resolve = resolveValue;
  });
  return { promise, resolve };
}

async function flushEffects() {
  await new Promise((resolve) => setTimeout(resolve, 0));
}
