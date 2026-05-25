# Workstream 01: Deployment Hardening

Goal:
- harden the deployment profile for NAS and CI/CD promotion

Initial inputs from Phase 3:
- reconcile currently executes inline through the API route
- the Dockerfile and GitHub deployment workflows exist but need production profile hardening
- target admin and metadata DB currently share a simple local-development pattern

Likely tasks:
- [x] separate development and production compose profiles
- [x] harden runtime secrets and deployment environment settings
- [ ] verify GHCR image promotion and NAS pull/update flow end to end

Status notes:
- 2026-05-25: Added `compose.dev.yaml` and `compose.prod.yaml` to separate development and production stack profiles.
- 2026-05-25: Added file-based secret loading for `DATABASE_URL` and `TARGET_ADMIN_DATABASE_URL` through `*_FILE` environment support in the store bootstrap.
- 2026-05-25: Added `deploy/env/prod.env.example` and updated the NAS deploy workflow to honor environment-level `IMAGE_NAME`, `NAS_COMPOSE_FILE`, and `NAS_ENV_FILE` settings.
- 2026-05-25: Validated `docker compose ... config` for both development and production profiles and successfully built the production image locally with SQLx offline metadata.

Remaining external validation:
- actual GHCR promotion and NAS pull/update still require repository secrets and target-environment access.
