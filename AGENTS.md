# AGENTS.md — Vulcanum

## Scope and precedence

This file defines repository-wide rules. Before changing a module, read its local `AGENTS.md`; local rules extend or override this file for that module.

Required local guides:

- `server/AGENTS.md` — Actix Web, SQLx, migrations, server layout, environment variables
- `worker-server/AGENTS.md` — daemon lifecycle, isolation, recovery, journal semantics
- `cli/AGENTS.md` — CLI-specific conventions
- `frontend/AGENTS.md` — Preact architecture, state, API layer, and design system

Do not introduce a second convention beside an existing one. Inspect neighboring code and reuse its established pattern.

## Project overview

Vulcanum is an agentic work orchestrator with task dispatch and monitoring, multi-tenant access control, and agent isolation.

| Module                     | Path             | Technology                  |
| -------------------------- | ---------------- | --------------------------- |
| CLI                        | `cli/`           | Rust                        |
| Worker daemon              | `worker-server/` | Rust, SQLite                |
| API server                 | `server/`        | Rust, Actix Web, PostgreSQL |
| Shared types and utilities | `shared/`        | Rust                        |
| Frontend                   | `frontend/`      | TypeScript, Preact          |

The repository is a pnpm/Turborepo monorepo. Rust crates also belong to the root Cargo workspace.

## Required workflow

1. Read this file and the affected module's `AGENTS.md`.
2. Inspect the relevant implementation, tests, and call sites before editing.
3. Make the smallest complete change. Reuse existing abstractions and naming.
4. Add or update tests only for application behavior affected by the change.
5. Run the required validation from the repository root.

A change is complete only when all affected callers, tests, and documentation agree with the new behavior. Do not leave compatibility aliases, deprecated paths, placeholders, or `TODO` implementations unless explicitly requested.

## Commands

Run repository-wide commands from the root:

```bash
pnpm install
pnpm run build
pnpm run lint
pnpm run type-check
pnpm run validate
pnpm run format
pnpm run test
pnpm run dev
```

Filter a workspace package when needed:

```bash
pnpm run build --filter=@repo/server
pnpm run build --filter=@repo/cli
```

Database commands:

```bash
pnpm migrate-server-up
pnpm migrate-server-down
pnpm prep-queries
```

Run Rust binaries:

```bash
cargo run -p vulcanum-server --bin vulcanum-web
cargo run -p vulcanum-worker-server --bin vulcanum-server
cargo run -p vulcanum-cli --bin vulcanum
```

## Completion checks

For code changes, all of these must pass with no warnings:

```bash
pnpm run format
pnpm run validate
pnpm run test
```

Also run `pnpm prep-queries` after changing SQL, migrations, or SQLx query macros. Commit the updated `.sqlx/` metadata.

For documentation-only changes, run the repository formatter and inspect the rendered Markdown. Do not run unrelated builds or tests.

## Repository-wide engineering rules

### General

- Prefer clear, direct code over speculative abstractions.
- Keep changes focused. Do not add retries, telemetry, validation, or cleanup unrelated to the request.
- Remove code made obsolete by the change.
- Comments explain non-obvious intent or constraints, not what the code already states.
- Public APIs and non-trivial behavior should have concise documentation comments.
- Avoid duplicate behavior, constants, and business rules. Extract a shared abstraction only when it has a clear owner and improves readability.
- Prefer composition and small, single-responsibility components.
- Never suppress a linter warning unless the suppression already exists and is still justified. Fix the cause.

### File and module layout

- Keep production code beside the domain and layer that owns it.
- Group three or more files with one responsibility into a semantic submodule directory such as `session/` or `transport/`.
- Inside a named submodule, remove redundant filename prefixes: use `session/metrics.rs`, not `session/session_metrics.rs`.
- Do not create one-file directories or speculative nesting.
- Keep files at or below 300 lines! Split earlier when a clear responsibility boundary exists.

## Rust rules

All Rust changes must follow `.agents/skills/rust-code-style/SKILL.md` and the affected crate's `AGENTS.md`.

### Safety and error handling

- No `unsafe` code.
- No `unwrap()`, `expect()`, or `panic!()` in production code. Return or handle errors explicitly; non-panicking fallbacks such as `unwrap_or` are allowed when they preserve the intended behavior.
- Use `thiserror` for structured library/domain errors. Use `anyhow` only at application boundaries.
- Use `tracing` for logging; never use `println!` for production logs.
- Repository methods map database errors to domain errors. Raw SQLx errors must not escape the repository layer.

### Types, ownership, and imports

- Use concrete domain structs instead of untyped collections such as `Vec<HashMap<String, Value>>`.
- Add explicit type annotations at public boundaries and wherever inference is not immediately clear; do not annotate obvious locals solely for verbosity.
- Accept borrowed forms: `&str` instead of `&String`, and `&[T]` instead of `&Vec<T>`.
- Avoid `clone()` when borrowing or ownership transfer is sufficient. Clone only when a distinct owned value is required.
- Prefer methods or traits when behavior belongs to a type.
- Prefer `match` when handling multiple variants or branches. A single clear `if let` is acceptable; do not build long `if let Some(...)` chains.
- No glob imports.
- No `pub use` re-exports. Import items from their defining module.

### Rust tests

- Test application-specific behavior: state transitions, validation, error handling, precedence, and business invariants.
- Do not test framework behavior, SQLx pool mechanics, Serde itself, or trivial glue.
- Never place inline `#[cfg(test)] mod tests { ... }` blocks in production files.
- Keep a small test suite in a sibling `*_tests.rs` file and register it with `#[cfg(test)] mod <name>_tests;`.
- When a suite needs multiple files, place all tests and helpers under one `<module>_tests/` directory with `<module>_tests/mod.rs`. Never keep both `<module>_tests.rs` and `<module>_tests/`.
- Server route tests may live beside their route module. Shared fixtures, end-to-end tests, and cross-module service tests belong under `server/src/tests/`.

## Frontend/backend API contract

`frontend/src/utils/api/client.ts` is the case-conversion boundary:

- Request body keys are converted from `camelCase` to `snake_case`.
- Response keys are converted from `snake_case` to `camelCase`.
- Rust request and response fields remain ordinary `snake_case` fields. Do not add `#[serde(rename = "...")]` or `#[serde(rename_all = "camelCase")]` to translate frontend casing.
- Wire enums use `#[serde(rename_all = "snake_case")]` when the frontend expects lowercase snake-case string unions.

### Preact signal dependencies

- In `useEffect`, include `<signal>.value` in the dependency array when the effect must rerun after that value changes. Read `.value` in the effect body.
- In event or submit callbacks, do not add `<signal>.value` to `useCallback` dependencies merely because the callback reads or writes it. Signal objects are stable, and their current value is read when invoked.

All other frontend structure, state, styling, and component rules live in `frontend/AGENTS.md`.

## Web service architecture

Web service crates use a strict layered architecture:

```text
HTTP routes/handlers → services → repositories → database
                     ↘ domain models ↗
```

| Layer      | Location                                    | Responsibility                                           |
| ---------- | ------------------------------------------- | -------------------------------------------------------- |
| HTTP       | `src/routes/` or `src/handlers/`            | Routing, extraction, request/response serialization      |
| Service    | `src/services/<domain>/`                    | Business rules, auth, validation, caching, orchestration |
| Repository | `src/db/<domain>.rs` and `src/db/<domain>/` | SQLx queries and database error mapping                  |
| Models     | `src/models/<domain>/`                      | Rows, DTOs, enums, principals, domain errors             |
| Utilities  | `src/util/`                                 | Stateless cross-domain helpers                           |

Boundaries are mandatory:

- HTTP handlers call services, never repositories or database pools.
- Services own business rules, authorization, validation, caching, and orchestration.
- Repositories are thin, stateless persistence adapters. They contain no auth, caching, or business invariants.
- Services return domain values and errors, not HTTP responses.
- Application state exposes services, not raw database pools.
- Models may be shared across adjacent layers but must not acquire layer-specific behavior.

### Server layout

Organize `server/src/` by architectural role first, then domain:

```text
server/src/
  routes/
  services/<domain>/
  db/<domain>.rs
  db/<domain>/
  models/<domain>/
  tests/
  util/
```

For split service operations, use:

```text
services/<domain>/
  service/
    mod.rs
    <operation>.rs
```

`service/mod.rs` owns the service type, constructor, and shared dependencies. Operation files own larger service methods. Do not keep both `service.rs` and a sibling `service/` directory.

Repositories use the `Queryer<'c>` pattern from `server/src/db/queryer.rs` so methods can accept a pool or transaction:

```rust
pub trait Queryer<'c>: sqlx::Executor<'c, Database = sqlx::Postgres> {}
impl<'c> Queryer<'c> for &PgPool {}
impl<'c> Queryer<'c> for &'c mut PgConnection {}
```

See `server/AGENTS.md` for migrations, SQLx macros, detailed layouts, and provider conventions.
