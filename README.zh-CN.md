# Issue Jumper

[English README](README.md)

Issue Jumper 用于从当前 Git 分支解析 Issue URL，并使用系统默认浏览器打开。项目采用约定优先的 CLI 设计，并提供 Zed 安装器，用于在编辑器工作区中通过一个快捷键跳转到当前分支对应的 Issue。

## 功能

- 识别常见 GitHub、GitLab、私有 GitLab、Bitbucket 和 Gitee remote。
- 从 `feature/GH-123`、`fix/issue-456`、`101-add-login`、`feature/ABC-456-login` 等分支名提取 Issue ID。
- 通过项目配置支持 Redmine、Jira、GitLab work items 和自定义 URL 模板。
- 通过 `issue-jumper install-zed` 安装 Zed task 和 keymap 绑定。
- 提供 `open`、`url`、`doctor` 命令，适用于编辑器、终端和脚本场景。

## 安装

安装最新 Release 并配置 Zed：

```sh
wget -qO- https://raw.githubusercontent.com/weirdo-adam/issue-jumper/main/scripts/install.sh | sh
```

安装器会识别当前平台，从 [GitHub Releases](https://github.com/weirdo-adam/issue-jumper/releases) 下载匹配的压缩包，将 `issue-jumper` 安装到 `~/.local/bin`，并执行 `issue-jumper install-zed`。

带参数安装：

```sh
wget -qO- https://raw.githubusercontent.com/weirdo-adam/issue-jumper/main/scripts/install.sh | sh -s -- --key cmd-alt-j
wget -qO- https://raw.githubusercontent.com/weirdo-adam/issue-jumper/main/scripts/install.sh | sh -s -- --no-zed
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
| 默认快捷键 | `alt-j`，macOS 对应 `Option+J` |
| 手动入口 | Command Palette -> `task: spawn` -> `Issue Jumper: Open Current Issue` |

选项：

```sh
issue-jumper install-zed --key cmd-alt-j
issue-jumper install-zed --force
issue-jumper install-zed --print
```

安装器会把 CLI 绝对路径写入 Zed task，避免 Zed task 环境与交互式终端的 `PATH` 差异。

## 配置

配置是可选的。Issue Jumper 按顺序读取第一个存在的文件：

1. `<repo>/.zed/issue-jumper.json`
2. `<repo>/.issue-jumper.json`

Redmine 覆盖示例：

```json
{
  "fallback_platform": "redmine",
  "redmine_base_url": "https://redmine.example.com",
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
```

开发调试示例：

```sh
cargo run -- url --repo /path/to/repo --print-url
cargo run -- open --repo /path/to/repo
cargo run -- doctor --repo /path/to/repo
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

Release 产物由本地构建，不通过 GitHub Actions release job 生成。

## 文档

- [技术设计](docs/design.md)
- [开发指南](docs/development.md)

## 许可证

MIT
