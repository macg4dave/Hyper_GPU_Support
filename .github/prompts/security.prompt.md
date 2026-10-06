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
