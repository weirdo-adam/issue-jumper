# Development Guide

## Principles

Issue Jumper follows convention over configuration:

- Default branch rules should cover common GitHub, GitLab, Redmine-style, Jira-style, and numeric branch names.
- Private GitLab hosts such as `gitlab.example.com` should work without custom platform config.
- Project config is optional and reserved for overrides such as Redmine base URLs, Jira templates, or GitLab work item URLs.
- Zed integration uses tasks and keymaps; slash commands are not the target UI for v0.1.0.
- Keep the CLI core editor-neutral. Target-specific setup belongs in focused `install-*` commands such as `install-zed`.

## Project Structure

```text
src/
├── main.rs              # Binary entrypoint.
├── lib.rs               # Library module exports.
├── cli/                 # CLI command parsing and subcommands.
├── browser/             # System browser opener boundary.
├── git/                 # Git command wrapper and remote parser.
├── issue/               # Branch-to-issue matching rules.
├── url/                 # Issue URL construction.
├── zed/                 # Zed tasks/keymap installer adapter.
├── config.rs            # Optional project config loading.
├── error.rs             # Shared error and exit code mapping.
├── jump.rs              # Main workflow orchestration.
└── platform.rs          # Issue platform enum.
```

## Local Quality Gate

Use the repository defaults:

```sh
make fmt
make check
```

Equivalent raw commands:

```sh
cargo fmt --all
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
```

Production-code line coverage is expected to stay at 100%:

```sh
make coverage
```

The coverage target runs `cargo test --all-targets` with Rust coverage instrumentation and reports with `llvm-cov`. Test files and non-current platform adapters are excluded from the production-code coverage gate.

## Local Mock Data

Local mock data lives under `local/mock-data/`. The whole `local/` directory is ignored by git.

The current local sample file is:

```text
local/mock-data/issue-links.json
```

It records representative GitLab, Redmine, and GitHub issue URLs for manual validation. Automated tests must not depend on this ignored file.

## Zed Integration Notes

v0.1.0 installs a task named `Issue Jumper: Open Current Issue` and binds a user-selected key to `task::Spawn`.

Use the one-command installer during local setup:

```sh
scripts/install-zed.sh
```

Pass `--key <key>` to choose a different keybinding, `--force` to replace an existing binding, or `--print` to preview the Zed config snippets. The public one-command installer and local source installer run `install-zed --force` by default so repeated installs refresh the selected binding; pass `--no-force` to the installer when conflicts should be preserved.

The same task can be run from the Zed Command Palette by selecting `task: spawn` and then `Issue Jumper: Open Current Issue`. Current public Zed extensions do not expose an arbitrary custom action registration API for adding `issue: open` directly to the Command Palette. Zed slash commands are surfaced in the Agent UI, which is not the desired entry point for this project.

The task/keymap integration keeps the shortcut path explicit:

- No Agent or Assistant interaction.
- No shell string concatenation for repo paths.
- The CLI receives `$ZED_WORKTREE_ROOT` via task args.

The shortcut path stays on built-in Zed task/keymap primitives so the one-key browser opening path works without relying on unpublished Zed action APIs.

Future editor or launcher integrations should follow this structure: add a narrow installer command or printable snippet generator, then call the same `open`, `url`, and `doctor` workflow rather than duplicating branch parsing or URL construction.

Current non-Zed integrations are printable examples, not automatic installers:

- VS Code: `issue-jumper integration print --target vscode` emits task/keybinding snippets that run `issue-jumper open --repo ${workspaceFolder}`.
- Cursor: `issue-jumper integration print --target cursor` reuses the VS Code-compatible task/keybinding shape.
- Generic editors: `issue-jumper integration print --target generic` documents command examples for tools that can bind a shortcut to a shell command.

Keep these integrations as thin adapters around the CLI. Editor-specific code should write configuration or examples only; branch parsing, remote parsing, and URL generation stay in the shared core.

## Release Automation

GitHub Actions owns the normal release path. Pushing a `v*` tag runs `.github/workflows/release.yml`, builds the release matrix, and uploads assets to the matching GitHub Release.

Current release targets:

- `aarch64-apple-darwin`
- `x86_64-unknown-linux-gnu`
- `x86_64-pc-windows-msvc`

The same workflow supports manual rebuilds from the Actions tab through `workflow_dispatch`; pass the release tag, for example `v0.1.0`.

## Homebrew Release

Homebrew installation is published through the external tap repository:

```text
weirdo-adam/homebrew-tap
```

Users install from the tap with:

```sh
brew install weirdo-adam/tap/issue-jumper
```

The main repository release workflow dispatches the tap repository's `Bottle` workflow after a new GitHub Release is created. The repository must have a `HOMEBREW_TAP_TOKEN` secret with permission to run workflows in `weirdo-adam/homebrew-tap`.

The dispatch sends:

- `formula`: `issue-jumper`
- `release_tag`: the Homebrew bottle tag, for example `issue-jumper-0.1.0`
- `source_url`: the GitHub source archive URL for the release tag
- `source_sha256`: the SHA-256 checksum for that source archive

Re-running the release workflow for an existing GitHub Release refreshes the release assets but does not dispatch Homebrew again.

The formula keeps `rust` as a build dependency for source fallback, but supported Homebrew installs should pour a bottle so users do not download Rust or LLVM during normal installation. The formula intentionally does not run `issue-jumper install-zed`, because Homebrew formulae should not mutate user editor configuration during install.

If a user has both Homebrew and the one-command installer copy, keep the sources explicit. The terminal uses the first `issue-jumper` in `PATH`, and Zed uses the absolute path written by the latest `install-zed` run. To move a macOS setup to Homebrew, run the one-command installer with `--uninstall`, then run `/opt/homebrew/bin/issue-jumper install-zed --force` after installing from the tap.

## Local Release

Local release scripts remain available for testing or emergency uploads.

Package the current host target:

```sh
scripts/package-release.sh --version v0.1.0
```

Package a specific target:

```sh
scripts/package-release.sh --target aarch64-apple-darwin --version v0.1.0
```

Build and upload to GitHub Releases:

```sh
scripts/publish-release.sh v0.1.0
```

Run the publish script once per local target you want to maintain in the release assets.
