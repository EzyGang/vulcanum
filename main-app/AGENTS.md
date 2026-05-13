# AGENTS.md - vulcanum-main-app

This crate is a member of the Vulcanum Cargo workspace.
Refer to the root-level `AGENTS.md` for shared project conventions, Rust code guidelines, and build instructions.

## Overview

Main Application Server for the Vulcanum orchestrator.

## Build & Run

```bash
# From the repo root
cargo run -p vulcanum-main-app

# Or from this directory
cd main-app
cargo run
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

## Supported .env Variables

- `DATABASE_URL`
- `MAX_CONNS` - max db connections (default: `32`)
