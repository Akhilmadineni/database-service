# Phase 1 Risk Register

Status: Ready

| ID | Risk | Impact | Likelihood | Mitigation |
| --- | --- | --- | --- | --- |
| R1 | `sqlx` checked-query workflow slows iteration | Medium | Medium | check in `.sqlx`, automate prepare checks, document workflow |
| R2 | schema evolves faster than API contract | High | Medium | co-review contract and schema changes |
| R3 | PostgreSQL admin privilege scope is wider than planned | High | Medium | prototype least-privilege admin role early |
| R4 | OpenTelemetry crate churn causes instability | Medium | Medium | isolate setup in `telemetry` crate and pin versions |
| R5 | NAS assumptions diverge from target operations | High | Medium | validate on the chosen UGREEN baseline early |
| R6 | idempotency semantics are underspecified in persistence | High | Low | implement unique operation identity and request-hash conflict checks first |
