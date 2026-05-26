.PHONY: format check build build-all build-release build-server build-server-release build-host-server build-cli build-cli-release build-shared run-server run-host-server run-cli test

# ── Formatting ──
check-fe:
	@cd frontend && pnpm fix

# Auto-format all code and apply clippy auto-fixes
format:
	@cargo fmt --all
	@cargo clippy --all-targets --workspace --fix --allow-dirty

# ── Checks ──

# Run full check: compile, formatting, and clippy lints
# Equivalent of `make check`:
check:
	@cargo check --workspace
	@cargo fmt --all -- --check
	@cargo clippy --all-targets --workspace -- -D warnings

prep-queries:
	@cargo sqlx prepare --workspace -- --all-targets


full-format-and-check: check-fe format check

# ── Build ──

# Build everything in the workspace
build:
	cargo build --workspace

build-all: build

# Build a specific crate
build-server:
	cargo build -p vulcanum-server

build-host-server:
	cargo build -p vulcanum-host-server

build-cli:
	cargo build -p vulcanum-cli

build-shared:
	cargo build -p vulcanum-shared

# Release builds (optimized)
build-release:
	cargo build --release --workspace

build-server-release:
	cargo build --release -p vulcanum-server

build-cli-release:
	cargo build --release -p vulcanum-cli

# ── Run ──

run-server:
	cargo run -p vulcanum-server

run-host-server:
	cargo run -p vulcanum-host-server

run-cli:
	cargo run -p vulcanum-cli

# ── Test ──

test:
	cargo test --workspace
