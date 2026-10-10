---
agent: agent
description: Update repository documentation to match verified behavior
---

Make the requested documentation change.

Follow [AGENTS.md](../../AGENTS.md) and
[documentation standards](../../docs/ENGINEERING.md#documentation).

- Describe current implemented behavior from source; label prior measured results
  as historical and preserve ROADMAP's open stability/acceptance holds. Distinguish
  implementation from untested, unsupported or experimental capability.
- Describe complete host-matched driver/runtime discovery and verified guest
  placement as the provisioning contract. Keep environment-specific counts/hashes
  in historical evidence/results; do not turn them into general requirements.
- Update only affected documents, contracts and examples. Fix trivial drift inline;
  leave historical experiments outside normal reading paths.
- Preserve required authorship/notices. Check touched links and runnable examples;
  avoid an unrelated documentation audit or repeated hardware validation.

[ROADMAP](../../docs/ROADMAP.md) and [BACKLOG](../../docs/BACKLOG.md) own priorities.
Follow [AGENTS testing and permission rules](../../AGENTS.md#testing-policy--codex);
report deferred validation. This prompt does not authorise live tests.
