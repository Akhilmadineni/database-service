# Workstream 01: ACR Mechanism and Claims

Goal:
- Define the novel Autonomous Capsule Reconciler (ACR) mechanism with precise technical boundaries.

Deliverables:
- `plan/phase-0/acr/acr-spec.md`
- `plan/phase-0/acr/acr-state-machine.md`
- `plan/phase-0/acr/prior-art-matrix.md`
- `plan/phase-0/acr/claim-candidates.md`

Supporting deliverables:
- `plan/phase-0/acr/benchmark-plan.md`

Tasks:
- [x] Define capsule model: desired state, observed state, and invariants.
- [x] Specify fingerprint construction and normalization rules.
- [x] Specify reconciliation planning stages and rollback checkpoints.
- [x] Document deterministic idempotency semantics for operations.
- [x] Build prior-art comparison table focused on DB autonomy mechanisms.
- [x] Draft independent and dependent claim candidates.

Done criteria:
- ACR spec is internally reviewed and accepted.
- Novelty points are concrete, implementation-tied, and testable.
- Claim draft can be handed to counsel without missing technical detail.

Status notes:
- 2026-05-23: Initial draft set created in `plan/phase-0/acr/` covering the capsule model, fingerprint model, state machine, prior-art comparison, and claim candidates.

Review focus:
- Pressure-test the mechanism against PostgreSQL operator prior art, especially declarative role and database reconciliation.
- Confirm whether create-time rollback rules are narrow enough to remain safely autonomous.
- Confirm whether the proposed fingerprint domains are sufficient for deterministic replay and audit reconstruction.
