# Workstream 02: Rust Platform Foundation

Goal:
- Lock the Rust stack, crate boundaries, and implementation constraints for Phase 1.

Target stack:
- Rust stable (current target: 1.95.0)
- `axum`, `tokio`, `sqlx`, `tracing`, `tracing-opentelemetry`

Deliverables:
- `plan/phase-0/foundation/adr/0001-rust-stack.md`
- `plan/phase-0/foundation/adr/0002-crate-boundaries.md`
- `plan/phase-0/foundation/adr/0003-data-access-and-migrations.md`
- `plan/phase-0/foundation/adr/0004-api-contract-style.md`

Supporting deliverables:
- `plan/phase-0/foundation/api/api-contract-v0.1.md`
- `plan/phase-0/foundation/schema/control-plane-schema-v0.1.md`
- `plan/phase-0/foundation/ci/baseline-plan.md`
- `plan/phase-0/foundation/deploy/ugreen-assumptions.md`

Tasks:
- [x] Define workspace structure (`api`, `core`, `store`, `worker`, `telemetry`).
- [x] Define error model and API response envelope.
- [x] Define migration strategy and SQL review workflow.
- [x] Define compile/test/lint toolchain (`cargo fmt`, `clippy`, test policy).
- [x] Define multi-arch container build strategy (`amd64`, `arm64`).
- [x] Define versioning and release tagging scheme.

Done criteria:
- ADRs are approved and linked from this file.
- No unresolved stack decision blocks Phase 1 implementation.
- Local developer bootstrap steps are documented and reproducible.

Status notes:
- 2026-05-23: ADR set drafted and aligned to the roadmap stack.
- 2026-05-23: API contract v0.1, control-plane schema v0.1, CI baseline plan, and UGREEN assumptions documented to satisfy Phase 0 readiness dependencies.

Review focus:
- Confirm `utoipa` remains the preferred OpenAPI generation choice for `axum`.
- Validate that `sqlx` checked-query workflow is acceptable for day-to-day developer ergonomics.
- Reconfirm the target UGREEN model baseline before production deployment planning.
