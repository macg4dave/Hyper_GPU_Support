---
name: security-recovery-review
description: Review a scoped change across authority, trust, and recovery boundaries.
---

Follow [AGENTS.md](../../AGENTS.md) and [security/recovery](../../docs/SECURITY.md). Consult [architecture](../../docs/ARCHITECTURE.md), [configuration](../../docs/CONFIGURATION.md), and the relevant [backlog](../../docs/BACKLOG.md) task.

- Trace inputs through operator/pair authority, fixed elevation, process/session authentication, bounds, artifact/ACL/path trust, operation locking, and fresh approval recomputation.
- Follow durable admission through possible effects, independent verification, publication, disconnect, and readback. Uncertainty must not become automatic effect replay.
- Keep setup, enrollment, GPU effects, standalone Verify, save-only, and import recovery distinct. Intent/imported files cannot create authority.
- Review credential lifetime and secret-free diagnostics; distinguish safe typed context from arbitrary adapter output.
- Record concrete findings, source evidence, affected scenarios, proposed corrections, and missing coverage. Mark untested boundaries as untested.

Do not probe installed IPC, edit protected files, access guests, or induce faults during source review. Tests/trials follow [testing](../../docs/TESTING.md) and the current user's authorization.
