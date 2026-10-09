---
agent: agent
description: Review and harden host, guest, API, driver, and parsing boundaries
---

Review or harden the requested area.

Follow [AGENTS.md](../../AGENTS.md) and relevant
[engineering standards](../../docs/ENGINEERING.md).

- Check untrusted inputs, FFI lifetimes/ranges, injection, paths, privilege,
  target identity, secrets, isolation and dependency provenance.
- Check bounded operations, cleanup and recovery after partial failure.
- Keep fixes focused; add meaningful regression coverage and update changed
  security contracts. Live tests stay within the authorised disposable target.
  Report exact checks and material remaining risks.
- Native migration must preserve enrolled VM/GPU/disk identity, parent protection,
  exact-SID rights, ACL/reparse guards, credential handling and reconciliation.
  Reject arbitrary elevated/guest command channels. A retained external interface
  needs DEC-027's narrow technical rationale and validated bounded results.

## Testing Policy — Codex

Stop running tests, builds, compilation checks, or launching the UI after every small change.

Follow these rules:

1. **During development:** Make changes without automatically running tests, `cargo check`, `cargo test`, `cargo build`, or launching the GUI.
2. **At milestone completion:** Run relevant tests and build checks once, after all work for that milestone is complete.
3. **On explicit request:** Run tests whenever I specifically instruct you to.
4. **Small changes:** Do not test individual edits, UI adjustments, layout changes, refactoring, or documentation updates.
5. **Failures:** If a milestone test fails, fix the relevant issue and rerun only the necessary checks. Avoid repeatedly running the entire test suite.
6. **Exceptions:** If you believe immediate testing is essential, explain why and request permission first.

Prioritise implementing the planned work over repeatedly validating intermediate states.

**Important:** Do not interpret completing an individual task or subtask as completing a milestone. A milestone is complete only when all its planned tasks are finished.

At the end of each task, briefly report what changed and whether it remains untested. Do not automatically start validation.

This policy overrides existing instructions to test continuously unless I explicitly tell you otherwise.
