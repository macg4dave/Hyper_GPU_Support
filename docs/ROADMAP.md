# Hyper GPU Support — v1.0 roadmap

**Updated:** 10 October 2026. **Objective:** complete the approved Slint Windows application over the existing Rust core.

This file owns delivery order and milestone gates. [BACKLOG](BACKLOG.md#next-implementation-action) owns executable tasks and status. The approved interface is fixed by [GUI_GUIDE](GUI_GUIDE.md); coding, review and authorization follow [AGENTS](../AGENTS.md). Historical results remain in [BACKLOG_HISTORY](BACKLOG_HISTORY.md) and [evidence](evidence/). Historical queues are not current instructions.

## Source baseline

Reviewed the current working tree, including uncommitted changes, on 10 October. This was static inspection only; no tests, builds, GUI launches or live checks were run. Existing evidence is historical, not fresh verification.

| Area | Present in source | Remaining product work |
|---|---|---|
| Application shell | No-argument Slint startup, explicit CLI dispatch, restricted internal worker, console attachment | Candidate packaging/acceptance; preserve the separate runner/guest/probe artifacts |
| Inventory and presentation | Live discovery, Refresh/System, partial observations, provenance; fixture and bounded historical snapshot modes | First-install discovery/setup route when no runner is installed; truthful action availability |
| Intent and persistence | Shared schema-2 `Configuration`/`Target`, GUID TOML, protected store/revisions, worker publication/import | Enrollment integration; full-resource schema; fixture intent still uses a separate `Saved` presentation model |
| Operations | Shared plan, native enable/disable/settings/raw VRAM, graphics Verify, worker progress, durable recovery, save-only retry and reconciliation | Specific presentation/recovery defects and remaining transfer supervision; no replacement worker/planner project |
| Privilege coordination | Authenticated IPC, trusted installed artifacts, common operation lock, host-wide recovery holds | Bounded first-install bootstrap and incremental exact-pair enrollment/removal |
| GPU editing | Physical GPU discovery/selector; optional raw VRAM triple and independent readback | Other resource triples; switching to another GPU is rejected by exact enrollment/attachment checks; slider mapping is unavailable |
| Approved UI | Dashboard/Details/System/Settings/About and dialogs | First-time setup, Forget pairing, stale strings and concrete interaction defects; no new pages or controls |

`src/worker.rs` requires protected enrollment and the installed main executable before launch. `runner::install` replaces the enrollment set; it is not a safe incremental GUI enrollment command. CLI `forget` deletes credentials, not pairing authority. Presence of Apply/Verify handlers is implementation evidence, not end-to-end acceptance.

## Delivery order

| Milestone | Implementation outcome | Dependencies | Completion gate |
|---|---|---|---|
| **M1 — Product foundation** | Existing-VM scope, native backend, protected runner, lab separation | Historical baseline | Accepted historically; preserve [M1](evidence/M1.md), do not restart |
| **M2 — NVIDIA baseline and host safety** | Retain working attach/render/disable paths; finish the already identified bounded transfer work | Existing core | Source completion, independent review of changed boundaries, then separately authorized affected-path stability qualification |
| **M3 — Functional Slint product** | Complete the implementation stages below through approved controls | Existing M1/core; M2 acceptance gates live validation, not safe source work | All M3 coding tasks implemented, material boundary reviews complete, then explicit authorization for proportionate milestone validation |
| **R1 — Windows distribution candidate** | Package actual artifacts and operator instructions; implement missing update/removal lifecycle | Stable implemented interface; package preparation may overlap M3 | M2 safety acceptance + M3 validation + separately authorized packaged acceptance; publication requires its own instruction |
| **M4 — Later capabilities** | Additional vendors, simultaneous GPU sharing and optional workload/performance work | Qualified baseline and explicit scope | Outside v1.0 queue |

### M3 implementation stages

1. **Enrollment backend:** add reviewed, incremental pair authorization and a fixed first-install bootstrap. Preserve other enrolled VMs and operator/artifact authority. Installation/enrollment does not apply GPU effects or fabricate committed intent.
2. **First-time GUI journey:** allow an eligible existing Gen 2 VM and discovered GPU to reach enrollment review from Details. After enrollment, reload protected state and obtain a separate fresh operation plan before Apply. No CLI dependency for this journey.
3. **Truthful supported editing:** extend shared schema, capability discovery, plan, native writes/readback and recovery to VRAM/Compute/Encode/Decode Min/Optimal/Max. Bind the existing fields only when supported. Implement reviewed GPU replacement with attribution and interruption handling. GPU Memory slider needs a demonstrated provider mapping; keep it unavailable until then.
4. **Complete existing workflows:** finish Forget pairing, recovery-state synchronization, accurate availability/help, and shared production-format fixture intent. Reuse existing Apply/Verify/publication/import/reconciliation handlers.
5. **Finish existing interactions:** repair concrete focus/dialog/scrolling/close defects within the approved layout. DPI, accessibility and software-renderer acceptance belong to milestone validation.

Independent ready tasks may run while enrollment or provider work is blocked. M2 containment/qualification is a separate lane; it must not become a prerequisite for every GUI edit. Each coding task has its own implementation criterion; completing one does not complete M3 or authorize validation.

### M3 completion criteria

- A normal user can select and enroll an eligible existing VM/GPU, draft settings, review and request supported operations through the approved UI.
- Enrollment authority, committed desired intent, observed attachment, historical graphics/preparation records and unsaved drafts remain distinct. Missing/unreadable/stale state never becomes an invented default or success.
- Four resource categories are represented through shared Rust types with explicit capability/unsupported/unset semantics and independent readback. Baseline support must be established before claiming the initial-release allocation requirement complete; disabling every missing field does not satisfy it. No invented GiB, percentages, hard limits or enforcement claims.
- GPU replacement and Forget pairing have bounded exact-target contracts; no implicit destructive guest cleanup, authority broadening or loss of unresolved recovery.
- Apply/disable, Verify, progress, verified saving, save-only retry and manual reconciliation use the shared core. Fresh plan/identity checks, credentials, UAC and separate guest downtime consent remain mandatory.
- Common operation admission and durable recovery holds work across GUI/CLI/worker, including after process exit. Save failures never replay GPU effects; uncertain outcomes never imply no mutation.
- Multiple VM configurations remain supported, with the approved selected-VM editing model and serialized operations. This is not simultaneous GPU-sharing qualification or a new batch-control requirement.
- Fixtures/snapshots are explicitly simulated/historical and strictly no-write/no-effect. No fixture fallback after live discovery failure, suppressed runner audit or guest probe in rehearsal.
- All planned M3 tasks are implemented and necessary independent boundary reviews are complete before the explicitly authorized validation gate. Deferred checks remain recorded as pending.

## Validation and release lanes

**M2 / ARCH-001 / GPU-012:** the earlier investigation was closed by user direction; preserve its history. The separate 10 October Reapply-associated host lockup remains unresolved, with no established cause. Admission reuse, WMI completion/payload limits, bootstrap publication and bounded acknowledged transfer source work are implemented and statically reviewed, pending validation; they are not a stability pass. Deferred closure gates and containment limits are described in [source containment](evidence/m2-source-containment-20261010.md). No hang reproduction, Reapply stress or new live exercise without explicit permission and a bounded safety decision.

**M3:** after all planned source tasks and applicable independent reviews, obtain explicit authorization for the relevant existing Rust/build gates and GUI acceptance. Use focused logic/failure coverage and approved UI checks; no new testing infrastructure campaign. Live Hyper-V/GPU/driver/VM-power, runner installation/update/exercise and guest changes require separate explicit authorization and M2 safety readiness.

**R1 / CORE-017 / DOC-003 / GPU-014:** ship the main application with its required protected runner, guest worker and checked D3D11 probe. Verify fixed sibling artifacts, prerequisites, protected ProgramData lifecycle, install/update/removal/recovery, CLI/GUI parity and notices against the candidate. Reuse native installation; no new installer framework or assumed existing uninstaller. Exclude laboratory data, disks/media, secrets and proprietary drivers without redistribution rights. A build or old render pass is not candidate acceptance.

## Non-negotiable boundaries

Rust owns product logic and normal installation/recovery. Only DEC-028's fixed PowerShell Direct session/transfer/bootstrap-integrity/launch bridge is retained; laboratory tooling stays outside the product graph. Reuse working modules rather than restart native migration, diagnostics or feasibility work.

Use exact discovered VM GUID/GPU identities and signed dynamic driver payloads. Preserve disks, CPU/RAM quantities, Secure Boot, ACL/reparse checks, trusted artifacts, authenticated IPC, audit and recovery. Never authorize operations from imported configuration or GUI draft text. Never reboot/shut down/log out the physical host without immediate explicit permission.

The next implementation task is always the first ready card in [BACKLOG](BACKLOG.md#next-implementation-action). GPU-010 is an initial-release requirement; optional CUDA/stress, HCS, vendor expansion, fair-sharing claims, persistent preferences, extra pages and removed prototype controls are outside the active path.
