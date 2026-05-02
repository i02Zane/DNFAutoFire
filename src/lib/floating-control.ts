// 悬浮窗共享常量：前端路由、Tauri 窗口 label、尺寸和本地位置缓存键。
import { getWebviewTextScale } from "./window-scale";

export const FLOATING_CONTROL_LABEL = "floating-control";
export const FLOATING_CONTROL_VIEW = "floating-control";
export const FLOATING_CONTROL_POSITION_KEY = "dnf-autofire:floating-control-position";
export const FLOATING_CONTROL_INITIAL_SIZE = {
  width: 260,
  height: 58,
};
export const FLOATING_CONTROL_MARGIN = 18;

export { getWebviewTextScale as getFloatingControlTextScale };
