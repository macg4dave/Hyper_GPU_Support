# GPU-PV Product Roadmap

**Updated:** 9 October 2026  
**Status:** Architecture rebase / planning; implementation not authorised by this document  
**Platform:** Windows 11 x64 · Rust · Hyper-V GPU-P · Slint  
**Related:** [BACKLOG.md](BACKLOG.md) · [GUI_GUIDE.md](GUI_GUIDE.md) · [GUI_ROADMAP.md](GUI_ROADMAP.md) · [GUI_PROMPT.md](GUI_PROMPT.md) · [SLINT_CODEX_PROMPT.md](SLINT_CODEX_PROMPT.md)

> **Source of truth:** This roadmap owns product direction and milestone gates; `BACKLOG.md` owns work-card status and dependencies; `GUI_GUIDE.md` owns approved GUI/architecture decisions; `GUI_ROADMAP.md` owns detailed Slint implementation slices. Reconcile conflicting cards before work. Historical results below come from the **8 October 2026 roadmap review**, not a new repository audit or new hardware tests. No source, host or VM modification is authorised by this planning document.

## 1. Product outcome

Deliver **one Windows executable**, `hyper-gpu-support.exe`, that opens a Slint GUI with no arguments and executes explicit CLI commands headlessly. Both interfaces share one Rust application/domain core and the same validated Hyper-V operations. The executable may launch a restricted elevated worker instance for an approved operation; there is no resident management service.

For **existing** Hyper-V Generation 2 VMs, users can:

- Discover all VMs and compatible host GPUs, including unconfigured/unsupported/unknown cases without conflating those states.
- Select a physical GPU, enrol or change an authorised VM/GPU pairing, and enable/disable GPU-P.
- Edit **per-VM** VRAM, compute, encode and decode **minimum/optimal/maximum** values where provider support and semantics are validated. Show truthful unsupported/unknown states rather than fabricated units or controls.
- Stage **one VM draft**, review the fresh plan, explicitly approve elevation and guest downtime as needed, apply changes, read back actual state, and save the resulting configuration.
- Prepare an eligible Windows guest using the dynamically discovered complete payload of the **current signed host NVIDIA driver**, and check PnP plus hardware D3D11 graphics where supported.
- Inspect real stages/results and incomplete-operation recovery state without automatic GPU rollback or blind retry.

**First target:** Windows 11 x64 host and Windows guest, NVIDIA first; RTX 5060 is a designated test baseline, not a universal-support claim. Additional vendors and simultaneous same-GPU sharing are separate qualification efforts.

**Out of production scope:** Creating/resetting/cloning VMs; golden-parent workflows; fixed driver manifests, versions, file counts or hashes; HCS experiments; laboratory coupling; arbitrary privileged command execution; host-wide GPU partition-count changes; schedulers/fairness promises; mandatory CUDA/stress testing; automatic host restart. The standalone lab belongs under contributor/test tooling, never as a production dependency.

## 2. Reuse the proven Rust work

The experimental `windows_gui.rs` is **disposable**. Do not port its Win32 controls, fixed positioning, thread-local UI state or event loop merely for compatibility. The Rust core may be refactored where justified, but keep working contracts and verified safeguards rather than recreating them.

| Area | 8 Oct baseline reported in previous roadmap | Required follow-up |
|---|---|---|
| M1 native boundary | Root Rust workspace, runtime schema 2, native install/enrollment, protected runner qualified | **Preserve capability**, not necessarily schema or internal layout |
| VM/GPU discovery | Native inventory and exact VM/GPU enrollment for multiple targets | Per-VM partial-access/unknown reporting; GUI selection and initial enrollment |
| NVIDIA preparation | Driver discovery, signature/catalog checks, dynamic complete payload and Rust guest writer | Check fresh transfer/writer path under current limits when affected |
| Apply/disable | Default attach, reapply, detach and attributable settings/power restoration exercised | Expand to selected GPU and twelve validated allocation fields |
| Shared planning | CORE-006 typed plan/apply effects and preview reported complete | Extend same plan for GPU selection, re-enrollment and allocations |
| Recovery | Per-VM journals, pending intent and uncertainty handling | Manual reconciliation under new worker, configuration and IPC boundaries |
| Verification | PnP and checked D3D11; running no-op preserved uptime | Distinguish **last verified** from **currently verified** |
| CLI | Inventory/install/plan/apply/enable/disable/status/verify/credentials/forget | Preserve headless behaviour, honest errors and shared new functionality |
| Current GUI | Win32 prototype | **Replace with Slint**; no Win32 feature-parity requirement |
| Allocation | Optional raw VRAM triple with range validation/native setter | Qualify units, unset values, readback, bounds, enforcement and all remaining fields |

This table preserves claims from the supplied roadmap; **Codex must verify against the current repository and `BACKLOG.md`** before treating a capability as implemented. In particular, inventory reportedly fails as a whole when one VM inspection fails; status is not a fresh guest graphics test; enable-plan may hash a full driver payload and is not a lightweight dashboard poll. Existing runner calls may write protected audit records even for ostensibly read-only requests.

Keep **DEC-028**'s limited PowerShell Direct transport/bootstrap bridge unless the current repository decisions supersede it. Preparation and application logic should remain native Rust; do not broaden the bridge into a general command API.

## 3. Non-negotiable architecture

### Application and state

- **One executable, three modes:** no args → Slint GUI; explicit commands → CLI (no Slint startup); internal authenticated invocation → elevated fixed-operation worker. Investigate Windows GUI-subsystem versus reliable console stdout/stderr/exit codes before choosing dispatch mechanics.
- **Three distinct states:** committed desired configuration, freshly observed Hyper-V/provider state, and one unsaved in-memory VM draft. Never treat any of these as interchangeable.
- **Per-VM files:** `%ProgramData%\HyperGpuSupport\config\vms\<VM-GUID>.<format>`; use stable VM identity, not display names. File format/version migration remains **OPEN-01**; `.toml` is illustrative, not approved.
- **Storage ownership:** restricted elevated worker exclusively writes machine-wide VM configurations after successful operation/readback; the same fixed protocol permits **save-only recovery** without reapplying GPU operations. Enrollment/recovery data has separate protected storage/ACLs; user window/theme/split preferences reside per-user.
- **Stale changes:** detect external edits to files and Hyper-V before apply/save; block, report and require refresh. Never auto-merge or silently overwrite a draft.
- **One modifying GPU operation host-wide:** shared cross-process coordination across GUI, CLI and elevated instances. Ordinary lock release does not clear unresolved durable recovery holds.

### Privilege and operation safety

- GUI normally unelevated; one **short-lived elevated instance** of the same executable per approved operation, then exit. Request necessary authorisation **before** any guest shutdown.
- **Local private Windows Named Pipe** for bounded, typed, versioned request/progress/result messaging. Restrict ACLs, authenticate peer/operation, bind VM/GPU identities and plan version; reject arbitrary shell commands, replay, malformed or stale requests.
- Preview must identify exact effects and be revalidated at execution; the privileged boundary independently checks permissions, device identities, current state and approved scope.
- Managed **graceful guest shutdown** requires explicit user approval. A failed graceful shutdown stops mutation; never force-off without separate consent. Restore previous running state only when safe and already authorised. **Never** restart or shut down the host without new explicit permission.
- On GPU operation failure or uncertainty: **stop modifications and require manual recovery**. Read back if possible, report completed/unknown stages, retain the minimal durable recovery information. No automatic GPU rollback, replay or corrective reassignment.
- **Startup after interruption:** open normal dashboard with persistent recovery warning. Read-only inspection remains; block new GPU modifications until reconciliation. Normal GUI close is deferred during an active operation; an unexpected crash or lost pipe must not falsely report cancellation or success.
- One GUI instance **per Windows session**; subsequent no-argument launches activate its existing window without discarding draft. CLI and workers remain unaffected.

### GPU and guest integrity

- Preserve VM disks, CPU/RAM quantities, Secure Boot, security devices, credentials, ACL/reparse protections and relevant preimages. Preview MMIO/cache/checkpoint-policy changes; restore attributable settings when safe.
- Use complete current signed NVIDIA payload discovery and correct guest destinations, not a fixed manifest. Existing preparation/PowerShell Direct constraints continue to apply.
- Host GPU partition count and host-wide capabilities are **read-only** in the GUI. Provider values do not inherently represent GiB, percentages, fairness or enforced limits.
- Simultaneous shared-GPU use remains **unqualified** until a separate two-VM test; supporting multiple configuration files or sequential GPU operations does not establish sharing safety.

## 4. Milestones and status

| Milestone | Exit condition | State / mapping |
|---|---|---|
| **M1 — Product boundary** | Native Rust boundary, protected runner and enrollment independent of lab | **Reported complete**; ARCH-001 |
| **M2 — Reliable NVIDIA core** | Default attach/render/reapply/disable, clean preservation and bounded recovery | **Observed current-build repeat passed; affected fresh preparation still needs qualification**; ARCH-001, GPU-012 |
| **M3 — Slint product + advanced per-VM GPU config** | New Slint UI, validated 12-field allocation and GPU selection, per-VM storage, secure worker/IPC, shared CLI, readback/recovery | **Planned architecture rebase**; GUI-001, GPU-010 plus new backend/security cards |
| **R1 — Release candidate** | M2 + M3 gates, usable CLI/GUI, packaging, instructions and authorised candidate validation | **Not started/verify actual backlog**; CORE-017, DOC-003, GPU-014 |
| **M4 — Further qualification/vendors** | Conditional two-VM shared-GPU validation and incremental additional vendor support | **Later**; GPU-015 and separately scoped vendor cards |

**Scope change:** The old roadmap put advanced allocation after the defaults-based release. The user has now required editable physical GPU selection and the complete four-category, twelve-field allocation UI **from the start**. This moves allocation capability and its necessary backend verification into **M3/R1**. If a provider or Windows build cannot actually support an advertised field, document the limitation and request an explicit scope decision; **do not silently rebrand a read-only/defaults-only release as complete**. Vendor expansion remains later.

### M2 remaining core work (small, bounded)

- **C1 / BLK-005 — Closed by direction on 8 Oct.** Revision `96152e7` reportedly completed an observed default attach/PnP/D3D11/reapply/verify/disable sequence, without reproducing the earlier host hang. **Cause remains unknown, not proven fixed.** Do not launch another hang campaign without new evidence/approval.
- **C2 / CORE-006 — Reported complete on 8 Oct.** Validated shared preview/effects for the then-supported operations. Extend rather than redo the preview system for selected GPU, full allocation and enrollment changes.
- **C3 / CORE-012, CORE-021 — Continue.** Distinguish desired/observed/recorded verification, missing versus denied versus unsupported/unknown, and partial inventory failures; provide precise redacted errors and next actions. Save failure must never trigger an Apply replay.
- **C4 / GPU-012 — Finish proportionally.** Validate affected current-driver fresh preparation/writing under the current child limits, preserving existing M2 evidence. No repeated long stress campaign, manufactured upgrade or mandatory CUDA/D3D12 gate.

## 5. M3 — Sequenced delivery (see GUI_ROADMAP.md)

These are **planned work packages**, not approval to implement them. Keep independently verifiable gates; mock/read-only work can progress without live GPU operations. Verify readiness and ownership against `BACKLOG.md` before claiming cards.

| Slice | Work | Exit / checkpoint |
|---|---|---|
| **M3.0 Audit** | Read current root Rust source, Cargo layout, project instructions, `BACKLOG.md`, configs, runner, existing journal, GPU provider and Slint capabilities. Identify what can be reused or simplified. | Reality/gap matrix; module plan; security/configuration open questions surfaced. **No code or live operation.** |
| **M3.1 Slint shell** | VS Code-previewable mock GUI: Fluent styling, navigation, VM cards, always-split adjustable panels, horizontal narrow-window scrolling and right-hand hybrid details. | Usable without Hyper-V; test long names, empty/unknown state, focus, scaling and scrolling. |
| **M3.2 Executable + contracts** | No-argument GUI, explicit headless CLI, restricted worker routing; shared typed VM/GPU identity, desired/observed/draft and operation-state interfaces. | CLI output/exit codes preserved; GUI does not duplicate core rules. |
| **M3.3 Configuration** | Per-VM schema/versioning after audit; ProgramData paths/ACLs; atomic worker-only writes, stale conflict detection, in-memory draft and independent save-only retry. | No config overwrite, identity mixup or mutation on mere editing. |
| **M3.4 GPU configuration** | Host/VM capability discovery; GPU assignment, new VM enrollment, all four expandable allocation groups with min/optimal/max; validate provider semantics and readback. | Every editable field demonstrated on supported target or explicitly blocked with evidence and scope decision; CLI/GUI share validation. |
| **M3.5 Privileged boundary** | Per-operation on-demand elevation, authenticated private Named Pipe, fixed request set, fresh-plan checks, host-wide cross-process lock and worker lifetime. | UAC denial/malformed/replayed/stale request cannot mutate; loss of pipe yields uncertainty. |
| **M3.6 Execution + recovery** | Adaptive Review & Apply; distinct shutdown consent; real stage events; verified readback; safe optional power restoration; stop/manual recovery; minimal durable journal. | Injected partial failures/timeout/crash/save failure do not cause blind retry or rollback; restart shows recovery warning. |
| **M3.7 Connect UI** | Replace mocks gradually with real discovery, GPU editor, credentials, single-VM draft prompts, progress/recovery and system information. | GUI/CLI produce same effective plan; conflicts block; incidental updates preserve user selection and draft. |
| **M3.8 Process + UX acceptance** | One GUI per session; activate-existing, deferred close, per-user preferences, renderer fallback, keyboard/accessibility, packaging inputs and negative tests. | Usable narrow/wide and at Windows scaling; no second GUI steals/drops state; tests gated before live qualification. |

Keep the detailed design in [`GUI_GUIDE.md`](GUI_GUIDE.md) and milestone acceptance criteria in [`GUI_ROADMAP.md`](GUI_ROADMAP.md). Do not maintain a second competing GUI implementation specification here.

## 6. R1 — Packaging and product acceptance

- **CORE-017:** Produce a clean, locked Windows x64 candidate with `hyper-gpu-support.exe` and only necessary ancillary guest/probe/runtime payloads. Verify fresh installation, UAC/worker invocation, ProgramData ACL creation, updates, missing prerequisites, CLI/GUI dispatch and safe removal. No unrelated disk or VM changes.
- **DOC-003:** Document actual install, discovery, first-time enrollment in right panel, allocation semantics, plan/apply/disable/verify, power approval, credentials, stale-state refresh, save-only retry and manual reconciliation. Use actual commands and paths only after verifying them; avoid developer-only paths.
- **GPU-014:** Test on explicitly designated disposable VM(s) **with user authorisation**: default and supported custom allocations, GPU selection/enrollment, fresh signed-driver preparation, PnP/checked D3D11, running no-op preservation, disable/restoration and representative uncertain/recovery path. Qualify changed privileged interfaces independently where repository workflow requires review.
- Include revision, checksums, target Windows requirements, licenses/notices and installation limitations. Ship no proprietary driver payload, VM disks, credentials or golden image. Original roadmap reported statically linked MSVC CRT; verify that remains true for the candidate rather than treating it as a perpetual guarantee.

**R1 exit:** M2 and M3 verified, no essential open blocker, shared CLI/GUI functionality and protected execution qualified, actual package/manual tested. Packaging does **not** authorise publication. Simultaneous same-GPU sharing is not required unless separately advertised.

## 7. M4 — After initial product

- **GPU-015 (conditional sharing):** Qualify two designated VMs independently using one GPU: attach/render, running no-op, and removal of one without disturbing the other. Document admission and support policy before advertising concurrent sharing; UI warnings alone do not enforce it. Do not introduce a scheduler without a demonstrated need.
- **Additional vendors:** Choose real supported hardware and signed preparation recipe; extend small Rust adapters and test changed behavior. No speculative generic vendor framework.
- **Optional research:** Provider units/enforcement beyond qualified targets, CUDA/stress, HCS or laboratory experiments only when tied to an approved backlog need. Host partition-count editing remains out of the chosen GUI scope.

## 8. Backlog reconciliation and unresolved evidence

Before implementation, Codex should **propose** (not silently execute) updates to `BACKLOG.md`:

1. Replace the old **GUI-001 Win32/table** acceptance text with Slint mock-first architecture, VM cards, right-hand hybrid panel and new operation workflow.
2. Bring **GPU-010** into the initial product's dependency path for all twelve validated allocation settings, keeping provider-specific uncertainty explicit.
3. Add/re-scope work cards for **single-binary GUI/CLI/worker dispatch**, **per-VM ProgramData storage**, **private authenticated Named Pipe**, **host-wide lock**, **save-only retry** and **startup reconciliation**; avoid duplicating existing root Rust capabilities.
4. Keep historical **M1/M2, BLK-005 and CORE-006** evidence/status accurately recorded; do not reopen completed diagnosis without reason.
5. Update **CORE-017 / DOC-003 / GPU-014** dependencies and acceptance for the new M3 scope. Remove Win32-preservation and defaults-only-v1 language, but retain test/qualification safeguards.

**Still to verify against repo and Windows host:** configuration format/migration (`OPEN-01`); lock and privileged-worker handshake (`OPEN-03`, `OPEN-05`); Windows 11 provider/API support (`OPEN-04`, `OPEN-10`); renderer fallback (`OPEN-06`); drafts across app pages/close (`OPEN-07`); precise manual reconciliation (`OPEN-08`); hybrid executable packaging (`OPEN-09`); accessibility (`OPEN-11`); securely scoped save-only authorisation (`OPEN-12`). Approved policies A18–A32 are not open design choices.

## 9. Immediate next step — planning only

**M3.0: Repository reality check.** Ask Codex to inspect the current root code and backlog using [`GUI_PROMPT.md`](GUI_PROMPT.md). The deliverable is an evidence-backed gap matrix, a compact module plan, necessary backlog changes and milestone refinements. **Stop for review.** No Rust/Slint/Cargo edits, builds, installation, privileged tests, VM power changes or GPU operations without separate instruction. Never touch the host lifecycle without explicit authorisation.
