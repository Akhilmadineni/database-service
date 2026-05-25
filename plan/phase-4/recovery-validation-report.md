# Recovery Validation Report

Status: Draft baseline

Validated assets:
- `ops/backup_metadata.sh`
- `ops/restore_metadata.sh`
- `ops/validate_recovery.sh`

Recommended drill:
1. export a metadata backup using `backup_metadata.sh`
2. run a capsule create/reconcile operation
3. restore into a fresh metadata database using `restore_metadata.sh`
4. run `validate_recovery.sh`
5. compare capsule, operation, and audit counts against expectations

Current limitation:
- this report captures the recovery workflow and scripts, but a full NAS-hosted restore drill still needs to be executed in the target environment.
