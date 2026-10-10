---
agent: agent
description: Replace one product PowerShell capability with demonstrated native Rust behavior
---

Implement the requested native migration card from
[BACKLOG.md](../../docs/BACKLOG.md), only when explicitly requested; otherwise use the next GUI action if no
ID is given. Follow [AGENTS.md](../../AGENTS.md) and
[ENGINEERING.md](../../docs/ENGINEERING.md); claim the selected work and preserve
overlapping edits. Read the affected [audit](../../scripts/PRODUCT-MIGRATION.md)
row and source, not unrelated historical experiments.

- Separate optional development/diagnostic tooling from capability required by
  ordinary CLI operation, installation or recovery. Embedded scripts also count.
- Reuse working Rust contracts and native helpers. Port one coherent boundary
  through Rust/native Windows bindings; retain bounded provider supervision and
  the enrolled fixed-operation protocol. Do not create an arbitrary command API.
- For provisioning, dynamically discover the selected signed host driver's complete
  associated payload and guest destinations; no fixed file count defines success.
  Historical manifests are regression data, not a permanent source list.
- Preserve exact targets, golden parent, isolation, credentials, ACL/reparse
  guards, preimages and uncertain-state reconciliation. Use the approved runner
  for known privileged testing; host lifecycle still requires explicit permission.
- Add meaningful success/failure/identity/timeout tests, retain compatibility coverage for the working implementation.
  Defer execution/qualification to the authorised milestone gate. Obtain independent review when the repository workflow requires it.
- Remove the production dependency only after the replacement is demonstrated.
  Keep a script only if useful as optional tooling. Record narrow interface
  limitations under DEC-027 with alternatives, exact invocation, validation,
  bounds and errors; convenience or a Rust shell wrapper is insufficient.
- Report actual changes/checks and remaining debt. Current adapter qualification
  does not prove native migration or close a port card.

[ROADMAP](../../docs/ROADMAP.md) and [BACKLOG](../../docs/BACKLOG.md) own priorities.
Follow [AGENTS testing and permission rules](../../AGENTS.md#testing-policy--codex);
report deferred validation. This prompt does not authorise live tests.
