# Workstream 02: Fingerprint Engine

Goal:
- implement canonical desired and observed fingerprint generation

Deliverables:
- desired fingerprint generator
- observed-state projection model
- fingerprint domain separation and hashing utilities

Tasks:
- [ ] move request-hash logic into a dedicated fingerprint module
- [ ] implement desired-state canonicalization exactly once
- [ ] define observed-state capture shape for catalog and runtime facts
- [ ] generate `desired_fingerprint`, `observed_fingerprint`, `plan_fingerprint`, and `checkpoint_fingerprint` through one shared interface
- [ ] add deterministic fixtures for equivalent-input hashing

Done criteria:
- identical normalized inputs always yield identical fingerprints
- fingerprint domains match the ACR spec
