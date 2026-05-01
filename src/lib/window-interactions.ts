// 桌面生产环境交互保护：禁用浏览器式右键、拖拽和非输入区文本选择。
import { isMockMode } from "./tauri-env";

function isEditableTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;

  return (
    target.isContentEditable ||
    target instanceof HTMLInputElement ||
    target instanceof HTMLTextAreaElement
  );
}

export function installWindowInteractionGuards(): void {
  if (isMockMode() || import.meta.env.DEV) return;

  // 只在打包桌面版启用，保留开发调试和浏览器预览中的默认浏览器行为。
  document.documentElement.classList.add("window-interactions-guarded");

  window.addEventListener(
    "contextmenu",
    (event) => {
      event.preventDefault();
    },
    { capture: true },
  );

  window.addEventListener(
    "selectstart",
    (event) => {
      if (!isEditableTarget(event.target)) {
        event.preventDefault();
      }
    },
    { capture: true },
  );

  window.addEventListener(
    "dragstart",
    (event) => {
      event.preventDefault();
    },
    { capture: true },
  );
}
