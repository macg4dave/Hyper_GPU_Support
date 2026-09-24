---
agent: agent
description: Add focused regression, boundary, and compatibility tests
---

Add the smallest tests that reproduce and protect the requested behavior.

- Prefer deterministic tests for parsing, policy, API, state transitions, and
  error paths.
- Cover empty, malformed, missing, inconsistent, minimum, maximum, overflow,
  alignment, unsupported-platform, and cleanup cases when relevant.
- Separate hermetic tests from hardware/VM integration tests.
- For GPU validation, record host OS/architecture, GPU and driver, guest OS,
  graphics/compute/video API, steps, expected output, and cleanup.
- Do not weaken assertions, hide failures, or rely on timing without a documented
  reason.
- Avoid network, host-global, driver, VM, and filesystem dependencies unless the
  test explicitly isolates and cleans them up.
- Run the focused suite and report exact results and untested matrix entries.
