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
- Once done implementing run `make format && make check`, both should succeed with no warnings.

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

## Web Service Architecture

All web service crates (e.g. `main-app`, future `agent-server`) must follow a strict layered architecture.

### Layers

| Layer | Responsibility | Location |
|-------|--------------|----------|
| HTTP | Routing, handlers, request/response serialization | `src/routes/` or `src/handlers/` |
| Service | Business logic, auth, validation, caching, orchestration | `src/services/<domain>/service/` |
| Repository | Database queries, SQLx execution | `src/services/<domain>/repository/` |

Rules:
- Each layer may only communicate with the layer directly above or below it.
- The HTTP layer **never** calls repositories directly.
- The repository layer **never** contains business logic, caching, or auth checks.

### File Organization

Domain logic is organized under `src/services/<domain>/`:

```
src/services/<domain>/
  model.rs          # Domain types and constants
  errors.rs         # Domain errors (thiserror)
  repository.rs     # Repository struct definition
  repository/
    <table>.rs      # Query implementations per table
  service.rs          # Service struct definition
  service/
    <operation>.rs  # Individual business operations
```

- Keep the HTTP layer in `src/routes/` or `src/handlers/`.
- Split files when they exceed 200 lines.
- Large domains may be extracted to separate workspace crates under `services/<domain>/`.

### Repository Conventions

- Repositories are thin, stateless wrappers around SQLx queries (one per domain/table).
- Use a `Queryer<'c>` trait so methods accept both `&PgPool` and `&mut PgConnection` for transaction support:
  ```rust
  pub trait Queryer<'c>: sqlx::Executor<'c, Database = sqlx::Postgres> {}
  impl<'c> Queryer<'c> for &PgPool {}
  impl<'c> Queryer<'c> for &'c mut PgConnection {}
  ```
- Map `sqlx::Error` to domain errors inside repository methods. Do not leak raw SQL errors.
- **No caching, no auth, no business invariants** in repositories.

### Service Conventions

- Services are structs holding repositories, the DB pool, and other infrastructure (`Arc<dyn Mailer>`, `Arc<Queue>`, cache, etc.).
- All business logic lives in service methods: auth checks, input validation, caching, orchestration.
- Caching is done **exclusively** at the service layer.
- Services return domain errors, not HTTP responses.

### HTTP Layer Conventions

- Handlers are thin: extract request data, call the appropriate service method, and return the response.
- Application state (`web::Data`) must expose **services**, not raw database pools.
- No business logic, no validation rules, and no direct DB access in handlers.

## Kaneo Task Management

Tasks for this project live in Kaneo (project `k5s7dwb5f89anmaui2d814h9`, slug `vulcanum`).
The local `.kaneo-conf.json` is pinned to the project — `kaneo task` commands work from this directory without extra flags.

### Skills to Load

When creating or updating tasks, always load these skills first:

- `skill name="kaneo"` — CLI reference (list, create, update, status, labels, comments, etc.)
- `skill name="kaneo-task-template"` — required structure: Goal, Requirements, Dependencies, Validation, Actions Log

### Column Statuses

| Slug | Status | Meaning |
|------|--------|---------|
| `planned` | Planned | Backlog — accepted but not ready to start |
| `to-do` | To Do | Ready for implementation |
| `in-progress` | In Progress | Currently being worked on |
| `in-review` | In Review | Implementation done, awaiting review |
| `done` | Done | Validated and complete (final) |

### Task Lifecycle

```
planned → to-do → in-progress → in-review → done
```

- **planned**: Task exists but may be blocked by dependencies.
- **to-do**: Dependencies resolved, ready to pick up.
- **in-progress**: Move here at the start of implementation.
- **in-review**: Tests pass, PR submitted — move here, not directly to done. The reviewer (user) validates.
- **done**: Only after explicit user validation.

### Priority Conventions

| Tier | Kaneo Priority | When to Use |
|------|---------------|-------------|
| P0 | `high` | Core infrastructure — nothing works without these |
| P1 | `medium` | Feature work that depends on P0 |
| P2 | `low` | Reliability, optimizations, CLI polish |
| P3 | `low` | Developer experience, tooling, documentation |

### Creating a Task

```bash
kaneo task create \
  --title "Short imperative title" \
  --status planned \
  --description "# Full markdown body with Goal, Requirements, Dependencies, Validation, Actions Log"
```

Never guess requirements. If context is insufficient, ask before creating.

### Updating a Task

```bash
# Move status
kaneo task update <id> --status "in-progress"

# Update description (always pass the complete body — Kaneo replaces the entire field)
kaneo task update <id> --description "<full updated body>"

# Set priority
kaneo task update <id> --priority medium
```

### Listing Tasks

```bash
kaneo task list                           # all tasks in the project
kaneo task list --status planned          # filter by status
kaneo task list --priority high           # filter by priority
```

### Valid Status Values

When using `--status`, always use the **slug** form (lowercase, hyphenated):
`planned`, `to-do`, `in-progress`, `in-review`, `done`

Do NOT use display names (`"To Do"`, `"In Progress"`) — these will fail with a 400 error.

## Crate-Specific Conventions

For crate-specific details (migrations, environment variables, SQLx workflows, etc.), refer to the local `AGENTS.md` in each crate directory:

- `main-app/AGENTS.md`
- `host-server/AGENTS.md`
- `cli/AGENTS.md`
