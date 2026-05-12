### Overview

CLI for the Vulcanum orchestrator.

## Build & Run

```bash
cd cli
cargo run
```

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

1. Documentation must describe _what_ your code does and mustn't describe _how_ your code does it and bla-bla-bla.
2. Be sure that your comments follow the grammar, including punctuation, the first capital letter and so on.
3. Do not use ending punctuation in short list items (usually containing just one phrase or sentence).
4. Link resources in your comments when possible.
5. Write crate names (e.g., `vulcanum`, `vulcanum-cli`) verbatim — not quoted or title-cased.

### Use `Self` where possible

When referring to the type for which block is implemented, prefer using `Self`, rather than the name of the type:

```rust
impl ErrorKind {
    fn print(&self) {
        Self::Io => println!("Io"),
        Self::Network => println!("Network"),
        Self::Json => println!("Json"),
    }
}
```

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

Use a generic parameter name `S` for streams, `Fut` for futures, `F` for functions (where possible).

### Deriving traits

Derive `Copy`, `Clone`, `Eq`, `PartialEq`, `Hash` and `Debug` for public types when possible.
Derive `Default` when there is a reasonable default value for the type.

### `Into`-polymorphism

Use `T: Into<Ty>` when this can simplify user code.

### `must_use`

Always mark functions as `#[must_use]` if they don't have side effects and the only reason to call them is to get the result.

### Creating boxed futures

Prefer `Box::pin(async { ... })` instead of `async { ... }.boxed()`.

### Full paths for logging

Always write `log::<op>!(...)` instead of importing `use log::<op>;` and invoking `<op>!(...)`.

### `&str` -> `String` conversion

Prefer using `.to_owned()`, rather than `.to_string()`, `.into()`, `String::from`, etc.

### Order of imports

Separate import groups with blank lines. Use one `use` per crate.
Module declarations come before the imports.
Order them in "suggested reading order" for a person new to the code base.

```rust
mod x;
mod y;

// First std.
use std::{ ... }

// Second, external crates (both crates.io crates and other rust-analyzer crates).
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

// BAD
impl std::fmt::Display for RenameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { .. }
}
```

Prefer `use crate::foo::bar` to `use super::bar` or `use self::bar::baz`.

### Order of Items

Optimize for the reader who sees the file for the first time.
If all items except one are private, always put the non-private item on top.
If there's a mixture of private and public items, put public items first.
Put structs and enums first, functions and impls last. Order type declarations in a top-down manner.

### Early Returns

Do use early returns:

```rust
// GOOD
fn foo() -> Option<Bar> {
    if !condition() {
        return None;
    }
    Some(...)
}

// BAD
fn foo() -> Option<Bar> {
    if condition() {
        Some(...)
    } else {
        None
    }
}
```

### If-let

Avoid the `if let ... { } else { }` construct, use `match` instead:

```rust
// GOOD
match ctx.expected_type.as_ref() {
    Some(expected_type) => completion_ty == expected_type && !expected_type.is_unit(),
    None => false,
}

// BAD
if let Some(expected_type) = ctx.expected_type.as_ref() {
    completion_ty == expected_type && !expected_type.is_unit()
} else {
    false
}
```

### Empty Match Arms

Use `=> (),` when a match arm is intentionally empty:

```rust
// GOOD
match result {
    Ok(_) => (),
    Err(err) => error!("{}", err),
}

// BAD
match result {
    Ok(_) => {},
    Err(err) => error!("{}", err),
}
```
