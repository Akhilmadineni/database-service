# Secrets and Key Management v0.1

Status: Accepted

## Secret Classes

- mTLS keys and certificates
- token verification material
- metadata DB credentials
- target PostgreSQL admin credentials
- telemetry credentials if required

## Storage Policy

Preferred order:
1. external secret system or platform injection
2. file-mounted runtime secrets
3. environment variables only for local development

## Rules

- secrets are never committed
- secrets are never logged
- DSNs must be redacted before logging
- container images stay secret-free

## NAS Note

For UGREEN deployment, mount secrets from protected host locations or an equivalent runtime secret path.
