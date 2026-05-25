# Agent System

These four agents coordinate delivery across phases. `plan/` remains the source of truth for what they do, what they read, and what they hand off.

## Agents

- [implementation-agent.md](./implementation-agent.md)
- [validation-agent.md](./validation-agent.md)
- [checkpoint-and-push-agent.md](./checkpoint-and-push-agent.md)
- [nas-deploy-agent.md](./nas-deploy-agent.md)

## Handoff Order

1. Implementation Agent
2. Validation Agent
3. Checkpoint and Push Agent
4. NAS Deploy Agent

## Core Rule

If implementation detail changes, the current phase folder must be updated before handoff to the next agent.
