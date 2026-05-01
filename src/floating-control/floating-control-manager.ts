// 悬浮控制管理器：主窗口唯一负责创建和隐藏 floating-control WebviewWindow。
import {
  FLOATING_CONTROL_INITIAL_SIZE,
  FLOATING_CONTROL_LABEL,
  FLOATING_CONTROL_MARGIN,
  FLOATING_CONTROL_POSITION_KEY,
  FLOATING_CONTROL_VIEW,
} from "../lib/floating-control";
import { FLOATING_CONTROL_WINDOW_TITLE } from "../lib/app-meta";
import { isTauriEnvironment } from "../lib/tauri-env";

type SavedWindowPosition = {
  x: number;
  y: number;
};

function readFloatingControlPosition(): SavedWindowPosition | null {
  const rawValue = window.localStorage.getItem(FLOATING_CONTROL_POSITION_KEY);
  if (!rawValue) return null;

  try {
    const position = JSON.parse(rawValue) as Partial<SavedWindowPosition>;
    if (typeof position.x === "number" && typeof position.y === "number") {
      return { x: position.x, y: position.y };
    }
  } catch {
    window.localStorage.removeItem(FLOATING_CONTROL_POSITION_KEY);
  }
  return null;
}

async function defaultFloatingControlPosition(): Promise<SavedWindowPosition | null> {
  const { primaryMonitor } = await import("@tauri-apps/api/window");
  const monitor = await primaryMonitor();
  if (!monitor) return null;

  const width = FLOATING_CONTROL_INITIAL_SIZE.width * monitor.scaleFactor;
  const height = FLOATING_CONTROL_INITIAL_SIZE.height * monitor.scaleFactor;
  return {
    x: Math.round(
      monitor.workArea.position.x + monitor.workArea.size.width - width - FLOATING_CONTROL_MARGIN,
    ),
    y: Math.round(
      monitor.workArea.position.y + monitor.workArea.size.height - height - FLOATING_CONTROL_MARGIN,
    ),
  };
}

function saveFloatingControlPosition(position: SavedWindowPosition) {
  window.localStorage.setItem(FLOATING_CONTROL_POSITION_KEY, JSON.stringify(position));
}

export async function showFloatingControlWindow(): Promise<boolean> {
  if (!isTauriEnvironment()) {
    throw new Error("当前是浏览器预览模式，悬浮窗只在 Tauri 应用中可用。");
  }

  const { WebviewWindow } = await import("@tauri-apps/api/webviewWindow");
  const { PhysicalPosition } = await import("@tauri-apps/api/window");
  // 悬浮控制保持全局唯一：已存在时只恢复显示和焦点，不再创建第二个窗口。
  const existingWindow = await WebviewWindow.getByLabel(FLOATING_CONTROL_LABEL);
  if (existingWindow) {
    await existingWindow.show();
    await existingWindow.setFocus();
    return true;
  }

  // 优先恢复上次拖动的位置；没有历史位置时，再按屏幕右下角计算初始点。
  const savedPosition = readFloatingControlPosition();
  const initialPosition = savedPosition ?? (await defaultFloatingControlPosition());
  const controlWindow = new WebviewWindow(FLOATING_CONTROL_LABEL, {
    title: FLOATING_CONTROL_WINDOW_TITLE,
    url: `/?view=${FLOATING_CONTROL_VIEW}`,
    width: FLOATING_CONTROL_INITIAL_SIZE.width,
    height: FLOATING_CONTROL_INITIAL_SIZE.height,
    resizable: false,
    decorations: false,
    alwaysOnTop: true,
    skipTaskbar: true,
    visible: false,
  });

  return new Promise((resolve, reject) => {
    void controlWindow.once("tauri://created", () => {
      void (async () => {
        try {
          if (initialPosition) {
            await controlWindow.setPosition(
              new PhysicalPosition(initialPosition.x, initialPosition.y),
            );
          }
          // 记录拖动后的物理坐标，下一次打开时继续沿用上次位置。
          void controlWindow.onMoved(({ payload }) => {
            saveFloatingControlPosition({ x: payload.x, y: payload.y });
          });
          await controlWindow.show();
          await controlWindow.setFocus();
          resolve(true);
        } catch (error) {
          reject(error);
        }
      })();
    });
    void controlWindow.once("tauri://error", (error) => {
      reject(new Error(`创建悬浮窗失败：${String(error.payload)}`));
    });
  });
}

export async function hideFloatingControlWindow(): Promise<boolean> {
  if (!isTauriEnvironment()) return false;

  const { getCurrentWebviewWindow, WebviewWindow } = await import("@tauri-apps/api/webviewWindow");
  const currentWindow = getCurrentWebviewWindow();
  // 当前如果就是悬浮控制窗口，直接隐藏自己即可。
  if (currentWindow.label === FLOATING_CONTROL_LABEL) {
    await currentWindow.hide();
    return true;
  }

  const controlWindow = await WebviewWindow.getByLabel(FLOATING_CONTROL_LABEL);
  if (controlWindow) {
    await controlWindow.hide();
  }
  return true;
}
