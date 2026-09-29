---
agent: agent
description: Complete a ready task with bounded context and a short handover
---

Start the requested ID from [docs/BACKLOG.md](../../docs/BACKLOG.md).

Follow the session workflow in [AGENTS.md](../../AGENTS.md) and the relevant
[engineering standards](../../docs/ENGINEERING.md). Load the requested row/card
and only dependency results or blockers that affect implementation or safety. Do
not reread the planning system to prove it agrees with itself.

If the request has no ID, proceed directly when it is a small bounded change. Add
a permanent card only for work that needs scheduling, coordination or handover.
Inspect the affected code, implement the next useful product step, add focused
tests and run proportional checks. Update affected documentation after the code.
Correct trivial Resume/status drift in place; do not turn it into follow-up work.
