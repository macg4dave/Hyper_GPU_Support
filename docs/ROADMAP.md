# Hyper GPU Support — v1.0 Product Roadmap

**Updated:** 10 October 2026  
**Objective:** Ship the approved Slint interface as a functional Windows application, backed by the existing Rust GPU-PV core. **Implement product features first; do not restart feasibility research or continuous test refinement.**

## Document ownership

- **This roadmap:** delivery order, milestones and release gates.
- **[BACKLOG.md](BACKLOG.md):** current next action, implementation cards, blockers and task status.
- **[GUI_GUIDE.md](GUI_GUIDE.md):** approved, fixed v1.0 interface; do not expand or redesign it.
- **[CONFIGURATION.md](CONFIGURATION.md), [ARCHITECTURE.md](ARCHITECTURE.md), [DECISIONS.md](DECISIONS.md):** data, security and architectural contracts.
- **[AGENTS.md](../AGENTS.md) and [ENGINEERING.md](ENGINEERING.md):** coding, testing, permission and review rules.
- **[BACKLOG_HISTORY.md](BACKLOG_HISTORY.md) and [evidence/](evidence/):** previous experiments, decisions and supporting records; read only for a specific question.

`GUI_ROADMAP.md` is historical cross-reference information, **not a second task queue**. If a summary conflicts with source, inspect the current code and correct the affected status rather than starting the task again.

## Delivery principles

1. **Working GUI behaviour is the measure of progress.** A useful change connects or repairs an existing control and its real Rust application behaviour.
2. **The prototype is the design.** Retain the present Dashboard, Details, System, Settings, About and approved dialogs. Do not reintroduce Activity, search/filter, new pages, persistent preferences, a new wizard or second-instance activation.
3. **One Rust backend for GUI and CLI.** Reuse the current `gui_model`, native discovery, planner, configuration, protected runner, worker, Named Pipes, lock and journal. The GUI must not implement a second GPU-PV system.
4. **Use capabilities already present.** Before coding a slice, check its actual entry points and identify the missing production behaviour. Fix integration gaps; do not recreate completed workers or test harnesses.
5. **Test proportionately, at the right time.** Write focused tests for meaningful new logic, but do not run builds, test suites, UI launches or qualification after every edit. Follow the milestone/explicit-request testing policy in `AGENTS.md`. Mark unvalidated code *implemented — validation pending*, never *qualified*.
6. **Preserve security.** Review, elevation, VM identity, driver trust, authenticated IPC, readback, recovery and separate downtime approval are product requirements, not dispensable testing overhead.
7. **Keep moving.** If a task is blocked by a real provider/API limitation or prohibited live test, record the exact blocker and take the next safe implementation slice. Do not open an investigation campaign merely because testing is unavailable.

## Current baseline — source inspection, 10 October 2026

The repository already contains:

- The approved Slint GUI at `src/gui/ui/`, with Rust presentation in `src/gui/`.
- No-argument GUI launch, explicit headless CLI commands, and Windows GUI/console handling reported complete under `APP-001`.
- Real live inventory/Refresh/System; separately labelled `--mock-gui` fixture and historical snapshot rehearsal.
- Shared Rust plan preview, recorded-state reporting, GUID-keyed configuration parser, protected committed configuration read, credential support and in-memory draft model.
- **Source implementations** of live GUI Apply, graphics Verify, save-only retry, manual reconciliation and progress callbacks in `src/gui/live.rs`, with restricted-worker functions in `src/worker.rs`. **Do not mistake the presence of these paths for complete end-to-end qualification.** Inspect them before scheduling more integration work.
- An unresolved first-time GUI enrollment path: `src/gui/live_state.rs` still directs unenrolled users to CLI installation. Compute/Encode/Decode editing is rejected there; the GPU Memory slider has no validated real-unit meaning.

These are code/status observations, **not claims that live GPU effects, installation, or release qualification passed**. Several older explanatory strings and backlog summaries still say Apply/Verify are unavailable; reconcile them with the implemented action handlers rather than duplicating those handlers.

## Delivery slices

| Slice | Outcome visible to user | Owner IDs | State / next gate |
|---|---|---|---|
| **D1 — Application shell** | Approved GUI starts normally; CLI remains headless; mock is explicit | GUI-001, APP-001, CORE-012 | **Implemented / historically checked.** No new design task. |
| **D2 — Accurate live configuration** | Per-VM committed intent, observed GPU state, selected VM and one unsaved draft stay separate; Refresh handles missing/invalid/stale data | GUI-002, CFG-001, CORE-021 | **Partially in source.** Finish actual binding defects and prove no accidental defaults; do not build another parser. |
| **D3 — First-time use** | Select an existing Gen 2 VM and discovered physical GPU; stage, review and securely enroll a pair from existing Details controls | GUI-002, SEC-001, CORE-021 | **Open priority.** CLI-only enrollment is not a complete GUI journey. |
| **D4 — Supported GPU settings** | Edit and preview truthful supported VRAM/Compute/Encode/Decode Min/Optimal/Max settings, with accurate capability limits and readback | GPU-010, GUI-002, CORE-028 | **Open.** Existing raw VRAM path is partial; unsupported categories remain clearly blocked until supported. |
| **D5 — Protected operations and recovery** | Existing Review/Apply/Verify/Disable, progress, protected saving, save-only retry and manual recovery work through the shared Rust backend | SEC-001, CORE-028, CFG-001, GUI-002 | **Substantial source exists; integration and acceptance open.** Inspect current handlers and worker before writing more. |
| **D6 — Finish the application** | Existing controls, dialogs, layout, keyboard/focus, DPI and software-renderer fallback work as approved | GUI-003, GUI-002 | **Partly checked.** Finish only user-visible gaps. |
| **R1 — Distribution candidate** | Packaged Windows executable, correct installer/uninstaller, operator guide, notices, supported-path acceptance | CORE-017, DOC-003, GPU-014 | **Not yet accepted.** Requires separate, controlled validation. |

The slices are **implementation order**, not an instruction to run a full qualification suite after each row. Independent, safely implementable work can proceed while a provider-dependent row is blocked.

## Milestone gates

### M1 — Existing-VM product foundation: accepted historically

Native Windows/Hyper-V discovery, existing-VM scope, enrollment boundaries, protected runner and lab separation. Retain the [M1 evidence](evidence/M1.md). Do not repeat M1 to earn progress.

### M2 — NVIDIA baseline: existing evidence; stability hold open

Preserve the previously reported attach/render/disable evidence in [M2](evidence/M2.md). A **10 October 2026 Reapply-associated host lockup** leaves ARCH-001/GPU-012 stability acceptance open; its cause has not been established. **No Reapply stress, hang reproduction or other potentially destabilising live qualification without new explicit permission and an appropriate safety decision.** This does not prohibit independent GUI/Rust coding. See [incident evidence](evidence/reapply-investigation-20261010.md).

### M3 — Working Slint product: principal development milestone

Complete D2–D6 using existing GUI controls and shared backend. Acceptance requires:
- Existing user-selected Gen 2 VM/GPU can be represented, enrolled and configured without the CLI for the approved GUI journey.
- Intended/observed/draft state are never confused; stale, missing and unavailable data are truthful.
- Only supported provider units and values can be applied; unsupported fields are not silently accepted.
- Real actions, if authorised, use fresh plans, independently checked protected identity, elevation and separate guest downtime consent.
- Saved per-VM intent is bound to verified readback; failed saving permits save-only retry, not GPU replay.
- Host-wide modifying-operation admission and unresolved recovery holds apply across GUI/CLI/worker. Uncertain effects require manual reconciliation.
- Existing UI interactions work across the target Windows environments; no retired feature is reintroduced.

Finish source implementation before one **explicitly authorised, proportionate milestone validation**. Record any untested conditions accurately rather than relaunching laboratory research.

### R1 — Packaged v1.0: release gate

Package and inspect the actual Windows candidate, CLI/GUI mode dispatch, ProgramData ownership and lifecycle, required payloads, notices, installer/update/removal and current user guide. Perform **separately authorised** changed-path host/guest qualification after the M2 stability hold is addressed. A build, mock replay or old baseline pass is not an end-to-end release pass; packaging does not grant permission to publish.

### M4 — Later, not v1.0 blockers

Additional vendors, simultaneous same-GPU VM sharing and optional CUDA/stress/optimization experiments are separate conditional work. No claim of a GPU scheduler, fair sharing or enforced physical-GB limits.

## Hard technical and safety boundaries

- Windows Hyper-V Generation 2 VMs, NVIDIA first; identify targets by VM GUID and exact discovered GPU interface, not name or test slot.
- Product Rust only, except the narrow approved PowerShell Direct bridge. Lab helpers belong outside the product dependency graph.
- Use the selected versioned per-VM TOML contract in [CONFIGURATION.md](CONFIGURATION.md); current schema 2 supports optional **raw VRAM** but not the other three categories. Extend schema explicitly when qualified; do not silently reinterpret or discard values.
- GPU Memory slider remains illustrative until its exact Windows/provider mapping is demonstrated. Never label opaque provider values as GiB or promise enforcement.
- Preserve VM disks, CPU/RAM, Secure Boot, driver trust, credentials, journals, ACL/reparse protection and the required runner audit. Mock rehearsal is strictly no-write/no-effect; an audited runner read is not a no-write mock read.
- Never silently reboot, shut down or log out the physical host. Further live Hyper-V/GPU/driver/VM-power operations for this GUI push require explicit permission. Never bypass uncertain-state holds.
- Do not ship drivers, guest disks/ISOs, credentials, keys, proprietary payloads without rights, or fixed laboratory paths.

## The handoff rule

At the start of a coding session, use [BACKLOG.md — Next implementation action](BACKLOG.md#next-implementation-action), inspect the affected source, and **make the product change**. At handoff, record what behaviour changed, files edited, what remains blocked, and whether validation was deferred. Do not substitute another global audit, test-suite improvement or documentation rewrite for an available GUI implementation task.
