.PHONY: fmt fmt-check lint test check coverage

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all --check

lint:
	cargo clippy --all-targets --all-features -- -D warnings

test:
	cargo test --all-targets

check: fmt-check lint test

coverage:
	scripts/coverage.sh
