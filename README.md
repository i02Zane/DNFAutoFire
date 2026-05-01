# DNF按键助手

DNF 按键连发工具，当前实现为 Tauri 2 + React 19 + Tailwind CSS 4 + Rust/Win32。

本仓库为源码公开项目，仅供学习、研究和个人使用，不是 OSI 意义上的开源许可项目。商业使用、收费分发、商业辅助服务或未经授权的重新许可均不被允许，详见 [LICENSE.md](./LICENSE.md)。

## 项目来源 / 贡献者

| 头像 | 名字 | 角色 |
| --- | --- | --- |
| <img src="https://github.com/mouyase.png?size=96" width="48" height="48" alt="mouyase" /> | [mouyase](https://github.com/mouyase) | 原项目作者 |
| <img src="https://github.com/i02Zane.png?size=96" width="48" height="48" alt="i02Zane" /> | [i02Zane](https://github.com/i02Zane) | 当前维护者 |

本项目基于[原项目dev分支](https://github.com/mouyase/DNFAutoFire/tree/dev)继续维护。

## 当前功能

- 按键连发：全局配置、职业配置矩阵、自定义配置、按键间隔、重复按键校验。
- 生效规则：支持“全局 + 当前配置”和“仅当前配置”。
- 启动控制：主界面按钮、全局启动/停止组合快捷键。
- 一键连招：按职业或自定义配置维护触发键、快捷栏动作和顺序手搓动作。
- 配置管理：职业配置是预设 id/name 且不可删除的配置，自定义配置可新增删除；两者运行逻辑保持一致。
- 悬浮窗：前端创建的唯一悬浮控制窗，可选择配置并启动/停止助手。
- 托盘：打开主界面、打开/关闭悬浮窗、显示当前配置、退出。
- 设置：开机时启动、启动到托盘、启动时自动打开悬浮窗。
- 浏览器预览（仅开发者）：`pnpm dev:web` 下自动进入 mock 模式。

## 技术栈

| 层级 | 技术 |
| --- | --- |
| 前端 | React 19、Vite 7、Tailwind CSS 4、lucide-react、Radix UI |
| 桌面壳 | Tauri 2 |
| 后端 | Rust、Win32 API、windows crate |
| 包管理 | pnpm |
| 检查 | TypeScript、oxlint、ESLint、oxfmt、Clippy、rustfmt |

## 常用命令

```bash
pnpm install
pnpm dev            # 详见 [CONTRIBUTING.md](./CONTRIBUTING.md)
pnpm dev:web
pnpm check          # 详细命令和校验要求见 [CONTRIBUTING.md](./CONTRIBUTING.md)
pnpm check:all
pnpm build:web
pnpm build
```

Rust 命令统一通过 `package.json` 脚本运行：

```bash
pnpm rust:check
pnpm rust:clippy
pnpm rust:fmt
pnpm rust:fmt:check
pnpm rust:test
pnpm rust:lint
```

## 项目结构

- `src/`：React 前端、配置工具、按键映射、悬浮窗页面。
- `src-tauri/`：Tauri 配置、Rust 命令、托盘、Win32 连发核心。
- `docs/`：当前仍适用的项目文档与踩坑记录。
- `scripts/check-encoding.mjs`：中文文本和编码安全检查。

## 运行要求

- Windows。
- 键盘钩子和输入模拟通常需要管理员权限。
- 仅前端预览可以在浏览器 mock 模式下运行，但悬浮窗和 Win32 连发能力只在 Tauri 应用中可用。

## 安装与构建

1. 安装 Node.js、pnpm 和 Rust 工具链。
2. 运行 `pnpm install` 安装前端依赖。
3. 运行 `pnpm dev:web` 预览前端 mock 模式，或运行 `pnpm dev` 启动 Tauri 开发模式。
4. 发布前运行 `pnpm check:all`。
5. 运行 `pnpm build` 打包桌面应用。

## 许可与声明

- 本项目源码按 [LICENSE.md](./LICENSE.md) 公开，仅供学习、研究和个人使用，禁止商业用途。
- 本项目不是游戏官方工具，使用风险由使用者自行承担。
- 本项目使用 Noto Sans SC 字体，字体遵循 [SIL Open Font License 1.1](https://scripts.sil.org/OFL)。
