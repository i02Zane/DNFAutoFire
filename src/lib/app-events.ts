// 前端多窗口事件总线：集中定义事件名和 payload，避免主窗口/悬浮窗各写一份字符串。
import type { AppStateSnapshot, RuntimeStateSnapshot } from "../types/app-config";
import type { BackendAppError } from "../types/app-error";
import { isMockMode } from "./tauri-env";

export const APP_EVENTS = {
  appConfigChanged: "app-config:changed",
  runtimeStateChanged: "runtime-state:changed",
  runtimeError: "runtime-error",
} as const;

export type RuntimeStateChangedPayload = RuntimeStateSnapshot;

export type RuntimeErrorPayload = BackendAppError;

type AppEventPayloads = {
  // 新增事件时先在这里登记 payload，再通过 emitAppEvent/listenAppEvent 调用。
  [APP_EVENTS.appConfigChanged]: AppStateSnapshot;
  [APP_EVENTS.runtimeStateChanged]: RuntimeStateChangedPayload;
  [APP_EVENTS.runtimeError]: RuntimeErrorPayload;
};

type AppEventName = keyof AppEventPayloads;
type UnlistenFn = () => void;

// 所有前端窗口事件都从这里进出，保持事件名和 payload 类型同步演进。
export async function emitAppEvent<EventName extends AppEventName>(
  eventName: EventName,
  payload: AppEventPayloads[EventName],
): Promise<void> {
  if (isMockMode()) {
    if (typeof window === "undefined") return;
    window.dispatchEvent(
      new CustomEvent(eventName, {
        detail: payload,
      }),
    );
    return;
  }

  const { emit } = await import("@tauri-apps/api/event");
  await emit(eventName, payload);
}

export async function listenAppEvent<EventName extends AppEventName>(
  eventName: EventName,
  handler: (payload: AppEventPayloads[EventName]) => void,
): Promise<UnlistenFn> {
  if (isMockMode()) {
    if (typeof window === "undefined") return () => undefined;

    const listener = (event: Event) => {
      handler((event as CustomEvent<AppEventPayloads[EventName]>).detail);
    };
    window.addEventListener(eventName, listener);
    return () => window.removeEventListener(eventName, listener);
  }

  const { listen } = await import("@tauri-apps/api/event");
  return listen<AppEventPayloads[EventName]>(eventName, (event) => handler(event.payload));
}
