# Repository Agent Guide

## Project

Build a Windows-native GPU-PV project for Windows 11 x64 host/guest and NVIDIA
RTX 5060 8 GB. Prefer Rust and native Windows/Hyper-V management.
AppSandbox is a technical reference for selective adaptation. First reproduce
its relevant GPU-PV behavior on the target, then improve it. Keep the core
GUI-independent; Linux/macOS and upstream architecture/API compatibility are
outside the initial scope.

## Read only what the task needs

| File | Owns / read when |
|---|---|
| [docs/BACKLOG.md](docs/BACKLOG.md) | Resume note, compact task register, task cards and blockers; normal session entry |
| [docs/ROADMAP.md](docs/ROADMAP.md) | Current milestone and exit criteria; selecting work or checking a milestone gate |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Current/proposed components, pinned upstream source map and validation contract; read linked sections |
| [docs/DECISIONS.md](docs/DECISIONS.md) | Important choices, rationale and upstream review log; only relevant decision IDs |
| [docs/CHANGELOG.md](docs/CHANGELOG.md) | Concise meaningful completed changes; append on completion, no routine history reread |

Task status/dependencies have one source: the backlog register. Cards hold
scope, acceptance and evidence. Blockers and the latest handover stay in that
same file. There is no separate manifest, blocker document or session log.
FORK_PLAN.md is a legacy redirect. Optional workflows in `.github/prompts/`
are read only when invoked or relevant; never load the entire collection.

## Session workflow

1. Read this guide, then BACKLOG's Resume, task register and blocker register.
   For a named ID, locate its heading with `rg -n "^## <ID>$" docs/BACKLOG.md`
   and read only that card, dependencies' result pointers and linked sections.
   If no ID is given, choose a ready task relevant to the user's scope; consult
   ROADMAP's current milestone. Do not start unrelated tasks automatically.
2. Check actual workspace state (Git status if metadata exists), dependencies,
   milestone gate, blockers and another agent's ownership. Before work, set
   the register status to `in progress` and add your session/agent to the card.
3. Work within acceptance criteria. Read affected code before editing; preserve
   unrelated user changes. Keep modules and native integration small.
4. Run relevant checks and verify acceptance. Set `completed` only with concise
   result evidence; otherwise record the exact next action or linked blocker.
   Update only affected architecture/decisions and add a short changelog entry
   for meaningful completed work. Update dependent readiness or the milestone
   pointer only when their gates are met.
5. Refresh the compact Resume note: completed IDs, changed files, outstanding
   problem/blocker and next recommended ID. Release an unfinished task to
   `ready` or `blocked` unless a named owner is still working. Durable results
   stay on the card; do not append a long transcript.

## Mandatory rules

- Use native Windows facilities; do not recreate Windows GPU-PV or import the
  surrounding AppSandbox product. Inspect upstream dependencies before adapting.
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

## Protected operations

Host drivers, network adapters, virtualization features, VMs, guest disks,
firmware settings, code-signing state, and files outside the repository are
protected. Read-only discovery is allowed when in scope. Obtain explicit user
approval before installing, removing, restarting, enabling, disabling, or
otherwise mutating any protected target; name the exact target, expected
effect, and recovery path.

Existing explicit authorization applies to its stated scope. Prepare concrete
operations before asking; a task status or handover note is not authorization.
