# Workstream 02: NAS Recovery Validation

Goal:
- run the backup and restore drill on the actual NAS-hosted metadata database

Inputs from Phase 4:
- `ops/backup_metadata.sh`
- `ops/restore_metadata.sh`
- `ops/validate_recovery.sh`
- `plan/phase-4/recovery-validation-report.md`

Tasks:
- [ ] execute backup on the NAS-hosted metadata database
- [ ] restore into a fresh target on the NAS
- [ ] run validation queries and compare counts
- [ ] document timing, operator steps, and any differences from local validation
