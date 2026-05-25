# Phase 2 Carry-Forward Notes

Use these notes to keep Phase 3 execution aligned with the normalized capsule and fingerprint model.

## Mechanism Notes

- Preserve the normalized capsule as the source of truth for planning and verification.
- Keep observed-state capture and drift classification deterministic; phase 3 should not reintroduce ad hoc planner logic.
- Maintain fingerprint updates across desired, observed, plan, and checkpoint artifacts during reconciliation.

## Execution Notes

- The Phase 3 implementation currently executes reconciliation inline through the API path while still using the operation model.
- A future phase can move the same orchestration into a dedicated worker/queue path without redefining the checkpoint or drift model.
