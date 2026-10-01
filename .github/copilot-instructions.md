# Copilot repository instructions

Follow [AGENTS.md](../AGENTS.md), including its bounded context-loading workflow
and [development/test authorization](../AGENTS.md#development-and-test-authorization).
Normal development and designated-disposable-VM testing proceed automatically;
physical-host lifecycle always requires permission immediately beforehand.

Use [docs/ENGINEERING.md](../docs/ENGINEERING.md) as the authoritative Rust,
testing, quality and [shell-script standard](../docs/ENGINEERING.md#shell-commands-and-development-scripts);
load the sections relevant to the active task.
Use [docs/BACKLOG.md](../docs/BACKLOG.md) for task status, acceptance, blockers,
and handover. Use a matching prompt in `.github/prompts/` only when invoked or
relevant; prompts add task-specific guidance to these shared rules.
