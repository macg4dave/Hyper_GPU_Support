---
agent: agent
description: Complete a ready task with bounded context and a short handover
---

Start the requested ID from [docs/BACKLOG.md](../../docs/BACKLOG.md).

Follow the session workflow in [AGENTS.md](../../AGENTS.md) and the relevant
[engineering standards](../../docs/ENGINEERING.md). Load the requested row/card
and affected source, plus a dependency result only when needed. Claim shared or
handover work, implement, test and record a concise result. The proven baseline
needs no new feasibility or reference-research task. A small direct request needs
no invented card; fix trivial tracking drift inline.

When asked for the next native migration task, select from the current register:
CORE-024 inventory first, then CORE-025 Hyper-V operations; CORE-026 guest writer
and CORE-027 setup/recovery have separate ownership. Preserve existing in-progress
work. Read only the affected [audit](../../scripts/PRODUCT-MIGRATION.md) row and
contract. Completed orchestration/current-adapter qualification does not close a
native backend port. Implement the smallest replacement, not a broad rewrite.
Provisioning tasks reproduce dynamic associated driver/runtime discovery and guest
mapping/verification. No historical file count is a general acceptance condition;
driver updates generate a new host-matched manifest rather than reuse old entries.
