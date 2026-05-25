# ADR 0001: Rust-First Stack

Status: Accepted

## Decision

Phase 1 uses:
- Rust stable `1.95.0`
- edition `2024`
- `axum`
- `tokio`
- `sqlx`
- `tracing` and `tracing-subscriber`
- `tracing-opentelemetry`, `opentelemetry`, `opentelemetry-otlp`
- `serde`, `serde_json`
- `utoipa`
- `thiserror`
- `uuid`
- `time`

## Why

- Rust 2024 is the current edition baseline and matches the roadmap
- `sqlx` supports PostgreSQL directly, checked queries, and embedded migrations
- `utoipa` gives code-generated OpenAPI for the `axum` API surface
- the stack supports deterministic background work, clear boundaries, and strong observability

## Toolchain Baseline

Required local tools:
- `rustup`
- stable toolchain pinned to `1.95.0`
- `cargo`
- `clippy`
- `rustfmt`
- `sqlx-cli`
- Docker Buildx for multi-arch images

## Bootstrap

1. `rustup toolchain install 1.95.0`
2. `rustup default 1.95.0`
3. `rustup component add clippy rustfmt`
4. `cargo install sqlx-cli --no-default-features --features rustls,postgres`
5. `cargo check --workspace`

## References

- [Rust 2024 edition guide](https://doc.rust-lang.org/edition-guide/rust-2024/index.html)
- [Cargo workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)
- [SQLx checked queries](https://docs.rs/sqlx/latest/sqlx/macro.query.html)
- [SQLx migrations](https://docs.rs/sqlx/latest/sqlx/macro.migrate.html)
- [Utoipa OpenAPI generation](https://docs.rs/utoipa/latest/utoipa/derive.OpenApi.html)
