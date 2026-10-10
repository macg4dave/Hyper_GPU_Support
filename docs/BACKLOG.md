# Hyper GPU Support — Active Product Backlog

**Updated:** 10 October 2026  
**Goal:** Finish the existing Slint GUI as a usable Windows GPU-PV management application. **Source implementation is the priority; test infrastructure and old feasibility exercises are not active feature work.**

**Authority:** [ROADMAP.md](ROADMAP.md) controls delivery order and acceptance; [GUI_GUIDE.md](GUI_GUIDE.md) freezes the GUI design; [AGENTS.md](../AGENTS.md) and [ENGINEERING.md](ENGINEERING.md) retain development/security rules; [CONFIGURATION.md](CONFIGURATION.md), [ARCHITECTURE.md](ARCHITECTURE.md) and [DECISIONS.md](DECISIONS.md) retain technical contracts. [BACKLOG_HISTORY.md](BACKLOG_HISTORY.md), existing Git history and [evidence/](evidence/) preserve earlier work. This file replaces the sprawling active queue, **not** those records.

## Next implementation action

**Primary: `GUI-002` + `SEC-001` — complete the first-time GUI configuration/enrollment journey.**

The Slint frontend already has live inventory, selection, draft/review controls, and source-level Apply/Verify/save-only/reconciliation handlers. But `src/gui/live_state.rs::target` still sends an unenrolled VM to CLI installation. The first user journey is therefore incomplete.

1. Inspect **only** the existing relevant handlers in `src/gui/live.rs`, `src/gui/live_state.rs`, `src/gui/ui/`, `src/gui_model.rs`, `src/runner.rs`, `src/worker.rs` and the necessary shared enrollment/plan code.
2. Reuse the current GPU/VM discovery, exact enrollment authority, authenticated worker, reviewed plan and committed configuration model. From the **existing Details controls**, permit a selected eligible VM/GPU to reach a safe proposed first-time setup without pretending an unenrolled pair is already authorized.
3. Complete the missing **protected authorization/enrollment** path for that journey; the worker must independently validate and authorize the exact pair before effects. Reuse the existing runner/protected contracts; do not grant authority from GUI draft or configuration text.
4. Retain all existing consent, stale-state, write/recovery and disabled-action protections. If a particular protected enrollment step genuinely cannot be implemented safely, record its exact technical blocker, then implement the next safe GUI slice below.
5. Make source changes now. Add focused tests for newly introduced nontrivial logic, **without automatically running them**. Do not start a project audit, GUI redesign, old research or qualification cycle.

**Why first:** This produces functionality a normal user cannot currently complete in the GUI. It is a better next increment than improving already passing rehearsal tests.

**Definition of this slice implemented:** selecting an eligible but unenrolled existing VM and a discovered GPU can reach the real reviewed enrollment route through the approved UI, with truthful error/consent states and no security bypass. Actual hardware acceptance may remain **pending**, and must not be claimed to have passed without authorisation and evidence.

## How to work this backlog

- **Pick an implementable product slice and edit its real code.** Consult only the selected card, relevant source and affected contracts. Do not run a global architecture/docs review as a prerequisite.
- **Do not create separate test, logging, refactor, CI or investigation cards** unless a concrete defect blocks the current product change. Apply a small fix in the owning card.
- **Use existing code first.** A function or handler existing in source is not proof of completed user behaviour, but it is also not a reason to rewrite it. Trace the missing boundary and patch that.
- **Keep tasks small enough to finish.** After a coherent implementation slice, record `implemented — validation pending` if testing was intentionally deferred, and move on. Do not mark a hardware path `qualified` without evidence.
- **Run checks only at full milestone completion or on explicit request**, per `AGENTS.md`. Focused tests may be authored with the code. If a permitted milestone check fails, fix the affected behaviour, not the testing ecosystem.
- **Skip safely, don't loop.** If API support, hardware observation or approval is unavailable, write one concrete blocker and take the next independent GUI task. Do not repeatedly "research" the same closed decision.
- **Security remains a hard gate.** Materially changed elevation, IPC, authorization or recovery boundaries require the existing independent-review step before live use. Never weaken a boundary to mark a feature complete.
- At handoff, report just: **implemented behaviour / changed code / blocking issue / next source task / tests run or deferred**. Avoid long evidence reports except when requested.

### Status vocabulary

| Status | Meaning |
|---|---|
| `ready` | Product-source work can start now. |
| `in progress` | An implementation slice is being edited. |
| `implemented — validation pending` | Code path is present; the selected milestone checks or live acceptance remain unrun. **Proceed to other implementation work.** |
| `blocked` | Specific external prerequisite or safety requirement prevents this slice; record it once and take another ready task. |
| `completed` | Scoped outcome and required existing evidence are established. |
| `deferred` | Not a v1.0 implementation target. |

**Do not conflate** a completed implementation slice, a card's overall acceptance and the v1.0 release gate.

## Implementation order

These are **incremental code deliverables**, not new milestone-validation campaigns. Source observations below are based on the main branch inspected 10 October; verify locally before editing because the working tree may have advanced.

| Order | Owner | Actual deliverable | Current observation |
|---|---|---|---|
| **1** | `GUI-002`, `SEC-001`, `CORE-021` | First-time VM/GPU enrollment and usable setup path from existing Details controls | **Ready.** CLI-only enrollment message remains in `live_state.rs`. |
| **2** | `GUI-002`, `CFG-001` | Complete trusted committed-config presentation and editing/stale-state semantics for enrolled and missing/invalid files | **Partly in source.** `read_committed`, `load_committed`, per-VM parser and conflict checks exist; fix only missing wiring. |
| **3** | `GPU-010`, `GUI-002` | Extend capability validation/plan/editor to genuine supported VRAM/Compute/Encode/Decode triples; settle slider only with proof | **Open.** `live_state.rs` rejects last nine fields; do not invent provider units. |
| **4** | `SEC-001`, `CORE-028`, `CFG-001` | Close any remaining real worker/progress/save-only/reconciliation and host-wide admission gaps used by GUI | **Substantial code exists.** Trace exact missing paths; do not rebuild worker. |
| **5** | `GUI-002`, `CORE-021` | Connect remaining **approved** existing interactions such as safe forget-pairing if backend support exists; remove stale misleading help/status text | **Partly available.** Mock-only and misleading live messages exist. |
| **6** | `GUI-003` | Finish layout, keyboard/focus, close deferral, DPI, accessibility and software-renderer defects within fixed UI | **Partly checked.** Leave systematic qualification until M3 gate. |
| **7** | `CORE-017`, `DOC-003`, `GPU-014` | Package and accept the actual product | **Release work.** Live qualification separately authorised. |

If task 1 encounters a provider-dependent hard block, move to task 2 or another safe part of tasks 4–6. Do **not** replace implementation time with new mock-test improvements.

## Active cards

### GUI-002
**Working GUI integration — P0 — in progress.**

**10 October slice — implemented, validation pending:** `src/gui/live.rs` and
`live_state.rs` now refresh inventory/readable committed intent despite a draft
conflict, retain and block that draft until discard, and distinguish unreadable
saved intent from absence. Corrected stale live help; regression tests authored, unrun.

**Already in source:** live inventory/Refresh/System, real-data presentation, historical snapshot rehearsal, one draft, selected-GPU draft, shared plan preview, guest credentials, and handlers for live Apply, Verify, save-only retry, progress and reconciliation. Previous tested GUI rehearsal evidence remains in [GUI-002-testing](evidence/GUI-002-testing.md) and [closure-sprint](evidence/closure-sprint.md).

**Do next:** first-time setup without CLI; ensure committed desired state, observation, proposed draft and protected enrollment remain distinct; fix stale error/help text and unconnected approved actions. Bind GPU-010's supported categories when the core can validate them. Preserve selection and invalid raw edits. An imported/snapshot plan never grants live authority.

**Implemented when:** the approved controls perform their intended Rust-backed work for supported use cases, or explain precisely why unavailable. **Acceptance pending:** actual Windows/GUI/CLI equivalence, safe negative/failure behaviour and M3 milestone checks. No extra pages.

### SEC-001
**Exact privileged enrollment, coordination and worker boundary — P0 — in progress / extension required.**

**First-time GUI blocker (source inspection):** `worker::trusted_executable` requires
existing protected enrollment/artifact pins before launch. `runner::install` is an
administrator installer that replaces the complete enrollment set, not a reviewed
one-pair GUI command. Next source step: a fixed authenticated bootstrap/enroll route
that preserves existing authority; no draft-to-enrollment shortcut. Continued CFG-001.

`src/worker.rs` contains same-executable elevated apply, save-only, verify, reconcile, import and progress transport; the protected runner and authenticated Named Pipe already exist. `APP-001` reviewed a restricted-worker integration, **not every expanded enrollment or concurrent-operation scenario**.

**Do next:** support the approved first-time GUI enrollment journey with independent exact VM/GPU revalidation. Check host-wide one-modifying-operation admission across GUI, CLI and worker; unresolved recovery holds remain even after process lock release. Secure plan/session/peer identity and no arbitrary worker command. A changed privileged boundary needs independent review before live use.

**Do not:** build a new IPC stack, weaken ACLs/audit, treat a selected UI row as enrollment, or test on an unrelated VM.

### CFG-001
**Per-VM configuration and protected persistence — P0 — in progress.**

**Selected contract:** one GUID-keyed TOML per VM, schema 2, shared `Configuration` / `Target` parser, optional raw VRAM triple; see [CONFIGURATION](CONFIGURATION.md) and DEC-032. Source already includes per-file reads/serialization, committed-store snapshot/revisions and worker-side save/recovery concepts. Treat old notes saying *no protected publication exists* as potentially stale; inspect the current code.

**Do next:** close actual GUI read/display/commit mismatches, error isolation (absent vs unreadable), stale revisions, import/conflict and verified-save-only defects. Per-VM files live under the approved protected ProgramData location. Only independently authorized worker code publishes machine-wide intent after matching readback. Draft editing writes nothing; never infer saved intent from enrollment.

**Acceptance pending:** installation ownership, correct atomic/conflict behaviour and recovery through GUI/CLI. Do not create another file format or automatic migration.

### GPU-010
**Real selected-GPU capability/allocation semantics — P0 — ready, provider-dependent.**

Deliver the physical GPU capability model and validated **Minimum / Optimal / Maximum** for VRAM, Compute, Encode and Decode through shared Rust types, planning and eventual readback. Current schema-2 production parser/GUI path supports only optional raw VRAM; schema extension needs an explicit version/import contract. Existing twelve UI inputs are not evidence of twelve supported backend controls.

**Do next:** consult official Windows/Hyper-V provider contracts and existing native code; implement accurate units, bounds, unsupported/unset semantics and capability reporting. Enable only those fields the backend can actually represent and validate. The GUI's "GPU Memory" slider is mock-only until its mapping is proven; do not convert raw provider integers to GiB or percentages by assumption.

**Blocker rule:** if a field requires unavailable live provider evidence, retain truthful disabled state, record the exact question and continue with independent GUI integration. No speculative enforcement promise.

### CORE-028
**Real operations, progress, reconciliation and verified saving — P0 — integration/acceptance open.**

Source includes shared planning, worker callbacks, durable operation phases, GUI Verify/Apply/reconcile and save-only paths. **Do not rewrite them as planned architecture.**

**Do next:** find the remaining mismatch between the approved dialogs and the actual execution/recovery contract. Enforce fresh plan, protection and guest-downtime consent; publish only after verified readback; expose real Pending/Running/Done/Failed/Unknown updates where supported. Ensure save failure cannot replay GPU effects. On crash, timeout or pipe loss, retain uncertain state and the host-wide modification hold until explicit manual reconciliation. No automatic rollback, restart, journal clearance or guessed success.

**Live safety hold:** the 10 October Reapply-associated host lockup is unresolved; do not rerun hazardous qualification as part of this integration card.

### CORE-021
**Shared CLI/GUI configuration and enrollment behaviour — P1 — in progress.**

Coordinate GUI-002 / SEC-001 / CFG-001 / GPU-010 so the same core rules apply from both interfaces. No duplicate parser, enrollment store, credential system or manual test-VM management feature. Close after the integrated existing-user journey is coherent.

### GUI-003
**Approved-interface completion — P1 — in progress; validation deferred.**

A 10 October scoped software-renderer/mock check covered minimum/workspace sizes, larger inventories, pages and wrapped Review. Remaining focus: real dialog lifetime/close deferral, pane split and scrolling, Windows DPI/text scaling, focus/keyboard and accessibility behaviour.

**Do next:** fix observed interaction/layout defects in the existing components; avoid speculative new controls. Perform a proportionate UI acceptance pass at the M3 milestone, not after every edit.

### ARCH-001
**M2 product baseline — in progress (stability acceptance blocked).**

Retain prior native NVIDIA/effect results. A 10 October fresh preparation/Reapply sequence returned functional results but was associated with an unresponsive Windows host and unclean restart; causality is unknown. No new destabilising testing without explicit permission. See [incident](evidence/reapply-investigation-20261010.md). **Not a reason to stop safe GUI coding.**

### GPU-012
**M2 bounded NVIDIA qualification — blocked by safety hold.**

No hang reproduction, Reapply stress, repeated payload-transfer experiments or extended GPU probes by default. Resume affected-path qualification only following fresh user authorisation and a suitable safety decision. Preserve prior passes as prior passes, not release proof.

### CORE-017
**R1 Windows candidate packaging — planned.**

Package the required binaries/payloads and installer lifecycle; preserve GUI/CLI output/exit semantics, UAC, ACLs, ProgramData, prerequisites and licenses. No lab data, drivers/disks or secrets bundled without rights.

### DOC-003
**R1 operator documentation — planned.**

Write user instructions against actual supported and tested candidate behaviour: startup/CLI, GUI setup, config, consent, verification, save-only and manual recovery. Not another internal planning audit.

### GPU-014
**R1 end-to-end acceptance — blocked until separate authorisation / M2 safety resolution.**

Use only explicitly designated disposable targets, relevant changed paths and existing evidence. Confirm product/package/UI/CLI parity and recovery; don't re-create an entire laboratory qualification programme.

## Completed scoped work — do not restart

### GUI-001
**Completed:** the approved Slint prototype and design promotion. Future layout/UX defects belong to GUI-003; no new UI-design task.

### APP-001
**Completed (reported 10 October):** single-executable GUI/CLI dispatch, Windows console handling, restricted-worker implementation/integration and independent review for that scope. Expanded SEC-001 cases remain open. See [closure evidence](evidence/closure-sprint.md).

### CORE-012
**Completed (reported 10 October):** partial/inaccessible inventory, recorded-state provenance, redacted diagnostics and appropriate operator guidance. Reuse from GUI; do not start another diagnostics initiative.

### CORE-006
**Completed existing baseline:** shared typed preview/planner. Extend only for real new GPU-010/CORE-028 operations. See [history](BACKLOG_HISTORY.md#core-006).

### PLAN-001
**Completed:** scoped architecture/source/docs audit. No need to repeat for each coding task.

## Deferred and retained historical references

**GPU-015** — deferred M4 simultaneous GPU sharing. Multiple VM configs and serialized operations do **not** prove concurrent GPU sharing safety.  
**OPEN-01** — resolved in DEC-032; schema-2 one-target GUID TOML, not an open design debate.  
**OPEN-03 / OPEN-05 / OPEN-08 / OPEN-12** — only specific SEC-001/CORE-028/CFG-001 security/recovery acceptance questions, not autonomous research cards.  
**OPEN-04 / OPEN-10** — GPU-010 provider semantics; **OPEN-06 / OPEN-11** — GUI-003 renderer/accessibility; **OPEN-09** — GUI/CLI packaging acceptance; **OPEN-07** — resolved draft navigation.

**BLK-005** — historical hang investigation closed by user direction; do not reopen. The separate 10 October Reapply incident is recorded under ARCH-001/GPU-012 and remains unresolved.

Other historical IDs retain their original meaning in [BACKLOG_HISTORY](BACKLOG_HISTORY.md); completed/merged/cancelled/deferred records are **not** a fresh implementation queue. This includes older native-port, golden-image, driver-staging, reference and optional stress/CUDA tracks.

## Permission and safety limits

- **No live Hyper-V/GPU/driver/VM-power tests for this GUI implementation push without explicit permission.** In particular, no Reapply qualification while its host-lockup concern remains unresolved. You may implement safe Rust/Slint changes without these tests.
- **Never reboot/shut down/log out the host without immediate explicit approval.** Don't schedule or accept automatic host restart.
- Keep the authorized protected worker, correct VM/GPU identity, enrollment, UAC, separate downtime consent, trusted signed payloads, audit, IPC ACL/authentication, host-wide locks and recovery holds.
- Mock/snapshot rehearsal is no-effect/no-write: no config, vault, enrollment, journal or audit writes and no guest probes. Do not suppress mandatory audit from the real protected runner to satisfy mock semantics.
- Preserve the CLI and the fixed approved GUI, using Rust for application logic. No test harness or development script is a production dependency.

## Close-out and next handoff

**When a coherent code slice is implemented:** update only its existing card with 2–4 lines: actual new behaviour, relevant source files, an exact blocker (if any), and `validation pending` when checks were deferred. Update a release milestone only after its real gate. Do not create a new test card because this slice was not tested.

**Then immediately choose the next ready source-code deliverable from the implementation-order table.** The default is implementation, not qualification, documentation synchronization, test refinement or another roadmap rewrite.
