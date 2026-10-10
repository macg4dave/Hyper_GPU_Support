---
agent: agent
description: Implement the next ready product slice with bounded context
---

Follow [AGENTS](../../AGENTS.md) and affected [ENGINEERING](../../docs/ENGINEERING.md)
sections. [ROADMAP](../../docs/ROADMAP.md) owns delivery order;
[BACKLOG](../../docs/BACKLOG.md#next-implementation-action) owns the next action.
Use the requested card, or the next ready GUI slice when none is named.

- Check Git state; preserve overlapping edits. Read the card, affected source and
  only contracts needed for implementation or safety.
- Trace existing Rust entry points before coding. Connect the approved Slint
  controls to working shared functionality; make routine decisions independently.
- Keep enrollment, reviewed plans, privilege, downtime consent and recovery intact.
  If blocked, record the exact missing boundary and take another safe GUI slice.
- Author focused tests for meaningful new logic. Follow AGENTS milestone testing
  policy; no automatic tests/builds/UI launches or live tests on task completion.
- Record a short result on the owning card: behavior, code, blocker, next source
  task and validation pending. No new audit, test campaign or planning document.
