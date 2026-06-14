<p align="center">
  <img src="assets/readme-banner.svg" alt="Issue Jumper - Jump from a Git branch to its issue page" width="100%">
</p>

<h1 align="center">Issue Jumper</h1>

<p align="center">
  面向 Git 分支、终端和编辑器工作区的约定优先 Issue 跳转工具。
</p>

<p align="center">
  <a href="README.md">English</a>
  ·
  <a href="docs/usage.md">使用文档</a>
  ·
  <a href="docs/architecture.md">架构</a>
  ·
  <a href="docs/development.md">开发</a>
</p>

<p align="center">
  <a href="LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-111111?style=flat-square"></a>
  <img alt="Language: Rust" src="https://img.shields.io/badge/Rust-CLI-4A4A4A?style=flat-square">
  <img alt="Editor: Zed" src="https://img.shields.io/badge/Zed-alt--alt-6F6F6F?style=flat-square">
  <img alt="Privacy: local only" src="https://img.shields.io/badge/privacy-local%20only-2F2F2F?style=flat-square">
  <a href="https://github.com/weirdo-adam/issue-jumper/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/weirdo-adam/issue-jumper/actions/workflows/ci.yml/badge.svg"></a>
  <a href="https://github.com/weirdo-adam/issue-jumper/actions/workflows/release.yml"><img alt="Release" src="https://github.com/weirdo-adam/issue-jumper/actions/workflows/release.yml/badge.svg"></a>
</p>

Issue Jumper 会从当前 Git 分支解析 Issue URL，并用系统默认浏览器打开。它既可以作为本地
CLI 使用，也可以安装 Zed task/keymap，让用户在编辑器中双击 Option/Alt 打开当前分支对应
的 Issue。

## 快速开始

使用 Homebrew 安装：

```sh
brew install weirdo-adam/tap/issue-jumper
issue-jumper install-zed --force
```

或使用 Release 安装脚本：

```sh
wget -qO- https://raw.githubusercontent.com/weirdo-adam/issue-jumper/main/scripts/install.sh | sh
```

在仓库中运行：

```sh
issue-jumper open --repo /path/to/repo
issue-jumper url --repo /path/to/repo --print-url
issue-jumper doctor --repo /path/to/repo
```

## 功能

- 识别 GitHub、GitLab、私有 GitLab、Bitbucket、Gitee、Redmine 和自定义平台。
- 从 `feature/GH-123`、`feature/ABC-456` 等常见分支名中提取 Issue ID。
- 分支没有 Issue ID 时，GitHub/GitLab 仓库默认打开项目首页。
- 支持全局和项目级 JSON 配置，包括自定义规则和 URL 模板。
- 提供 Zed、VS Code、Cursor 和通用编辑器集成示例。
- 核心解析在本机完成，不包含遥测。

## 文档

| 文档 | 用途 |
| --- | --- |
| [使用文档](docs/usage.md) | 安装、CLI、编辑器集成和配置示例 |
| [架构](docs/architecture.md) | 运行链路、模块职责和架构图 |
| [技术设计](docs/technical-design.md) | 设计约束和实现决策 |
| [开发指南](docs/development.md) | 本地检查、代码规范、发布流程和 Homebrew 说明 |
| [集成示例](docs/integrations.md) | VS Code、Cursor 和通用编辑器片段 |

## 隐私

分支解析、配置读取、remote 解析和 URL 构建均在本机完成。Issue Jumper 不采集遥测，也不
上传分支名、仓库路径、Git remote、配置值或 Issue ID。

## 贡献

提交 PR 前请阅读 [CONTRIBUTING.md](CONTRIBUTING.md)，并执行 `make check`。

## 许可证

MIT
