## Overview

Main Application Server for the Vulcanum orchestrator.

## Build & Run

```bash
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

## Code Style

All code must follow the `rust-code-style` skill rules defined in `.agents/skills/rust-code-style/SKILL.md`.

### Generics

All trait bounds should be written in `where`:

```rust
// GOOD
pub fn new<N, T, P, E>(user_id: i32, name: N, title: T, png_sticker: P, emojis: E) -> Self
where
    N: Into<String>,
    T: Into<String>,
    P: Into<InputFile>,
    E: Into<String>,
{ ... }

// BAD
pub fn new<N: Into<String>,
           T: Into<String>,
           P: Into<InputFile>,
           E: Into<String>>
    (user_id: i32, name: N, title: T, png_sticker: P, emojis: E) -> Self { ... }
```

```rust
// GOOD
impl<T> Trait for Wrap<T>
where
    T: Trait
{ ... }

// BAD
impl<T: Trait> Trait for Wrap<T> { ... }
```

### Documentation comments

1. Documentation must describe _what_ your code does and mustn't describe _how_.
2. Be sure that your comments follow grammar, punctuation, and capitalization.
3. Do not use ending punctuation in short list items.
4. Link resources in your comments when possible.

### Use `Self` where possible

When referring to the type for which block is implemented, prefer using `Self`.

### Avoid duplication in fields names

```rust
struct Message {
    // GOOD
    #[serde(rename = "message_id")]
    id: MessageId,

    // BAD
    message_id: MessageId,
}
```

### Conventional generic names

Use `S` for streams, `Fut` for futures, `F` for functions where possible.

### Deriving traits

Derive `Copy`, `Clone`, `Eq`, `PartialEq`, `Hash` and `Debug` for public types when possible.
Derive `Default` when there is a reasonable default value.

### `Into`-polymorphism

Use `T: Into<Ty>` when this can simplify user code.

### `must_use`

Always mark functions as `#[must_use]` if they don't have side effects.

### Creating boxed futures

Prefer `Box::pin(async { ... })` instead of `async { ... }.boxed()`.

### Full paths for logging

Always write `log::<op>!(...)` instead of importing `use log::<op>;`.

### `&str` -> `String` conversion

Prefer `.to_owned()` over `.to_string()`, `.into()`, `String::from`, etc.

### Order of imports

Separate import groups with blank lines. Use one `use` per crate.
Module declarations come before the imports.

```rust
mod x;
mod y;

// First std.
use std::{ ... }

// Second, external crates.
use crate_foo::{ ... }
use crate_bar::{ ... }

// Then current crate.
use crate::{}

// Finally, parent and child modules, but prefer `use crate::`.
use super::{}

// Re-exports go after imports.
pub use crate::x::Z;
```

### Import Style

When implementing traits from `std::fmt` import the module:

```rust
// GOOD
use std::fmt;

impl fmt::Display for RenameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { .. }
}
```

Prefer `use crate::foo::bar` to `use super::bar` or `use self::bar::baz`.

### Order of Items

Optimize for first-time readers. Put public items first, structs/enums before functions/impls, and order top-down.

### Early Returns

Use early returns instead of if/else where possible.

### If-let

Avoid `if let ... { } else { }`, use `match` instead.

### Empty Match Arms

Use `=> (),` when a match arm is intentionally empty.
