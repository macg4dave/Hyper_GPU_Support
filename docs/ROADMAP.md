# Hyper GPU Support — v1.0 roadmap

**Updated:** 10 October 2026. **Objective:** complete the approved Slint Windows application over the existing Rust core.

This file owns delivery order and milestone gates. [BACKLOG](BACKLOG.md#next-implementation-action) owns executable tasks and status. The approved interface is fixed by [GUI_GUIDE](GUI_GUIDE.md); coding, review and authorization follow [AGENTS](../AGENTS.md). Historical results remain in [BACKLOG_HISTORY](BACKLOG_HISTORY.md) and [evidence](evidence/). Historical queues are not current instructions.

## Source baseline

Reviewed the current working tree, including uncommitted changes, on 10 October. This was static inspection only; no tests, builds, GUI launches or live checks were run. Existing evidence is historical, not fresh verification.

| Area | Present in source | Remaining product work |
|---|---|---|
| Application shell | No-argument Slint startup, explicit CLI dispatch, restricted internal worker, console attachment | Candidate packaging/acceptance; preserve the separate runner/guest/probe artifacts |
| Inventory and presentation | Live discovery, Refresh/System, partial observations, provenance; fixture and bounded historical snapshot modes; fixed first-install UAC discovery/setup source | Acceptance of first-install discovery/setup and readiness text remains deferred |
| Intent and persistence | Shared schema-2 `Configuration`/`Target`, GUID TOML, protected store/revisions, worker publication/import; separate Details enrollment review source | Enrollment journey validation; full-resource schema; fixture intent still uses a separate `Saved` presentation model |
| Operations | Shared plan, native enable/disable/settings/raw VRAM, graphics Verify, worker progress, durable recovery, save-only retry and reconciliation | Specific presentation/recovery defects and remaining transfer supervision; no replacement worker/planner project |
| Privilege coordination | Authenticated IPC, trusted installed artifacts, common operation lock, host-wide recovery holds; reviewed incremental pair enrollment and fixed setup/recovery source | Deferred bootstrap/enrollment validation; protected removal and GPU replacement |
| GPU editing | Physical GPU discovery/selector; optional raw VRAM triple and independent readback | Other resource triples; switching to another GPU is rejected by exact enrollment/attachment checks; slider mapping is unavailable |
| Approved UI | Dashboard/Details/System/Settings/About and dialogs; existing Details/Review setup/enrollment bindings and corrected readiness/help text | Deferred setup/enrollment UI acceptance, Forget pairing, recovery synchronization and concrete interaction defects; no new pages or controls |

`src/worker.rs` requires protected enrollment and the installed main executable before launch. Installed GUI enrollment now uses fixed EnrollPair with exact pair/operator/revision review and durable recovery; `runner::install` remains the complete CLI installation route. The separate fixed `setup` path handles initial discovery/install and original-scope interruption recovery without replacing existing authority. These changes received independent static boundary review. Authorized checks passed on 10 October (145 tests, strict Clippy, formatting, Windows x64/MSVC debug build and docs); separately authorized protected artifact update/hash readback and installed live GUI startup/Discover succeeded. Enrollment/setup interruption and GPU-effect acceptance remain pending; M3 is not complete. CLI `forget` deletes credentials, not pairing authority. Presence of handlers is implementation evidence, not end-to-end acceptance.

## Delivery order

| Milestone | Implementation outcome | Dependencies | Completion gate |
|---|---|---|---|
| **M1 — Product foundation** | Existing-VM scope, native backend, protected runner, lab separation | Historical baseline | Accepted historically; preserve [M1](evidence/M1.md), do not restart |
| **M2 — NVIDIA baseline and host safety** | Completed; NVIDIA baseline and bounded transfer accepted | Existing core | Closed 10 October on user-confirmed live testing, independent source review and passing quality gate; no further M2 live tests required |
| **M3 — Functional Slint product** | Complete the implementation stages below through approved controls | Accepted M1/M2 core | All M3 coding tasks implemented, material boundary reviews complete, then explicit authorization for proportionate milestone validation |
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

**M2 / ARCH-001 / GPU-012: complete, 10 October.** The user confirmed that live tests were done and the product works, accepted the baseline as known good, and directed closure without further live qualification. Admission reuse, WMI completion/payload limits, bootstrap publication and bounded acknowledged transfer are implemented and independently reviewed; the repository quality gate passed. [M2 acceptance](evidence/M2.md) records this closure. Incident evidence is retained as history, with no established lockup cause; it is not an open milestone blocker or a requirement to repeat testing.

**M3:** after all planned source tasks and applicable independent reviews, obtain explicit authorization for the relevant existing Rust/build gates and GUI acceptance. Use focused logic/failure coverage and approved UI checks; no new testing infrastructure campaign. Live Hyper-V/GPU/driver/VM-power, runner installation/update/exercise and guest changes require separate explicit authorization. M2 is accepted and needs no repeat qualification.

**R1 / CORE-017 / DOC-003 / GPU-014:** ship the main application with its required protected runner, guest worker and checked D3D11 probe. Verify fixed sibling artifacts, prerequisites, protected ProgramData lifecycle, install/update/removal/recovery, CLI/GUI parity and notices against the candidate. Reuse native installation; no new installer framework or assumed existing uninstaller. Exclude laboratory data, disks/media, secrets and proprietary drivers without redistribution rights. A build or old render pass is not candidate acceptance.

## Non-negotiable boundaries

Rust owns product logic and normal installation/recovery. Only DEC-028's fixed PowerShell Direct session/transfer/bootstrap-integrity/launch bridge is retained; laboratory tooling stays outside the product graph. Reuse working modules rather than restart native migration, diagnostics or feasibility work.

Use exact discovered VM GUID/GPU identities and signed dynamic driver payloads. Preserve disks, CPU/RAM quantities, Secure Boot, ACL/reparse checks, trusted artifacts, authenticated IPC, audit and recovery. Never authorize operations from imported configuration or GUI draft text. Never reboot/shut down/log out the physical host without immediate explicit permission.

The next implementation task is always the first ready card in [BACKLOG](BACKLOG.md#next-implementation-action). GPU-010 is an initial-release requirement; optional CUDA/stress, HCS, vendor expansion, fair-sharing claims, persistent preferences, extra pages and removed prototype controls are outside the active path.
