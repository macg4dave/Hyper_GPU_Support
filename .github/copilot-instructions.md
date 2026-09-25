# Copilot repository instructions

Follow [AGENTS.md](../AGENTS.md), including its bounded context-loading workflow
and [permission boundary](../AGENTS.md#permission-boundary). Routine repository
development proceeds automatically; only new privileged, destructive or
host-wide effects require approval.

Use [docs/ENGINEERING.md](../docs/ENGINEERING.md) as the authoritative Rust,
testing, and quality standard; load the sections relevant to the active task.
Use [docs/BACKLOG.md](../docs/BACKLOG.md) for task status, acceptance, blockers,
and handover. Use a matching prompt in `.github/prompts/` only when invoked or
relevant; prompts add task-specific guidance to these shared rules.
