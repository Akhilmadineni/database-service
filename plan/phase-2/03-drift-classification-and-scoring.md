# Workstream 03: Drift Classification and Scoring

Goal:
- implement deterministic drift classification and initial scoring

Deliverables:
- diff model between desired and observed state
- drift classes: `safe`, `warning`, `critical`
- initial drift scoring rules

Tasks:
- [x] compare normalized desired state against observed state
- [x] classify drift items by mutation risk and scope
- [x] add score inputs for ownership, grants, role settings, and runtime safety conditions
- [x] produce deterministic planner-facing drift output

Done criteria:
- repeated comparison of the same desired and observed inputs yields identical drift output
- drift classes match the ACR state machine and safety model

Status notes:
- 2026-05-24: Added `core/src/drift.rs` with deterministic drift items, severity classes, and scoring.
- 2026-05-24: Added `core/src/plan.rs` so drift output deterministically maps to planner stages.
