# Workstream 02: Rust Platform Foundation

Goal:
- Lock the Rust stack, crate boundaries, and implementation constraints for Phase 1.

Target stack:
- Rust stable (current target: 1.95.0)
- `axum`, `tokio`, `sqlx`, `tracing`, `tracing-opentelemetry`

Deliverables:
- `docs/adr/0001-rust-stack.md`
- `docs/adr/0002-crate-boundaries.md`
- `docs/adr/0003-data-access-and-migrations.md`
- `docs/adr/0004-api-contract-style.md`

Tasks:
- [ ] Define workspace structure (`api`, `core`, `store`, `worker`, `telemetry`).
- [ ] Define error model and API response envelope.
- [ ] Define migration strategy and SQL review workflow.
- [ ] Define compile/test/lint toolchain (`cargo fmt`, `clippy`, test policy).
- [ ] Define multi-arch container build strategy (`amd64`, `arm64`).
- [ ] Define versioning and release tagging scheme.

Done criteria:
- ADRs are approved and linked from this file.
- No unresolved stack decision blocks Phase 1 implementation.
- Local developer bootstrap steps are documented and reproducible.

