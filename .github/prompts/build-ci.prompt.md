---
agent: agent
description: Update build or CI automation without weakening quality gates
---

Make the requested build, packaging, or CI change.

- Inspect the Rust, native Windows, script, and workflow configuration that
  actually exists; do not invent build commands before implementation exists.
- Preserve failures and exit codes; do not hide, ignore, or silently retry them.
- Target Windows 11 x64 and keep automation minimal; do not add cross-platform
  build infrastructure or an elaborate release process.
- Do not add privileged host, driver, VM, network, or signing mutations to CI.
- Pin downloaded tools or artifacts where practical and document their source,
  checksum strategy, and license implications.
- Do not remove an existing check unless the task explicitly requires it and
  the replacement is documented.
- Update build/test documentation and backlog notes when commands or coverage
  change.
- Validate the smallest representative matrix available and report environment
  blockers exactly.
