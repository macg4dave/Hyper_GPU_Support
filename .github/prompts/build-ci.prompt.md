---
agent: agent
description: Update build or CI automation without weakening quality gates
---

Make the requested build, packaging, or CI change.

Follow [AGENTS.md](../../AGENTS.md) and the engineering standards for
[Rust and Windows](../../docs/ENGINEERING.md#rust-and-native-windows),
[toolchains and dependencies](../../docs/ENGINEERING.md#toolchain-dependencies-and-features),
and [required checks and CI](../../docs/ENGINEERING.md#required-checks-and-ci).

- Inspect the actual Cargo, toolchain, lockfile, feature, and workflow setup;
  distinguish established commands from checks still awaiting implementation.
- Keep every-PR quality checks reproducible on the supported Windows target.
  Keep hardware or privileged validation separate from the default test suite.
- Preserve failures and exit codes; correct their causes instead of suppressing
  warnings, weakening assertions, or bypassing gates.
- Keep automation proportional to this project; avoid unrelated platforms or
  release infrastructure.
- Pin downloaded tools or artifacts where practical and document their source,
  checksum strategy, and license implications.
- Document the reason and replacement coverage for any intentionally changed
  check. Update build/test documentation when commands or coverage change.
- Validate the affected supported feature combinations and report exact commands,
  results, and environment blockers; do not claim an unrun CI or hardware result.
