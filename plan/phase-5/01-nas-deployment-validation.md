# Workstream 01: NAS Deployment Validation

Goal:
- execute the GitHub Actions deployment flow against the real NAS environment

Inputs from Phase 4:
- `compose.prod.yaml`
- `deploy/env/prod.env.example`
- NAS deploy workflow in `.github/workflows/nas-deploy-agent.yml`

Tasks:
- [ ] provide GitHub secrets and environment approvals
- [ ] push a tagged image through GHCR
- [ ] run the NAS deploy workflow end to end
- [ ] capture resulting deployment evidence and any follow-up hardening
