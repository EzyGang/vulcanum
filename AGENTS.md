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

# Run migrations up
pnpm migrate-server-up

# Run migrations down
pnpm migrate-server-down

# Prepare queries cache
pnpm prep-queries

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

- When using `useEffect` with Preact signals, include `<signal>.value` in the dependency array when the effect must rerun after the signal value changes. Reactivity depends on reading the `.value` property inside the hook body.
- When using `useCallback` for event handlers or submit handlers, do not include `<signal>.value` in the dependency array just because the callback reads or writes it. Signal objects are stable and `.value` is read at invocation time, so value dependencies recreate handlers on every signal change without improving correctness.

## Rust Code Guidelines

### Important Rules

<important_rules>

- Comments should explain **why**, not **what** — only add them when the intent is genuinely hard to infer from the code.
- Doc comments (`///`) are for public API surfaces, and for non-trivial logic where a single-line description prevents confusion.
- The length of a single file should not be more than 300 lines; if it exceeds that, split it, the only exceptions to this rule are the files where splitting introduces too much complexity and it is semantically better to have the code centralized.
- You should strive to write as low amount of code as possible, not in a cutting corners way, but if something can be achieved with fewer LOC - it should always be a preferred implementation path.
- Must follow DRY (do not repeat yourself) principle. No code repetition should exist for any reason.
- No `unwrap()`, `expect()`, or `panic!()` in production code — use proper error handling with `Result`, or alternatives that don't panic, like `unwrap_or`, `unwrap_or_else`, etc.
- No `pub use` re-exports — use direct imports of what is needed.
- No glob imports (`use module::*`) — always be explicit.
- No `Vec<HashMap<String, Value>>` or raw collection returns — use proper structs/vectors of structs.
- Prefer struct methods and traits over free functions when operations belong to a type.
- Everything must have explicit types (use type annotations when inference is ambiguous).
- Use `&str` over `&String`, `&[T]` over `&Vec<T>` for function parameters.
- No `unsafe` code whatsoever.
- No `clone()` unless necessary — leverage lifetimes and borrowing.
- Use `match` over `if let Some(...)` chains for clarity.
- Use `thiserror` for structured error types, `anyhow` only at application boundaries.
- Use `tracing` for structured logging, not `println!`.
- Don't silence clippy warnings with `#[allow(...)]` unless already present — fix the issue instead.
- Prefer **composition over inheritance**. Build behavior by combining small single-responsibility components rather than deep class hierarchies.
- No inline test modules (`#[cfg(test)] mod tests { ... }` inside source files). Always place tests in separate `*_tests.rs` files alongside the module and include them in `mod.rs` with `#[cfg(test)] mod tests_module;`.
- Only test application-specific business logic. Do not write tests for framework internals (actix-web routing, SQLx pool management, serde serialization), third-party library behavior, or trivial glue code. Focus tests on: state transitions, input validation, error handling, and business rules.
  </important_rules>

### Style & Formatting

All formatting, import ordering, and style conventions (generics, documentation, import groups, `Self`, early returns, etc.) are enforced by the `rust-code-style` skill. Load it with `skill name="rust-code-style"` when editing Rust files.

## Web Service Architecture

All web service crates (e.g. `server`, future `agent-server`) must follow a strict layered architecture.

### Layers

| Layer      | Responsibility                                           | Location                         |
| ---------- | -------------------------------------------------------- | -------------------------------- |
| HTTP       | Routing, handlers, request/response serialization        | `src/routes/` or `src/handlers/` |
| Service    | Business logic, auth, validation, caching, orchestration | `src/services/<domain>/`         |
| Repository | Database queries, SQLx execution                         | `src/db/<domain>.rs`             |
| Models     | Domain rows, DTOs, enums, errors, shared principals      | `src/models/<domain>/`           |
| Utilities  | Cross-domain helpers with no business state              | `src/util/`                      |

Rules:

- Each layer may only communicate with the layer directly above or below it.
- The HTTP layer **never** calls repositories directly.
- The repository layer **never** contains business logic, caching, or auth checks.

### File Organization

The `server` crate is organized by architectural role first, then by domain. This keeps layer boundaries visible at the top level and avoids mixing HTTP, business logic, SQL, and DTOs in one domain directory.

```
server/src/
  routes/                    # HTTP route registration, handlers, extractors
    jobs.rs
    jobs_tests.rs
  services/                  # Business logic and infrastructure owned by services
    work_runs/
      mod.rs
      service/               # Service root plus one file per larger operation
        mod.rs               # WorkRunsService type and constructor
        poll.rs
        submit_result.rs
  db/                        # Repository structs and SQLx query implementations
    work_runs.rs             # WorkRunsRepository type
    work_runs/
      queries.rs
      queries/
        limits.rs
  models/                    # Domain rows, DTOs, enums, errors, shared principals
    work_runs/
      mod.rs
      model.rs
      errors.rs
  tests/                     # Shared helpers, e2e tests, and cross-module service tests
    helpers.rs
    e2e_integration_tests.rs
    work_runs_service/
      mod.rs
      work_runs_tests.rs
  util/                      # Cross-domain helpers with no business state
```

- Put HTTP concerns in `src/routes/`. Route tests can live beside the route file as `*_tests.rs` when they only exercise that route module.
- Put business logic in `src/services/<domain>/`. When service logic is split, keep the service root in `src/services/<domain>/service/mod.rs` and operation modules in `src/services/<domain>/service/<operation>.rs`; do not keep both `service.rs` and a sibling `service/` directory.
- Put repository structs in `src/db/<domain>.rs` and SQLx query modules under `src/db/<domain>/`.
- Put database row structs, request/response DTOs, enums, shared principals, and domain errors in `src/models/<domain>/`.
- Put reusable server test helpers, e2e tests, and cross-module service tests under `src/tests/` instead of using `#[path]` from production modules.
- Split files when they exceed 200 lines.
- Large domains may be extracted to separate workspace crates under `services/<domain>/`.

### Repository Conventions

- Repositories are thin, stateless wrappers around SQLx queries (one per domain/table).
- The shared `Queryer<'c>` trait lives in `src/db/queryer.rs`.
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
