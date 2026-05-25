# Dashboard Specification

Use this as the operator-facing dashboard baseline.

Panels:
- API request rate by route and status from `database_service_http_requests_total`
- reconciliation outcomes by state from `database_service_reconcile_results_total`
- capsule states from SQL queries or future metrics
- latest blocked and failed operations from the metadata database

Primary questions:
- are reconcile operations succeeding or rolling back?
- are specific routes failing more often than expected?
- are capsules getting stuck outside `converged`?
