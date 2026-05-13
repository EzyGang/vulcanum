# AGENTS.md - Vulcanum

## Overview

Vulcanum is a symphony-like agentic work orchestrator. It provides:

- An **agentic control system** for dispatching and monitoring agent tasks.
- **Fine-grained permissions and access controls** for multi-tenant, multi-agent environments.
- **Agent isolation controls** to sandbox and constrain individual agents.

## Repository Layout

| Module | Path | Technology | Status |
|--------|------|------------|--------|
| CLI | `cli/` | Rust | Active |
| Host Machine Server | `host-server/` | Rust | Active |
| Main Application Server | `main-app/` | Rust | Active |
| Shared Types & Utilities | `shared/` | Rust | Active |
| Frontend UI | *(omitted for now)* | — | — |
| Agent Server | *(omitted for now)* | — | — |

Non-Rust components (Frontend UI, Agent Server) are **not** part of the Cargo workspace and will live in their own top-level directories when added.

## Rust Workspace

The Rust crates (`cli/`, `host-server/`, `main-app/`, `shared/`) are organized as a Cargo workspace defined in the root `Cargo.toml`. This enables:

- Shared dependency resolution and locking.
- The `shared` crate to be used as a path dependency by other workspace members.
- A single `cargo check --workspace` or `cargo build --workspace` command.

## Build & Run

From the repository root you can build any workspace member:

```bash
# Build everything
cargo build --workspace

# Build a specific crate
cargo build -p vulcanum-main-app
cargo build -p vulcanum-host-server
cargo build -p vulcanum-cli
cargo build -p vulcanum-shared

# Run from the root
cargo run -p vulcanum-main-app
cargo run -p vulcanum-host-server
cargo run -p vulcanum-cli
```

You can also enter a crate directory and build it independently:

```bash
cd main-app/
cargo run
```

## Conventions

- Every Rust crate keeps its own `AGENTS.md` with **crate-specific** conventions.
- All Rust crates share the same `rustfmt.toml` at the repository root.
- All Rust code must follow the `rust-code-style` skill rules defined in `.agents/skills/rust-code-style/SKILL.md`.

## Rust Code Guidelines

### Important Rules

<important_rules>
- Comments should explain **why**, not **what** — only add them when the intent is genuinely hard to infer from the code.
- Doc comments (`///`) are for public API surfaces, and for non-trivial logic where a single-line description prevents confusion.
- The length of a single file should not be more than 200 lines; if it exceeds that, split it.
- MUST FOLLOW `DRY` (DO NOT REPEAT YOURSELF) principle. NO code repetition should exist for ANY reason.
- NO `unwrap()`, `expect()`, or `panic!()` in production code — use proper error handling with `Result`.
- NO `pub use` re-exports — use direct imports of what is needed.
- NO glob imports (`use module::*`) — always be explicit.
- NO `Vec<HashMap<String, Value>>` or raw collection returns — use proper structs/vectors of structs.
- Prefer struct methods and traits over free functions when operations belong to a type.
- Everything must have explicit types (use type annotations when inference is ambiguous).
- Use `&str` over `&String`, `&[T]` over `&Vec<T>` for function parameters.
- NO `unsafe` code whatsoever.
- NO `clone()` unless necessary — leverage lifetimes and borrowing.
- Use `match` over `if let Some(...)` chains for clarity.
- Use `thiserror` for structured error types, `anyhow` only at application boundaries.
- Use `tracing` for structured logging, not `println!`.
- Don't silence clippy warnings with `#[allow(...)]` unless already present — fix the issue instead.
- Prefer **composition over inheritance**. Build behavior by combining small single-responsibility components rather than deep class hierarchies.
- NO inline test modules (`#[cfg(test)] mod tests { ... }` inside source files). Always place tests in separate `*_tests.rs` files alongside the module and include them in `mod.rs` with `#[cfg(test)] mod tests_module;`.
- Only test application-specific business logic. Do not write tests for framework internals (actix-web routing, SQLx pool management, serde serialization), third-party library behavior, or trivial glue code. Focus tests on: state transitions, input validation, error handling, and business rules.
</important_rules>

### Style & Formatting

All formatting, import ordering, and style conventions (generics, documentation, import groups, `Self`, early returns, etc.) are enforced by the `rust-code-style` skill. Load it with `skill name="rust-code-style"` when editing Rust files.

## Crate-Specific Conventions

For crate-specific details (migrations, environment variables, SQLx workflows, etc.), refer to the local `AGENTS.md` in each crate directory:

- `main-app/AGENTS.md`
- `host-server/AGENTS.md`
- `cli/AGENTS.md`
