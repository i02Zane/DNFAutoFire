# 贡献指南

感谢你愿意改进 DNF按键助手。本仓库以源码公开、学习和个人使用为目标。开始前请阅读 `README.md`、`LICENSE.md`、`AGENTS.md`、`docs/README.md` 和 `docs/pitfalls.json`。

本文件同时约束人工贡献者和代码代理。若其他文档与本文件冲突，以 `AGENTS.md` 和本文件为准。

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

## 修改前

- 文档和注释使用中文，代码标识使用英文，文件名使用 kebab-case。
- 注释只解释状态边界、跨模块链路、并发/异步顺序和平台限制等不易从代码看出的原因。
- 优先复用现有模式，小步修改；不要混入无关重构、格式化、依赖升级或本地配置。
- 不读写 `node_modules/`、`src-tauri/gen/`、`src-tauri/target/`。
- 跨前后端契约改动必须同步类型、实现、校验、迁移、文档和测试。
- 变更优先在独立分支上完成，再合并到 `main`，保持主干提交整洁。

## 修改中

Windows PowerShell 5.1 默认读写文件可能产生旧编码或 UTF-16LE，容易损坏中文文本：

- 不要用 PowerShell 默认读取或写入源码、文档。
- 不要用 PowerShell 默认写文件命令修改源码或文档。
- 避免使用 `Set-Content`、`Out-File`、`Add-Content`、`>`、`>>` 写源码，除非显式指定并验证 UTF-8。
- 手工修改优先使用 `apply_patch`。
- 批量重写使用明确写入 UTF-8 的运行时，例如 Node.js `fs.writeFile(path, text, "utf8")`。

常见修改还要遵守：

- 只改本次需求相关内容，不顺手带入无关重构、格式化或依赖升级。
- 仓库开发流程以 `package.json` 中的 `pnpm` 脚本为准，不维护旧 `scripts/*.ps1` 或 `cargo make` 文档。
- 涉及 Win32 钩子、输入模拟、托盘、开机启动和 Tauri 窗口时，在 Windows Tauri 应用里验证。

## 修改后

- 修改中文文档或源码后运行 `pnpm encoding:check`。
- 前端改动运行 `pnpm typecheck`、`pnpm lint`、`pnpm format:check`、`pnpm encoding:check`。
- Rust 改动运行 `pnpm rust:check`、`pnpm rust:clippy`、`pnpm rust:fmt:check`、`pnpm rust:test`。
- 跨端契约或发布前检查运行 `pnpm check:all` 或 `pnpm run ci`。
- 如果无法运行必要检查，提交或 PR 说明原因和剩余风险。
- 每完成一个需求并通过必要校验后，默认单独创建一次 git 提交；除非明确要求暂不提交，不要把已完成改动留在未提交状态。

## 提交规范

提交信息使用 Conventional Commits：

- `feat:` 新功能
- `fix:` 修复问题
- `docs:` 文档
- `test:` 测试
- `refactor:` 不改变行为的重构
- `style:` UI 样式或格式调整
- `chore:` 维护性修改
- `ci:` CI 配置
- `build:` 构建系统

示例：

```text
docs: clarify source-available license
fix: keep floating control state in sync
ci: add Windows quality checks
```

## Pull Request 建议

- 一个 PR 解决一个主题。
- 说明改动目的、影响范围和已运行的检查命令。
- 不要提交构建产物、日志或个人本地设置。

## 版本发布

- 版本号变更时，必须同步更新 `package.json`、`src-tauri/Cargo.toml`、`src-tauri/tauri.conf.toml` 和 `src/app-meta.ts`。
- 发布 tag 统一使用 `v` 前缀格式，并且必须与实际版本号一致，例如 `v0.3.0` 对应 `0.3.0`。
- 同步补写 `CHANGELOG.md`。
- 更新后必须运行 `pnpm build`，确认构建通过，并检查 `src-tauri/Cargo.lock` 是否随之更新。
