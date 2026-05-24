# Workstream 01: ACR Mechanism and Claims

Goal:
- Define the novel Autonomous Capsule Reconciler (ACR) mechanism with precise technical boundaries.

Deliverables:
- `docs/acr/acr-spec.md`
- `docs/acr/acr-state-machine.md`
- `docs/acr/prior-art-matrix.md`
- `docs/acr/claim-candidates.md`

Tasks:
- [ ] Define capsule model: desired state, observed state, and invariants.
- [ ] Specify fingerprint construction and normalization rules.
- [ ] Specify reconciliation planning stages and rollback checkpoints.
- [ ] Document deterministic idempotency semantics for operations.
- [ ] Build prior-art comparison table focused on DB autonomy mechanisms.
- [ ] Draft independent and dependent claim candidates.

Done criteria:
- ACR spec is internally reviewed and accepted.
- Novelty points are concrete, implementation-tied, and testable.
- Claim draft can be handed to counsel without missing technical detail.

