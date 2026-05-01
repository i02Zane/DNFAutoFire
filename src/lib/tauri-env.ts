// Tauri 环境探测：用于区分桌面运行和浏览器预览/mock 模式。
export function isTauriEnvironment(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export function isMockMode(): boolean {
  return !isTauriEnvironment();
}
