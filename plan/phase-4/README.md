# Phase 4 Work Plan

Phase window: 2026-07-20 to 2026-08-02

Purpose:
- harden the NAS deployment profile
- expand observability and dashboards
- complete recovery drills and deployment runbooks
- prepare the filing evidence package

Workstreams:
- [01-deployment-hardening.md](./01-deployment-hardening.md)
- [02-observability-and-dashboards.md](./02-observability-and-dashboards.md)
- [03-recovery-and-runbooks.md](./03-recovery-and-runbooks.md)
- [04-filing-package-and-carry-forward-notes.md](./04-filing-package-and-carry-forward-notes.md)

How to use this folder:
- keep future Phase 4 implementation detail here
- carry forward any remaining hardening items from Phase 3

Current status:
- deployment profiles, secret-file config, and Docker image build validation are in place
- metrics endpoint and Prometheus-friendly counters are implemented
- backup/restore scripts and a local recovery drill succeeded against the metadata database
- filing evidence summary and dashboard/recovery docs are present

Reality check:
- end-to-end GHCR to NAS deployment was not executed because this machine does not have the NAS secrets or environment approvals
- the recovery runbook was validated locally, but not on the target NAS yet

Phase closeout status:
- Phase 4 is implementation-complete on this PC
- external environment validation is carried forward to Phase 5
