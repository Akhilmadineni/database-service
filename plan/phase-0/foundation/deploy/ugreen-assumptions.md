# UGREEN Deployment Assumptions v0.1

Status: Accepted

## Baseline Target

Planning target:
- UGREEN NASync `DXP4800 Plus`

## Publicly Verified Assumptions

- UGREEN documents Docker and Docker Compose usage on UGOS Pro
- DXP-series devices are Docker-friendly
- DXP4800 Plus is an Intel-based NAS with enough baseline headroom for a small internal control plane

## Deployment Shape

Phase 1 assumes a single Compose project with:
- database control-plane service
- PostgreSQL metadata store
- optional telemetry sidecar or peer service

## Constraints

- persistent data must live on mapped NAS volumes
- secrets must be mounted or injected, never baked into images
- health endpoints must support Compose restart policies
- image strategy should remain compatible with `linux/amd64` first and `linux/arm64` second

## References

- [UGREEN Docker and Compose guide](https://nas.ugreen.com/blogs/knowledge/docker-docker-compose-ugreen-nas)
- [UGREEN DXP4800 Plus product page](https://nas.ugreen.com/products/ugreen-nasync-dxp4800-plus-nas-storage)
