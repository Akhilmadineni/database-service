# NAS Deploy Agent

Mission:
- build, publish, and deploy validated code to the NAS using GitHub Actions

Primary inputs:
- validated ref or checkpoint tag
- deployment environment selection
- NAS and registry secrets

Responsibilities:
- build the container image for the target ref
- publish the image to GHCR
- deploy the image to the NAS over SSH using a Compose-based update flow
- keep deployment gated behind GitHub environment protections

Baseline assumptions:
- images are published to `ghcr.io`
- the NAS pulls images from GHCR
- the NAS deployment path already contains the Compose file for the service stack
- GitHub environment protections are used for production-like deployment approval

Required GitHub secrets:
- `NAS_HOST`
- `NAS_SSH_USER`
- `NAS_SSH_PRIVATE_KEY`
- `NAS_SSH_KNOWN_HOSTS`
- `NAS_DEPLOY_PATH`
- `GHCR_DEPLOY_USERNAME`
- `GHCR_DEPLOY_TOKEN`

Recommended GitHub environment and variables:
- environment name `nas-prod`
- variable `NAS_COMPOSE_FILE` with default `compose.prod.yaml`
- variable `IMAGE_NAME` if you want to override the default image path

Outputs:
- published image reference
- deployment result summary
- target environment record in GitHub Actions

Automation:
- GitHub Actions workflow: `.github/workflows/nas-deploy-agent.yml`
