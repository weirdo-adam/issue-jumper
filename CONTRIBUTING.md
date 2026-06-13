# Contributing

Thank you for considering a contribution to Issue Jumper.

## Development

Use the stable Rust toolchain pinned by `rust-toolchain.toml`.

Before submitting a change, run:

```sh
make check
shellcheck scripts/*.sh
cargo package --allow-dirty --no-verify
```

## Pull Requests

- Keep changes focused on one problem.
- Update documentation when user-visible behavior changes.
- Add or update tests for parsing, config, URL generation, and Zed integration changes.
- Do not include private hostnames, internal issue IDs, local paths, credentials, or tokens.
- Release artifacts are built locally with repository scripts; this project does not use GitHub CI for packaging.

## Releases

Release tags should be treated as immutable after publication. Publish follow-up fixes as a new patch release instead of rewriting an existing tag.
