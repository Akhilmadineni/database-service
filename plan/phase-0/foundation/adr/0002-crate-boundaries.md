# ADR 0002: Workspace and Crate Boundaries

Status: Accepted

## Decision

Use a Cargo workspace with:
- `api`
- `core`
- `store`
- `worker`
- `telemetry`

## Responsibilities

`core`
- capsule model
- drift classification
- invariants
- domain errors

`store`
- `sqlx` repositories
- transactions
- migration embedding

`api`
- `axum` routes
- validation
- response envelopes
- idempotency header handling

`worker`
- queue polling and leasing
- run orchestration
- retry and dead-letter policy

`telemetry`
- tracing subscriber setup
- OpenTelemetry setup
- naming conventions for logs, traces, and metrics

## Dependency Direction

- `api -> core, store, telemetry`
- `worker -> core, store, telemetry`
- `store -> core`
- `telemetry -> none of the project crates`
- `core -> none of the project crates`

## Boundary Rules

- only `store` contains SQL text
- only `api` translates HTTP into domain commands
- `core` stays transport-agnostic and persistence-agnostic
- `worker` orchestrates runs but does not define reconciliation rules
