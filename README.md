<p align="center">
  <img src="assets/readme-banner.svg" alt="Issue Jumper - Jump from a Git branch to its issue page" width="100%">
</p>

<h1 align="center">Issue Jumper</h1>

<p align="center">
  Convention-first issue navigation for Git branches, terminals, and Zed workspaces.
</p>

<p align="center">
  <a href="README.zh-CN.md">中文文档</a>
  ·
  <a href="docs/technical-design.md">Technical design</a>
  ·
  <a href="docs/development.md">Development guide</a>
</p>

<p align="center">
  <a href="LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-111111?style=flat-square"></a>
  <img alt="Language: Rust" src="https://img.shields.io/badge/Rust-CLI-4A4A4A?style=flat-square">
  <img alt="Editor: Zed" src="https://img.shields.io/badge/Zed-alt--alt-6F6F6F?style=flat-square">
  <img alt="Privacy: local only" src="https://img.shields.io/badge/privacy-local%20only-2F2F2F?style=flat-square">
  <a href="https://github.com/weirdo-adam/issue-jumper/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/weirdo-adam/issue-jumper/actions/workflows/ci.yml/badge.svg"></a>
  <a href="https://github.com/weirdo-adam/issue-jumper/actions/workflows/release.yml"><img alt="Release" src="https://github.com/weirdo-adam/issue-jumper/actions/workflows/release.yml/badge.svg"></a>
</p>

Issue Jumper resolves an issue URL from the current Git branch and opens it in the system browser. It is a convention-first CLI with a Zed installer for one-key navigation from an editor workspace.

Double-tap Option/Alt in Zed, or run `issue-jumper open`, to jump from a branch such as `feature/GH-123-add-login` to the matching issue page.

## Quick Start

Install the latest release and configure the default Zed shortcut:

```sh
wget -qO- https://raw.githubusercontent.com/weirdo-adam/issue-jumper/main/scripts/install.sh | sh
```

Then use one of the built-in entry points:

```sh
issue-jumper open --repo /path/to/repo
issue-jumper url --repo /path/to/repo --print-url
issue-jumper doctor --repo /path/to/repo
```

## Features

- Resolves common GitHub, GitLab, private GitLab, Bitbucket, and Gitee remotes.
- Extracts issue IDs from branch names such as `feature/GH-123`, `fix/issue-456`, `101-add-login`, and `feature/ABC-456-login`.
- Opens the GitHub or GitLab repository page when the branch has no recognizable issue ID.
- Supports Redmine, Jira, GitLab work items, and custom URL templates through global or project config.
- Installs a Zed task and keymap binding with `issue-jumper install-zed`.
- Provides `open`, `url`, and `doctor` commands for editor, terminal, and script usage.
- Runs locally without telemetry or customer data collection.

## Privacy and Offline Use

Issue Jumper performs branch parsing, config loading, remote parsing, and URL generation on the local machine. It does not collect telemetry and does not upload branch names, repository paths, Git remotes, config values, or issue IDs.

Core commands such as `url` and `doctor` work offline once the binary is installed. Network access is only needed for the install script to download release assets, and for the browser or external issue tracker when opening the generated URL.

## Installation

On macOS, install with Homebrew:

```sh
brew install weirdo-adam/tap/issue-jumper
issue-jumper install-zed --force
```

Homebrew installs the CLI only. Run `issue-jumper install-zed --force` after installation when you want the Zed task and keymap integration. This keeps CLI upgrades under Homebrew while letting the CLI write the Zed task/keymap files explicitly.

If you do not use Homebrew, install the latest release and configure Zed with the one-command installer:

```sh
wget -qO- https://raw.githubusercontent.com/weirdo-adam/issue-jumper/main/scripts/install.sh | sh
```

The installer downloads the supported Unix host archive from [GitHub Releases](https://github.com/weirdo-adam/issue-jumper/releases), installs `issue-jumper` to `~/.local/bin`, and runs `issue-jumper install-zed --force`.

Repeated runs replace the existing `issue-jumper` binary and refresh the Zed task/keymap binding. Pass `--no-force` if key conflicts should fail instead of being replaced.

Avoid mixing install sources when possible. If Homebrew and `~/.local/bin/issue-jumper` both exist, the first directory in `PATH` wins in the terminal, while Zed uses the absolute command path written by the last `install-zed` run. To move an existing setup to Homebrew, remove or de-prioritize the `~/.local/bin` copy, then run:

```sh
/opt/homebrew/bin/issue-jumper install-zed --force
```

GitHub Releases publish prebuilt archives for Apple Silicon macOS, Linux x64, and Windows x64. The shell installer supports Apple Silicon macOS and Linux x64. Windows users can download the `.zip` asset and place `issue-jumper.exe` on `PATH`.

Install with options:

```sh
wget -qO- https://raw.githubusercontent.com/weirdo-adam/issue-jumper/main/scripts/install.sh | sh -s -- --key ctrl-shift-j
wget -qO- https://raw.githubusercontent.com/weirdo-adam/issue-jumper/main/scripts/install.sh | sh -s -- --no-force
wget -qO- https://raw.githubusercontent.com/weirdo-adam/issue-jumper/main/scripts/install.sh | sh -s -- --no-zed
wget -qO- https://raw.githubusercontent.com/weirdo-adam/issue-jumper/main/scripts/install.sh | sh -s -- --version v0.1.0 --install-dir ~/.local/bin
```

For local development, build from source and install the Zed integration:

```sh
scripts/install-zed.sh
```

## Zed Integration

`issue-jumper install-zed` writes or updates the global Zed `tasks.json` and `keymap.json` files.

| Entry | Value |
| --- | --- |
| Task label | `Issue Jumper: Open Current Issue` |
| Task command | `issue-jumper open --repo $ZED_WORKTREE_ROOT` |
| Default Zed keymap entry | `alt alt` |
| Manual entry | Command Palette -> `task: spawn` -> `Issue Jumper: Open Current Issue` |

The default is documented as the key sequence written to Zed `keymap.json`; `alt alt` means pressing and releasing Option/Alt twice. Use `--key <key>` to select a binding that matches your platform and keyboard layout.

Options:

```sh
issue-jumper install-zed --key ctrl-shift-j
issue-jumper install-zed --force
issue-jumper install-zed --print
```

`scripts/install.sh` and `scripts/install-zed.sh` pass `--force` by default for repeatable installs. Direct `issue-jumper install-zed` keeps key conflicts explicit unless `--force` is passed.

The installer writes the absolute CLI path into the Zed task to avoid shell `PATH` differences between Zed and an interactive terminal.

## Configuration

Configuration is optional. Issue Jumper loads global config first, then overlays the first matching project config:

1. `$XDG_CONFIG_HOME/issue-jumper/config.json` or `~/.config/issue-jumper/config.json`
2. `<repo>/.zed/issue-jumper.json`
3. `<repo>/.issue-jumper.json`

On Windows, the global path is `%APPDATA%\issue-jumper\config.json`.

Example Redmine override:

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

Config files are strict JSON. Unknown fields are rejected.

## CLI

```sh
issue-jumper open [--repo <path>] [--platform <name>] [--rule <name>]
issue-jumper url [--repo <path>] [--platform <name>] [--rule <name>] [--print-url]
issue-jumper install-zed [--key <key>] [--force] [--print]
issue-jumper doctor [--repo <path>]
```

Development examples:

```sh
cargo run -- url --repo /path/to/repo --print-url
cargo run -- open --repo /path/to/repo
cargo run -- doctor --repo /path/to/repo
```

## Development

Run the standard local gate:

```sh
make check
```

Useful commands:

```sh
make fmt
make lint
make test
make coverage
```

Validate the remote installer script:

```sh
sh -n scripts/install.sh
```

Build a local release archive:

```sh
scripts/package-release.sh --version v0.1.0
```

Publish a local release artifact:

```sh
scripts/publish-release.sh v0.1.0
```

Release artifacts are normally built by GitHub Actions when a `v*` tag is pushed. A release can also be rebuilt manually from the Actions tab by running the `Release` workflow with a tag such as `v0.1.0`.

## Documentation

- [Technical design](docs/technical-design.md)
- [Development guide](docs/development.md)

## License

MIT
