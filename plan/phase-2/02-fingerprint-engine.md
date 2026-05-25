# Workstream 02: Fingerprint Engine

Goal:
- implement canonical desired and observed fingerprint generation

Deliverables:
- desired fingerprint generator
- observed-state projection model
- fingerprint domain separation and hashing utilities

Tasks:
- [x] move request-hash logic into a dedicated fingerprint module
- [x] implement desired-state canonicalization exactly once
- [x] define observed-state capture shape for catalog and runtime facts
- [x] generate `desired_fingerprint`, `observed_fingerprint`, `plan_fingerprint`, and `checkpoint_fingerprint` through one shared interface
- [x] add deterministic fixtures for equivalent-input hashing

Done criteria:
- identical normalized inputs always yield identical fingerprints
- fingerprint domains match the ACR spec

Status notes:
- 2026-05-24: Added `core/src/fingerprint.rs` with domain-separated hashing for request, desired, observed, plan, and checkpoint artifacts.
- 2026-05-24: Added `core/src/observed.rs` to define the observed-state projection used by drift and reconciliation.
- 2026-05-25: Refreshed `.sqlx` metadata after expanding checked query usage in the store layer.
