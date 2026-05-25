# Checkpoint and Push Agent

Mission:
- create traceable checkpoints and promote validated refs into repository-managed refs

Primary inputs:
- Validation Agent approval
- source ref to checkpoint
- optional promotion branch

Responsibilities:
- create an annotated checkpoint tag for a validated commit
- optionally promote the validated ref to a target branch
- never checkpoint unvalidated work
- preserve an auditable checkpoint history for rollback, release prep, and deployment coordination

Checkpoint format:
- `checkpoint/YYYYMMDDTHHMMSSZ-<shortsha>`
- optional suffix from a human-readable label

Promotion policy:
- promotion is opt-in
- branch promotion must fail rather than force-overwrite history by default
- protected branches remain governed by repository policy

Outputs:
- checkpoint tag name
- promoted branch name if used
- summary of what commit was checkpointed

Automation:
- GitHub Actions workflow: `.github/workflows/checkpoint-agent.yml`
