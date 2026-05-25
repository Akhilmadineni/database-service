# Workstream 01: Capsule Spec and Validation

Goal:
- implement the normalized capsule spec parser and validation pipeline

Deliverables:
- typed capsule domain model aligned to ACR scope
- validation rules for identity, role, privilege, protection, and safety sections
- deterministic normalization pass

Tasks:
- [ ] split external API payload types from normalized internal capsule types
- [ ] enforce capsule validation rules from the ACR spec
- [ ] normalize unordered sets and defaulted values before fingerprinting
- [ ] reject out-of-scope or unsupported policy sections explicitly

Done criteria:
- semantically equivalent inputs normalize to the same internal capsule representation
- invalid capsule specs fail before planning begins
