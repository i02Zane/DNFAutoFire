// 前端多窗口事件总线：集中定义事件名和 payload，避免主窗口/悬浮窗各写一份字符串。
import { type AppConfig, type ClassDetectionResult } from "../types/app-config";
import { isMockMode } from "./tauri-env";

export const APP_EVENTS = {
  classDetectionResult: "class-detection:result",
  floatingControlClassChanged: "floating-control:class-changed",
  floatingControlToggleRequest: "floating-control:toggle-request",
  floatingControlUpdate: "floating-control:update",
  floatingControlVisibilityChanged: "floating-control:visibility-changed",
} as const;

export type FloatingControlUpdatePayload = {
  config: AppConfig;
  running: boolean;
};

export type FloatingControlVisibilityPayload = {
  visible: boolean;
};

export type FloatingControlClassChangedPayload = {
  activeClassId: string | null;
};

type AppEventPayloads = {
  // 新增事件时先在这里登记 payload，再通过 emitAppEvent/listenAppEvent 调用。
  [APP_EVENTS.classDetectionResult]: ClassDetectionResult;
  [APP_EVENTS.floatingControlClassChanged]: FloatingControlClassChangedPayload;
  [APP_EVENTS.floatingControlToggleRequest]: undefined;
  [APP_EVENTS.floatingControlUpdate]: FloatingControlUpdatePayload;
  [APP_EVENTS.floatingControlVisibilityChanged]: FloatingControlVisibilityPayload;
};

type AppEventName = keyof AppEventPayloads;
type UnlistenFn = () => void;

// 所有前端窗口事件都从这里进出，保持事件名和 payload 类型同步演进。
export async function emitAppEvent<EventName extends AppEventName>(
  eventName: EventName,
  payload: AppEventPayloads[EventName],
): Promise<void> {
  if (isMockMode()) return;

  const { emit } = await import("@tauri-apps/api/event");
  await emit(eventName, payload);
}

export async function listenAppEvent<EventName extends AppEventName>(
  eventName: EventName,
  handler: (payload: AppEventPayloads[EventName]) => void,
): Promise<UnlistenFn> {
  if (isMockMode()) return () => undefined;

  const { listen } = await import("@tauri-apps/api/event");
  return listen<AppEventPayloads[EventName]>(eventName, (event) => handler(event.payload));
}
