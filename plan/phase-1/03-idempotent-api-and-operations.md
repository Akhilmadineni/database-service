# Workstream 03: Idempotent API and Operations

Goal:
- implement the first async write path with deterministic idempotency and operation tracking

Deliverables:
- contract-aligned capsule write endpoint
- reconcile trigger endpoint
- operation lookup endpoint
- idempotency-key and request-hash enforcement
- operation journal persistence

Tasks:
- [x] implement success and error envelopes
- [x] enforce `Idempotency-Key` on mutating routes
- [x] persist operation identity and request hash
- [x] reject same-key different-payload conflicts
- [x] return stable operation handles for retries

Done criteria:
- repeated same-key same-payload writes do not duplicate side effects
- same-key conflicting payloads return conflict

Status notes:
- 2026-05-24: Added store-backed `PUT /v1/capsules/{app}/{environment}`, `GET /v1/capsules/{app}/{environment}`, `POST /v1/capsules/{app}/{environment}/reconcile`, and `GET /v1/operations/{operation_id}` endpoints.
- 2026-05-24: Added SHA-256 request hashing, operation persistence, and idempotency conflict handling in the `store` crate.
- 2026-05-24: Store-level tests now verify same-key same-payload reuse, same-key different-payload conflict, reconcile operation creation, and live migration behavior against PostgreSQL.
- 2026-05-24: A live API smoke test was run against the local PostgreSQL container and verified capsule creation, revision persistence, reconcile operation creation, and operation lookup.

Implementation notes:
- The reconcile route is implemented as `/v1/capsules/{app}/{environment}/reconcile`.
- `X-Caller-Principal` is used as the current caller identity placeholder until the full auth model from the security plan is implemented in a later phase.
