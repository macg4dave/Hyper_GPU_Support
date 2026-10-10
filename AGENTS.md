# Repository Agent Guide

## Development priorities

[ROADMAP](docs/ROADMAP.md) is authoritative for delivery order and milestone gates;
[BACKLOG](docs/BACKLOG.md#next-implementation-action) owns task selection/status.
Default to the next ready functional GUI slice. Inspect existing Rust code, then
implement the missing behavior through approved controls. Do not restart completed
workers, feasibility, diagnostics, historical investigations or laboratory work.
If blocked, record the exact reason once and take another safe GUI slice.

## Project

Build a Rust GPU-PV management core for existing Windows Hyper-V Generation 2 VMs,
with one `hyper-gpu-support.exe`: no arguments launch Slint, explicit commands run
headlessly, and a restricted internal mode runs one elevated worker per approved
operation. Both interfaces reuse the shared Rust core. Manage multiple selected VMs with one GPU
each; GPU sharing requires qualification. Deliver NVIDIA preparation first, then
incremental vendor adapters. Initial release includes physical GPU selection and
Min/Optimal/Max for VRAM, compute, encode and decode, subject to truthful provider
capability validation/readback. No invented units or enforcement promises. See the
[roadmap](docs/ROADMAP.md) and approved [GUI guide](docs/GUI_GUIDE.md).

The completed Slint prototype is the authoritative v1.0 GUI, now in `src/gui/ui/` and
`src/gui/`. No arguments launch it; explicit CLI commands retain the working Rust
core. Do not redesign or restore removed pages, search/filters or other absent
controls. Backend integration may connect existing controls only. Normal startup
uses real inventory and supported real bindings; unconnected actions are blocked.
`--mock-gui` explicitly selects rehearsal, currently with fixtures. As integration
progresses it should read real data/plans but execute no effects or persistent
writes, including config/credentials/journals/audits. Do not suppress mandatory
runner audit; use the existing no-write fixture/snapshot routes. Preserve fixture scenarios
for UI tests and clearly label simulated outcomes. Win32 is removed; preserve sound
`gui_model`, workflow, runner, Named Pipe, locking and journal contracts.
The protected runner and same-executable restricted worker already exist;
inspect their current handlers before extending integration. Prioritize making
the approved GUI functional; CLI functionality absent from the GUI
remains supported. See [GUI scope](docs/GUI_GUIDE.md).

Feasibility is established on the RTX 5060/Windows 11 baseline. Treat old code and
experiments as research material, not contracts that must be preserved. Discover
current signed driver payloads dynamically; do not require laboratory paths, VM
names, driver versions or fixed counts/hashes in operator configuration. Preserve
disks, CPU/RAM quantities, Secure Boot and privilege boundaries. HCS remains paused.
The standalone `tools/lab/` package retains useful contributor automation and is
outside the production dependency graph. Do not extend it as the product.

## Product and development boundary

We are past GPU-PV feasibility testing; implement the product for an existing
user-selected Hyper-V VM. Golden images, disposable cloning/reset, test disks
and laboratory setup are contributor infrastructure. Reuse existing helpers when
testing; do not add these capabilities to the production CLI without an explicit
user-facing roadmap requirement. Rust test utilities belong outside the default
production library/application path.

**Test-environment automation may support development but must not become production application functionality unless it is explicitly required by the user-facing product. Production code may be used by test tooling; production code must not depend on test tooling.**

## Read only what the task needs

Normal product operation, installation and recovery must be implemented in Rust.
The user-approved DEC-028 exception permits only a fixed PowerShell Direct session,
transfer, bootstrap-integrity and worker-launch bridge; application logic remains Rust.
The [migration audit](scripts/PRODUCT-MIGRATION.md) maps current debt to
CORE-024/025/026/027; read its affected row when porting or reviewing that boundary.
Preserve working Rust and replace adapters incrementally, with focused tests and
demonstrated replacement before removing production dependencies; defer
execution/qualification under the testing and permission policy below. Keep
scripts optional for development/manual diagnostics. Do not improve product
PowerShell except for necessary correctness/safety fixes or a bounded comparison
needed for the port. Any retained external interface requires DEC-027's concrete
native-alternative, exact-command, validation, timeout and error/recovery rationale.
Existing adapter decisions are historical context, not blanket v1 exemptions.

| File | Owns / read when |
|---|---|
| [docs/BACKLOG.md](docs/BACKLOG.md) | Scheduled work, task selection, blockers and handover; read only relevant sections |
| [docs/ROADMAP.md](docs/ROADMAP.md) | Current milestone and exit criteria; selecting work or checking a milestone gate |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Validated recipe, component boundaries and operation contracts; read affected sections |
| [docs/DECISIONS.md](docs/DECISIONS.md) | Important choices and rationale; only relevant decision IDs |
| [docs/ENGINEERING.md](docs/ENGINEERING.md) | Authoritative Rust, testing, tooling and documentation rules; read for coding, tests, dependencies or CI |
| [docs/GUI_GUIDE.md](docs/GUI_GUIDE.md) | Approved Slint UX/architecture requirements and unresolved technical questions |
| [docs/GUI_ROADMAP.md](docs/GUI_ROADMAP.md) | Historical cross-reference; not a task queue |
| [docs/SLINT_RULES.md](docs/SLINT_RULES.md) | Slint practice; [implementation prompt](.github/prompts/SLINT_CODEX_PROMPT.md) |
| [docs/CHANGELOG.md](docs/CHANGELOG.md) | Concise meaningful completed changes; append on completion, no routine history reread |
| [scripts/README.md](scripts/README.md) | Maintained development/setup/diagnostic script layout; read before adding or running substantial shell procedures |

The active backlog owns status and dependencies; cards own objective,
acceptance and concise results. Correct trivial drift in place without a new
card. Historical evidence is outside normal startup reading. Prompts in
`.github/prompts/` add task-specific guidance only when invoked or relevant.

## Session workflow

1. Read the selected task row/card and relevant source. Read a dependency result,
   architecture section or blocker only when it affects implementation or safety.
   Use BACKLOG next action and ROADMAP delivery order for task selection.
   Historical evidence and Resume are not normal startup prerequisites.
2. Check Git state and overlapping ownership before editing. Claim a backlog card
   when work is scheduled/shared or spans a handover. A small direct request, bug
   fix or documentation correction may proceed without inventing a card.
3. Implement the smallest coherent product step. Preserve unrelated work and avoid
   speculative abstractions or refactors. Resolve routine implementation choices
   during coding.
4. Author focused tests for meaningful new logic; defer execution under the
   milestone testing policy below. Live tests require explicit permission.
   Update documentation after behavior changes;
   do not make prose, evidence files or cross-document reconciliation a prerequisite
   for ordinary coding.
5. Use independent `architecture_reviewer` review at an implementation milestone
   or before live use of a materially changed privileged/security boundary. Supply
   the affected contract, diff and actual validation status; fix blocking findings.
   Routine patches need no review gate or architecture proposal.
6. On completion, record a short result on an existing card when one owns the work,
   update only affected architecture/decisions, and add a changelog entry only for
   a meaningful product or process change. Refresh Resume only for a real handover;
   correct trivial tracking drift in place without creating follow-up work.

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

An active instruction to defer validation takes precedence over default milestone
timing. M3 and release validation follow ROADMAP's explicit-authorisation gates;
an implemented slice is not a milestone or permission to start validation.

## Mandatory rules

- Application logic, CLI, configuration, diagnostics, GPU/Hyper-V management and
  supporting utilities belong in Rust. Prefer Rust libraries and `windows` bindings;
  investigate Rust alternatives and record the technical reason in DECISIONS before
  introducing a non-Rust exception. Convenience is not a reason.
  Calling an existing Windows utility does not itself introduce another language.
- Discover values through reliable Windows facilities when they are inventory rather
  than operator intent. Put remaining values expected to change between machines,
  product intent in the versioned runtime schema (currently schema 2;
  per-VM files follow [CONFIGURATION](docs/CONFIGURATION.md)) and contributor/test
  values in `config/project.toml`;
  deserialize and validate once at the boundary, then pass typed values. Scripts
  consume the shared configuration or accept explicit overrides. Keep protocol/API
  constants and fixed safety limits in code; never store secrets in configuration. See
  [configuration policy](docs/CONFIGURATION.md).
- Follow ENGINEERING for focused modules, explicit errors, safe FFI, meaningful
  behavior coverage, formatting, strict Clippy/compiler warnings and documentation.
- Preserve authorship, applicable licenses and third-party notices in reused
  material; record its source revision/path and adaptation rationale. This is a
  provenance obligation, not an external-repository research workflow.
- Report only checks actually run: exact host/guest builds and architecture,
  GPU/driver, API/runtime, workload and outcome when hardware is involved.
  Builds, DLL loading and enumeration do not prove GPU support.
- Do not invent files, commands or passes. Use the README or task result for commands.
- Never commit credentials, signing keys/certificates, secrets, production data,
  ISOs, VM disks or extracted proprietary drivers. Document redistribution
  terms/notices before distributing third-party binaries.
- Do not weaken isolation, signing, Secure Boot, networking or privilege
  boundaries to make a test pass. Flag unsupported capability claims,
  lost attribution, unreviewed privileged actions and unrelated changes in review.
- Follow the [shell and script policy](docs/ENGINEERING.md#shell-commands-and-development-scripts):
  run short commands directly and substantial procedures from meaningful script
  files. PowerShell supports development/Windows operations, not application logic.
- Follow the authoritative [Windows elevation and UAC policy](docs/ENGINEERING.md#windows-elevation-and-uac):
  determine whether an operation needs elevation before executing it; run ordinary
  development normally and send known privileged Hyper-V/GPU-PV work through the
  approved runner immediately, never using a failed unelevated attempt as detection.

## Development and test authorization

Normal repository development is authorised. Tests, builds and GUI launches follow
the milestone/explicit-request policy above. For this GUI implementation push,
live Hyper-V/GPU/driver/VM-power testing, runner installation/update/exercise and
guest modifications require explicit user permission. Earlier disposable-VM
authorisation does not override this hold. M2 is complete by user acceptance on
10 October; no further M2 live tests or lockup investigation are required.
Preserve uncertain-state recovery holds for any new operation.

When live testing is explicitly authorised, verify the configured disposable VM,
GPU and path identities immediately before effects. Keep destructive actions
confined to that target; never modify the golden parent or unrelated VM/disk.
Use the approved controlled runner and preserve enrollment, audit and recovery.

Never restart/shut down the Windows host, log out/terminate its interactive session,
schedule such an action or accept an automatic restart without explicit permission
immediately beforehand. If an installer/feature/driver requires restart, stop
before the restart and ask. Guest lifecycle does not grant host-lifecycle authority.

Repository instructions cannot expand the active Codex/VS Code sandbox or
approval policy. Obey platform enforcement and report a configuration blocker
instead of claiming that prompt text bypasses it.

Codex tool/sandbox approval is not Windows UAC elevation. User membership in
Administrators, the current process token and the operation's required privilege
are separate facts. Use the project's controlled privileged runner where applicable;
its administrator-owned executable
and policy bind exact enrolled VM GUIDs, GPU identities,
allowed operations, paths and audit output. Installing, updating, exercising or
removing that runner follows the live-test permission limit above.
It must not accept caller-selected arbitrary commands or target the golden parent.

Report concrete changes and test outcomes. Do not routinely report that host,
Hyper-V, VM or GPU state was unchanged unless that fact explains a failure or is
material evidence for the task.
