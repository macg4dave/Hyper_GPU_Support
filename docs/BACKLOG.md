# GPU-PV Product Backlog

**Updated:** 9 October 2026  
**Status:** Architecture/planning rebase. No implementation or live testing is authorised by this document.  
**Companion documents:** [ROADMAP.md](ROADMAP.md) · [GUI_ROADMAP.md](GUI_ROADMAP.md) · [GUI_GUIDE.md](GUI_GUIDE.md) · [GUI_PROMPT.md](GUI_PROMPT.md) · [SLINT_CODEX_PROMPT.md](SLINT_CODEX_PROMPT.md)  
**Historical record:** [BACKLOG_HISTORY.md](BACKLOG_HISTORY.md) preserves the entire supplied 8 October backlog, including detailed card results, older scope, measurements and evidence references.

> **Ownership:** `ROADMAP.md` owns product direction and milestones; this file owns **task IDs, current planned status, priorities and dependencies**; `GUI_GUIDE.md` owns the user's agreed requirements; `GUI_ROADMAP.md` details the gated Slint slices. The 8 October status/evidence is inherited from the supplied backlog, **not** freshly validated against the current repository or host. The new task breakdown is a planning proposal until Codex's read-only audit confirms ownership and feasibility.

## Resume / next action

**Next: PLAN-001, read-only repository reality check (M3.0).** Examine the actual root Rust code, project instructions and existing evidence against the approved Slint architecture. Return a concise capability/gap matrix, module ownership, API/Windows constraints, necessary task adjustments and any decisions requiring user approval. **Stop for review; do not implement.**

The earlier Win32 `windows_gui.rs` is a disposable prototype, **not** the foundation of M3. Its previously reported tests do not qualify the new Slint GUI. A new `hyper-gpu-support.exe` must support no-argument Slint startup, explicit headless CLI commands, and a restricted elevated-worker mode using the same Rust core.

**Preserve reported progress:** M1 accepted on 8 October; M2 one-VM current-build NVIDIA attach/render/reapply/disable checks passed, but affected fresh driver preparation under current child limits remains to qualify. CORE-006 shared preview is reported complete. BLK-005 was closed by user instruction; its historical host-hang cause is unknown, **not** proven fixed. No further hang campaign is scheduled.

**Approved product shift:** Full physical GPU selection, first-time enrollment and the four allocation triples (VRAM, compute, encode, decode) are now **M3/R1 scope**, not deferred to post-v1. Host-wide GPU partition count remains read-only. Inability to support an advertised provider field requires evidence and a specific user scope decision; never silently turn the feature into a read-only/default-only release.

## Status and work rules

- **planned**: scoped but not authorised for implementation; **ready**: dependencies met for the stated, explicitly permitted action; **in progress**: evidence-backed ongoing work; **blocked**: observed impediment; **completed**: acceptance evidenced; **merged**: superseded/absorbed, not an independent gate; **deferred**: outside initial release; **cancelled**: no longer scheduled.
- `PLAN-001` is the sole **ready planning task**. Other newly proposed M3 cards are **planned**, pending its audit and later explicit authorisation. Do not infer implementation approval from their priority.
- Before acting, read the chosen card, relevant source, `AGENTS.md` / `ENGINEERING.md` where present and applicable decisions. Claim shared work only when the repo workflow requires it. Avoid unnecessary historical experiments.
- Reuse proven Rust contracts and native fixed-operation helpers. Product never depends on laboratory scripts, fixed VM slots, golden images, cloning/reset or arbitrary privileged commands. DEC-028's bounded PowerShell Direct bridge remains limited to the approved guest transport/bootstrap functions unless a new reviewed decision changes it.
- Apply proportional tests and review **changed** privileged boundaries. No tests-of-tests, broad telemetry framework, speculative refactor or repeated investigation of closed findings. Real VM operations, elevation, driver staging or VM power changes require **separate explicit permission**. Never restart/shut down/log out the **host** without immediate permission.
- Report what was inspected or tested versus what is merely planned. Record narrowly scoped results on the affected card; do not mark a gate complete from mocks, read-only parity or old Win32 evidence alone.

## Milestone status

| Milestone | Current status | Gate / ownership |
|---|---|---|
| **M1 — Product boundary** | **Completed (reported 2026-10-08)** | ARCH-001; protected native runner, existing-VM enrollment, lab separation. [M1 evidence](evidence/M1.md). |
| **M2 — NVIDIA core** | **In progress** | ARCH-001, GPU-012, CORE-012. Current-build observed repeat passed; fresh preparation under new limits remains. [M2 evidence](evidence/M2.md). |
| **M3 — Slint and editable GPU config** | **Planned; design agreed** | PLAN-001, GUI-001, APP-001, CFG-001, GPU-010, SEC-001, CORE-028, GUI-002, GUI-003; no Slint implementation claimed. |
| **R1 — Packaged candidate** | **Planned** | CORE-017, DOC-003, GPU-014 after M2/M3 acceptance. |
| **M4 — Sharing and vendors** | **Deferred** | GPU-015 and individually justified vendor/optional tasks. |

## Active and proposed task register

This register is authoritative for **new scope/status/dependency planning**. Task summaries follow; retained historical records are in [BACKLOG_HISTORY.md](BACKLOG_HISTORY.md). IDs already used by the old backlog are never repurposed to mean unrelated work.

| ID | Milestone | Priority | Status | Prerequisites / notes |
|---|---|---|---|---|
| [PLAN-001](#plan-001) | M3.0 | P0 | **ready — read-only only** | Approved design docs and current repo available; no source edits |
| [ARCH-001](#arch-001) | M1/M2 | P0 | **in progress** | M1 completed; affected M2 fresh preparation still open |
| [GPU-012](#gpu-012) | M2 | P1 | **in progress** | Explicitly authorised bounded test; existing M1/M2 evidence |
| [CORE-012](#core-012) | M2/M3 | P1 | **ready (after task authorisation)** | Existing observed-state/journal/error contracts |
| [CORE-021](#core-021) | M2/M3 | P1 | **in progress (rescope at audit)** | Schema-2 baseline; avoid duplicate CFG-001/GUI-002 ownership |
| [CORE-006](#core-006) | M2 | P1 | **completed (reported)** | Reuse shared plan/apply preview; extend only for new operations |
| [GUI-001](#gui-001) | M3.1 | P1 | **planned — re-scoped** | PLAN-001; Slint mock shell, **not** Win32 continuation |
| [APP-001](#app-001) | M3.2 | P0 | **planned — new** | PLAN-001; single exe GUI/CLI/worker routing |
| [CFG-001](#cfg-001) | M3.3 | P0 | **planned — new** | PLAN-001, shared model audit; worker write integration depends on SEC-001 |
| [GPU-010](#gpu-010) | M3.4 | P0 | **planned — promoted from M4** | PLAN-001; provider/API validation and selected-GPU capability model |
| [SEC-001](#sec-001) | M3.5 | P0 | **planned — new** | PLAN-001, APP-001; fixed elevated worker, Named Pipe, host lock |
| [CORE-028](#core-028) | M3.6 | P0 | **planned — new** | CORE-006, CFG-001, GPU-010, SEC-001; real progress/manual recovery |
| [GUI-002](#gui-002) | M3.7 | P1 | **planned — new** | GUI-001, contracts + safe backend; incremental real binding |
| [GUI-003](#gui-003) | M3.8 | P1 | **planned — new** | GUI-002 and APP-001; accessibility, lifetime, scaling and UX checks |
| [CORE-017](#core-017) | R1 | P1 | **planned** | M2/M3 gate; packaging can be prepared independently |
| [DOC-003](#doc-003) | R1 | P1 | **planned** | Verified CLI/GUI, configuration and recovery behaviour |
| [GPU-014](#gpu-014) | R1 | P1 | **planned** | M2, M3, CORE-017, DOC-003; authorised candidate-only VM test |
| [GPU-015](#gpu-015) | M4 | P2 | **deferred** | Two explicitly designated VMs, support/admission policy |

**Task-ID changes:** `PLAN-001`, `APP-001`, `CFG-001`, `SEC-001`, `CORE-028`, `GUI-002` and `GUI-003` are **new proposals**. `GUI-001` remains the GUI card but its unfinished Win32 acceptance is retired in favour of Slint. `GPU-010` keeps its ID and is promoted to M3. Historical Win32 progress remains archived, not deleted or converted into Slint progress.

## M2 — Retain existing core progress

### ARCH-001

**Native existing-VM Rust core.** M1 was independently reviewed and live-qualified on 8 October. The root product reportedly has native discovery, protected exact VM/GPU enrollment, signed current-NVIDIA-driver preparation, fixed runner, per-VM recovery journals and ordinary health/checked D3D11 verification. The observed repeat on revision `96152e7` passed default attach, PnP, D3D11, off/running reapply, verification and disable without reported slowdown/beeps. The earlier hang cause is unknown and its investigation is closed by user direction.

**Remaining M2 gate:** proportionately qualify affected **fresh** preparation under the current child/resource limits and reconcile actual current root-code status. Preserve existing VM disks, CPU/RAM, security devices, credentials, ACL/reparse guards, preimages and uncertain-state handling. Do not recreate an old laboratory migration programme.

### GPU-012

**Finish bounded NVIDIA stability/maintenance qualification.** Reuse reported M1/M2 passes; focus on paths changed since those results. Qualified work includes signed current-driver preparation, default GPU attach, PnP/checked D3D11, running no-op preservation, reapply, disable and initial-state restoration. Do not require a manufactured driver upgrade, prolonged stress, two recreated children, CUDA/D3D12 or a new hang-reproduction campaign.

**Exit:** a precise affected-path test report on the designated target **after explicit test authorisation**, including any unresolved limitations. No host lifecycle operations without separate authorisation.

### CORE-012

**Truthful observed state and operator errors.** Distinguish desired configuration, observed GPU attachment, previous driver preparation, pending recovery and **last** successful graphics verification. Report missing/denied/unavailable/unsupported/unknown separately; avoid turning one VM inventory failure into an empty result for all. Provide bounded, redacted stage/error and safe-next-action messages, shared by CLI and Slint.

**Exit:** isolated partial-access, stale-state, missing-provider and secret-redaction tests; accurately explained provenance/freshness; no persistent diagnostic database. Ordinary visible history is session-only; minimal durable recovery records remain separate.

### CORE-021

**Configuration and enrollment UX — reconcile with new architecture.** Previously implemented runtime schema 2 and exact native enrollment are starting points, **not** an instruction to preserve a single config file or make the Win32 prototype authoritative. M3 requires per-VM GUID-keyed configuration, direct first-time GPU enrollment from the right-hand panel and one source of rules for CLI/GUI.

**Audit action:** decide which existing parser, enrollment and credential pieces move under CFG-001 / GUI-002; eliminate duplicate ownership. Retain completed work/evidence. Do not mark CORE-021 complete until its residual contract/documentation responsibilities are explicitly assigned and tested.

### CORE-006

**Shared effect preview — completed baseline, extend not rebuild.** The 8 October card reports typed plan/apply summaries and read-only preview tests for existing operations, including state, driver, credentials and downtime. New physical GPU selection and twelve-field allocation require corresponding validated extensions under GPU-010 / CORE-028. Fresh execution must independently recheck approved targets; no preview may broaden enrollment. [Historical result](BACKLOG_HISTORY.md#core-006).

## M3 — Slint and expanded product architecture

### PLAN-001

**Read-only repository reality check — next task.** Inspect `ROADMAP.md`, `GUI_GUIDE.md`, `GUI_ROADMAP.md`, applicable `AGENTS.md` / `ENGINEERING.md`, root Cargo manifests, CLI entry point, model/parser, inventory, workflow, runner, enrollment, journaling, guest writer and Windows API integration. Consult up-to-date Slint/provider documentation where assumptions depend on actual APIs.

**Deliver:** (1) evidence-backed existing/missing/conditional capabilities; (2) compact Rust/Slint module plan; (3) differences from approved A18–A32 policies; (4) security/IPC/config/schema feasibility and risk; (5) proposed revised card dependencies and explicit open decisions. Preserve proven functions and identify unnecessary compatibility/laboratory coupling.

**Exit:** independent review of the audit results **with the user**. No implementation, dependency edits, build, installation, elevated helper, guest credentials, VM/host operation or live GPU-P test under this card.

### GUI-001

**New modular Slint shell — mock-first; old Win32 work is historical.** Build once authorised using standard Slint Fluent widgets, shared theme/spacing, sidebar, vertically scrollable **VM cards** (all discovered states), search/status filters, left/right split with adjustable divider, always-two-panel layout and **horizontal scrolling** at narrow widths. Right-hand hybrid details with expandable advanced information and four resource categories. Do not port `windows_gui.rs`, implement monolithic `.rs`/`.slint` files, or place Hyper-V rules in UI components.

**Exit:** previews without Hyper-V or elevation; usable mock data for empty, missing, denied, unknown, long-name, multi-VM and pending-draft states; practical resize/DPI/keyboard/focus checks. No claim of real GPU functionality from mocks.

### APP-001

**Single executable, three entry modes.** No arguments open the Slint GUI and begin read-only background discovery; explicit subcommands run CLI with no Slint initialisation; an internal constrained worker mode may run only following validated privileged launch. Normal second GUI launch activates the existing window **within the same Windows session** without losing drafts. Different sessions may inspect simultaneously.

**Exit:** on supported Windows versions, GUI startup has no unwanted console; CLI reliably writes stdout/stderr and exit status; worker entry cannot be invoked as an unrestricted public command; tests cover dispatch, malformed arguments and session activation. Confirm Windows subsystem constraints before locking implementation.

### CFG-001

**Per-VM machine-wide configuration and safe saving.** Replace/extend schema 2 as justified by the repository audit; use stable Hyper-V VM GUID files under `%ProgramData%\HyperGpuSupport\config\vms\` (format/version **OPEN-01**). Keep **observed**, **committed desired** and **one in-memory draft** separate. Store per-user window/theme/split preferences elsewhere. Enrollment/operational recovery records remain protected separately, with appropriate installer-created ACLs.

**Exit:** no saving while merely editing; after authorised Apply and verified readback, **only the elevated worker** atomically commits the VM configuration. Detect externally modified files/VM state and **block with refresh**, never auto-merge. Verified GPU success followed by save failure offers **save-only retry**, never repeat Apply. Test wrong GUID/name collision, torn writes, ACL/reparse issues, external edits and recovery. Protected write integration requires SEC-001.

### GPU-010

**Selected physical GPU and complete allocation model — required for initial release.** Extend the existing raw VRAM capability rather than rewrite it. Discover eligible GPUs and report capability/unknown state. Implement validated per-VM **VRAM, Compute, Encode and Decode** triples (Min/Optimal/Max), selected GPU identity and first-time/re-enrollment plan semantics. Provider-reported initial values may be suggested but must be labelled as such; no invented units, GiB, percent or performance guarantees. Host GPU partition count remains **read-only**.

**Exit:** for the selected host/build/vendor, each editable field has known input semantics, bounds and effective readback; CLI and GUI share validation/plan. If provider APIs or a category are unavailable, show a truthful block and bring the scoped limitation to the user **before** changing product requirements. No automatic concurrent-sharing claim, fairness promise or host-wide tuning.

### SEC-001

**Short-lived authorised worker, private Named Pipe and host-wide coordination.** One approved modifying operation launches a restricted elevated instance of the **same executable**. Reuse the protected runner/fixed-operation policy. The frontend and worker communicate through private local Windows Named Pipes with strict ACLs, authenticated peer/operation/plan identity, bounded versioned typed messages, exact VM/GPU identity checks, timeouts and replay rejection. The worker independently revalidates every request; it never accepts arbitrary scripts, files or shell commands.

**Coordination:** at most **one modifying GPU operation across the entire host**, including GUI, CLI and workers. Read-only inventory can continue where safe. Lock release after exit/crash is **not** equivalent to recovery resolution. Elevate before any VM shutdown; UAC denial retains the draft and causes no VM mutation. Distinguish UI approval, Windows elevation and guest credentials.

**Exit:** mock/negative tests for denied UAC, malicious/malformed/oversized requests, stale plan and altered device identity, wrong peer/session, worker failure, timeouts, pipe loss and concurrent CLI/GUI requests. Confirm changed privileged boundary through repository-required independent review before any live use.

### CORE-028

**Adaptive operation execution, progress, verified persistence and manual recovery.** Reuse CORE-006's shared planner for first-time enrollment, selected-GPU changes, four resource triples and enable/disable/reapply. For complex operations disclose approved protected actions and request **separate explicit graceful shutdown approval**. Before effects, record sufficient durable recovery intent; then execute validated fixed steps with real Pending/Running/Done/Failed/Unknown stage messages over the pipe. Read back real Hyper-V state; save the matching per-VM config via the worker after success.

**Failure policy:** stop further GPU modifications; report completed/uncertain stages; no blind retries, automatic GPU rollback, corrective reassignment or automatic recovery-record clearing. Restore originally running VM only when genuinely safe and already authorised; never silently force-off or modify host power. Worker crash/disconnect is **uncertain**. On restart keep a persistent dashboard recovery warning and block new GPU modifications until **manual reconciliation** verifies safety. Session-only human diagnostics are distinct from minimal durable recovery records.

**Exit:** fault-injected crash, timeout, partial state, unsafe restart, UAC refusal, stale plan, save failure and save-only retry; no GPU operation replay. Exact reconciliation mechanics remain **OPEN-08** until independently reviewed.

### GUI-002

**Connect the Slint interface to the verified shared Rust core.** Replace mock data incrementally: automatic discovery with partial-access states; GUID-keyed VM cards/selection; physical-GPU dropdown; expandable allocation editor; one draft; pending-draft dialog before **switching VMs**; fresh Review & Apply; UAC/downtime approvals; credential access through existing protected facilities; true progress and persistent recovery banner. No full-list rebuild for incidental UI changes.

**Exit:** identical effective plans and error semantics in CLI and GUI; switching/stale change never silently discards or applies a draft; unsupported GPU/provider states are explained; partial provider failure does not empty the list; side effects occur only through the shared core and restricted worker.

### GUI-003

**Process lifetime and usability acceptance.** One GUI per Windows session; second launch activates/raises existing window with current draft intact. Defer normal close while worker is active, permit minimisation; handle unreachable workers and crashes without indefinite hang. Preserve user preferences separately from operation records. Verify permanent two-panel horizontal-scroll design, draggable splitter, independent vertical scrolling, resizing, high DPI/text scaling, theme, keyboard use, accessible labels/focus, long names and GUI rendering when the graphics driver is unhealthy.

**Exit:** mock/negative UX checks and verified Slint renderer/accessibility limitations; no claimed screen-reader parity without testing; no old Win32 feature-parity requirement.

## R1 — Packaging and candidate acceptance

### CORE-017

**Build the combined Windows x64 candidate.** Package `hyper-gpu-support.exe` plus only required fixed guest/worker/probe payloads. Verify mode dispatch, UAC launch/pipe ACLs, ProgramData directory installation/upgrades/removal, signed-driver discovery, missing prerequisites, read-only startup, installer interruption and licensing/notices. The executable may spawn its elevated instance, but no always-on management service is required. Exclude proprietary host drivers, VM disks/media, secrets, keys and laboratory fixtures. Verify current CRT/linking assumptions instead of relying on historical build statements.

**Exit:** clean candidate and recorded version/revision/checksums; no developer paths or essential missing binary. Packaging does not authorise publication.

### DOC-003

**Tested GUI/CLI operator guide.** Document single-exe startup/commands; per-VM ProgramData configuration, direct right-panel GPU enrollment and all supported allocation fields; review/apply/UAC/shutdown approvals; driver preparation and graphics verification; session diagnostics; stale-state refresh, save-only retry, failure/manual reconciliation; and honest provider, vendor and concurrent-sharing limitations. Do not document unimplemented commands or use historic lab/golden-image paths as product requirements.

**Exit:** instructions validated against the actual packaged candidate and supported host/guest combinations.

### GPU-014

**Authorised end-to-end product qualification.** On explicitly selected disposable existing VM(s), exercise clean packaged startup, discovery, enrollment, GPU selection and supported custom twelve-field allocations, signed-driver preparation, exact attachment, PnP/checked D3D11, running no-op, reapply, disable/restoration, CLI/GUI parity and representative interrupted/recovery/save-only scenarios. Reuse earlier M1/M2 evidence and run only relevant changed paths. A second concurrent VM is **not** a gate unless sharing is advertised.

**Exit:** reviewed actual run report, state/host safety, security boundaries, package/manual parity and no essential unresolved blocker. Any live VM or host-affecting action requires specific permission. No automatic host restart.

## Deferred, merged and completed records

These IDs are preserved. This is a **status index**, not a request to rerun their original procedures; see [BACKLOG_HISTORY.md](BACKLOG_HISTORY.md) for the full source cards and evidence. Completed claims are the historical backlog's claims, not new tests.

| IDs | Historical disposition / contribution |
|---|---|
| CORE-022, CORE-023, CORE-003, CORE-024 | **Completed** historical standalone-lab driver staging, VM profile, probe automation and native discovery; production equivalent audited under ARCH-001. |
| CORE-027 | **Completed** root native installation/enrollment, M1 qualified. |
| CORE-025 | **Deferred lab history**; root native product operations belong to ARCH-001, no legacy port release gate. |
| CORE-026, CORE-010, CORE-015 | **Merged** into ARCH-001; guest provisioning, removal/recovery and driver drift refresh. |
| CORE-011 | **Deferred** optional standalone guest lifecycle commands, not required for ordinary GPU-P journey. |
| GPU-006 | **Deferred lab history**; clean-child laboratory reproduction is not production setup. |
| GPU-007, GPU-013, GPU-016, GPU-017 | **Deferred/post-v1** optional API workload, real driver transition, CUDA identity correlation and recipe optimisation. |
| DOC-001, DOC-002, DOC-007, DOC-008, DOC-009, DOC-010, DOC-011 | **Completed** historical documentation, design, engineering and review practices. |
| CORE-019, CORE-001, CORE-002, CORE-004, CORE-005, CORE-008, CORE-009, CORE-020 | **Completed** earlier Rust core, inventory, bounded runner, guest writer, tests and probes. |
| GPU-001, GPU-002, GPU-003, GPU-005, GPU-008, GPU-009, HV-001, HV-002, HV-003, REF-001, REF-002, REF-004 | **Completed** historical research/baseline/provider/laboratory evidence; see archive for exact details. |
| GPU-004 | **Cancelled** reference comparison; no replacement release task. |
| CORE-007, CORE-013, CORE-014, CORE-016, CORE-018, GPU-011, DOC-004, DOC-005, DOC-006, REF-003 | **Merged/retired** into original ownership; IDs never reused. |

### BLK-005 and earlier blockers

**BLK-005: closed by user direction 8 October 2026.** Current-build repeat did not reproduce host slowdown/beeps; root cause remains unknown and is not claimed fixed. No repeat investigation without fresh evidence and approval. **BLK-001, BLK-002 and BLK-004 resolved; BLK-003 closed** per historical register. Retain the original details/evidence in [BACKLOG_HISTORY.md](BACKLOG_HISTORY.md#blocker-register).

## Unresolved technical verification — not permission to change approved policy

Codex must report evidence and proposed choices for the outstanding `GUI_GUIDE.md` items, especially:

| ID | Verification needed |
|---|---|
| OPEN-01 | Per-VM config format/schema version and safe migration of existing real files |
| OPEN-03 / OPEN-05 | Cross-process lock/worker handover; Named Pipe ACL, peer authentication, plan binding and launch semantics |
| OPEN-04 / OPEN-10 | Actual Windows 11 GPU-P provider APIs, GPU selection, 12 field units/bounds/unset/readback/enforcement |
| OPEN-06 | Slint renderer and practical software fallback for unhealthy GPU drivers |
| OPEN-07 | Draft behaviour when switching non-VM pages or closing idle GUI |
| OPEN-08 | Exact manual reconciliation/clearance rules for incomplete or uncertain operations |
| OPEN-09 | Windows GUI/CLI console subsystem, installer/UAC/ProgramData and session activation |
| OPEN-11 | Actual keyboard, text-scale and assistive-tech support |
| OPEN-12 | Restrict save-only worker authorisation without allowing arbitrary config rewriting |

The **policies A18–A32 are agreed**; the details above are implementation investigations, not invitations to silently reverse those user decisions.

## Acceptance and next handoff

A card closes only with its own acceptance evidence. M3 closes only after the common Rust core, Slint frontend, editable supported allocation, restricted worker, typed Named Pipe, host-wide coordination, ProgramData persistence, crash recovery and verified GUI/CLI behaviour pass their checks. R1 then requires actual packaging and explicitly authorised target qualification.

**Immediate Codex prompt:** “Read `GUI_PROMPT.md`, `GUI_GUIDE.md`, `GUI_ROADMAP.md`, `ROADMAP.md` and `BACKLOG.md`. Perform **PLAN-001** only: read-only audit, gap matrix and proposed card refinements. Do not implement or test on real Hyper-V until separately approved.”
