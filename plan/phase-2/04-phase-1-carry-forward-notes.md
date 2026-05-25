# Phase 1 Carry-Forward Notes

Use these notes to keep Phase 2 focused on the next real mechanism layer instead of reopening settled Phase 1 groundwork.

## Foundation Notes

- The workspace, migrations, and idempotent operation path are in place; build on them rather than reworking them.
- Keep `.sqlx` metadata current as checked queries expand.
- Preserve strict crate boundaries: API in `api`, domain logic in `core`, persistence in `store`, orchestration in `worker`.

## API Notes

- Current caller identity is still carried through `X-Caller-Principal`; Phase 2 or later should begin moving toward the full auth model from the security plan.
- The reconcile path is `/v1/capsules/{app}/{environment}/reconcile`.

## Mechanism Notes

- Current desired fingerprint is still based on serialized request content; replace that with the dedicated Phase 2 canonical fingerprint engine.
- Observed-state capture is still minimal; Phase 2 should define the real observed projection shape from the ACR spec.
- Reconcile operations are currently recorded, not executed; actual planning and execution belong to later phases.

Phase 2 resolution notes:
- Desired/request/observed/plan/checkpoint fingerprinting is now implemented in `core`.
- Observed-state capture and drift classification are now implemented and feed the reconciliation planner.
- Reconcile operations now execute through the Phase 3 path rather than being record-only.
