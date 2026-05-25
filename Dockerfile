FROM rust:1.95.0-bookworm AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY api ./api
COPY core ./core
COPY migrations ./migrations
COPY store ./store
COPY telemetry ./telemetry
COPY worker ./worker

RUN cargo build --release --locked -p database-service-api

FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update \
  && apt-get install --yes --no-install-recommends ca-certificates \
  && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/database-service-api /usr/local/bin/database-service-api

EXPOSE 8080
CMD ["database-service-api"]
