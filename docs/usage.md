# Usage

This guide covers installation, CLI usage, editor integrations, and configuration.

## Installation

### Homebrew

```sh
brew install weirdo-adam/tap/issue-jumper
issue-jumper install-zed --force
```

Homebrew installs the CLI only. Run `issue-jumper install-zed --force` when you want Zed task and
keymap integration. This keeps CLI upgrades under Homebrew while letting the CLI explicitly write
editor configuration.

If `issue-jumper` is not found after a Homebrew install, add Homebrew to your shell `PATH`:

```sh
eval "$(/opt/homebrew/bin/brew shellenv)"
```

### Shell Installer

```sh
wget -qO- https://raw.githubusercontent.com/weirdo-adam/issue-jumper/main/scripts/install.sh | sh
```

The installer downloads the supported Unix host archive from GitHub Releases, installs
`issue-jumper` to `~/.local/bin`, and runs `issue-jumper install-zed --force`.

Useful installer options:

```sh
wget -qO- https://raw.githubusercontent.com/weirdo-adam/issue-jumper/main/scripts/install.sh | sh -s -- --key ctrl-shift-j
wget -qO- https://raw.githubusercontent.com/weirdo-adam/issue-jumper/main/scripts/install.sh | sh -s -- --no-force
wget -qO- https://raw.githubusercontent.com/weirdo-adam/issue-jumper/main/scripts/install.sh | sh -s -- --no-zed
wget -qO- https://raw.githubusercontent.com/weirdo-adam/issue-jumper/main/scripts/install.sh | sh -s -- --version v0.1.2 --install-dir ~/.local/bin
```

Uninstall the copy installed by the shell installer:

```sh
wget -qO- https://raw.githubusercontent.com/weirdo-adam/issue-jumper/main/scripts/install.sh | sh -s -- --uninstall
```

The uninstall path verifies that the target looks like Issue Jumper before deleting it. Use
`--force-uninstall` only when you intentionally want to remove an unknown same-name file.

### Windows

GitHub Releases publish a Windows x64 `.zip` asset. Download it, extract `issue-jumper.exe`, and
place it on `PATH`.

## CLI

```sh
issue-jumper open [--repo <path>] [--platform <name>] [--rule <name>]
issue-jumper url [--repo <path>] [--platform <name>] [--rule <name>] [--print-url]
issue-jumper install-zed [--key <key>] [--force] [--print]
issue-jumper doctor [--repo <path>]
issue-jumper config lint [--repo <path>] [--path <file>]
issue-jumper integration [print] [--target vscode|cursor|generic|all] [--command <path>]
```

Common commands:

```sh
issue-jumper open --repo /path/to/repo
issue-jumper url --repo /path/to/repo --print-url
issue-jumper doctor --repo /path/to/repo
issue-jumper config lint --repo /path/to/repo
```

## Zed

Install or refresh the Zed task and keymap:

```sh
issue-jumper install-zed --force
```

Default Zed entries:

| Entry | Value |
| --- | --- |
| Task label | `Issue Jumper: Open Current Issue` |
| Task command | `issue-jumper open --repo $ZED_WORKTREE_ROOT` |
| Default keymap | `alt alt` |
| Manual entry | Command Palette -> `task: spawn` -> `Issue Jumper: Open Current Issue` |

`alt alt` means pressing and releasing Option/Alt twice. Use `--key <key>` when your platform or
keyboard layout needs a different binding:

```sh
issue-jumper install-zed --key ctrl-shift-j
issue-jumper install-zed --print
```

The installer writes the absolute CLI path into the Zed task so Zed does not depend on the same
`PATH` as your interactive shell.

## Other Editors

Print ready-to-copy snippets for VS Code, Cursor, and generic launchers:

```sh
issue-jumper integration print --target vscode
issue-jumper integration print --target cursor
issue-jumper integration print --target generic
```

`print` is optional:

```sh
issue-jumper integration --target vscode
```

Use `--command <path>` when the editor cannot see the same `PATH` as your shell:

```sh
issue-jumper integration print --target vscode --command /opt/homebrew/bin/issue-jumper
```

See [Integrations](integrations.md) for full snippets.

## Configuration

Configuration is optional. Issue Jumper loads global config first, then overlays the first matching
project config:

1. `$XDG_CONFIG_HOME/issue-jumper/config.json` or `~/.config/issue-jumper/config.json`
2. `<repo>/.zed/issue-jumper.json`
3. `<repo>/.issue-jumper.json`

On Windows, the global path is `%APPDATA%\issue-jumper\config.json`.

Set `"clear_inherited_config": true` in a project config when that project should ignore inherited
global config before applying its own fields.

Example Redmine/Jira-style override:

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
  ],
  "custom_platforms": [
    {
      "name": "jira",
      "host_patterns": ["jira.example.com"],
      "url_template": "https://jira.example.com/browse/{id}"
    }
  ]
}
```

Validate config files:

```sh
issue-jumper config lint
issue-jumper config lint --repo /path/to/repo
issue-jumper config lint --path /path/to/issue-jumper.json
```

Config files are strict JSON. Unknown fields, invalid regexes, unknown platform references, and
unsupported URL template placeholders are rejected.

## Mixed Install Sources

Avoid mixing Homebrew and shell-installer copies. If both Homebrew and `~/.local/bin/issue-jumper`
exist, the first directory in `PATH` wins in the terminal, while Zed uses the absolute command path
written by the last `install-zed` run.

To move an existing shell-installed setup to Homebrew:

```sh
wget -qO- https://raw.githubusercontent.com/weirdo-adam/issue-jumper/main/scripts/install.sh | sh -s -- --uninstall
/opt/homebrew/bin/issue-jumper install-zed --force
```
