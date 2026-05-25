# Workstream 02: Checkpoints and Rollback

Goal:
- capture reversible checkpoint data before mutation and roll back on failure

Tasks:
- [x] capture checkpoint state for database, role, and schema resources
- [x] persist checkpoint data and checkpoint fingerprint
- [x] roll back newly created or modified resources when execution or verification fails
- [x] verify rollback behavior with a failure-path test

Done criteria:
- failed reconciliation attempts can restore the checkpointed safe state for covered mutation classes

Status notes:
- 2026-05-25: Added checkpoint capture and checkpoint persistence using the existing `checkpoint` table.
- 2026-05-25: Added rollback logic for created databases, created roles, role connection limits, and created non-`public` schemas.
- 2026-05-25: Added a failpoint-backed reconciliation test that verifies rollback on a staged failure.
