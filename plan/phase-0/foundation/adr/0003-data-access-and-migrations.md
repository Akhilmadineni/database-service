# ADR 0003: Data Access and Migration Strategy

Status: Accepted

## Decision

Use PostgreSQL as the control-plane metadata store and `sqlx` for data access. Schema evolution is forward-only SQL managed with `sqlx migrate`.

## Rules

1. runtime SQL lives only in `store`
2. prefer checked `sqlx` query macros for stable queries
3. dynamic SQL is isolated behind repository methods
4. multi-table write paths use explicit transactions
5. API and worker layers never construct SQL

## Migration Workflow

- forward-only migrations
- one logical change set per migration
- PR review includes lock/risk notes
- destructive rollback migrations are not the recovery strategy

## Offline Query Metadata

- check in `.sqlx`
- enforce `cargo sqlx prepare --check` in CI
- keep local development flow aligned with prepared metadata
