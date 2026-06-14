# Technical Design

Issue Jumper is a local-first Rust CLI for turning the current Git branch into an issue URL. The
project keeps editor integrations thin so Zed, VS Code, Cursor, and generic launchers all reuse the
same branch parsing, config loading, remote parsing, and URL generation logic.

## Goals

- Open the correct issue URL from the current Git branch with no intermediate confirmation.
- Work from terminals, scripts, and editor tasks.
- Prefer conventions for GitHub/GitLab-style branches and remotes.
- Allow Redmine, Jira, GitLab work items, and internal trackers through JSON config.
- Keep all core parsing local and telemetry-free.
- Provide clear diagnostics through CLI errors and `doctor`.

## Non-Goals

- Do not call issue tracker APIs or require user tokens.
- Do not create issues.
- Do not depend on Zed Agent slash commands for the primary workflow.
- Do not register a custom Command Palette action until the editor exposes a stable public API for
  that path.

## Core Decisions

| Decision | Reason |
| --- | --- |
| Rust CLI as the core | Fast startup, simple distribution, explicit platform handling |
| Editor integrations call the CLI | Avoid duplicating branch, remote, config, and URL logic |
| Zed task/keymap integration | Public Zed tasks and keybindings support the required workflow |
| Absolute CLI path in Zed tasks | Avoid differences between Zed task `PATH` and interactive shell `PATH` |
| JSON config with `deny_unknown_fields` | Fail fast on misspelled or unsupported config keys |
| `config lint` command | Let users validate config without opening a browser |
| GitHub/GitLab repository fallback | Main branches without issue IDs should open the project homepage |
| No fallback when `--rule` is explicit | A user-selected rule should fail clearly when it does not match |

## Runtime Model

1. Resolve the repository from `--repo` or the current working directory.
2. Load config from global config, then the first project config.
3. Read the current branch from Git.
4. Match an issue ID with custom rules first, then built-in rules.
5. Read `origin` or `upstream` remote when needed.
6. Resolve the platform from command override, rule hint, remote, or fallback config.
7. Build a validated `http` or `https` URL.
8. Open the URL or print it, depending on the command.

See [Architecture](architecture.md) for diagrams and module responsibilities.

## Branch Rules

Built-in rules cover:

| Rule | Example | ID |
| --- | --- | --- |
| GitHub prefix | `feature/GH-123` | `123` |
| issue prefix | `fix/issue-456` | `456` |
| hash number | `bug/#789` | `789` |
| leading number | `101-add-login` | `101` |
| Jira-like key | `feature/ABC-456-login` | `ABC-456` |
| trailing number | `feature/login-789` | `789` |

Custom rules are regular expressions with a named `id` capture group. Config lint rejects custom
rules without `(?P<id>...)`.

## Platform Resolution

Platform precedence:

1. `--platform`
2. matched rule `platform`
3. parsed Git remote
4. `fallback_platform`

Built-in platforms are GitHub, GitLab, Bitbucket, Gitee, and Redmine. Custom platforms use
`custom_platforms[].url_template`.

## Configuration

Config paths:

1. `$XDG_CONFIG_HOME/issue-jumper/config.json` or `~/.config/issue-jumper/config.json`
2. `<repo>/.zed/issue-jumper.json`
3. `<repo>/.issue-jumper.json`

Windows global config path is `%APPDATA%\issue-jumper\config.json`.

Project config can set `clear_inherited_config` to ignore inherited global config before applying
project-specific fields.

See [Usage](usage.md#configuration) for config examples.

## Error Handling

Errors map to stable exit code groups:

| Code | Meaning |
| --- | --- |
| 0 | Success |
| 1 | Business error such as no matching issue or unresolved platform |
| 2 | Invalid config |
| 3 | Git or repository error |
| 4 | Browser opener failure |
| 5 | Zed config installation failure |

Errors are user-facing and should include enough context to fix the issue without debug logs.

## Release and Install Design

- GitHub Actions builds macOS Apple Silicon, Linux x64, and Windows x64 archives.
- Homebrew installs the CLI only and prints post-install guidance through formula caveats.
- The shell installer installs to `~/.local/bin` and can install or refresh Zed config.
- The uninstall path validates the target before deleting it, with `--force-uninstall` for explicit
  manual override.
- Release reruns refresh assets, generated release notes, source SHA-256, and the Homebrew bottle
  workflow dispatch.

## Quality Gates

Every change should pass:

```sh
make check
```

The check includes formatting, clippy with warnings denied, script syntax checks, file length
limits, and tests. See [Development](development.md) for the coding standard.
