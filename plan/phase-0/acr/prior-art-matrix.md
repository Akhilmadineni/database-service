# ACR Prior-Art Matrix v0.1

Status: Phase 0 baseline

This is an engineering novelty map, not a legal opinion.

## Comparison Lens

We compare whether a system clearly teaches:
- database-scoped desired state
- deterministic desired vs observed fingerprinting
- minimal staged remediation planning
- reversible checkpoints before mutation
- session-aware safety gates
- tamper-evident operation journaling

## Matrix

| Reference family | Primary source | What it clearly teaches | What it appears not to teach relative to ACR |
| --- | --- | --- | --- |
| Kubernetes controllers | [kubernetes.io/docs/concepts/architecture/controller/](https://kubernetes.io/docs/concepts/architecture/controller/) | Generic desired-state reconciliation loop | No database capsule boundary, no catalog fingerprint model, no SQL checkpoint package, no database-session safety model |
| Oracle Autonomous Database | [docs.oracle.com/en/cloud/paas/autonomous-database/adbsa/autonomous-intro-adb.html](https://docs.oracle.com/en/cloud/paas/autonomous-database/adbsa/autonomous-intro-adb.html) | Broad database autonomy across provisioning and operations | Public docs do not clearly teach tenant-scoped desired/observed/plan/checkpoint fingerprints or explicit rollback checkpoints |
| CloudNativePG | [cloudnative-pg.io/docs/devel/declarative_database_management](https://cloudnative-pg.io/docs/devel/declarative_database_management) | Declarative PostgreSQL management with reconciliation behavior | Does not clearly disclose the multi-fingerprint control model, reversible checkpoint artifact, or tenant-scoped rollback discipline central to ACR |
| Crunchy PostgreSQL Operator | [access.crunchydata.com/documentation/postgres-operator/4.6.1/](https://access.crunchydata.com/documentation/postgres-operator/4.6.1/) | Cluster-oriented PostgreSQL automation | Public materials emphasize cluster lifecycle, not a database-tenant compiler with checkpoint-before-mutate semantics |

## Strongest Novelty Threads

1. Database Capsule as the management boundary
2. Separate desired, observed, plan, and checkpoint fingerprints
3. Reversible remediation as a required stage
4. Session-aware safety gates
5. Tamper-evident lifecycle journal

## Follow-Up Pressure Tests

- compare ACR against declarative role and database reconciliation in PostgreSQL operators
- search for prior art around reversible database reconciliation checkpoints
- search for prior art around hashing observed catalog state for convergence logic
