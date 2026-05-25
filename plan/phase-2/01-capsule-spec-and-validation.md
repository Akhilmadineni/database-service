# Workstream 01: Capsule Spec and Validation

Goal:
- implement the normalized capsule spec parser and validation pipeline

Deliverables:
- typed capsule domain model aligned to ACR scope
- validation rules for identity, role, privilege, protection, and safety sections
- deterministic normalization pass

Tasks:
- [x] split external API payload types from normalized internal capsule types
- [x] enforce capsule validation rules from the ACR spec
- [x] normalize unordered sets and defaulted values before fingerprinting
- [x] reject out-of-scope or unsupported policy sections explicitly

Done criteria:
- semantically equivalent inputs normalize to the same internal capsule representation
- invalid capsule specs fail before planning begins

Status notes:
- 2026-05-24: Added `NormalizedCapsule` and related typed normalized structures in `core/src/capsule.rs`.
- 2026-05-24: Validation now enforces identifier shape, owner-role presence, connection-limit bounds, and normalized ordering of roles and schemas.
