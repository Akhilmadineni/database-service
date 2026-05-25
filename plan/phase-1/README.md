# Phase 1 Work Plan

Phase window: 2026-06-08 to 2026-06-21

Purpose:
- implement the Rust workspace baseline
- add the control-plane schema and migrations
- make create/update/reconcile operations deterministic and idempotent
- expose the first contract-aligned API surface

Workstreams:
- [01-workspace-and-scaffolding.md](./01-workspace-and-scaffolding.md)
- [02-schema-and-migrations.md](./02-schema-and-migrations.md)
- [03-idempotent-api-and-operations.md](./03-idempotent-api-and-operations.md)
- [04-phase-0-carry-forward-notes.md](./04-phase-0-carry-forward-notes.md)

Agent flow:
- Implementation Agent carries the work
- Validation Agent checks each step
- Checkpoint and Push Agent records validated refs
- NAS Deploy Agent builds and deploys validated refs through GitHub Actions

How to use this folder:
- keep active implementation detail here
- update status notes as work lands
- if a concern should be handled in Phase 2, record it here before Phase 1 closes

Current status:
- Workstream 01 complete with a compiling Rust workspace, pinned toolchain, Dockerfile, and baseline tracing/config bootstrap
- Workstream 02 complete with live PostgreSQL-backed migration validation and checked SQLx metadata
- Workstream 03 complete with store-backed capsule, reconcile, and operation endpoints plus idempotency validation

Phase closeout status:
- Workstream 01 complete
- Workstream 02 complete
- Workstream 03 complete
- Phase 2 planning folder created for compiler, fingerprint, and drift work
