# Authentication and Authorization Model v0.1

Status: Accepted

## Authentication

Two layers:
1. mTLS between trusted services
2. signed service token on each request

## Authorization

Service-to-service scopes:
- `capsule.read`
- `capsule.write`
- `reconcile.run`
- `operation.read`

Optional resource narrowing:
- app prefixes
- environment restrictions

## Evaluation Order

1. verify TLS peer identity
2. verify token signature and expiry
3. verify issuer and audience
4. map scopes
5. authorize requested capsule action

## Least-Privilege DB Access

Metadata DB role:
- owns control-plane tables

Target PostgreSQL admin role:
- creates managed databases
- creates and alters managed roles
- alters ownership and grants inside capsule scope

Phase 1 note:
- minimize superuser usage and step down after bootstrap if possible
