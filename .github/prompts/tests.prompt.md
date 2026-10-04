---
agent: agent
description: Add meaningful behavioral, failure, and regression coverage
---

Add focused tests that verify the requested behavior and its relevant failure paths.

Follow [AGENTS.md](../../AGENTS.md) and the engineering standards for
[testing](../../docs/ENGINEERING.md#testing),
[Windows elevation and UAC](../../docs/ENGINEERING.md#windows-elevation-and-uac), and
[required checks](../../docs/ENGINEERING.md#required-checks-and-ci).

- Prefer deterministic tests for parsing, policy, API, state transitions, and
  error paths. Choose unit, integration, regression, or documentation tests to
  verify behavior at the appropriate boundary; avoid implementation-mirroring tests.
- Cover the meaningful normal, boundary and failure cases for the changed behavior;
  do not add a mechanical matrix of cases that cannot affect the implementation.
- Separate hardware-independent tests from explicitly selected administrator,
  Hyper-V, and GPU tests; use simple controlled interfaces where needed.
- Use condition-based synchronization and bounded waits instead of arbitrary
  delays. Isolate temporary files and resources and verify cleanup on failure.
- Do not weaken assertions or hide failures. Document coverage gaps and the
  alternative validation for logic that cannot reasonably be automated.
- Follow the repository's hardware evidence and designated-target rules when
  validating GPU-PV; run required disposable-VM operations autonomously and report
  exact checks and untested matrix entries.
