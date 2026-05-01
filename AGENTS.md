# DNF按键助手 项目协作指南

本文件为 Claude Code、Codex 以及其他代码代理提供本仓库的统一开发约定。`CLAUDE.md` 只保留到本文件的引用；修改项目规则时只更新本文件。

## 快速参考

- 文档索引：`docs/README.md`
- 踩坑记录与经验总结：`docs/pitfalls.json`
- 编码检查脚本：`scripts/check-encoding.mjs`

## 开发规范

```json
{
  "语言": { "文档": "中文", "注释": "中文", "代码": "英文" },
  "文件命名": "kebab-case",
  "提交信息": "Conventional Commits",
  "工作流程": ["设计先行", "小步修改", "反复校验", "零警告", "完成即提交"],
  "代码原则": ["高内聚低耦合", "优先复用现有模式", "遵循官方最佳实践", "复杂逻辑保留必要中文注释"]
}
```

注释只用于解释状态边界、跨模块链路、并发/异步顺序、平台限制等不容易从代码本身看出的原因；避免给简单赋值、直观渲染和自解释函数添加噪音注释。

每完成一个用户需求并通过必要校验后，默认单独创建一次 git 提交；除非用户明确要求暂不提交，不要把已完成改动留在未提交状态。

提交信息使用 Conventional Commits，例如 `feat:`、`fix:`、`docs:`、`test:`、`refactor:`、`style:`、`chore:`、`ci:`、`build:`。

## 编码安全

Windows PowerShell 5.1 的默认写文件行为可能使用旧编码或 UTF-16LE，容易损坏本仓库中的中文文本。后续修改必须遵守：

- 不要用 PowerShell 的默认写文件命令修改源码。
- 避免用 `Set-Content`、`Out-File`、`Add-Content`、`>`、`>>` 写源码，除非显式指定并验证 UTF-8。
- 手工源码修改优先使用 `apply_patch`。
- 批量重写使用明确写入 UTF-8 的运行时，例如 Node.js `fs.writeFile(path, text, "utf8")`。
- 修改后运行 `pnpm encoding:check` 或包含它的 `pnpm check`。

## 常用命令

```bash
# 安装依赖
pnpm install

# 开发
pnpm dev            # Tauri 完整开发模式
pnpm dev:web        # 仅前端预览，自动进入 mock 模式

# 前端质量检查
pnpm typecheck      # TypeScript --noEmit
pnpm lint           # oxlint + eslint
pnpm lint:fix       # 自动修复可修复的前端 lint 问题
pnpm format         # oxfmt 格式化 src
pnpm format:check   # 检查前端格式
pnpm encoding:check # 检查乱码和替换字符
pnpm check          # typecheck + lint + format:check + encoding:check

# Rust
pnpm rust:check
pnpm rust:clippy
pnpm rust:fmt
pnpm rust:fmt:check
pnpm rust:test
pnpm rust:lint

# 全量检查
pnpm check:all
pnpm run ci

# 构建
pnpm build:web
pnpm build
```

## 技术栈

| 层级 | 技术 |
| --- | --- |
| 前端 | React 19 + Vite 7 + Tailwind CSS 4 |
| 桌面壳 | Tauri 2 |
| 后端 | Rust + Win32 API |
| UI 辅助 | lucide-react、Radix Tooltip/Dialog/Slot |
| 检查 | TypeScript、oxlint、ESLint、oxfmt |
| 包管理 | pnpm |

## 前端结构

- 入口：`src/main.tsx`
- 主应用：`src/app.tsx`
- 通用 UI：`src/components/app-ui.tsx`
- Tauri 兼容导出层：`src/lib/tauri.ts`
- Tauri 命令封装：`src/lib/tauri-commands.ts`
- Tauri 环境判断：`src/lib/tauri-env.ts`
- 浏览器预览 mock：`src/lib/mock-tauri.ts`
- 配置工具：`src/lib/config.ts`
- 按键映射：`src/lib/keys.ts`、`src/lib/browser-keys.ts`
- 职业数据：`src/data/classes.ts`
- 悬浮窗事件常量与类型：`src/lib/floating-control.ts`
- 悬浮控制视图：`src/floating-control/floating-control-view.tsx`
- 悬浮控制管理器：`src/floating-control/floating-control-manager.ts`

前端根据 URL 参数选择视图：

- 默认视图渲染主界面。
- `?view=floating-control` 渲染悬浮窗。

浏览器预览模式通过 `isMockMode()` 判断 Tauri API 是否存在，并在 `src/lib/tauri.ts` 中返回 mock 数据。

当前主界面包括按键连发、一键连招、配置管理、设置、关于。一键连招 v1 按职业配置或自定义配置维护，支持快捷栏按键动作和顺序手搓动作；不支持全局连招、同时按键、长按组合、步骤循环或排队执行。

配置抽象上，职业配置应被视为项目预设好 id 和名称、不可删除的自定义配置；除显示/隐藏和删除权限外，连发、连招、生效规则、运行时选择等功能逻辑应尽量保持一致。

## 后端结构

- `src-tauri/src/lib.rs`：Tauri 应用装配、插件注册、窗口启动行为和命令注册。
- `src-tauri/src/commands.rs`：前端可调用的 Tauri 命令入口。
- `src-tauri/src/config.rs`：配置数据结构、默认配置、读写、迁移和校验。
- `src-tauri/src/state.rs`：应用共享状态和配置保存入口。
- `src-tauri/src/tray.rs`：系统托盘、托盘菜单和悬浮窗菜单文案同步。
- `src-tauri/src/hotkey.rs`：全局启动/停止快捷键注册与触发逻辑。
- `src-tauri/src/startup.rs`：Windows 开机自启动注册表逻辑。
- 核心模块位于 `src-tauri/src/core/`：
  - `autofire.rs`：连发引擎，负责键盘钩子、运行状态、按键循环。
  - `combo.rs`：一键连招引擎，负责触发键钩子、触发键拦截和动作序列执行。
  - `keyboard.rs`：Win32 `SendInput` 封装。
  - `window.rs`：DNF/记事本前台窗口检测。

后端使用 `Arc<Mutex<...>>` 管理运行状态、配置、运行时按键和快捷键注册。

## 配置系统

配置文件位于 `{exe_dir}/configs/app-config.json`，当前 schema 版本为 `6`。主要字段包括：

- `globalKeys`：全局连发按键。
- `comboDefs`：根级兼容字段，当前保持为空，不参与前端编辑或运行时下发。
- `classes`：预设职业配置，职业 id 和名称来自 `src/data/classes.ts`，不可删除。
- `customConfigs`：用户自定义配置，独立保存名称、连发按键、生效规则和一键连招定义。
- `hiddenClassIds`：被用户隐藏的未配置职业入口；已有配置职业始终显示。
- `activeClassId`：当前生效配置 id，可指向职业配置或自定义配置。
- `toggleHotkey`：全局启动/停止快捷键，必须是组合键。
- `detection`：职业识别相关设置，当前结构保留。
- `settings.launchAtStartup`：开机时启动。
- `settings.startMinimized`：启动时最小化或隐藏到托盘。
- `settings.minimizeToTray`：最小化按钮和启动时最小化是否隐藏到托盘。
- `settings.openFloatingControlOnStart`：启动时自动打开悬浮窗。
- `settings.logLevel`：后端日志等级，支持 `trace` / `debug` / `info` / `warn` / `error` / `off`。

## Tauri 命令

前端通过 `src/lib/tauri.ts` 导出的 `tauriCommands.*` 调用后端；实际命令实现分布在 `src/lib/tauri-commands.ts`、`src/lib/tauri-env.ts` 和 `src/lib/mock-tauri.ts`：

- `load_app_config` / `save_app_config`：配置持久化。
- `set_runtime_profile`：同步当前生效的连发键和职业连招运行时快照。
- `start_assistant` / `stop_assistant` / `is_assistant_running`：统一控制连发和一键连招。
- `start_autofire` / `stop_autofire` / `is_running`：旧连发引擎控制入口，保留兼容。
- `set_runtime_keys`：运行时更新按键配置。
- `register_toggle_hotkey`：注册全局启动/停止快捷键。
- `update_tray_current_config`：更新托盘中展示的当前配置。
- `is_elevated` / `restart_as_admin`：管理员权限检测与提权重启。
- `set_launch_at_startup`：写入或移除 Windows 当前用户 Run 注册表项。

## 托盘与悬浮窗链路

悬浮窗是全局唯一的 `floating-control` 窗口，由前端 `src/lib/tauri.ts` 创建和关闭，入口为 `/?view=floating-control`。不要再新增 Rust 侧同名窗口创建链路。

托盘菜单中的“打开悬浮窗 / 关闭悬浮窗”不直接操作窗口，而是：

1. Rust 托盘菜单触发 `floating-control:toggle-request`。
2. 主窗口前端监听该事件并更新唯一的 `floatingControlEnabled` 状态。
3. 前端根据状态调用 `showFloatingControlWindow()` 或 `hideFloatingControlWindow()`。
4. 前端发送 `floating-control:visibility-changed`。
5. Rust 监听可见性事件并更新托盘菜单文字。

这样主窗口开关、托盘菜单和实际悬浮窗保持同一条状态链路。

## VK 码系统

项目统一使用 Windows Virtual Key 码（`u16` / number）表示键盘按键。完整映射在 `src/lib/keys.ts`，浏览器 `KeyboardEvent.code` 到 VK 码的转换在 `src/lib/browser-keys.ts`。

## 文档维护

- 已废弃的 Native Windows GUI（NWG）方案不再属于当前项目架构，不要新增相关实现文档。
- 开发流程以 `package.json` 中的 `pnpm` 脚本为准，不再维护旧的 `scripts/*.ps1` 或 `cargo make` 文档。
- 如果新增长期有效的架构说明，先更新 `docs/README.md` 索引。

## 运行要求

- Windows。
- 依赖 Win32 API，不支持跨平台运行。
- 键盘钩子和输入模拟通常需要管理员权限。
