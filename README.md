# Issue Jumper

[中文文档](README.zh-CN.md)

Issue Jumper resolves an issue URL from the current Git branch and opens it in the system browser. It is a convention-first CLI with a Zed installer for one-key navigation from an editor workspace.

## Features

- Resolves common GitHub, GitLab, private GitLab, Bitbucket, and Gitee remotes.
- Extracts issue IDs from branch names such as `feature/GH-123`, `fix/issue-456`, `101-add-login`, and `feature/ABC-456-login`.
- Supports Redmine, Jira, GitLab work items, and custom URL templates through project config.
- Installs a Zed task and keymap binding with `issue-jumper install-zed`.
- Provides `open`, `url`, and `doctor` commands for editor, terminal, and script usage.

## Installation

Install the latest release and configure Zed:

```sh
wget -qO- https://raw.githubusercontent.com/weirdo-adam/issue-jumper/main/scripts/install.sh | sh
```

The installer detects the host platform, downloads the matching archive from [GitHub Releases](https://github.com/weirdo-adam/issue-jumper/releases), installs `issue-jumper` to `~/.local/bin`, and runs `issue-jumper install-zed`.

Install with options:

```sh
wget -qO- https://raw.githubusercontent.com/weirdo-adam/issue-jumper/main/scripts/install.sh | sh -s -- --key cmd-alt-j
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
| Default key | `alt-j` (`Option+J` on macOS) |
| Manual entry | Command Palette -> `task: spawn` -> `Issue Jumper: Open Current Issue` |

Options:

```sh
issue-jumper install-zed --key cmd-alt-j
issue-jumper install-zed --force
issue-jumper install-zed --print
```

The installer writes the absolute CLI path into the Zed task to avoid shell `PATH` differences between Zed and an interactive terminal.

## Configuration

Configuration is optional. Issue Jumper loads the first matching file:

1. `<repo>/.zed/issue-jumper.json`
2. `<repo>/.issue-jumper.json`

Example Redmine override:

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

Release artifacts are built locally, not by a GitHub Actions release job.

## Documentation

- [Technical design](docs/design.md)
- [Development guide](docs/development.md)

## License

MIT
