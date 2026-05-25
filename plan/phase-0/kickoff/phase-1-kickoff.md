# Phase 1 Kickoff

Status: Ready

## Objective

Build the Phase 1 control-plane data model and idempotent operation baseline for the Rust-first database control plane.

## Entry Criteria

- ACR mechanism locked
- prior-art and claim draft exist
- Rust ADR set exists
- API contract exists
- control-plane schema exists
- security baseline exists
- CI baseline exists
- UGREEN assumptions exist

## Phase 1 Scope

- create Cargo workspace
- implement metadata schema and migrations
- implement capsule and operation persistence
- implement idempotent create/update/reconcile request handling
- implement operation journal writes
- expose health, capsule, reconcile, and operation endpoints
