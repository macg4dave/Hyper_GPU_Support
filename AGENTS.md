# Repository Agent Guide

## Project

Build a Windows-native GPU-PV project for Windows 11 x64 host/guest and NVIDIA
RTX 5060 8 GB. Implement in Rust wherever technically possible, using native
Windows/Hyper-V facilities. Follow the [engineering standards](docs/ENGINEERING.md).
AppSandbox is a technical reference for selective adaptation. First reproduce
its relevant GPU-PV behavior on the target, then improve it. Keep the core
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
   change. Protected host/VM operations still require the permission boundary and
   technical preconditions below. Update documentation after behavior changes;
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
- Put values expected to change between machines, VM/image/driver revisions or test
  runs in the authoritative `config/project.toml`; deserialize and validate once at
  the boundary, then pass typed values. Scripts consume the shared configuration or
  accept explicit overrides. Keep protocol/API constants and fixed safety limits in
  code, do not duplicate current environment values in prose, and never store secrets
  in repository configuration. See [configuration policy](docs/CONFIGURATION.md).
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

## Permission boundary

Permission follows an operation's effect, not whether it writes a file or runs a
command:

| Category | Agent behavior |
|---|---|
| Routine repository development | Proceed without approval. This includes creating, editing, moving and deleting project files; Rust modules, documentation, prompts, task records, dependencies and test fixtures; Cargo build/check/fmt/Clippy/test commands; non-destructive Git inspection; small refactors and warning fixes; and targeted cleanup of generated artifacts inside this repository after verifying the target. |
| Previously authorized test operation | Repeat without asking while the exact approved target, operation set, identity, paths and recovery boundary still match. Stop when the authorization or observed target no longer matches. |
| New privileged, destructive or host-wide operation | Obtain explicit user approval after naming the exact target, effect and recovery path. |

The last category includes Windows administrator elevation; host drivers,
network adapters, virtualization features, security settings, firmware or
code-signing changes; mutations to VMs or guest disks outside an approved test
mechanism; physical-disk operations; deletion outside the repository; changes
to the golden VM image or unrelated VMs/disks; destructive Git operations that
discard work; unnecessary credential/secret access; and broad cleanup with an
unverified target. Read-only discovery is allowed when it is in scope.

Routine repository work is authorized by the assigned task and needs no extra
confirmation. A task status alone does not authorize a protected operation.
Existing explicit authorization applies only to its stated scope.

Host-session termination has an additional hard boundary: never restart or shut
down the Windows host, log out or terminate its user session, schedule a restart,
or permit an installer/update to restart it automatically without the user's
explicit permission first. Explain why the lifecycle action is required and wait
for that permission before proceeding. Each occurrence needs its own permission
and cannot use the reusable-authorization category. Administrator access and the
approved elevated test mechanism do not imply host lifecycle consent.

An explicitly authorized disposable Hyper-V test VM is a guest, not the host. Its
start, graceful stop, restart, reset or recreation may proceed as normal GPU-PV
testing only after immediately verifying its pinned identity and the authorization's
target/operation scope. Never modify, restart or delete the golden master or an
unrelated VM without explicit permission. Guest lifecycle commands must be scoped
so they cannot invoke or schedule a host lifecycle action.

Repository instructions cannot expand the active Codex/VS Code sandbox or
approval policy. Obey platform enforcement and report a configuration blocker
instead of claiming that prompt text bypasses it.

Codex tool/sandbox approval is not Windows UAC elevation. Never run the editor,
agent or arbitrary repository binaries with a general administrator token merely
to make hardware iteration easier. A user-approved privileged test runner may
repeat an already authorized operation only when its administrator-owned executable
and policy pin one disposable slot, its runner-owned current VM-GUID enrollment,
GPU identity, allowed operation set, paths and audit output. The agent must not be
able to replace that executable, edit policy/enrollment, choose a replacement VM,
target the golden parent image or submit arbitrary commands. Installing,
updating, broadening or removing the runner is itself a protected operation needing
new explicit approval and a recovery/revocation path.
