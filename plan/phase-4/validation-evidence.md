# Validation Evidence

This file captures the concrete local validation outcomes produced during the Phase 4 pass.

## Build and Test

- `cargo fmt --all`
- `cargo check --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo build --workspace --locked`
- `cargo test --workspace`
- `cargo sqlx prepare --check --workspace` with `DATABASE_URL` set

## Deployment Profile Validation

- `docker compose -f compose.dev.yaml config` succeeded
- `docker compose --env-file deploy/env/prod.env.example -f compose.prod.yaml config` succeeded
- `docker build -t database-service:phase4-check .` succeeded after copying `.sqlx` into the builder image and enabling `SQLX_OFFLINE`

## Observability Validation

- `/metrics` returns Prometheus text
- sample output included:
  - `database_service_http_requests_total{route="/healthz",method="GET",status="200"}`
  - `database_service_http_requests_total{route="/readyz",method="GET",status="200"}`
  - `database_service_reconcile_results_total{state="succeeded"}`

## Recovery Drill Validation

- metadata backup created successfully with `ops/backup_metadata.sh`
- metadata restore completed successfully with `ops/restore_metadata.sh`
- `ops/validate_recovery.sh` returned non-zero counts for `capsule`, `operation`, and `audit_event` in the restored database

## Remaining External Validation

- GitHub Actions deployment to the target NAS still requires environment secrets and approval
- target-NAS execution of the recovery runbook is still pending
