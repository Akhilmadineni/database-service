# Workstream 01: Reconciliation Executor

Goal:
- execute deterministic reconciliation stages from normalized desired state and observed state

Tasks:
- [x] observe target PostgreSQL state from the admin connection
- [x] classify drift and derive a deterministic stage plan
- [x] execute the stage plan against PostgreSQL
- [x] verify the observed state after apply

Done criteria:
- reconciliation success moves a capsule to `converged`
- verification runs after apply and can fail the reconciliation

Status notes:
- 2026-05-25: Added target-state observation, planner integration, and stage execution in `store/src/reconcile.rs`.
- 2026-05-25: The reconcile API path now records and executes reconciliation inline through the operation model.
