# Audit Ledger Model v0.1

Status: Accepted

## Model

The audit ledger is append-only and hash-chained.

Each `audit_event` contains:
- `audit_event_id`
- `occurred_at`
- `capsule_id`
- `operation_id`
- `reconciliation_run_id`
- `event_type`
- `payload_json`
- `prev_hash`
- `event_hash`

## Minimum Event Families

- request accepted
- request denied
- capsule revision created
- observation captured
- drift classified
- plan generated
- checkpoint written
- stage applied
- verification passed
- rollback started
- rollback completed
- manual intervention required

## Rules

- audit rows are never updated in place
- canonical payloads exclude secret material
- chain mismatch is a security incident
