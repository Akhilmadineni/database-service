# database-service Roadmap

## Mission
Build a Rust-based, DB-only autonomous database control plane for NAS environments that provisions, secures, and self-heals PostgreSQL tenants with deterministic behavior and auditable operations.

## Product Positioning
- This service is the database control layer, not a generic secret-management product.
- Primary target: Oracle ADB-like operational autonomy for PostgreSQL tenant lifecycle.
- Core differentiator: novel autonomous reconciliation mechanism with reversible remediation.

## Novel Mechanism: Autonomous Capsule Reconciler (ACR)
Each app/environment is managed as a Database Capsule:
- Desired state: database, roles, grants, connection/session limits, backup class, SLO class.
- Compiler: translates capsule spec into executable policy + SQL plan.
- Fingerprint: canonical hash of intended state and observed catalog state.
- Reconciler: computes minimal remediation, applies staged changes, verifies invariants.
- Safety: reversible checkpoints with automatic rollback on failed verification.
- Audit: tamper-evident operation ledger for lifecycle events.

## Program Timeline
Program start date: 2026-05-25
Program duration: 10 weeks

## Implementation Stack (Rust-First)
- Language/runtime: Rust stable (currently 1.95.0), edition 2024.
- HTTP API: `axum` + OpenAPI contract generation.
- Async runtime: `tokio`.
- PostgreSQL access: `sqlx` with compile-time query validation.
- Data model and migrations: SQL migrations with `sqlx migrate`.
- Background reconciliation workers: Postgres-backed queue using `FOR UPDATE SKIP LOCKED`.
- Observability: OpenTelemetry (`tracing`, `tracing-opentelemetry`) + Prometheus metrics.
- Packaging and deploy: multi-arch Docker images (`amd64` + `arm64`) via UGOS Pro Compose projects.
- Security baseline: mTLS between trusted services, signed API auth at edge, tamper-evident operation ledger.

## Phase 0 - Invention Freeze and Claims Mapping (Week 1-2, 2026-05-25 to 2026-06-07)
- Freeze ACR mechanism boundaries and terminology.
- Author system architecture and lifecycle state-machine diagrams.
- Produce a prior-art matrix and claim-ready novelty statements.
- Define measurable technical advantages and benchmark plan.
- Freeze Rust implementation boundaries (crate layout, persistence strategy, async/concurrency model).

Exit criteria:
- ACR specification is approved as the source of truth.
- Novelty statements are concrete enough for provisional filing draft.

## Phase 1 - Control-Plane Data Model and Idempotent Ops (Week 3-4, 2026-06-08 to 2026-06-21)
- Add metadata schema: `capsule`, `capsule_revision`, `operation`, `reconciliation_run`, `audit_event`.
- Add idempotency keys for create/update/reconcile APIs.
- Add deterministic operation replay and failure recovery semantics.
- Add API contracts for `provision`, `inspect`, and `reconcile` actions.
- Bootstrap Rust workspace modules: `api`, `core`, `store`, `worker`, `telemetry`.

Exit criteria:
- Repeated requests produce deterministic outcomes.
- Operation journal is complete for all write paths.

## Phase 2 - Capsule Compiler and Fingerprint Engine (Week 5-6, 2026-06-22 to 2026-07-05)
- Implement capsule spec parser and validation pipeline.
- Implement SQL plan generation from normalized capsule definitions.
- Implement canonical fingerprint generation for desired and observed state.
- Implement drift classification (safe, warning, critical) and drift scoring.

Exit criteria:
- Capsule-to-plan compiler is deterministic for identical inputs.
- Fingerprint and drift outputs are stable across repeated inspections.

## Phase 3 - Autonomous Reconciliation and Safe Rollback (Week 7-8, 2026-07-06 to 2026-07-19)
- Implement staged reconciliation executor with pre/post verification checks.
- Implement reversible checkpoints for grants, role settings, and ownership mutations.
- Add session-aware safety checks using PostgreSQL runtime telemetry.
- Implement automatic rollback with root-cause annotation in operation journal.

Exit criteria:
- Failed reconciliation attempts auto-rollback without manual SQL intervention.
- Verification guards prevent unsafe destructive operations by default.

## Phase 4 - NAS Production Hardening and Filing Package (Week 9-10, 2026-07-20 to 2026-08-02)
- Ship production NAS deployment profile (compose, persistence, health probes).
- Add observability stack: metrics, structured logs, reconciliation dashboards.
- Complete backup/recovery drill scripts and recovery validation report.
- Prepare patent evidence package: diagrams, benchmarks, novelty mapping, and claim-support tables.
- Produce hardened Rust release profile and SBOM for deployment artifacts.

Exit criteria:
- Production deployment runbook is validated end-to-end on NAS.
- Filing package contains reproducible technical evidence for the mechanism.

## Success Metrics
- 99.95%+ successful provisioning and reconciliation operations.
- Mean drift-detection to corrected-state time under 5 minutes for safe class drift.
- 100% of write operations captured in tamper-evident ledger.
- Zero non-reversible mutations in automated reconciliation mode.

## Immediate Next Steps (Execution Order)
1. Create Rust workspace skeleton and dependency baseline.
2. Implement Phase 1 schema and operation journal in `sqlx`.
3. Add capsule API contracts and idempotency middleware in `axum`.
4. Build compiler + fingerprint module behind feature flags.
