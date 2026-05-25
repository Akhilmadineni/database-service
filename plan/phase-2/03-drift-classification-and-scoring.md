# Workstream 03: Drift Classification and Scoring

Goal:
- implement deterministic drift classification and initial scoring

Deliverables:
- diff model between desired and observed state
- drift classes: `safe`, `warning`, `critical`
- initial drift scoring rules

Tasks:
- [ ] compare normalized desired state against observed state
- [ ] classify drift items by mutation risk and scope
- [ ] add score inputs for ownership, grants, role settings, and runtime safety conditions
- [ ] produce deterministic planner-facing drift output

Done criteria:
- repeated comparison of the same desired and observed inputs yields identical drift output
- drift classes match the ACR state machine and safety model
