---
agent: agent
description: Replace one product PowerShell capability with demonstrated native Rust behavior
---

Implement the requested native migration card from
[BACKLOG.md](../../docs/BACKLOG.md), or select the next ready migration card if no
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
- Add meaningful success/failure/identity/timeout tests, compare affected output
  with the working implementation and qualify changed behavior against the proven
  baseline. Obtain independent review when the repository workflow requires it.
- Remove the production dependency only after the replacement is demonstrated.
  Keep a script only if useful as optional tooling. Record narrow interface
  limitations under DEC-027 with alternatives, exact invocation, validation,
  bounds and errors; convenience or a Rust shell wrapper is insufficient.
- Report actual changes/checks and remaining debt. Current adapter qualification
  does not prove native migration or close a port card.

GPU-PV feasibility is established; implement and validate the product for an
existing selected VM. Reuse disposable-VM tooling for tests. Golden-image copying,
cloning, disk reset and laboratory setup stay outside the production path unless
an explicit user-facing roadmap task requires them. Test tooling may depend on
product code; product code must not depend on test tooling.

The user-approved architecture rebase takes precedence over historical task scope.
Use runtime existing-VM identities and the revised core → native GUI → allocation/
vendor roadmap. Old fixed-slot code is research/contributor tooling in `tools/lab/`.
Do not preserve laboratory coupling, baseline driver pins or repeated diagnosis as
product architecture. Default operation verifies health plus checked graphics;
extended CUDA/stress remain optional. The core never imports the laboratory.
