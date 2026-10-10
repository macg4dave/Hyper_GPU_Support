---
agent: agent
description: Update build or CI automation without weakening quality gates
---

Make the requested build, packaging, or CI change.

Follow [AGENTS.md](../../AGENTS.md) and
[required checks/CI](../../docs/ENGINEERING.md#required-checks-and-ci).

- Inspect the actual Cargo, toolchain, lockfile and workflows. Preserve strict
  Windows checks, failure reporting and separately selected privileged tests.
- Keep automation proportional to the CLI; avoid unrelated platforms or release
  infrastructure. Pin downloaded inputs and retain license/provenance details.
- At an authorised validation gate, check affected commands and feature sets. Update build documentation only
  when commands or coverage change; report exact results and unrun checks.

[ROADMAP](../../docs/ROADMAP.md) and [BACKLOG](../../docs/BACKLOG.md) own priorities.
Follow [AGENTS testing and permission rules](../../AGENTS.md#testing-policy--codex);
report deferred validation. This prompt does not authorise live tests.
