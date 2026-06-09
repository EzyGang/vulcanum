FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
RUN apt update && apt install --no-install-recommends -y pkg-config libssl-dev clang && rm -rf /var/lib/apt/lists/*
COPY --from=planner /app/recipe.json recipe.json
ENV SQLX_OFFLINE=true
ARG BUILD_PROFILE=release
RUN cargo chef cook --profile $BUILD_PROFILE --recipe-path recipe.json
COPY . .
ARG BUILD_PROFILE=release
RUN cargo build --profile $BUILD_PROFILE --bin vulcanum-web --bin vulcanum-dispatcher

FROM debian:bookworm-slim AS runtime
ARG BUILD_PROFILE=release
WORKDIR /app
RUN apt update && apt install --no-install-recommends -y ca-certificates && rm -rf /var/lib/apt/lists/*
RUN useradd --no-create-home --shell /usr/sbin/nologin app
COPY --from=builder /app/target/${BUILD_PROFILE}/vulcanum-web /usr/local/bin/
COPY --from=builder /app/target/${BUILD_PROFILE}/vulcanum-dispatcher /usr/local/bin/
USER app
ENTRYPOINT ["/usr/local/bin/vulcanum-web"]