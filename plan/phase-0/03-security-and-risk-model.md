# Workstream 03: Security and Risk Model

Goal:
- Define security baseline and risk controls before autonomous reconciliation code is introduced.

Deliverables:
- `plan/phase-0/security/threat-model.md`
- `plan/phase-0/security/authz-authn.md`
- `plan/phase-0/security/audit-ledger-model.md`
- `plan/phase-0/security/secrets-and-key-management.md`

Tasks:
- [x] Define trust boundaries (caller, control plane, PostgreSQL admin target).
- [x] Define mTLS and token/API-key validation model.
- [x] Define ledger hash-chain structure and tamper detection checks.
- [x] Define least-privilege DB admin role and permissions scope.
- [x] Define sensitive-data logging rules and redaction requirements.
- [x] Define incident runbook triggers for failed reconciliation/rollback.

Done criteria:
- Threat model covers top abuse and failure paths.
- Security controls map directly to Phase 1 API and schema design.
- Audit model is sufficient for forensic reconstruction.

Status notes:
- 2026-05-23: Security baseline docs created for threat model, authn/authz, audit ledger, and secrets handling.

Review focus:
- Validate whether bootstrap requires superuser and how quickly runtime can step down to a narrower admin role.
- Confirm that the chosen audit chain model is sufficient before external digest anchoring is added.
