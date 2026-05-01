# 贡献指南

感谢你愿意改进 DNF按键助手。这个仓库目前以源码公开、学习和个人使用为目标，贡献前请先阅读 `README.md`、`LICENSE.md`、`AGENTS.md` 和 `docs/README.md`。

## 开发流程

```bash
pnpm install
pnpm dev:web
pnpm check:all
```

- 只验证 React 界面时使用 `pnpm dev:web`。
- 涉及 Win32 钩子、输入模拟、托盘、开机启动和 Tauri 窗口时，需要在 Windows Tauri 应用里验证。
- 修改中文文档或源码后运行 `pnpm encoding:check`，避免编码损坏。

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

- 保持改动聚焦，一个 PR 解决一个主题。
- 说明改动目的、影响范围和已运行的检查命令。
- 不要把功能改动、格式化、依赖升级混在同一个 PR 中。
- 不要提交个人本地设置、构建产物、日志。

## 历史整理建议

如果需要公开一个干净远程仓库，推荐新建公开仓库并以当前文件树创建一次 `initial commit`。本地开发仓库可以继续保留完整历史；不要对已有多人协作远程仓库直接 force push 改写历史。
