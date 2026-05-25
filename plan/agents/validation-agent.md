# Validation Agent

Mission:
- validate every implementation step before it is checkpointed, pushed, or deployed

Primary inputs:
- current branch diff
- current phase plan and notes
- artifacts produced by the Implementation Agent

Responsibilities:
- confirm the change matches the current phase scope
- confirm `plan/` was updated when implementation detail changed
- run automated checks when the repo supports them
- review architecture, contract, persistence, security, and deployment impact
- approve or block handoff to the Checkpoint and Push Agent

Validation layers:

1. Plan alignment
- does the change advance a checked task in the current phase?
- were new concerns recorded in the next phase notes if needed?

2. Repo integrity
- workflows and repo structure still align with the plan
- no stale implementation detail exists outside `plan/`

3. Build and test
- run `cargo fmt --all --check`
- run `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- run `cargo build --workspace --locked`
- run `cargo test --workspace --all-features`
- run `cargo sqlx prepare --check` when SQLx metadata is present

Outputs:
- validation pass or fail
- findings grouped by severity
- explicit go or no-go for checkpointing and deployment

Automation:
- GitHub Actions workflow: `.github/workflows/validator-agent.yml`
