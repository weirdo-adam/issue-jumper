<p align="center">
  <img src="assets/readme-banner.svg" alt="Issue Jumper - 从 Git 分支跳转到对应 Issue 页面" width="100%">
</p>

<h1 align="center">Issue Jumper</h1>

<p align="center">
  面向 Git 分支、终端和 Zed 工作区的约定优先 Issue 跳转工具。
</p>

<p align="center">
  <a href="README.md">English README</a>
  ·
  <a href="docs/technical-design.md">技术设计</a>
  ·
  <a href="docs/development.md">开发指南</a>
</p>

<p align="center">
  <a href="LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-111111?style=flat-square"></a>
  <img alt="Language: Rust" src="https://img.shields.io/badge/Rust-CLI-4A4A4A?style=flat-square">
  <img alt="Editor: Zed" src="https://img.shields.io/badge/Zed-alt--alt-6F6F6F?style=flat-square">
  <img alt="Privacy: local only" src="https://img.shields.io/badge/privacy-local%20only-2F2F2F?style=flat-square">
  <a href="https://github.com/weirdo-adam/issue-jumper/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/weirdo-adam/issue-jumper/actions/workflows/ci.yml/badge.svg"></a>
  <a href="https://github.com/weirdo-adam/issue-jumper/actions/workflows/release.yml"><img alt="Release" src="https://github.com/weirdo-adam/issue-jumper/actions/workflows/release.yml/badge.svg"></a>
</p>

Issue Jumper 用于从当前 Git 分支解析 Issue URL，并使用系统默认浏览器打开。项目采用约定优先的 CLI 设计，并提供 Zed 安装器，用于在编辑器工作区中通过一个快捷键跳转到当前分支对应的 Issue。

在 Zed 中双击 Option/Alt，或在终端运行 `issue-jumper open`，即可从 `feature/GH-123-add-login` 这类分支跳转到对应 Issue 页面。

## 快速开始

安装最新 Release，并配置默认 Zed 快捷键：

```sh
wget -qO- https://raw.githubusercontent.com/weirdo-adam/issue-jumper/main/scripts/install.sh | sh
```

然后使用内置入口：

```sh
issue-jumper open --repo /path/to/repo
issue-jumper url --repo /path/to/repo --print-url
issue-jumper doctor --repo /path/to/repo
```

## 功能

- 识别常见 GitHub、GitLab、私有 GitLab、Bitbucket 和 Gitee remote。
- 从 `feature/GH-123`、`fix/issue-456`、`101-add-login`、`feature/ABC-456-login` 等分支名提取 Issue ID。
- 当 GitHub 或 GitLab 仓库的分支无法识别 Issue ID 时，默认打开远程仓库页面。
- 通过全局或项目配置支持 Redmine、Jira、GitLab work items 和自定义 URL 模板。
- 通过 `issue-jumper install-zed` 安装 Zed task 和 keymap 绑定。
- 提供 `open`、`url`、`doctor` 命令，适用于编辑器、终端和脚本场景。
- 本地运行，不包含遥测，不收集客户数据。

## 隐私与离线使用

Issue Jumper 在本机完成分支解析、配置读取、remote 解析和 URL 生成。项目不包含遥测，不上传分支名、仓库路径、Git remote、配置值或 Issue ID。

安装二进制后，`url`、`doctor` 等核心命令可离线使用。只有安装脚本下载 Release 资产，以及浏览器打开生成的远端 Issue 页面时，需要网络访问。

## 安装

使用 Homebrew 安装：

```sh
brew install weirdo-adam/tap/issue-jumper
issue-jumper install-zed --force
```

Homebrew 只安装 CLI。如需 Zed task 和 keymap 集成，安装后再执行 `issue-jumper install-zed --force`。

安装最新 Release 并配置 Zed：

```sh
wget -qO- https://raw.githubusercontent.com/weirdo-adam/issue-jumper/main/scripts/install.sh | sh
```

安装器会从 [GitHub Releases](https://github.com/weirdo-adam/issue-jumper/releases) 下载受支持 Unix 主机对应的压缩包，将 `issue-jumper` 安装到 `~/.local/bin`，并执行 `issue-jumper install-zed --force`。

重复执行安装命令会覆盖已有的 `issue-jumper` 二进制，并刷新 Zed task/keymap 绑定。如需在快捷键冲突时失败而不是覆盖，可传入 `--no-force`。如需删除该脚本安装的副本，传入 `--uninstall`；卸载时会先校验目标，只有确认要删除未知同名文件时才使用 `--force-uninstall`。

GitHub Releases 会发布 Apple Silicon macOS、Linux x64 和 Windows x64 的预构建压缩包。shell 安装器支持 Apple Silicon macOS 和 Linux x64。Windows 用户可下载 `.zip` 资产，并将 `issue-jumper.exe` 放到 `PATH` 中。

带参数安装：

```sh
wget -qO- https://raw.githubusercontent.com/weirdo-adam/issue-jumper/main/scripts/install.sh | sh -s -- --key ctrl-shift-j
wget -qO- https://raw.githubusercontent.com/weirdo-adam/issue-jumper/main/scripts/install.sh | sh -s -- --no-force
wget -qO- https://raw.githubusercontent.com/weirdo-adam/issue-jumper/main/scripts/install.sh | sh -s -- --no-zed
wget -qO- https://raw.githubusercontent.com/weirdo-adam/issue-jumper/main/scripts/install.sh | sh -s -- --uninstall
wget -qO- https://raw.githubusercontent.com/weirdo-adam/issue-jumper/main/scripts/install.sh | sh -s -- --version v0.1.0 --install-dir ~/.local/bin
```

本地开发时，从源码构建并安装 Zed 集成：

```sh
scripts/install-zed.sh
```

## Zed 集成

`issue-jumper install-zed` 会写入或更新全局 Zed `tasks.json` 和 `keymap.json`。

| 入口 | 值 |
| --- | --- |
| Task label | `Issue Jumper: Open Current Issue` |
| Task command | `issue-jumper open --repo $ZED_WORKTREE_ROOT` |
| 默认 Zed keymap 项 | `alt alt` |
| 手动入口 | Command Palette -> `task: spawn` -> `Issue Jumper: Open Current Issue` |

这里记录的是写入 Zed `keymap.json` 的按键序列；`alt alt` 表示连续按下并松开两次 Option/Alt。不同平台或键盘布局需要其他绑定时，使用 `--key <key>` 指定。

选项：

```sh
issue-jumper install-zed --key ctrl-shift-j
issue-jumper install-zed --force
issue-jumper install-zed --print
```

`scripts/install.sh` 和 `scripts/install-zed.sh` 默认传入 `--force`，用于支持可重复安装。直接执行 `issue-jumper install-zed` 时仍会保留快捷键冲突，需要显式传入 `--force` 才覆盖。

安装器会把 CLI 绝对路径写入 Zed task，避免 Zed task 环境与交互式终端的 `PATH` 差异。

## 配置

配置是可选的。Issue Jumper 会先读取全局配置，再叠加第一个存在的项目配置：

1. `$XDG_CONFIG_HOME/issue-jumper/config.json` 或 `~/.config/issue-jumper/config.json`
2. `<repo>/.zed/issue-jumper.json`
3. `<repo>/.issue-jumper.json`

Windows 全局路径为 `%APPDATA%\issue-jumper\config.json`。

当某个项目不希望继承全局配置时，可以在项目配置中设置 `"clear_inherited_config": true`，再声明该项目自己的字段。

校验当前仓库发现的所有配置文件：

```sh
issue-jumper config lint
issue-jumper config lint --repo /path/to/repo
issue-jumper config lint --path /path/to/issue-jumper.json
```

Redmine 覆盖示例：

```json
{
  "fallback_platform": "redmine",
  "redmine_base_url": "https://redmine.example.com",
  "disabled_rules": ["global-redmine-number"],
  "issue_rules": [
    {
      "name": "redmine-number",
      "pattern": "(?i)redmine[-_](?P<id>\\d+)",
      "platform": "redmine"
    }
  ]
}
```

配置文件使用严格 JSON，未知字段会报错。

## CLI

```sh
issue-jumper open [--repo <path>] [--platform <name>] [--rule <name>]
issue-jumper url [--repo <path>] [--platform <name>] [--rule <name>] [--print-url]
issue-jumper install-zed [--key <key>] [--force] [--print]
issue-jumper doctor [--repo <path>]
issue-jumper config lint [--repo <path>] [--path <file>]
issue-jumper integration [print] [--target vscode|cursor|generic|all] [--command <path>]
```

开发调试示例：

```sh
cargo run -- url --repo /path/to/repo --print-url
cargo run -- open --repo /path/to/repo
cargo run -- doctor --repo /path/to/repo
cargo run -- config lint --repo /path/to/repo
cargo run -- integration print --target vscode
```

## 开发

执行标准本地检查：

```sh
make check
```

常用命令：

```sh
make fmt
make lint
make test
make coverage
```

校验远程安装脚本语法：

```sh
sh -n scripts/install.sh
```

构建本地 Release 压缩包：

```sh
scripts/package-release.sh --version v0.1.0
```

发布本地 Release 产物：

```sh
scripts/publish-release.sh v0.1.0
```

Release 产物通常由 GitHub Actions 在推送 `v*` tag 时构建。也可以在 Actions 页面手动运行 `Release` workflow，并传入 `v0.1.0` 这类 tag 来重新构建。

## 文档

- [技术设计](docs/technical-design.md)
- [开发指南](docs/development.md)

## 许可证

MIT
