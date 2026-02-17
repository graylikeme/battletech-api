# syntax=docker/dockerfile:1
ARG RUST_VERSION=1.89
ARG APP_NAME=api

# ── Stage 1: chef prepare ─────────────────────────────────────────────────────
FROM rust:${RUST_VERSION}-alpine AS chef
RUN apk add --no-cache musl-dev pkgconfig openssl-dev && \
    cargo install cargo-chef --locked
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ── Stage 2: cargo cook (cache deps) ─────────────────────────────────────────
FROM chef AS builder
ARG SQLX_OFFLINE=true
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json

COPY . .
ENV SQLX_OFFLINE=${SQLX_OFFLINE}
RUN cargo build --release --target x86_64-unknown-linux-musl -p api

# ── Stage 3: minimal runtime ──────────────────────────────────────────────────
FROM alpine:3.20 AS runtime
RUN apk add --no-cache ca-certificates tzdata
WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/api ./api
COPY migrations ./migrations

EXPOSE 8080
ENV RUST_LOG=info
ENTRYPOINT ["/app/api"]
