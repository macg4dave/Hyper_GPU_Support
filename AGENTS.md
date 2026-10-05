# Repository Agent Guide

## Project

Build a Windows-native GPU-PV project for Windows 11 x64 host/guest and NVIDIA
RTX 5060 8 GB. Implement in Rust wherever technically possible, using native
Windows/Hyper-V facilities. Follow the [engineering standards](docs/ENGINEERING.md).
Easy-GPU-PV is the primary reference for normal Hyper-V VM configuration,
GPU assignment and driver staging. AppSandbox is a secondary GPU-PV technical
reference for selective adaptation. HCS-owned-guest work is paused; keep the
product around a normal Generation 2 Hyper-V Windows 11 VM. First reproduce
the relevant GPU-PV behavior on the target, then improve it. Keep the core
GUI-independent; Linux/macOS and upstream architecture/API compatibility are
outside the initial scope.

## Read only what the task needs

| File | Owns / read when |
|---|---|
| [docs/BACKLOG.md](docs/BACKLOG.md) | Scheduled work, task selection, blockers and handover; read only relevant sections |
| [docs/ROADMAP.md](docs/ROADMAP.md) | Current milestone and exit criteria; selecting work or checking a milestone gate |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Current/proposed components, pinned upstream source map and validation contract; read linked sections |
| [docs/DECISIONS.md](docs/DECISIONS.md) | Important choices, rationale and upstream review log; only relevant decision IDs |
| [docs/ENGINEERING.md](docs/ENGINEERING.md) | Authoritative Rust, testing, tooling and documentation rules; read for coding, tests, dependencies or CI |
| [docs/CHANGELOG.md](docs/CHANGELOG.md) | Concise meaningful completed changes; append on completion, no routine history reread |
| [scripts/README.md](scripts/README.md) | Maintained development/setup/diagnostic script layout; read before adding or running substantial shell procedures |

Task status/dependencies have one source: the backlog register. Cards stay
concise: objective, genuine dependencies, acceptance and result. Blockers and
the latest handover stay in that same file. There is no separate manifest,
blocker document or session log. A stale summary or status is corrected in
place; it is not a new task unless it conceals a technical or safety risk.
Optional workflows in `.github/prompts/` add task-specific guidance; read them
only when invoked or relevant, never the entire collection. Agent-specific
instructions link here and to engineering standards instead of duplicating them.

## Session workflow

1. Start from the user's requested outcome. For a named ID, read its register row,
   card and only the dependency results or linked technical sections needed to
   implement it. Read the Resume, blocker register or milestone gate only when
   selecting work, resolving an actual impediment or closing a milestone.
2. Check Git state and overlapping ownership before editing. Claim a backlog card
   when work is scheduled/shared or spans a handover. A small direct request, bug
   fix or documentation correction may proceed without inventing a card.
3. Inspect the affected implementation and choose the smallest useful product
   step. Reuse established test results unless the change or a suspected regression
   makes a fresh baseline useful. Preserve unrelated work and avoid speculative
   refactors. Small implementation choices may be made during coding when they do
   not change architecture, security boundaries or public contracts.
4. Add focused tests for meaningful logic and run checks proportional to the
   change. Normal testing against the designated disposable VM proceeds under the
   authorization below. Update documentation after behavior changes;
   do not make prose, evidence files or cross-document reconciliation a prerequisite
   for ordinary coding.
5. Use independent `architecture_reviewer` review when closing an implementation
   milestone or before deploying a materially changed privileged/security boundary,
   not for every routine patch. Give the reviewer the applicable gate, diff and
   actual test evidence. Fix blocking findings and rerun affected checks.
6. On completion, record a short result on an existing card when one owns the work,
   update only affected architecture/decisions, and add a changelog entry only for
   a meaningful product or process change. Refresh Resume only for a real handover;
   correct trivial tracking drift in place without creating follow-up work.

## Mandatory rules

- Use native Windows facilities; do not recreate Windows GPU-PV or import the
  surrounding AppSandbox product. Inspect upstream dependencies before adapting.
- Application logic, CLI, configuration, diagnostics, GPU/Hyper-V management and
  supporting utilities belong in Rust. Prefer Rust libraries and `windows` bindings;
  investigate Rust alternatives and record the technical reason in DECISIONS before
  introducing a non-Rust exception. Upstream language or convenience is not a reason.
  Calling an existing Windows utility does not itself introduce another language.
- Keep code idiomatic, focused and modular, with explicit errors and safe interfaces
  around small, justified `unsafe` blocks. Cover every function with meaningful logic
  through behavior tests or document why testing is impractical and how it is validated.
- Discover values through reliable Windows facilities when they are inventory rather
  than operator intent. Put remaining values expected to change between machines,
  VM/image/driver revisions or test runs in the authoritative `config/project.toml`;
  deserialize and validate once at the boundary, then pass typed values. Scripts
  consume the shared configuration or accept explicit overrides. Keep protocol/API
  constants and fixed safety limits in code, do not duplicate current environment
  values in prose, and never store secrets in repository configuration. See
  [configuration policy](docs/CONFIGURATION.md).
- Enforce formatting, strict Clippy and compiler warnings; fix causes instead of
  broad lint suppressions. Document public interfaces, invariants and non-obvious
  Windows/FFI behavior. Follow ENGINEERING for the detailed rules and test lanes.
- Preserve upstream authorship/history in references and applicable MIT and
  third-party notices in reused material. Record source commit/path and rationale.
  Review selectively; do not routinely merge upstream into this codebase.
- Report only checks actually run: exact host/guest builds and architecture,
  GPU/driver, API/runtime, workload and outcome when hardware is involved.
  Builds, DLL loading and enumeration do not prove GPU support.
- Do not invent missing files, commands or passing results. Consult a task's
  result for established build/test commands; if none exist, state that.
- Never commit credentials, signing keys/certificates, secrets, production data,
  ISOs, VM disks or extracted proprietary drivers. Document redistribution
  terms/notices before distributing third-party binaries.
- Do not weaken isolation, signing, Secure Boot, networking or privilege
  boundaries to make a test pass. Flag unsupported capability claims,
  lost attribution, unreviewed privileged actions and unrelated changes in review.
- Follow the [shell and script policy](docs/ENGINEERING.md#shell-commands-and-development-scripts):
  run short commands directly, but put substantial PowerShell, conditional logic,
  complex pipelines and multi-step procedures in meaningful script files and run
  those files. Keep reusable tooling under `scripts/`; keep temporary scripts clearly
  separate. PowerShell supports development and Windows operations, not application logic.
- Follow the authoritative [Windows elevation and UAC policy](docs/ENGINEERING.md#windows-elevation-and-uac):
  determine whether an operation needs elevation before executing it; run ordinary
  development normally and send known privileged Hyper-V/GPU-PV work through the
  approved runner immediately, never using a failed unelevated attempt as detection.

## Development and test authorization

The AI may autonomously perform normal project development and approved testing
against the designated disposable VM. This includes building and running project
code; installing, updating and executing the controlled privileged runner;
configuring Hyper-V and GPU-PV; changing GPU resources; starting, stopping,
restarting, resetting or recreating the disposable guest; replacing its
differencing disk and configuration; provisioning and modifying guest files,
registry and NVIDIA components; running PowerShell Direct, probes and diagnostics;
and repeating experiments. Administrative access and non-rebooting host changes
needed for that workflow do not require another permission prompt.

Immediately verify the configured disposable VM, GPU and path identities before
effects. Never experimentally modify the golden parent or an unrelated VM/disk,
and keep destructive actions confined to the verified disposable target. These are
targeting requirements, not additional approval gates.

The sole recurring approval boundary for the agreed workflow is physical-host
lifecycle: never restart or shut down the Windows host, log out or terminate its
interactive session, schedule such an action, or accept an automatic restart
without the user's explicit permission immediately beforehand. If an installer,
feature, driver or update reports that a host restart is required, stop before the
restart and ask. Guest lifecycle is not host lifecycle.

Repository instructions cannot expand the active Codex/VS Code sandbox or
approval policy. Obey platform enforcement and report a configuration blocker
instead of claiming that prompt text bypasses it.

Codex tool/sandbox approval is not Windows UAC elevation. User membership in
Administrators, the current process token and the operation's required privilege
are separate facts. Use the project's controlled privileged runner where applicable;
its administrator-owned executable
and policy pin one disposable slot, the current VM-GUID enrollment, GPU identity,
allowed operations, paths and audit output. Installing, updating, exercising or
removing that runner is normal project testing and may proceed autonomously.
It must not accept caller-selected arbitrary commands or target the golden parent.

Report concrete changes and test outcomes. Do not routinely report that host,
Hyper-V, VM or GPU state was unchanged unless that fact explains a failure or is
material evidence for the task.
