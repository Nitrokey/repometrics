.PHONY: all
all: check lint

.PHONY: check
check:
	cargo check

.PHONY: lint
lint:
	cargo clippy
	cargo fmt -- --check
