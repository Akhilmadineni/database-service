# ACR Claim Candidates v0.1

Status: Phase 0 baseline

These are engineering claim candidates for counsel refinement.

## Claim Strategy

Anchor the invention on:
- a database-scoped capsule boundary
- canonical desired and observed fingerprints
- deterministic staged remediation planning
- checkpoint-before-mutate
- invariant-based verification and automatic rollback
- a journal suitable for tamper-evident chaining

## Candidate Independent Method Claim

A computer-implemented method for autonomously reconciling a database tenant, comprising:

1. receiving a capsule specification for a database tenant identified by application and environment
2. normalizing the specification into canonical desired state
3. generating a desired-state fingerprint
4. observing catalog and runtime state for resources inside the capsule management boundary
5. generating an observed-state fingerprint
6. classifying drift from the desired and observed state
7. generating a deterministic ordered remediation plan limited to the management boundary
8. recording a reversible checkpoint package before executing the remediation plan
9. evaluating safety guards using runtime session information
10. executing plan stages when the safety guards pass
11. verifying invariants after execution
12. automatically rolling back executed stages using the checkpoint package when verification fails

## Candidate System Claim

A control-plane system comprising:
- an API component
- a compiler for canonical desired state
- an observer for catalog and runtime facts
- a planner for deterministic remediation stages
- a checkpoint manager
- an executor
- a verifier
- a journal subsystem

where the system automatically rolls back executed mutations when post-mutation verification fails.

## Candidate Dependent Themes

- sorted privilege-set canonicalization before fingerprinting
- exclusion of secret material from fingerprints
- checkpoint packages containing prior ownership, role settings, grants, and object-existence indicators
- session-count thresholds as safety gates
- rejection of same idempotency key with different request hash
- separate blocked and manual-intervention states

## Claim Risks

- broad “autonomous database” claims are crowded by operator and managed-service prior art
- claims that omit checkpoints and verification look too much like generic control loops
- claims that omit the capsule boundary look too much like cluster operators
