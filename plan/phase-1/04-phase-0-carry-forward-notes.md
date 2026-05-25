# Phase 0 Carry-Forward Notes

Use these notes if a decision needs to be validated or tightened during Phase 1.

## Architecture Notes

- Keep the API async-by-default for writes so checkpoint, verification, and rollback semantics stay uniform.
- Map `desired_fingerprint`, `observed_fingerprint`, `plan_fingerprint`, and `checkpoint_fingerprint` cleanly into persistence from the start.
- Preserve explicit separation between `BlockedUnsafe` and `ManualIntervention`.
- The reconcile route shape is `/v1/capsules/{app}/{environment}/reconcile`; keep plan and implementation aligned there.

## Foundation Notes

- Validate that `utoipa` still feels right once the `axum` routes exist.
- Validate day-to-day developer ergonomics of the `sqlx` checked-query workflow and `.sqlx` maintenance.
- Keep crate boundaries strict; do not let SQL leak into `api` or `worker`.
- Keep local Postgres validation reproducible; Docker CLI alone is not enough if the daemon is unavailable.

## Security Notes

- Confirm whether bootstrap truly needs superuser and, if so, how quickly runtime can step down to a narrower admin role.
- Keep auth built around mTLS plus signed service identity, not static API keys.
- Make audit-event writes part of the happy path early so later ledger hardening is additive instead of invasive.

## Phase 2 Preview Notes

If these are not fully settled by Phase 1, carry them forward:
- richer capsule spec validation
- deterministic plan generation fixtures
- drift scoring and classification tuning
- observed-state capture shape for runtime telemetry
