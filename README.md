<p align="center">
  <img src="assets/readme-banner.svg" alt="Issue Jumper - Jump from a Git branch to its issue page" width="100%">
</p>

<h1 align="center">Issue Jumper</h1>

<p align="center">
  Convention-first issue navigation for Git branches, terminals, and editor workspaces.
</p>

<p align="center">
  <a href="README.zh-CN.md">中文</a>
  ·
  <a href="docs/usage.md">Usage</a>
  ·
  <a href="docs/architecture.md">Architecture</a>
  ·
  <a href="docs/development.md">Development</a>
</p>

<p align="center">
  <a href="LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-111111?style=flat-square"></a>
  <img alt="Language: Rust" src="https://img.shields.io/badge/Rust-CLI-4A4A4A?style=flat-square">
  <img alt="Editor: Zed" src="https://img.shields.io/badge/Zed-alt--alt-6F6F6F?style=flat-square">
  <img alt="Privacy: local only" src="https://img.shields.io/badge/privacy-local%20only-2F2F2F?style=flat-square">
  <a href="https://github.com/weirdo-adam/issue-jumper/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/weirdo-adam/issue-jumper/actions/workflows/ci.yml/badge.svg"></a>
  <a href="https://github.com/weirdo-adam/issue-jumper/actions/workflows/release.yml"><img alt="Release" src="https://github.com/weirdo-adam/issue-jumper/actions/workflows/release.yml/badge.svg"></a>
</p>

Issue Jumper resolves an issue URL from the current Git branch and opens it in the system browser.
It works as a local CLI and can install a Zed task/keymap so double-tapping Option/Alt opens the
current branch's issue from the editor.

## Quick Start

Install with Homebrew:

```sh
brew install weirdo-adam/tap/issue-jumper
issue-jumper install-zed --force
```

Or install the latest release with the shell installer:

```sh
wget -qO- https://raw.githubusercontent.com/weirdo-adam/issue-jumper/main/scripts/install.sh | sh
```

Run from a repository:

```sh
issue-jumper open --repo /path/to/repo
issue-jumper url --repo /path/to/repo --print-url
issue-jumper doctor --repo /path/to/repo
```

## Features

- Detects GitHub, GitLab, private GitLab, Bitbucket, Gitee, Redmine, and custom trackers.
- Extracts issue IDs from common branch names such as `feature/GH-123` and `feature/ABC-456`.
- Falls back to the GitHub/GitLab repository homepage when a branch has no issue ID.
- Supports global and per-project JSON config, including custom rules and URL templates.
- Provides Zed, VS Code, Cursor, and generic editor integration examples.
- Runs locally without telemetry or customer data collection.

## Documentation

| Document | Purpose |
| --- | --- |
| [Usage](docs/usage.md) | Installation, CLI, editor integrations, and config examples |
| [Architecture](docs/architecture.md) | Runtime flow, module map, and architecture diagram |
| [Technical design](docs/technical-design.md) | Design constraints and implementation decisions |
| [Development](docs/development.md) | Local checks, coding rules, release workflow, and Homebrew notes |
| [Integrations](docs/integrations.md) | VS Code, Cursor, and generic editor snippets |

## Privacy

Branch parsing, config loading, remote parsing, and URL generation happen locally. Issue Jumper does
not collect telemetry and does not upload branch names, repository paths, Git remotes, config values,
or issue IDs.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) and run `make check` before opening a pull request.

## License

MIT
