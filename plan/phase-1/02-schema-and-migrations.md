# Workstream 02: Schema and Migrations

Goal:
- implement the metadata schema and migration workflow for the control plane

Deliverables:
- initial migrations for `capsule`, `capsule_revision`, `operation`, `reconciliation_run`, `checkpoint`, and `audit_event`
- `sqlx` migration workflow
- `.sqlx` metadata baseline

Tasks:
- [x] create initial migrations
- [x] enforce key uniqueness and foreign-key relationships
- [x] wire migration startup path
- [x] document local migration/test flow
- [x] enable `cargo sqlx prepare --check`

Done criteria:
- migrations apply cleanly to an empty database
- schema matches the Phase 0 model closely enough for Phase 1 work

Status notes:
- 2026-05-24: Added `migrations/0001_control_plane_metadata.sql` covering `capsule`, `capsule_revision`, `operation`, `reconciliation_run`, `checkpoint`, and `audit_event`.
- 2026-05-24: Wired the `store` crate to build an optional `PgPool` and run migrations automatically when `DATABASE_URL` is set.
- 2026-05-24: Added a migration smoke test in the `store` crate that runs when `DATABASE_URL` is present.
- 2026-05-24: Docker Desktop was brought back online locally and live Postgres validation completed successfully against a containerized PostgreSQL 17 instance.
- 2026-05-24: `.sqlx` metadata was generated and `cargo sqlx prepare --check --workspace` now passes when `DATABASE_URL` is set.

Local migration and test flow:
1. Start PostgreSQL locally, for example with Docker or the target NAS dev stack.
2. Set `DATABASE_URL=postgresql://<user>:<password>@<host>:<port>/<db>`.
3. Optionally leave `RUN_MIGRATIONS=true` to auto-apply migrations on API or worker startup.
4. Run `cargo test -p database-service-store runs_migrations_when_database_url_is_present -- --exact --nocapture`.

Next note:
- Keep the checked-query surface meaningful as query complexity grows; do not let `.sqlx` become stale ceremony.
