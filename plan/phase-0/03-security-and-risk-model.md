# Workstream 03: Security and Risk Model

Goal:
- Define security baseline and risk controls before autonomous reconciliation code is introduced.

Deliverables:
- `docs/security/threat-model.md`
- `docs/security/authz-authn.md`
- `docs/security/audit-ledger-model.md`
- `docs/security/secrets-and-key-management.md`

Tasks:
- [ ] Define trust boundaries (caller, control plane, PostgreSQL admin target).
- [ ] Define mTLS and token/API-key validation model.
- [ ] Define ledger hash-chain structure and tamper detection checks.
- [ ] Define least-privilege DB admin role and permissions scope.
- [ ] Define sensitive-data logging rules and redaction requirements.
- [ ] Define incident runbook triggers for failed reconciliation/rollback.

Done criteria:
- Threat model covers top abuse and failure paths.
- Security controls map directly to Phase 1 API and schema design.
- Audit model is sufficient for forensic reconstruction.

