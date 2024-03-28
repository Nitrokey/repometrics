.PHONY: all
all: check lint

.PHONY: ci
ci: check lint test

.PHONY: check
check:
	cargo check

.PHONY: lint
lint:
	cargo clippy
	cargo fmt -- --check

.PHONY: test
test:
	cargo test

.PHONY: example
example:
	mkdir -p examples/data
	cargo run --quiet -- --config examples/repometrics.toml generate examples/a > examples/data/a.toml
	cargo run --quiet -- --config examples/repometrics.toml generate examples/b > examples/data/b.toml
	cargo run --quiet -- --config examples/repometrics.toml compare examples/data/a.toml examples/data/b.toml
	cargo run --quiet -- --config examples/repometrics.toml compare examples/data/b.toml examples/data/a.toml
	cargo run --quiet -- --config examples/repometrics.toml compare examples/data/a.toml examples/data/a.toml
	cargo run --quiet -- --config examples/repometrics.toml compare examples/data/a.toml examples/data/b.toml --output-format markdown
	cargo run --quiet -- --config examples/repometrics.toml compare examples/data/a.toml examples/data/a.toml --output-format markdown
