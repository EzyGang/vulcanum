FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
RUN apt update && apt install --no-install-recommends -y pkg-config libssl-dev clang && rm -rf /var/lib/apt/lists/*
COPY --from=planner /app/recipe.json recipe.json
ENV SQLX_OFFLINE=true
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin vulcanum-main-app

FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt update && apt install --no-install-recommends -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/vulcanum-main-app /usr/local/bin/
ENTRYPOINT ["/usr/local/bin/vulcanum-main-app"]
