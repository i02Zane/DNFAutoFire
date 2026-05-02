// 悬浮窗共享常量：前端路由、Tauri 窗口 label、尺寸和本地位置缓存键。
export const FLOATING_CONTROL_LABEL = "floating-control";
export const FLOATING_CONTROL_VIEW = "floating-control";
export const FLOATING_CONTROL_POSITION_KEY = "dnf-autofire:floating-control-position";
export const FLOATING_CONTROL_INITIAL_SIZE = {
  width: 260,
  height: 58,
};
export const FLOATING_CONTROL_MARGIN = 18;

export function getFloatingControlTextScale(monitorScaleFactor: number): number {
  if (typeof window === "undefined") return 1;

  const rasterizationScale = window.devicePixelRatio || 1;
  const textScale = rasterizationScale / monitorScaleFactor;
  return Number.isFinite(textScale) && textScale > 0 ? textScale : 1;
}
