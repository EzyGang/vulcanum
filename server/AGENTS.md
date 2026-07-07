# AGENTS.md - vulcanum-server

This crate is a member of the Vulcanum Cargo workspace.
Refer to the root-level `AGENTS.md` for shared project conventions, Rust code guidelines, and build instructions.

## Overview

Server for the Vulcanum orchestrator.

## Build & Run

```bash
# From the repo root
cargo run -p vulcanum-server --bin vulcanum-web

# Or from this directory
cd server
cargo run --bin vulcanum-web
```

## Migrations

Migrations are written in raw SQL (see `./migrations`).

To add migrations, use the [sqlx CLI tool](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md).

Steps:

1. Put `DATABASE_URL=<your local db>` in `.env` for SQLx to check queries live.
2. Run existing migrations via `sqlx migrate run`.
3. Add a new migration with `sqlx migrate add -r <migration_name>`
   (creates `<migration_name>.up.sql` and `<migration_name>.down.sql`).
4. Fill in the migration code.
5. Check migrations do not fail by running `sqlx migrate run` on your local db.

## SQLx Guide

- Use `query_as!()` and `query!()` macros if possible.
- Fall back to `query_as()` and `query()` only if necessary.

## Architecture

This crate follows the **Web Service Architecture** defined in the root `AGENTS.md`.

### Actix-Web Conventions

- Use `web::Data<Arc<AppState>>` for application state.
- `AppState` exposes **service structs**, not raw `PgPool`.
- Route configuration uses `App::configure(...)` in `src/routes/mod.rs`.
- Handlers are thin: they extract request data, delegate to the service layer, and serialize responses.
- No business logic, auth checks, or direct repository calls in handlers.

### SQLx Conventions

- Use `query!` and `query_as!` macros when possible.
- Repository methods must map `sqlx::Error` to domain errors; never leak raw SQL errors into the service or HTTP layers.
- Use the `Queryer<'c>` trait pattern from `src/db/queryer.rs` for transaction support:
  ```rust
  pub trait Queryer<'c>: sqlx::Executor<'c, Database = sqlx::Postgres> {}
  impl<'c> Queryer<'c> for &PgPool {}
  impl<'c> Queryer<'c> for &'c mut PgConnection {}
  ```

## Module Layout

`src/` is role-first, then domain-specific. Do not create a domain root such as `src/work_runs/` that contains routes, services, models, and SQL together. Instead, put each piece in the layer that owns it.

```
src/
  routes/                    # Actix route registration, handlers, extractors
    jobs.rs                  # Thin HTTP handlers for job endpoints
    jobs_tests.rs            # Route tests scoped to the jobs routes module
  services/                  # Business logic, orchestration, caches, external clients
    work_runs/
      mod.rs                 # Domain service module declarations
      service/               # Service root and larger service methods
        mod.rs               # WorkRunsService type, constructor, shared deps
        poll.rs
        submit_result.rs
        record_review.rs
    dispatcher/
      dispatch_store.rs      # Service-owned workflow store abstraction
      cancel_store.rs
  db/                        # Repository structs and SQLx query implementations
    work_runs.rs             # WorkRunsRepository type
    work_runs/
      queries.rs             # Query module declarations or small query impls
      queries/
        limits.rs            # Focused SQLx query implementations
  models/                    # Domain data types and domain errors
    work_runs/
      mod.rs
      model.rs               # Rows, DTOs, enums
      errors.rs              # WorkRunsError and related domain errors
  tests/                     # Shared helpers, e2e tests, cross-module service tests
    helpers.rs
    e2e_integration_tests.rs
    e2e_worker_flow_tests.rs
    work_runs_service/
      mod.rs
      work_runs_tests.rs
      record_review_tests.rs
  util/                      # Cross-domain helpers with no business state
```

When adding or changing a domain, place each concern by layer: route handlers in `src/routes/`, business behavior in `src/services/<domain>/`, persistence in `src/db/<domain>.rs` and `src/db/<domain>/`, and data/error types in `src/models/<domain>/`. For example, a `work_runs` API change usually touches `routes/work_runs.rs`, one or more files under `services/work_runs/`, `db/work_runs.rs` or `db/work_runs/queries/`, and `models/work_runs/model.rs` or `models/work_runs/errors.rs`.

Keep reusable test fixtures in `src/tests/helpers.rs`. Put e2e-style tests and service tests that need cross-module access under `src/tests/`. Route tests may stay next to route modules as `src/routes/<route>_tests.rs`. Do not use `#[path]` attributes from production modules to pull in test files.

### Provider Namespace (`src/services/providers/`)

All external-provider client code lives under a single `providers/` directory so adding a future alternative only requires adding one sibling directory.

```
src/services/providers/
  client.rs      # Dispatcher enum + TaskFetcher trait (e.g. IntegrationClient)
  kaneo/         # Kaneo-specific HTTP client
    client.rs
    errors.rs

src/models/providers/
  errors.rs      # Shared provider error types
  model.rs       # Shared provider model types (e.g. IntegrationType)
```

### Provider Configuration

Provider configuration rows (name, URL, API key) are stored through `src/db/provider_configs.rs`. The domain remains named `provider_configs` to avoid colliding with the `providers` external-client namespace.

### Repository Conventions

Each domain keeps query module declarations in `src/db/<domain>/queries.rs`. Small modules may keep all query implementations there, but split modules should keep `queries.rs` declaration-only and place implementations in named child files. Do not put implementation code in `mod.rs`. Example:

```
src/db/
  <domain>.rs
  <domain>/
    queries.rs        # SQLx query implementations for small modules

# or, when split:
src/db/
  <domain>.rs
  <domain>/
    queries.rs        # Module declarations only
    queries/
      <area>.rs       # Focused query implementations
```

### Service Conventions

Business operations are split into individual files under `service/<operation>.rs`. Keep the service root in `service/mod.rs` when a domain has operation modules:

```
src/services/<domain>/
  service/
    mod.rs           # Service type, constructor, shared deps
    <operation>.rs   # One file per service method (e.g. poll.rs, acknowledge.rs)
```

### Dispatcher Stores

- `dispatch_store.rs` — implements the `DispatchStore` trait (previously `flag_store.rs`).
- `cancel_store.rs` — implements the `CancelStore` trait.

### Worker Registration

- `registration_code_store.rs` — abstract + Redis + in-memory stores for worker registration codes (previously `code_store.rs`).

## Supported .env Variables

- `DATABASE_URL`
- `MAX_CONNS` - max db connections (default: `32`)
- `IS_SINGLE_USER` - `true` keeps instance-password auth; `false` enables GitHub OAuth user auth
- `GITHUB_OAUTH_CLIENT_ID`
- `GITHUB_OAUTH_CLIENT_SECRET`
- `GITHUB_OAUTH_REDIRECT_URL`
