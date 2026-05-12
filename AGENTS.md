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
| Frontend UI | *(omitted for now)* | — | — |
| Agent Server | *(omitted for now)* | — | — |

## Conventions

- Every Rust crate keeps its own `AGENTS.md` with project-specific conventions.
- All Rust crates share the same `rustfmt.toml` and the same code style rules enforced by the `rust-code-style` skill in `.agents/skills/rust-code-style/SKILL.md`.
- The monorepo root is intentionally minimal; each crate is independently buildable with `cargo`.

## Build & Run

1. Enter the desired crate directory (e.g. `cd main-app/`).
2. Run `cargo build` / `cargo run` as usual.
3. For the `main-app` server, refer to its local `AGENTS.md` for SQLx and migration workflows.
