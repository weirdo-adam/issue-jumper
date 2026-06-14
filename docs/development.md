# Development Guide

This guide covers repository maintenance, local validation, release automation, and quality rules for Issue Jumper.

## Principles

- Keep the CLI core editor-neutral.
- Keep branch parsing, remote parsing, and URL generation in shared Rust modules.
- Add editor integrations as thin adapters that call `open`, `url`, `doctor`, or `install-*` commands.
- Treat project configuration as optional; defaults should cover common GitHub, GitLab, Redmine, Jira, and numeric branch names.
- Do not install Windows Rust targets for local validation. Windows packaging is verified by GitHub Actions.

## Project Structure

```text
src/
├── main.rs              # Binary entrypoint.
├── lib.rs               # Library module exports.
├── cli/                 # CLI command parsing and subcommands.
├── browser/             # System browser opener boundary.
├── config.rs            # Config types, loading, validation entrypoints.
├── config/              # Config lint rules and tests.
├── git/                 # Git command wrapper and remote parser.
├── issue/               # Branch-to-issue matching rules.
├── url/                 # Issue URL construction.
├── zed/                 # Zed tasks/keymap installer adapter.
├── error.rs             # Shared error and exit code mapping.
├── jump.rs              # Main workflow orchestration.
└── platform.rs          # Issue platform enum.
```

## Coding Standard

- Every checked source, script, workflow, config, and Markdown file must stay at or below 500 lines.
- Rust code must pass `cargo fmt --all --check`.
- Rust code must pass `cargo clippy --all-targets --all-features -- -D warnings`.
- Shell scripts must pass syntax checks.
- New behavior needs focused tests near the owning module or CLI surface.
- Shared behavior belongs in the core library, not in editor-specific installers.

Run the full local gate:

```sh
make check
```

`make check` runs:

```sh
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
scripts/check-file-lines.sh
bash -n scripts/*.sh
sh -n scripts/install.sh
cargo test --all-targets
```

Format before committing:

```sh
make fmt
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

## Editor Integration Rules

Zed integration installs a task named `Issue Jumper: Open Current Issue` and binds a user-selected key to `task::Spawn`.

Use the local Zed installer during development:

```sh
scripts/install-zed.sh
```

Pass `--key <key>` to choose a different keybinding, `--force` to replace an existing binding, or `--print` to preview the Zed config snippets. The public one-command installer and local source installer run `install-zed --force` by default so repeated installs refresh the selected binding; pass `--no-force` when existing conflicts should be preserved.

Current non-Zed integrations are printable examples, not automatic installers:

- VS Code: `issue-jumper integration print --target vscode`
- Cursor: `issue-jumper integration print --target cursor`
- Generic editors: `issue-jumper integration print --target generic`

Keep these integrations thin. Editor-specific code may write configuration or examples only.

## Release Automation

GitHub Actions owns the normal release path. Pushing a `v*` tag runs `.github/workflows/release.yml`, builds the release matrix, and uploads assets to the matching GitHub Release.

Current release targets:

- `aarch64-apple-darwin`
- `x86_64-unknown-linux-gnu`
- `x86_64-pc-windows-msvc`

The workflow supports manual rebuilds from the Actions tab through `workflow_dispatch`; pass the release tag, for example `v0.1.2`.

## Homebrew Release

Homebrew installation is published through:

```text
weirdo-adam/homebrew-tap
```

Users install from the tap with:

```sh
brew install weirdo-adam/tap/issue-jumper
```

The main release workflow dispatches the tap repository's `Bottle` workflow after release assets are uploaded. The repository must have a `HOMEBREW_TAP_TOKEN` secret with permission to run workflows in `weirdo-adam/homebrew-tap`.

The dispatch sends:

- `formula`: `issue-jumper`
- `release_tag`: the Homebrew bottle tag, for example `issue-jumper-0.1.2`
- `source_url`: the GitHub source archive URL for the release tag
- `source_sha256`: the SHA-256 checksum for that source archive

The formula keeps `rust` as a build dependency for source fallback, but supported Homebrew installs should pour a bottle so users do not download Rust or LLVM during normal installation. The formula intentionally does not run `issue-jumper install-zed`, because Homebrew formulae should not mutate user editor configuration during install.

If a user has both Homebrew and the one-command installer copy, keep the sources explicit. The terminal uses the first `issue-jumper` in `PATH`, and Zed uses the absolute path written by the latest `install-zed` run. To move a macOS setup to Homebrew, run the one-command installer with `--uninstall`, then run `/opt/homebrew/bin/issue-jumper install-zed --force` after installing from the tap.

## Local Release

Local release scripts remain available for testing or emergency uploads.

Package the current host target:

```sh
scripts/package-release.sh --version v0.1.2
```

Package a specific target:

```sh
scripts/package-release.sh --target aarch64-apple-darwin --version v0.1.2
```

Build and upload to GitHub Releases:

```sh
scripts/publish-release.sh v0.1.2
```

Run the publish script once per local target you want to maintain in the release assets.
