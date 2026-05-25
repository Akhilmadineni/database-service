# Workstream 03: Recovery and Runbooks

Goal:
- validate recovery behavior and document operator runbooks

Initial inputs from Phase 3:
- rollback exists for covered mutation classes
- audit rows capture reconciliation lifecycle events
- metadata schema and operation records are live

Likely tasks:
- [x] run backup and restore drills for the metadata database
- [x] document reconciliation failure/manual-intervention procedures
- [ ] validate runbook steps on the target NAS environment

Status notes:
- 2026-05-25: Added `ops/backup_metadata.sh`, `ops/restore_metadata.sh`, and `ops/validate_recovery.sh`.
- 2026-05-25: Verified backup, restore, and validation locally against a restored metadata database using containerized `pg_dump`, `pg_restore`, and `psql`.
- 2026-05-25: Added `plan/phase-4/recovery-validation-report.md` documenting the recovery drill flow and the remaining NAS-hosted validation gap.
