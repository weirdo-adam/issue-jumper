# Contributing

Thanks for considering a contribution to Issue Jumper.

## Development

Use the stable Rust toolchain pinned by `rust-toolchain.toml`.

Before submitting a change, run:

```sh
make check
```

`make check` covers Rust formatting, clippy with warnings denied, file length limits, shell syntax
checks, and tests.

## Code Standards

- Keep every checked source, script, workflow, config, and Markdown file at or below 500 lines.
- Keep Rust modules focused and close tests to the behavior they verify.
- Prefer shared CLI/library logic over editor-specific duplicate logic.
- Update documentation when user-visible behavior changes.

## Pull Requests

- Keep changes focused on one problem.
- Update documentation when user-visible behavior changes.
- Add or update tests for parsing, config, URL generation, and Zed integration changes.
- Do not include private hostnames, internal issue IDs, local paths, credentials, or tokens.
- Do not install Windows Rust targets for local validation; Windows is covered by GitHub Actions.

## Releases

Release artifacts are built by GitHub Actions when a `v*` tag is pushed. Release tags should be
treated as immutable after publication. Publish follow-up fixes as a new patch release instead of
rewriting an existing tag.
