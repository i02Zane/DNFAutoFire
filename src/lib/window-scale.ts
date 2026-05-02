export function getWebviewTextScale(monitorScaleFactor: number): number {
  if (typeof window === "undefined") return 1;

  const rasterizationScale = window.devicePixelRatio || 1;
  const textScale = rasterizationScale / monitorScaleFactor;
  return Number.isFinite(textScale) && textScale > 0 ? textScale : 1;
}
