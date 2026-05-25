# ACR Benchmark Plan v0.1

Status: Phase 0 baseline

## Purpose

Measure the technical advantages of ACR in a reproducible way.

## Claims to Validate

- deterministic plan generation
- stable canonical fingerprints
- fast safe drift correction
- reversible remediation for covered mutation classes
- safety gating under runtime session and lock pressure

## Benchmarks

### B1: Canonicalization Stability

Input:
- semantically equivalent capsule specs with different formatting and ordering

Success:
- identical desired fingerprints

### B2: Plan Determinism

Input:
- repeated planning runs over the same observed-state fixture

Success:
- identical plan fingerprints and stage ordering

### B3: Safe Drift Mean Time to Correct

Input:
- seeded safe drift such as missing grants, owner mismatch without active sessions, connection-limit mismatch

Success:
- drift-detection-to-converged time remains inside the target SLO window

### B4: Rollback Success Rate

Input:
- injected failures after apply stage N

Success:
- rollback-eligible runs restore checkpointed safe state without manual SQL

### B5: Safety Gate Precision

Input:
- active sessions, long-running locks, conflicting ownership cases

Success:
- unsafe mutations are blocked or deferred with no false-safe outcomes in the benchmark corpus

## Evidence Package

Capture for each run:
- capsule input
- normalized desired state
- observed-state fixture or capture
- all fingerprints
- stage timeline
- final status
- rollback artifacts if any
