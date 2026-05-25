# API Contract v0.1

Status: Phase 0 baseline

## Principles

- internal-only API
- JSON over HTTPS
- versioned under `/v1`
- asynchronous writes
- required idempotency for mutation

## Common Headers

Required on writes:
- `Authorization: Bearer <token>`
- `Idempotency-Key: <opaque-key>`

Recommended:
- `X-Request-Id: <opaque-request-id>`

## Endpoints

### GET `/healthz`
- liveness

### GET `/readyz`
- dependency readiness

### PUT `/v1/capsules/{app}/{environment}`
- create or replace desired capsule state
- returns `202 Accepted`

### GET `/v1/capsules/{app}/{environment}`
- inspect current desired state and latest reconciliation status
- returns `200 OK`

### POST `/v1/capsules/{app}/{environment}/reconcile`
- request an immediate reconciliation run
- returns `202 Accepted`

### GET `/v1/operations/{operation_id}`
- read async operation state
- returns `200 OK`

## Success Envelope

```json
{
  "data": {},
  "meta": {
    "request_id": "req_01",
    "operation_id": "op_01"
  }
}
```

## Error Envelope

```json
{
  "error": {
    "code": "capsule_blocked_unsafe",
    "message": "reconciliation blocked by active sessions",
    "details": {},
    "request_id": "req_01"
  }
}
```
