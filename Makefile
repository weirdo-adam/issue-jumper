.PHONY: fmt fmt-check lint line-check script-check style test check coverage

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all --check

lint:
	cargo clippy --all-targets --all-features -- -D warnings

line-check:
	scripts/check-file-lines.sh

script-check:
	bash -n scripts/*.sh
	sh -n scripts/install.sh

style: fmt-check lint line-check script-check

test:
	cargo test --all-targets

check: style test

coverage:
	scripts/coverage.sh
