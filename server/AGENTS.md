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
- Use the `Queryer<'c>` trait pattern for transaction support:
  ```rust
  pub trait Queryer<'c>: sqlx::Executor<'c, Database = sqlx::Postgres> {}
  impl<'c> Queryer<'c> for &PgPool {}
  impl<'c> Queryer<'c> for &'c mut PgConnection {}
  ```

## Supported .env Variables

- `DATABASE_URL`
- `MAX_CONNS` - max db connections (default: `32`)
