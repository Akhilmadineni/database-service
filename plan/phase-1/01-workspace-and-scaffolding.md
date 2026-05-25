# Workstream 01: Workspace and Scaffolding

Goal:
- create the Rust workspace and baseline crates required by the Phase 1 architecture

Deliverables:
- virtual Cargo workspace
- crates: `api`, `core`, `store`, `worker`, `telemetry`
- baseline dependency and lint configuration

Tasks:
- [x] create workspace `Cargo.toml`
- [x] scaffold all five crates
- [x] pin toolchain and edition settings
- [x] add tracing and config bootstrap
- [x] add first `cargo check` baseline

Done criteria:
- workspace compiles
- crate boundaries match Phase 0 ADRs

Status notes:
- 2026-05-24: Added the root Cargo workspace, pinned `rust-toolchain.toml`, scaffolded `api`, `core`, `store`, `telemetry`, and `worker`, and added baseline tracing/config bootstrap.
- 2026-05-24: Added a baseline Dockerfile for the API service so the NAS deploy workflow has a concrete build target.
