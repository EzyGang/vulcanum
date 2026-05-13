---
name: actix-web
description: Reference for building web services with the Actix Web framework. Covers server setup, routing, handlers, extractors, errors, testing, and application state. Use when writing or modifying actix-web code in the Vulcanum monorepo.
---

# Actix Web

Reference for building web services with [Actix Web](https://actix.rs). Full API docs: <https://docs.rs/actix-web>.

## Cargo Dependencies

```toml
[dependencies]
actix-web = "4"
actix-rt = "2"
```

## Entry Point

Use `#[actix_web::main]` to start the actix runtime (replaces `#[tokio::main]`):

```rust
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Server (`HttpServer`)

- `HttpServer::bind("0.0.0.0:8080")` binds a socket.
- `HttpServer::workers(n)` overrides the default worker count (default = physical CPU count).
- Workers each get their own `App` instance; application factories must be `Send + Sync`.
- `HttpServer::shutdown_timeout(secs)` sets the graceful shutdown window (default 30 s).
- `HttpServer::disable_signals()` disables CTRL‑C / signal handling.

### Keep-Alive

```rust
HttpServer::new(|| App::new()...)
    .keep_alive(Duration::from_secs(75))
```

## Application (`App`)

The `App` holds routes, middleware, and scoped state.

### Configuration helper

Use `App::configure(...)` to split route registration into modules:

```rust
App::new().configure(routes::configure)
```

Then in `routes/mod.rs`:

```rust
use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/resource", web::get().to(handler))
    );
}
```

## Application State

### Read-only shared state (preferred)

Use `web::Data<T>` (wraps `Arc<T>`). Create it **outside** `HttpServer::new` and call `app_data`:

```rust
let pool = create_pool().await;

HttpServer::new(move || {
    App::new()
        .app_data(web::Data::new(pool.clone()))
        .route("/", web::get().to(handler))
})
```

Extract in handlers:

```rust
async fn handler(pool: web::Data<SqlitePool>) -> impl Responder { ... }
```

### Note on blocking primitives

Do **not** hold `std::sync::Mutex` across `.await` points inside handlers. Use `tokio::sync::Mutex` if locking is unavoidable across async boundaries.

## Routing

### Route directly on App

```rust
App::new()
    .route("/path", web::get().to(handler))
    .route("/path", web::post().to(handler2))
```

### Scoped routes (namespaces)

```rust
web::scope("/api/v1")
    .route("/users", web::get().to(users::list))
    .route("/users/{id}", web::get().to(users::show))
```

### Dynamic path segments

```rust
// /users/{id} with Path extractor
async fn show(path: web::Path<u32>) -> impl Responder {
    format!("User {}", path.into_inner())
}

// Deserialize into a struct
#[derive(Deserialize)]
struct Info {
    user_id: u32,
}
// pattern: /users/{user_id}
async fn show(info: web::Path<Info>) -> impl Responder { ... }
```

Pattern syntax: `{name}` matches up to the next `/`. `{name:regex}` applies a regex filter. A trailing `/` does **not** match (e.g. `foo/1/2/` does not match `foo/{a}/{b}`).

## Handlers

A handler is an async function returning something that implements `Responder`:

```rust
async fn handler() -> impl Responder { ... }
async fn handler() -> Result<impl Responder, AppError> { ... }
```

Built-in `Responder` impls: `&'static str`, `String`, `web::Json<T>`, `HttpResponse`, `Result<T, E>` (where E implements `ResponseError`), `(StatusCode, T)`, and more.

## Extractors

Typed arguments to handlers that pull data from the request. Up to 12 allowed per handler.

| Extractor | Purpose |
|-----------|---------|
| `web::Json<T>` | Deserializes JSON request body (T: Deserialize) |
| `web::Query<T>` | Deserializes query string (T: Deserialize) |
| `web::Path<T>` | Dynamic path segments (T: Deserialize or tuple) |
| `web::Data<T>` | Application state |
| `web::Form<T>` | URL-encoded form data |
| `HttpRequest` | Raw request metadata |
| `String` / `web::Bytes` | Raw request body |

**IMPORTANT**: If an extractor **consumes** the request body stream (e.g. `Json`, `String`, `Bytes`), only the first such extractor will succeed. Don't put two body-consuming extractors in the same handler signature.

## Error Handling

Implement `ResponseError` for typed errors:

```rust
use actix_web::{HttpResponse, ResponseError};

#[derive(Debug)]
enum AppError { ... }

impl fmt::Display for AppError { ... }

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            Self::NotFound => HttpResponse::NotFound().json(...),
            Self::BadRequest => HttpResponse::BadRequest().json(...),
        }
    }
}
```

Handler `Result` types: `Result<impl Responder, E>` works when `E` implements `ResponseError`. Actix automatically logs errors at `WARN` level (set `RUST_LOG=actix_web=debug` for more).

Best practice: split errors into **user-facing** (return descriptive messages) and **internal** (map to generic "Internal Server Error").

## Testing

Use `actix_web::test`:

```rust
use actix_web::{test, web, App};

#[actix_web::test]
async fn test_handler() {
    let app = test::init_service(
        App::new().route("/", web::get().to(handler))
    ).await;

    let req = test::TestRequest::get().uri("/").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}
```

### Testing with state

```rust
let pool = test_db().await;
let app = test::init_service(
    App::new()
        .app_data(web::Data::new(pool))
        .configure(routes::configure)
).await;
```

### Test POST with JSON

```rust
let req = test::TestRequest::post()
    .uri("/api/v1/login")
    .set_json(&serde_json::json!({"email": "test@example.com"}))
    .to_request();
```

## Further Reading

- Full API docs: <https://docs.rs/actix-web/4>
- Extractor implementors: <https://docs.rs/actix-web/latest/actix_web/trait.FromRequest.html#implementors>
- Responder foreign impls: <https://docs.rs/actix-web/4/actix_web/trait.Responder.html#foreign-impls>
- Error helpers: <https://docs.rs/actix-web/4/actix_web/error/struct.Error.html>
- Guard functions: <https://docs.rs/actix-web/4/actix_web/guard/index.html#functions>
