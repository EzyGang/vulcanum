# AGENTS.md - Vulcanum

## Overview

Vulcanum is a symphony-like agentic work orchestrator. It provides:

- An **agentic control system** for dispatching and monitoring agent tasks.
- **Fine-grained permissions and access controls** for multi-tenant, multi-agent environments.
- **Agent isolation controls** to sandbox and constrain individual agents.

## Repository Layout

| Module                   | Path                | Technology        | Status |
| ------------------------ | ------------------- | ----------------- | ------ |
| CLI                      | `cli/`              | Rust              | Active |
| Worker Daemon            | `worker-server/`    | Rust, SQLite      | Active |
| Server                   | `server/`           | Rust              | Active |
| Shared Types & Utilities | `shared/`           | Rust              | Active |
| Frontend UI              | `frontend/`         | TypeScript/Preact | Active |
| Agent Server             | _(omitted for now)_ | —                 | —      |

All packages (Rust and JS/TS) are managed as a single monorepo via **pnpm workspaces** and **Turborepo**. The Rust crates are also part of a Cargo workspace defined in the root `Cargo.toml`.

## Build & Run

From the repository root, use turbo commands to build, lint, test, and format:

```bash
# Install all dependencies
pnpm install

# Build everything (Rust + frontend)
pnpm run build

# Build a specific package
pnpm run build --filter=@repo/server
pnpm run build --filter=@repo/cli

# Run lints (Rust clippy + frontend biome)
pnpm run lint

# Type-check (frontend only)
pnpm run type-check

# Lint + type-check (full validation)
pnpm run validate

# Format everything
pnpm run format

# Run all tests
pnpm run test

# Dev server (frontend)
pnpm run dev

# Run a specific Rust binary
cargo run -p vulcanum-server --bin vulcanum-web
cargo run -p vulcanum-worker-server --bin vulcanum-server
cargo run -p vulcanum-cli --bin vulcanum
```

You can also enter a crate directory and build it independently:

```bash
cd server/
cargo run --bin vulcanum-web
```

## Conventions

- Every Rust crate keeps its own `AGENTS.md` with **crate-specific** conventions.
- All Rust crates share the same `rustfmt.toml` at the repository root.
- All Rust code must follow the `rust-code-style` skill rules defined in `.agents/skills/rust-code-style/SKILL.md`.
- All frontend UI component work must follow the `base-ui` skill rules defined in `.agents/skills/base-ui/SKILL.md`.
- Once done implementing run `pnpm run format && pnpm run validate`, both should succeed with no warnings.
- If you have changed/added new queries, run `pnpm run prep-queries` before committing.

## Frontend / Backend API Contract

- The frontend's `fetchApi` wrapper (`frontend/src/utils/api/client.ts`) converts request body keys to `snake_case` and response keys to `camelCase` automatically.
- **Do NOT add `#[serde(rename = "...")]` or `#[serde(rename_all = "camelCase")]` attributes in Rust backend models** to translate between cases. Keep Rust struct field names plain (snake_case) and let the frontend handle bidirectional conversion.
- Enum variants that map to wire values should use `#[serde(rename_all = "snake_case")]` (e.g. `WorkerStatus`) so they serialize as lowercase strings matching the frontend's TypeScript string-union types.

### Signal Reactivity in Hooks

- When using `useEffect` or `useCallback` with Preact signals, always include `<signal>.value` in the dependency array, not the signal object itself. Reactivity depends on reading the `.value` property inside the hook body.

## Rust Code Guidelines

### Important Rules

<important_rules>

- Comments should explain **why**, not **what** — only add them when the intent is genuinely hard to infer from the code.
- Doc comments (`///`) are for public API surfaces, and for non-trivial logic where a single-line description prevents confusion.
- The length of a single file should not be more than 200 lines; if it exceeds that, split it, unless it is logically correct to keep this as a single file and splitting it will introduce complexity rather than simplificity.
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

All web service crates (e.g. `server`, future `agent-server`) must follow a strict layered architecture.

### Layers

| Layer      | Responsibility                                           | Location                            |
| ---------- | -------------------------------------------------------- | ----------------------------------- |
| HTTP       | Routing, handlers, request/response serialization        | `src/routes/` or `src/handlers/`    |
| Service    | Business logic, auth, validation, caching, orchestration | `src/services/<domain>/service/`    |
| Repository | Database queries, SQLx execution                         | `src/services/<domain>/repository/` |

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

| Slug          | Status      | Meaning                                   |
| ------------- | ----------- | ----------------------------------------- |
| `planned`     | Planned     | Backlog — accepted but not ready to start |
| `to-do`       | To Do       | Ready for implementation                  |
| `in-progress` | In Progress | Currently being worked on                 |
| `in-review`   | In Review   | Implementation done, awaiting review      |
| `done`        | Done        | Validated and complete (final)            |

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

| Tier | Kaneo Priority | When to Use                                       |
| ---- | -------------- | ------------------------------------------------- |
| P0   | `high`         | Core infrastructure — nothing works without these |
| P1   | `medium`       | Feature work that depends on P0                   |
| P2   | `low`          | Reliability, optimizations, CLI polish            |
| P3   | `low`          | Developer experience, tooling, documentation      |

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

## Module-Specific Conventions

For module-specific details, refer to the local `AGENTS.md` in each module directory:

- `server/AGENTS.md` — Rust backend (migrations, SQLx, actix-web, env vars)
- `worker-server/AGENTS.md`
- `cli/AGENTS.md`
- `frontend/AGENTS.md` — TypeScript/Preact UI (component patterns, API layer, design system)

**Always read the module's local `AGENTS.md` before working on code in that directory.**

### Worker Daemon Architecture

The `worker-server` crate uses an embedded SQLite journal for crash-robust job execution.
Jobs run concurrently governed by a `Semaphore` sized to `max_concurrent_jobs` (received from the server at registration).
On restart, the journal is reconciled against Docker/subprocess reality to recover or retire in-flight jobs.
See `worker-server/AGENTS.md` for the full architecture (daemon loop, recovery, harnesses, state journal).

## Feature implementation checklist

[ ] - tests for relevant logic blocks worth covering are added
[ ] - `pnpm prep-queries` is run if any backend work has been done
[ ] - `pnpm format` in root is successful
[ ] - `pnpm validate` passes
[ ] - `pnpm test` passes
