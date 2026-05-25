# Workstream 01: Deployment Hardening

Goal:
- harden the deployment profile for NAS and CI/CD promotion

Initial inputs from Phase 3:
- reconcile currently executes inline through the API route
- the Dockerfile and GitHub deployment workflows exist but need production profile hardening
- target admin and metadata DB currently share a simple local-development pattern

Likely tasks:
- [ ] separate development and production compose profiles
- [ ] harden runtime secrets and deployment environment settings
- [ ] verify GHCR image promotion and NAS pull/update flow end to end
