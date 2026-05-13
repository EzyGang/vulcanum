---
name: actix-web
description: Reference for building web services with Actix Web in the Vulcanum architecture. Covers the HTTP layer, routing, handlers, extractors, errors, application state, and how actix-web fits into the layered service/repository pattern.
---

# Actix Web (HTTP Layer)

Actix Web implements the **HTTP layer** of the Vulcanum web service architecture. It handles routing, request extraction, and response serialization. All business logic and database access must be delegated to the **service** and **repository** layers.

## Layered Architecture

```
HTTP Layer (actix-web handlers) → Service Layer → Repository Layer
```

- **HTTP Layer**: Deserializes requests, delegates to services, serializes responses.
- **Service Layer**: Contains all business logic (auth, validation, caching, invariants).
- **Repository Layer**: Encapsulates SQLx queries. No business logic. No caching.

**Rule**: Handlers may **not** call repositories directly. They may only call service methods.

## Cargo Dependencies

```toml
[dependencies]
actix-web = "4"
```

## Entry Point

```rust
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let state = create_app_state().await;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .configure(routes::configure)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Application State

State must expose **services**, not raw infrastructure:

```rust
pub struct AppState {
    pub users: UsersService,
    pub projects: ProjectsService,
}

impl AppState {
    pub async fn new(config: &Config) -> Result<Self, Error> {
        let db = pg::create_pool(&config.db_url).await?;
        let queue = Arc::new(Queue::new(...));
        let mailer = Arc::new(mailer::create(...));

        Ok(Self {
            users: UsersService::new(
                UsersRepository::new(),
                config,
                db.clone(),
                queue.clone(),
                mailer.clone(),
            ),
            projects: ProjectsService::new(ProjectsRepository::new(), db.clone()),
        })
    }
}
```

Extract in handlers:

```rust
async fn get_user(
    state: web::Data<Arc<AppState>>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let user = state
        .users
        .get_user(GetUserInput { id: path.into_inner() })
        .await?;
    Ok(web::Json(user))
}
```

## Routing

Split route registration into modules using `App::configure`:

```rust
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/users/{id}", web::get().to(users::get))
    );
}
```

## Handlers

Handlers are **thin**. They only:
1. Extract request data (path, query, body, state).
2. Call the appropriate service method.
3. Return the response.

```rust
async fn get_user(
    state: web::Data<Arc<AppState>>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let user = state
        .users
        .get_user(GetUserInput { id: path.into_inner() })
        .await?;
    Ok(web::Json(user))
}
```

## Extractors

| Extractor | Purpose |
|-----------|---------|
| `web::Json<T>` | JSON body |
| `web::Query<T>` | Query string |
| `web::Path<T>` | Path segments |
| `web::Data<T>` | Application state |
| `HttpRequest` | Raw metadata |

Only one body-consuming extractor per handler.

## Error Handling

Use `thiserror` for domain errors and implement `ResponseError` at the HTTP layer boundary:

```rust
use actix_web::{HttpResponse, ResponseError};
use thiserror::Error;

#[derive(Debug, Error)]
enum AppError {
    #[error("not found")]
    NotFound,
    #[error("bad request")]
    BadRequest,
    #[error("internal error")]
    Internal(#[from] users::Error),
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            Self::NotFound => HttpResponse::NotFound().json(...),
            Self::BadRequest => HttpResponse::BadRequest().json(...),
            Self::Internal(_) => HttpResponse::InternalServerError().json("Internal Server Error"),
        }
    }
}
```

Services return structured errors. Repositories map `sqlx::Error` to domain errors.

## Testing

Use `actix_web::test` with the full app state:

```rust
#[actix_web::test]
async fn test_get_user() {
    let state = Arc::new(AppState::new(&test_config()).await.unwrap());
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure)
    ).await;

    let req = test::TestRequest::get().uri("/api/v1/users/1").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}
```

## Further Reading

- API docs: <https://docs.rs/actix-web/4>
- Extractors: <https://docs.rs/actix-web/latest/actix_web/trait.FromRequest.html#implementors>
