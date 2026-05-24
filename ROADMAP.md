# database-service Roadmap

## Mission
Deliver a secure database control plane that provisions isolated data stores for each application and environment across the NAS platform.

## Product Scope
- Provision per-app/per-environment PostgreSQL database + credentials.
- Rotate credentials with minimal/no downtime patterns.
- Store and distribute DB connection material through `secret-service`.
- Enforce policy, auditability, and lifecycle controls.

## Phase Plan

## Phase 0 - Core Provisioning (Week 0-2)
- Stable API for create/inspect provisioning actions.
- Idempotent provisioning behavior for repeated requests.
- Strict naming conventions and input validation.
- API-key or mTLS authentication for service-to-service calls.

Exit criteria:
- App DB provisioning works reliably for dev/staging/prod.
- Duplicate calls are safe and deterministic.

## Phase 1 - Security and Secret Integration (Week 2-4)
- Tight integration with `secret-service` for credential escrow.
- Credential rotation endpoint and runbook.
- Audit logs for all credential and database lifecycle actions.
- Least-privilege DB roles and grants by default.

Exit criteria:
- All issued DB credentials are available through secret-service.
- Rotation flows are tested without service interruption.

## Phase 2 - Reliability and Backups (Week 4-6)
- Backup policy orchestration hooks (full + incremental strategy).
- Health checks for DB reachability, role drift, and grants drift.
- Alerting on failed provisioning/rotation and backup lag.
- Recovery drill automation and documented RPO/RTO targets.

Exit criteria:
- Recoverability validated in scheduled restore drills.

## Phase 3 - Multi-Tenant Governance (Week 6-8)
- Application registration and ownership model.
- Quotas and resource limits per app/environment.
- Change approval flow for destructive operations.
- Metadata catalog (who owns which DB, retention class, backup policy).

Exit criteria:
- Platform can scale to many apps with clear ownership and controls.

## Phase 4 - Advanced Platform Features (Week 8-10)
- Optional read replica provisioning workflows.
- Optional point-in-time restore orchestration.
- Tenant-specific maintenance windows and policy profiles.
- Integration with CI/CD for ephemeral preview environments.

Exit criteria:
- Database lifecycle is fully automatable for application teams.

## Success Metrics
- 99.95%+ successful provisioning operations.
- 100% database credentials managed via secret-service.
- Database onboarding time reduced from hours to minutes.
