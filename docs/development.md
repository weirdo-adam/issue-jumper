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

Future editor or launcher integrations should follow this structure: add a narrow installer command and target module, then call the same `open`, `url`, and `doctor` workflow rather than duplicating branch parsing or URL construction.

## Local Release

Release artifacts are built and uploaded from a local machine, not from a GitHub Actions release job.

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
