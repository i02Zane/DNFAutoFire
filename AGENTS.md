# DNF按键助手 Agent 指南

本文件只保留代码代理需要额外注意的入口和架构边界。通用协作、命令、编码安全、校验、提交和发布要求统一见 `CONTRIBUTING.md`。

## 必读

- 贡献与协作要求：`CONTRIBUTING.md`
- 项目概览与运行要求：`README.md`
- 当前架构说明：`ARCHITECTURE.md`
- 文档索引：`docs/README.md`
- 踩坑记录：`docs/pitfalls.json`

## 关键架构

前端：
- 入口：`src/main.tsx`
- 主应用：`src/app.tsx`
- 通用 UI：`src/components/app-ui.tsx`
- 全局状态：`src/store/`
- 生成 DTO：`src/generated/backend-types.ts`
- 配置和本地 helper 类型：`src/types/app-config.ts`
- 结构化错误类型：`src/types/app-error.ts`
- 功能页面：`src/features/`
- 业务钩子：`src/hooks/`
- Tauri 命令封装：`src/lib/tauri-commands.ts`
- Tauri 环境判断：`src/lib/tauri-env.ts`
- 配置工具：`src/lib/config.ts`
- 按键映射：`src/lib/keys.ts`、`src/lib/browser-keys.ts`
- 悬浮窗：`src/features/floating-control/`

后端入口和职责：
- 后端入口：`src-tauri/src/lib.rs`、`src-tauri/src/main.rs`
- 应用装配：`src-tauri/src/app/`
- 统一错误类型：`src-tauri/src/error.rs`
- IPC 命令入口：`src-tauri/src/ipc/`
- 配置系统：`src-tauri/src/config/`
- 领域数据：`src-tauri/src/domain/`
- 运行时协调：`src-tauri/src/runtime/`
- 职业识别：`src-tauri/src/vision/`
- Win32 核心：`src-tauri/src/platform/`
- 托盘与提示：`src-tauri/src/platform/tray.rs`、`src-tauri/src/platform/notify.rs`
- 启动与日志：`src-tauri/src/platform/startup.rs`、`src-tauri/src/platform/logging.rs`
- 全局快捷键：`src-tauri/src/platform/hotkey.rs`

## 架构边界

- 前端通过 `src/lib/tauri-commands.ts` 导出的 `tauriCommands.*` 调用后端，不在页面中散落底层 Tauri API。
- 配置文件拆为 `{exe_dir}/configs/settings.json` 和 `{exe_dir}/configs/profiles.json`；旧版 `{exe_dir}/configs/app-config.json` 仅作为迁移来源。
- 当前 schema 版本：`settings.json` 为 `1`，`profiles.json` 为 `1`，后端通过 bootstrap / 事件下发显式配置快照和运行态快照。
- 从应用版本 `0.5.0` 开始视为已有用户数据。变更 `settings.json` / `profiles.json` 结构时必须补迁移、校验、测试和对应 schema version；同一发布周期内同一配置文件 schema version 只递增一次，不随每次开发改动反复递增。
- 跨端 DTO 由 Rust `ts-rs` 生成到 `src/generated/backend-types.ts`；改动 DTO 后运行 `pnpm generate:types` 和 `pnpm types:check`。
- IPC 错误统一返回 `{ kind, message }` 结构化 `AppError`；前端只保留 transport 兜底，不新增旧字符串错误兼容。
- `AppStateSnapshot`、`RuntimeStateSnapshot`、bootstrap payload 都带 `revision`，前端必须丢弃旧 revision 事件。
- `comboDefs` 根字段当前保留兼容和未来全局连招扩展，不作为现有编辑主路径。
- 职业目录和职业识别 ID 映射由后端维护，前端通过 bootstrap / 事件消费，不新增前端职业数据表。
- 配置选择器选项、职业显示/隐藏、已配置标记等展示规则由后端 `profileDisplay` 快照下发，前端只做搜索过滤和渲染。
- `activeClassId` 和 `hiddenClassIds` 属于 `profiles.json`，不再放进 `settings.json`。
- 职业配置是预设 id/name 且不可删除的配置；除显示/隐藏和删除权限外，应与自定义配置保持同一套运行逻辑。
- 项目统一使用 Windows Virtual Key 码（`u16` / number）表示按键。

## 检查补充

- 用户经常会在另一个终端运行 `pnpm dev` 做测试，`src-tauri/target` 可能被 Tauri dev 进程占用。
- 如果 `pnpm rust:clippy` 因 `src-tauri/target` 文件锁、Tauri build script `os error 32` 或资源文件占用失败，改用临时 target 目录重跑：
  `cmd.exe /d /c "set ""CARGO_TARGET_DIR=src-tauri\target-clippy"" && pnpm rust:clippy"`。
- 临时检查结束后删除 `src-tauri/target-clippy`，不要提交该目录。

## 悬浮窗链路

悬浮窗是全局唯一的 `floating-control` 窗口，由后端 `src-tauri/src/platform/floating_control.rs` 创建、显示、隐藏和恢复位置，入口为 `/?view=floating-control`。前端只负责悬浮窗 React 内容、尺寸调整，以及把用户拖动后的新位置作为意图回传后端。

主窗口最小化、关闭和关闭前清理悬浮窗也归后端 `RuntimeSupervisor` 管理。前端标题栏只调用 `tauriCommands.minimizeMainWindow()` / `closeMainWindow()`，不要直接调用 Tauri window API 决定生命周期。

托盘菜单和主窗口中的“打开悬浮窗 / 关闭悬浮窗”统一走后端运行态：

1. 前端调用 `tauriCommands.showFloatingControlWindow()` / `hideFloatingControlWindow()`，托盘直接调用 `RuntimeSupervisor`。
2. 后端 `RuntimeSupervisor` 调用 `FloatingControlRuntime` 创建或隐藏窗口。
3. 后端通过 `settings.json` 保存悬浮窗位置，通过 `runtime-state:changed` 下发 `floatingControlVisible`。
4. 主窗口、悬浮窗和托盘菜单都消费同一份运行态快照。
5. 不要恢复旧的 `floating-control:toggle-request` / `floating-control:visibility-changed` 前端同步链路。
