# Phase 1 Delivery Plan

Status: Ready

## Sequencing

### Week 3

1. create virtual workspace and crate skeleton
2. add baseline dependencies and lint configuration
3. implement initial migrations for:
- `capsule`
- `capsule_revision`
- `operation`
- `reconciliation_run`
- `checkpoint`
- `audit_event`
4. wire metadata DB connectivity

### Week 4

1. implement repository layer in `store`
2. implement capsule create/update and operation lookup endpoints
3. implement idempotency enforcement
4. implement reconcile request acceptance path and operation journaling
5. pass CI baseline checks
