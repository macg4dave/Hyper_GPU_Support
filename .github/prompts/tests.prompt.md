---
agent: agent
description: Add meaningful behavioral, failure, and regression coverage
---

Add focused tests that verify the requested behavior and its relevant failure paths.

Follow [AGENTS.md](../../AGENTS.md) and the engineering standards for
[testing](../../docs/ENGINEERING.md#testing) and
[required checks](../../docs/ENGINEERING.md#required-checks-and-ci).

- Prefer deterministic tests for parsing, policy, API, state transitions, and
  error paths. Choose unit, integration, regression, or documentation tests to
  verify behavior at the appropriate boundary; avoid implementation-mirroring tests.
- Cover empty, malformed, missing, inconsistent, minimum, maximum, overflow,
  alignment, unsupported-platform, and cleanup cases when relevant.
- Separate hardware-independent tests from explicitly selected administrator,
  Hyper-V, and GPU tests; use simple controlled interfaces where needed.
- Use condition-based synchronization and bounded waits instead of arbitrary
  delays. Isolate temporary files and resources and verify cleanup on failure.
- Do not weaken assertions or hide failures. Document coverage gaps and the
  alternative validation for logic that cannot reasonably be automated.
- Follow the repository's hardware evidence and protected-operation rules when
  validating GPU-PV; report exact checks and untested matrix entries.
