# ADR 0004: API Contract Style

Status: Accepted

## Decision

The control plane uses:
- REST-style JSON over HTTPS
- OpenAPI `3.1`
- versioned routes under `/v1`
- asynchronous semantics for mutating operations
- a consistent `data/meta` success envelope
- a structured error envelope

## Contract Rules

- JSON request fields use `snake_case`
- mutating requests require `Idempotency-Key`
- reads are synchronous
- writes return operation handles
- same idempotency key plus same request hash returns the original operation
- same idempotency key plus different request hash returns conflict

## Why Async Writes

Reconciliation is staged and may:
- serialize by capsule
- checkpoint before mutation
- verify after mutation
- roll back on failure

An async operation model keeps that behavior uniform.
