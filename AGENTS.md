# DNF按键助手 Agent 指南

本文件只保留代码代理需要额外注意的入口和架构边界。通用协作、命令、编码安全、校验、提交和发布要求统一见 `CONTRIBUTING.md`。

## 必读

- 贡献与协作要求：`CONTRIBUTING.md`
- 项目概览与运行要求：`README.md`
- 文档索引：`docs/README.md`
- 踩坑记录：`docs/pitfalls.json`

## 关键架构

前端：
- 入口：`src/main.tsx`
- 主应用：`src/app.tsx`
- 通用 UI：`src/components/app-ui.tsx`
- 配置类型：`src/types/app-config.ts`
- 页面：`src/pages/`
- 业务钩子：`src/hooks/`
- Tauri 兼容导出层：`src/lib/tauri.ts`
- Tauri 命令封装：`src/lib/tauri-commands.ts`
- 浏览器预览 mock：`src/lib/mock-tauri.ts`
- 配置工具：`src/lib/config.ts`
- 按键映射：`src/lib/keys.ts`、`src/lib/browser-keys.ts`
- 职业数据：`src/data/classes.ts`
- 悬浮窗：`src/floating-control/`

后端入口和职责：
- 后端入口：`src-tauri/src/lib.rs`、`src-tauri/src/main.rs`
- 后端命令入口：`src-tauri/src/commands.rs`
- 后端配置系统：`src-tauri/src/config.rs`
- 后端共享状态：`src-tauri/src/state.rs`
- 后端托盘与提示：`src-tauri/src/tray.rs`、`src-tauri/src/notify.rs`
- 启动与日志：`src-tauri/src/startup.rs`、`src-tauri/src/logging.rs`
- 全局快捷键：`src-tauri/src/hotkey.rs`
- Win32 核心：`src-tauri/src/core/`

## 架构边界

- 前端通过 `src/lib/tauri.ts` 导出的 `tauriCommands.*` 调用后端，不绕过兼容层散落调用 Tauri API。
- 配置文件位于 `{exe_dir}/configs/app-config.json`，当前 schema 版本为 `6`；配置字段变更要同步前端类型、后端结构、迁移、校验和文档。
- `comboDefs` 根字段只做兼容保留，当前不参与前端编辑或运行时下发。
- 职业配置是预设 id/name 且不可删除的配置；除显示/隐藏和删除权限外，应与自定义配置保持同一套运行逻辑。
- 项目统一使用 Windows Virtual Key 码（`u16` / number）表示按键。

## 悬浮窗链路

悬浮窗是全局唯一的 `floating-control` 窗口，由前端创建和关闭，入口为 `/?view=floating-control`。不要新增 Rust 侧同名窗口创建链路。

托盘菜单中的“打开悬浮窗 / 关闭悬浮窗”不直接操作窗口，而是：

1. Rust 托盘菜单触发 `floating-control:toggle-request`。
2. 主窗口前端更新唯一的 `floatingControlEnabled` 状态。
3. 前端调用 `showFloatingControlWindow()` 或 `hideFloatingControlWindow()`。
4. 前端发送 `floating-control:visibility-changed`。
5. Rust 监听可见性事件并更新托盘菜单文字。
