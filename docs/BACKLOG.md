# GPU-PV Product Backlog

**Updated:** 10 October 2026
**Status:** Main Slint GUI promotion implemented; backend binding and candidate validation remain.
**Companion documents:** [ROADMAP.md](ROADMAP.md) · [GUI_ROADMAP.md](GUI_ROADMAP.md) · [GUI_GUIDE.md](GUI_GUIDE.md) · [GUI_PROMPT.md](GUI_PROMPT.md) · [Slint implementation prompt](../.github/prompts/SLINT_CODEX_PROMPT.md)
**Historical record:** [BACKLOG_HISTORY.md](BACKLOG_HISTORY.md) preserves the entire supplied 8 October backlog, including detailed card results, older scope, measurements and evidence references.

> **Ownership:** `ROADMAP.md` owns product milestones; this file owns task status,
> dependencies and acceptance; `GUI_GUIDE.md` owns approved requirements;
> `GUI_ROADMAP.md` maps slices to cards. The 9 October [source audit](ARCHITECTURE.md#repository-audit--9-october-2026)
> confirms reuse/gaps, not new hardware results. Preserve 8 October evidence.

## Resume / next action

**v1.0 priority:** Deliver the approved completed Slint prototype as the main
Windows application, using existing Rust backend functionality. GUI-001 is completed
history. APP-001's GUI/CLI dispatch, restricted-worker integration and console
packaging are completed and reviewed. **OPEN-01's format/import contract is resolved;
CFG-001's storage implementation remains open.** Use the shared per-VM schema,
samples and saved/observed/draft split in [CONFIGURATION](CONFIGURATION.md)
when binding existing controls; no redesign or restored search/filter/Activity.

Preserve M1/M2/CORE-006 evidence and BLK-005's closed investigation. Normal startup
uses actual discovery and blocks unconnected actions; `--mock-gui` selects rehearsal.
Next reuse the selected production parser/model for read-only
configuration and plan rehearsal, without persistent writes.
No new hardware qualification is claimed.
The CLI/backend keep functionality absent from the interface. For this promotion
task, live Hyper-V/GPU/driver/VM-power operations require explicit permission.

## Status and work rules

- **planned**: scoped but not authorised for implementation; **ready**: dependencies met for the stated, explicitly permitted action; **in progress**: evidence-backed ongoing work; **blocked**: observed impediment; **completed**: acceptance evidenced; **merged**: superseded/absorbed, not an independent gate; **deferred**: outside initial release; **cancelled**: no longer scheduled.
- PLAN-001 and prototype design are complete; remaining cards own backend binding
  and release acceptance. The active user request authorizes development checks.
- Before acting, read the chosen card, relevant source, `AGENTS.md` / `ENGINEERING.md` where present and applicable decisions. Claim shared work only when the repo workflow requires it. Avoid unnecessary historical experiments.
- Reuse proven Rust contracts and native fixed-operation helpers. Product never depends on laboratory scripts, fixed VM slots, golden images, cloning/reset or arbitrary privileged commands. DEC-028's bounded PowerShell Direct bridge remains limited to the approved guest transport/bootstrap functions unless a new reviewed decision changes it.
- Apply proportional tests and review changed privileged boundaries. Follow AGENTS
  for designated-disposable testing authorisation and immediate host lifecycle
  permission. Mock GUI development never triggers real Hyper-V effects. This
  promotion task permits builds and mock checks, not live effects without permission.
- Report what was inspected or tested versus what is merely planned. Record narrowly scoped results on the affected card; do not mark a gate complete from mocks, read-only parity or old Win32 evidence alone.

## Milestone status

| Milestone | Current status | Gate / ownership |
|---|---|---|
| **M1 — Product boundary** | **Completed (reported 2026-10-08)** | ARCH-001; protected native runner, existing-VM enrollment, lab separation. [M1 evidence](evidence/M1.md). |
| **M2 — NVIDIA core** | **In progress** | ARCH-001, GPU-012, CORE-012. Current-build observed repeat passed; fresh preparation under new limits remains. [M2 evidence](evidence/M2.md). |
| **M3 — Slint and editable GPU config** | **In progress; approved prototype promoted** | PLAN-001, GUI-001, APP-001, CFG-001, GPU-010, SEC-001, CORE-028, GUI-002, GUI-003; main Slint GUI has live discovery and explicit mock rehearsal; configuration/effect bindings remain. |
| **R1 — Packaged candidate** | **Planned** | CORE-017, DOC-003, GPU-014 after M2/M3 acceptance. |
| **M4 — Sharing and vendors** | **Deferred** | GPU-015 and individually justified vendor/optional tasks. |

## Active and proposed task register

This register is authoritative for **new scope/status/dependency planning**. Task summaries follow; retained historical records are in [BACKLOG_HISTORY.md](BACKLOG_HISTORY.md). IDs already used by the old backlog are never repurposed to mean unrelated work.

| ID | Milestone | Priority | Status | Prerequisites / notes |
|---|---|---|---|---|
| [PLAN-001](#plan-001) | M3.0 | P0 | **completed — repository/docs scope** | 9 Oct source audit; runtime/provider feasibility remains on owning cards |
| [ARCH-001](#arch-001) | M1/M2 | P0 | **in progress** | M1 completed; affected M2 fresh preparation still open |
| [GPU-012](#gpu-012) | M2 | P1 | **in progress** | Explicitly authorised bounded test; existing M1/M2 evidence |
| [CORE-012](#core-012) | M2/M3 | P1 | **completed** | Shared truthful reporting, partial inventory and redaction qualified 10 October |
| [CORE-021](#core-021) | M2/M3 | P1 | **in progress** | Integration/doc residuals; parser/storage CFG-001, enrollment SEC-001, UI GUI-002 |
| [CORE-006](#core-006) | M2 | P1 | **completed (reported)** | Reuse shared plan/apply preview; extend only for new operations |
| [APP-001](#app-001) | M3.2 | P0 | **completed** | GUI console packaging and installed restricted-worker integration qualified and reviewed 10 October |
| [CFG-001](#cfg-001) | M3.3 | P0 | **in progress — format selected** | OPEN-01 format/import contract resolved; protected store/write integration depends on SEC-001 |
| [GPU-010](#gpu-010) | M3.4 | P0 | **planned — promoted from M4** | PLAN-001; provider/API validation and selected-GPU capability model |
| [SEC-001](#sec-001) | M3.5 | P0 | **planned — extension** | APP-001; adapt existing protected runner/pipe/lock, no replacement stack |
| [CORE-028](#core-028) | M3.6 | P0 | **planned — new** | CORE-006, CFG-001, GPU-010, SEC-001; real progress/manual recovery |
| [GUI-002](#gui-002) | M3.7 | P0 | **in progress — live discovery and explicit mock mode** | Existing cards/Refresh/System; effects need CFG-001/GPU-010/SEC-001/CORE-028; no-write real-data rehearsal source remains |
| [GUI-003](#gui-003) | M3.8 | P1 | **in progress** | Mock layout/themes/keyboard/draft/recovery checks qualified; lifetime and remaining Windows checks open |
| [CORE-017](#core-017) | R1 | P1 | **planned** | M2/M3 gate; packaging can be prepared independently |
| [DOC-003](#doc-003) | R1 | P1 | **planned** | Verified CLI/GUI, configuration and recovery behaviour |
| [GPU-014](#gpu-014) | R1 | P1 | **planned** | M2, M3, CORE-017, DOC-003; authorised candidate-only VM test |
| [GPU-015](#gpu-015) | M4 | P2 | **deferred** | Two explicitly designated VMs, support/admission policy |

## Completed prototype history

| ID | Result |
|---|---|
| [GUI-001](#gui-001) | Approved completed prototype; promoted under APP-001. Historical results retained below; accessibility/scaling qualification belongs to GUI-003. |
| PLAN-001 | Repository/source documentation audit complete; retained below. |

Earlier GUI requirements absent from the approved prototype are retired from v1.0.
Task IDs and backend work remain; no duplicate design cards or restored controls.

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

**Completed, 10 October:** shared public diagnostics and recorded-state provenance;
partial failures preserve available inventory, raw errors are redacted and recovery
guidance remains conditional. Isolated behavior tests, CLI processes and actual
Slint rehearsal passed; independent review cleared the result. See
[closure evidence](evidence/closure-sprint.md).

### CORE-021

**Configuration and enrollment UX — reconcile with new architecture.** Previously implemented runtime schema 2 and exact native enrollment are starting points, **not** an instruction to preserve a single config file or make the Win32 prototype authoritative. M3 requires per-VM GUID-keyed configuration, direct first-time GPU enrollment from the right-hand panel and one source of rules for CLI/GUI.

**Residual ownership:** CFG-001 extends parser/storage/conflict handling; SEC-001
owns enrollment/elevated launch adaptation; GPU-010 owns selection/allocation;
GUI-002 reuses credentials and presentation state. CORE-021 retains cross-interface
configuration/enrollment guidance and integration acceptance; it does not build
another parser, vault or enrollment backend. Retain completed work/evidence.

### CORE-006

**Shared effect preview — completed baseline, extend not rebuild.** The 8 October card reports typed plan/apply summaries and read-only preview tests for existing operations, including state, driver, credentials and downtime. New physical GPU selection and twelve-field allocation require corresponding validated extensions under GPU-010 / CORE-028. Fresh execution must independently recheck approved targets; no preview may broaden enrollment. [Historical result](BACKLOG_HISTORY.md#core-006).

## M3 — Slint and expanded product architecture

### PLAN-001

**Repository audit and documentation reconciliation — completed scoped result,
9 October.** Inspected manifests, CLI/model/GUI state, native allocation, runner,
Named Pipe, workflow/journals and relevant test source; corrected current docs.
The [audit matrix](ARCHITECTURE.md#repository-audit--9-october-2026) records existing
capabilities/gaps; GUI_GUIDE module responsibilities remain the compact UI plan.
Existing IPC, lock, journals, parser and backend are reused. Historical passes were
preserved, not rerun. No source/config/script edits, builds or host/VM access.

**Remaining investigation reassigned:** actual provider semantics/qualification to
GPU-010; Slint version/layout/runtime checks to GUI-001/GUI-003; console/dispatch to
APP-001; schema/save binding to CFG-001; elevation/handshake/coordination to SEC-001;
manual reconciliation to CORE-028. Completion covers the requested source/docs audit,
not live/API feasibility or implementation of those policies.

**Scoped exit:** source-backed corrections and task ownership recorded; check local
links/task dependencies and diff hygiene. No code, builds or live operations. The
user can review the concrete documentation diff; implementation remains planned.

### GUI-001

**Header and diagnostics:** Removed VM Selection; heading/Refresh share one row.
VM cards have measured 12px layout gaps in narrow/wide and 35-VM scenarios.
Reusable read-only TextInput/ScrollView keeps dialog logs black-on-white and
monospaced in both themes, with bounded height and the existing bottom toggle.
Running-app mouse partial copy, keyboard line copy/Ctrl+C, blocked edit/cut/paste,
two-axis wheel scrolling, Home/End and 700x520 layouts passed; a 500-line live
viewer fixture passed the same interaction checks without moving actions offscreen.
Viewer compile, prototype build, format, strict Clippy and allocation test passed.

**Layout usability:** Removed prototype VM search/filter controls and state;
names now use responsive left-aligned ellipsis with full-name tooltips and
keyboard-accessible card selection. All dialogs share a bottom technical toggle
below their actions and a bounded technical scroll area. Viewer/live preview,
prototype build, format, strict Clippy/test and verified application checks at
1240x860 / 700x520, minimum 250px divider, long names, light/dark review and
recovery layouts passed. Toggle geometry stays 210x32 in both states.

**Visual polish:** Neutral light/dark surfaces and restrained accent use; removed
navigation headings; shared 210×32 technical-detail button verified in both dialog
states; VM dots reflect displayed power and OS icons use mock metadata with generic
fallback. Local assets require no new dependency. Viewer compilation, wide/narrow
renders, verified running-app light/dark/dialog checks and prototype format,
strict Clippy/test passed. Backend operations remain untouched.

**UI refinement:** Advanced Allocation now defaults to collapsed with a clickable
chevron heading. GPU Memory has an independent mock 1–8 GB slider, separate from
provider allocations. Navigation and VM selection have contrasting panel regions;
Activity and unused history state are removed. Viewer/live preview, build,
formatting, strict prototype Clippy/test, expand/edit/collapse preservation,
slider snap/discard, page navigation, theme switching and narrow layout checked.

**9 October prototype:** User explicitly requested the exploratory UI without
backend integration. Implemented the isolated Rust/Slint 1.18.1 package at
`tools/gui-prototype/`: initially Dashboard, Activity, System, Settings and About; mock
inventory/scenarios, draft editing, twelve fields, review/progress/recovery,
themes and adjustable scrolling workspace. Activity was subsequently removed;
the completed Dashboard/System/Settings/About prototype is now authoritative.
Launch and limits are documented
in its README. Production integration remains with its own cards. Full screen
reader, Windows text-scaling and multi-monitor qualification remain open;
this does not close M3 or qualify GPU functionality.

**Checks:** Slint viewer compile/live preview and large/narrow layout renders;
verified Rust application navigation, edit/discard, dirty-VM switch, themes,
splitter drag, detail scrolling, mock Apply, failure/reconciliation and save-only
recovery. Prototype build, formatting, strict Clippy and its allocation test pass;
root formatting/strict Clippy and 69 core + 3 CLI tests pass (one privileged
integration test remains ignored). Independent prototype review cleared.

**Promotion:** Approved prototype sources moved to `src/gui/ui/` and `src/gui/`.
No further prototype design is scheduled. Search/filter/Activity requirements and
Win32 parity are retired. Remaining visual/accessibility release checks belong to
GUI-003, not unfinished prototype development.

### APP-001

**Main application promotion — completed.**
No arguments open the approved Slint GUI; explicit CLI commands execute the existing
headless path. Sources were moved, not duplicated. Deleted old Win32 presentation
and `hyper-gpu-gui` entry; retain `gui_model` and all shared core/runner functionality.
The original promotion used explicit mock callbacks. GUI-002 now owns normal live
discovery and the isolated `--mock-gui` rehearsal path.

**Completed, 10 October:** Windows GUI-subsystem packaging preserves redirected
CLI stdout/stderr/exit codes and actual parent-console output. The existing
restricted same-executable worker was independently reviewed and exercised through
an installed authenticated exchange; bounded pipe and internal-mode rejection tests
passed. The protected SYSTEM runner remains for its existing fixed-operation role.
See [closure evidence](evidence/closure-sprint.md). Expanded SEC-001 enrollment and
coordination acceptance remains on its own card. Second-instance
activation and persistent preferences absent from the prototype are retired GUI
requirements. No arbitrary internal worker mode is introduced by this promotion.

**Validation (9 October promotion):** Windows x64 MSVC `cargo build --locked`,
`cargo fmt --all -- --check`, strict workspace/all-target/all-feature Clippy and
workspace/all-feature tests passed: 69 core, 2 GUI mock and 4 CLI process tests;
one installed-runner enrollment test intentionally ignored. CLI tests exercise
help/version/error paths with an invalid Slint backend to prove independent dispatch.
No live CLI effect, GPU/driver modification or guest-power operation was performed.

The main executable launched with no arguments. Development-only `slint/mcp`
inspection checked all four pages, Light/Dark/Follow Windows controls, drafts across
page navigation, VM-switch confirmation, discard, advanced allocation, technical
review, mock Apply and failure/reconciliation. Renders inspected at 1240x860 and
704x561; splitter drag readback moved x495 to x435. Corrected the Follow Windows
Unknown-sentinel surface mismatch without changing palette colors or layout.
Sources now use the user's final `src/gui/ui/` location; build and current links
match. Local development evidence is under ignored `local/evidence/gui-promotion/`.

Independent architecture review found no blocking issues and checked preserved
UI/CLI/security contracts and corrected theme renders. It did not rerun the reported
build/test suite. Documentation link/diff checks passed for 57 Markdown files.
Screen readers, DPI/text scaling, multi-monitor, broken-driver rendering and live
backend binding remain unqualified. `cargo-audit`/`cargo-deny` were unavailable;
distribution license/advisory review remains part of CORE-017, not a claimed pass.

### CFG-001

**Active closure work:** protected store, revision/readback-bound publication and
durable save-only recovery; integrate the same contract into GUI and CLI. Do not
mark complete before implementation, qualification and independent boundary review.

**Per-VM machine-wide configuration and safe saving.** Replace/extend schema 2 as justified by the repository audit; use stable Hyper-V VM GUID files under `%ProgramData%\HyperGpuSupport\config\vms\` (format/version **OPEN-01**). Keep **observed**, **committed desired** and **one in-memory draft** separate. Current appearance/window/split preferences remain session-only; persistence absent from the approved prototype is retired GUI scope. Enrollment/operational recovery records remain protected separately, with appropriate installer-created ACLs.

**9 October format investigation / OPEN-01 proposal:** recommend existing TOML
schema 2, exactly one `[[targets]]` entry per canonical `<vm-guid>.toml`; retain
`Configuration` / `Target` / `Allocation`, strict parsing and existing CLI input
compatibility. Required schema/GUID/GPU interface/enabled; optional complete raw
VRAM triple. Omission means no allocation write, not reset. Full field/read/state
contract, storage audit and migration gaps are in [CONFIGURATION](CONFIGURATION.md).
Compute/encode/decode and slider semantics remain GPU-010 gaps; do not invent
sample support. A later expanded-resource schema requires versioned import.

Added three synthetic production-format files under `config/samples/vms/`, bounded
read-only `Configuration::read_vm_file/parse_vm_file` with filename/identity binding,
and `View::from_vm_file/saved_desired_text` as the GUI loading/display boundary.
Added fixture/identity/parser/separation test source; **no builds, tests or GUI
launches run**. This is an unvalidated proposal/read boundary, not trusted-store
deployment. Superseded by the OPEN-01 result below; CFG-001 is not complete.

Subsequent GUI-002 integration must stop
substituting enrollment for saved configuration, replace the old mock `Saved`
intent with `Target`, retain raw invalid edits separately, and keep fixture reads
explicit/read-only. Directory aggregation, ACL/reparse trust, protected TOML
commits, revision conflicts, explicit bundle import and durable save-only recovery
remain on CFG-001/SEC-001/CORE-028; no migration or live configuration write added.

**OPEN-01 result, 9 October:** selected the schema-2 one-target/GUID-file contract
under DEC-032. Implemented `Configuration::vm_documents` to validate and prepare
canonical per-VM TOML from existing bundles, preserving typed intent and leaving
the source untouched. Added round-trip/duplicate/invalid/version test cases (unrun).
Defined explicit conflict handling, protected per-file commit and partial-import
recovery requirements; no automatic migration or import CLI/publisher exists.
OPEN-01 is resolved as a format/identity/import decision. Reader/serializer code
passed the 9 October validation below; protected deployment and CFG-001 acceptance
remain open. No import CLI or protected publisher exists.

**9 October testing:** six configuration-file tests passed, including CLI/GUI
parity, GUID/filename binding, duplicate/version rejection, bundle round trips,
64 KiB/UTF-8 boundaries and omitted-versus-zero allocation semantics. The wider
103-test suite, strict Clippy and build/docs checks passed. Protected writes,
ACL/reparse/torn-write admission and durable recovery remain unimplemented and
unqualified. See [testing results](evidence/GUI-002-testing.md).

**Exit:** no saving while merely editing; after authorised Apply and verified readback, **only the elevated worker** atomically commits the VM configuration. Detect externally modified files/VM state and **block with refresh**, never auto-merge. Verified GPU success followed by save failure offers **save-only retry**, never repeat Apply. Test wrong GUID/name collision, torn writes, ACL/reparse issues, external edits and recovery. Protected write integration requires SEC-001.

### GPU-010

**Selected physical GPU and complete allocation model — required for initial release.** Extend the existing raw VRAM capability rather than rewrite it. Discover eligible GPUs and report capability/unknown state. Implement validated per-VM **VRAM, Compute, Encode and Decode** triples (Min/Optimal/Max), selected GPU identity and first-time/re-enrollment plan semantics. Provider-reported initial values may be suggested but must be labelled as such; no invented units, GiB, percent or performance guarantees. Host GPU partition count remains **read-only**.

**Also resolve:** the existing GPU Memory slider has mock meaning independent of advanced allocation; evidence is required for a real mapping, with no physical GB/enforcement claim. Do not redesign it. **Exit:** for the selected host/build/vendor, each editable field has known input semantics, bounds and effective readback; CLI and GUI share validation/plan. If provider APIs or a category are unavailable, show a truthful block and bring the scoped limitation to the user **before** changing product requirements. No automatic concurrent-sharing claim, fairness promise or host-wide tuning.

### SEC-001

**Short-lived authorised worker, private Named Pipe and host-wide coordination.** One approved modifying operation launches a restricted elevated instance of the **same executable**. Reuse the protected runner/fixed-operation policy. The frontend and worker communicate through private local Windows Named Pipes with strict ACLs, authenticated peer/operation/plan identity, bounded versioned typed messages, exact VM/GPU identity checks, timeouts and replay rejection. The worker independently revalidates every request; it never accepts arbitrary scripts, files or shell commands.

**Existing foundation:** `runner` already pins protected artifacts/enrollment,
nonces and durable admission; `windows_pipe` supplies local ACL/SID authentication
and bounded framing; exclusive file-handle locking already serializes requests.
Adapt these to per-operation elevation, plan/progress/session binding and host-wide
recovery admission. Owner SID alone cannot identify a particular same-user worker;
review the launch handshake, server waits and artifact trust rather than adding IPC.

**Coordination:** at most **one modifying GPU operation across the entire host**, including GUI, CLI and workers. Read-only inventory can continue where safe. Lock release after exit/crash is **not** equivalent to recovery resolution. Elevate before any VM shutdown; UAC denial retains the draft and causes no VM mutation. Distinguish UI approval, Windows elevation and guest credentials.

**Exit:** mock/negative tests for denied UAC, malicious/malformed/oversized requests, stale plan and altered device identity, wrong peer/session, worker failure, timeouts, pipe loss and concurrent CLI/GUI requests. Confirm changed privileged boundary through repository-required independent review before any live use.

### CORE-028

**Adaptive operation execution, progress, verified persistence and manual recovery.** Reuse CORE-006's shared planner for first-time enrollment, selected-GPU changes, four resource triples and enable/disable/reapply. For complex operations disclose approved protected actions and request **separate explicit graceful shutdown approval**. Before effects, record sufficient durable recovery intent; then execute validated fixed steps with real Pending/Running/Done/Failed/Unknown stage messages over the pipe. Read back real Hyper-V state; save the matching per-VM config via the worker after success.

Extend `workflow`'s pre-effect journals/readback and native pending markers. Current
Apply can resume pending work; change admission to explicit manual reconciliation
and a durable host-wide hold. Reuse `gui_model`'s successful-but-unsaved separation;
make save-only recovery protected and durable through CFG-001/SEC-001.

**Failure policy:** stop further GPU modifications; report completed/uncertain stages; no blind retries, automatic GPU rollback, corrective reassignment or automatic recovery-record clearing. Restore originally running VM only when genuinely safe and already authorised; never silently force-off or modify host power. Worker crash/disconnect is **uncertain**. On restart keep a persistent dashboard recovery warning and block new GPU modifications until **manual reconciliation** verifies safety. Session-only human diagnostics are distinct from minimal durable recovery records.

**Exit:** fault-injected crash, timeout, partial state, unsafe restart, UAC refusal, stale plan, save failure and save-only retry; no GPU operation replay. Exact reconciliation mechanics remain **OPEN-08** until independently reviewed.

### GUI-002

**Active closure work:** trusted saved-intent loading, worker/progress/recovery
bindings and approved-control parity, including Review text wrapping. Dependencies
SEC-001/CORE-028/GPU-010 remain acceptance gates, not exemptions.

**10 October scoped result:** fixed Review clipping and inspected the enabled
historical plan in the actual Slint executable; snapshot rehearsal/conflict/no-write
checks passed. [Closure evidence](evidence/closure-sprint.md). Broader dependency
acceptance and narrow/DPI/accessibility checks remain open.

**9 October configuration/recorded-plan rehearsal slice:** stopped treating
enrollment as saved desired configuration. Added optional
`--mock-gui --snapshot FILE --config GUID.toml` via the production per-VM reader;
candidate intent, observed state, enrollment and one draft remain separate.
Snapshot pairs can edit raw VRAM/toggle intent and recompute the shared Rust plan
from captured inventory/managed records. Enabled previews require recorded plan
payload context for the same VM/GPU/driver; missing context blocks truthfully.
Review confirmation lists simulated stages, retains the unapplied draft and
performs no effects/saving/credentials/recovery clearance. Changed input bytes
block Refresh/Review/confirmation without silently discarding drafts. Added focused
separation/conflict, shared-plan parity, missing-context and mutation-refusal test
source. **9 October qualification:** implemented behavior passed automated and
headless runtime testing below; protected effects/saving and M3 remain open.
Next: trusted live configuration reads/commits (CFG-001), GPU-010 field semantics,
SEC-001 worker and CORE-028 progress/recovery, then GUI parity/qualification.

**9 October testing:** 103 tests passed (one privileged enrollment test ignored),
with formatting, strict Clippy, Windows x64 build and docs checks. Actual executable
headless MCP testing passed snapshot candidate/observed separation, enabled/disabled
shared-plan rehearsal, unapplied draft retention and external-input blocking.
No filesystem activity was observed in the three existing product directories
during snapshot rehearsal. Live startup/System/Refresh passed through the audited
runner; previously recorded graphics checks remain historical and saved desired
configuration is not inferred. **Open visual defect:** long Review body text clips
at the right edge at the default window size. Full protected saving/effects and
desktop accessibility/DPI acceptance remain open. See
[testing results](evidence/GUI-002-testing.md) for exact scope and commands.

**Mode split:** no arguments launch live inventory; `--mock-gui` selects isolated
fixture rehearsal. Live Refresh/System reuse native discovery when already elevated
or the existing authenticated fixed runner for ordinary tokens. No installation or
automatic elevation. Failed reads never substitute sample VMs. Enrollment, guest
health and desired configuration are not inferred; unconnected actions are blocked.

**Next:** finish trusted live configuration/effect bindings through the listed
dependencies and qualify the recorded-data rehearsal slice above. Preserve
fixture scenarios for isolated tests. The current runner audits reads; a strict
no-write real-data route must use authorized native reads/an existing snapshot and
be qualified without suppressing security records. Mock save/forget/credentials/
recovery stay in memory; guest verification/probes and power transitions are effects.

**Current bindings:** enrolled in-memory toggles/raw VRAM drafts, fresh shared
plan previews and native credential storage are connected. Real GUI
Apply/Verify/enrollment/configuration saving remain unavailable. Normal mode does
not present demonstrations as real operations.

**9 October integration in progress (GUI-002 / CORE-012):** wired preparation,
recovery and last graphics-check records into existing live cards/details through
shared Rust presentation logic. Unread/absent/unprepared records are distinct;
schema or enrolled-pair mismatches cannot supply a graphics pass. Driver parity
and current guest health remain unknown until checked. Failed Refresh labels
retained power/status/attachment observations historical; a disappeared draft VM
shows unavailable details while preserving its draft. Added focused record and
row behavior tests. Untested: no tests, builds or GUI launches run; M3 remains open.

**Snapshot follow-on:** added explicit `--mock-gui --snapshot FILE` to read an
existing native/protected CLI inventory JSON capture through the real-data UI.
The 8 MiB-bounded reader performs file reads only; Refresh rereads the input.
All observations are labelled historical; snapshot enrollment never authorizes
operations. Rust source routing refuses live plans and credential access, while
editing/effects remain blocked. Plain `--mock-gui` retains fixture scenarios and
simulated stages. Added dispatch, malformed/oversized/native/protected input and
source-refusal tests. Untested/unqualified: no builds, tests, UI launches or
no-write runtime qualification performed. Recorded-plan/stage rehearsal and
candidate configuration reads were subsequently added in the slice above;
mandatory runner audit is unchanged.

**Validation (9 October mode split):** Windows x64 MSVC build, strict Clippy and
77 tests passed (one privileged enrollment test ignored). Runtime checks covered
normal inventory/Refresh/System, unavailable actions and mock-only settings;
`--mock-gui` covered draft/review/progress/recovery and themes. Actual inventory
reported the disposable VM and RTX 5060 driver 32.0.16.1692; no guest rendering or
GPU sharing qualification is claimed. Independent source review found no blockers.

**Connect the Slint interface to the verified shared Rust core.** Replace mock data incrementally: automatic discovery with partial-access states; GUID-keyed VM cards/selection; physical-GPU dropdown; expandable allocation editor; one draft; pending-draft dialog before **switching VMs**; fresh Review & Apply; UAC/downtime approvals; credential access through existing protected facilities; true progress and the existing recovery banner. No additional pages or controls. No full-list rebuild for incidental UI changes.

**Exit:** identical effective plans and error semantics in CLI and GUI; switching/stale change never silently discards or applies a draft; unsupported GPU/provider states are explained; partial provider failure does not empty the list; side effects occur only through the shared core and restricted worker.

### GUI-003

**Qualify the approved interface.** Verify Dashboard/System/Settings/About, current
dialogs, draft preservation, close deferral during active work, light/dark/Windows
themes, splitter, pane scrolling, narrow overflow, keyboard/focus, DPI/text scaling,
long names, accessibility and software rendering. No new controls/pages, persisted
preferences, session activation or Win32 parity gate.

**Exit:** Actual mock interaction/render checks and recorded Windows qualification
limitations; no screen-reader parity claim without testing.

**10 October scoped result:** opt-in software-renderer checks cover the 700x520
minimum, 1600x1000 workspace, 35-VM inventory, all four pages and wrapped Review.
Actual MCP mock interactions passed Light/Dark/Windows themes, Tab/Return navigation,
draft-switch protection and uncertain-outcome recovery. Lifecycle/close deferral,
splitter/pane interactions, Windows text scaling and screen-reader behavior are
not claimed. See [closure evidence](evidence/closure-sprint.md).

## R1 — Packaging and candidate acceptance

### CORE-017

**Build the combined Windows x64 candidate.** Package `hyper-gpu-support.exe` plus only required fixed guest/worker/probe payloads. Verify mode dispatch, UAC launch/pipe ACLs, ProgramData directory installation/upgrades/removal, signed-driver discovery, missing prerequisites, read-only startup, installer interruption and licensing/notices. The executable may spawn its elevated instance, but no always-on management service is required. Exclude proprietary host drivers, VM disks/media, secrets, keys and laboratory fixtures. Verify current CRT/linking assumptions instead of relying on historical build statements.

**Exit:** clean candidate and recorded version/revision/checksums; no developer paths or essential missing binary. Packaging does not authorise publication.

### DOC-003

**Tested GUI/CLI operator guide.** Document single-exe startup/commands; per-VM ProgramData configuration, direct right-panel GPU enrollment and all supported allocation fields; review/apply/UAC/shutdown approvals; driver preparation and graphics verification; session diagnostics; stale-state refresh, save-only retry, failure/manual reconciliation; and honest provider, vendor and concurrent-sharing limitations. Do not document unimplemented commands or use historic lab/golden-image paths as product requirements.

**Exit:** instructions validated against the actual packaged candidate and supported host/guest combinations.

### GPU-014

**Authorised end-to-end product qualification.** On explicitly selected disposable existing VM(s), exercise clean packaged startup, discovery, enrollment, GPU selection and supported custom twelve-field allocations, signed-driver preparation, exact attachment, PnP/checked D3D11, running no-op, reapply, disable/restoration, CLI/GUI parity and representative interrupted/recovery/save-only scenarios. Reuse earlier M1/M2 evidence and run only relevant changed paths. A second concurrent VM is **not** a gate unless sharing is advertised.

**Exit:** reviewed actual run report, state/host safety, security boundaries, package/manual parity and no essential unresolved blocker. Follow AGENTS designated-target authorisation and identity checks; no automatic host restart.

### GPU-015

**Conditional sharing qualification — deferred M4.** Before advertising simultaneous
same-GPU use, qualify two explicitly designated VMs independently and together,
including removing one without disturbing the other. Define admission/support policy;
multiple configuration files and serialized operations do not prove sharing safety.
Reuse existing core and relevant evidence; no scheduler or new release gate unless
sharing is advertised. Follow AGENTS target checks and testing authorisation.

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

Retain former OPEN IDs as references to backend/qualification questions, not an
independent GUI requirement register. DEC-030 supersedes absent-interface scope.

| ID | Verification needed |
|---|---|
| OPEN-01 | Resolved: schema-2 one-target/GUID-file and explicit import contract ([contract](CONFIGURATION.md#open-01-per-vm-format), DEC-032). Code untested; protected publication remains CFG-001/SEC-001; resource schema extension remains GPU-010 |
| OPEN-03 / OPEN-05 | Cross-process lock/worker handover; Named Pipe ACL, peer authentication, plan binding and launch semantics |
| OPEN-04 / OPEN-10 | Actual Windows 11 GPU-P provider APIs, GPU selection, 12 field units/bounds/unset/readback/enforcement |
| OPEN-06 | Slint renderer and practical software fallback for unhealthy GPU drivers |
| OPEN-07 | Resolved by approved prototype: page navigation preserves draft; dirty close prompts |
| OPEN-08 | Exact manual reconciliation/clearance rules for incomplete or uncertain operations |
| OPEN-09 | Windows GUI/CLI console subsystem and installer/UAC/ProgramData; session activation retired |
| OPEN-11 | Actual keyboard, text-scale and assistive-tech support |
| OPEN-12 | Restrict save-only worker authorisation without allowing arbitrary config rewriting |

The completed prototype defines GUI scope. Retain applicable backend safeguards;
do not turn historical A18–A32 or OPEN questions into extra interface requirements.

## Acceptance and next handoff

A card closes only with its own acceptance evidence. M3 closes only after the common Rust core, Slint frontend, editable supported allocation, restricted worker, typed Named Pipe, host-wide coordination, ProgramData persistence, crash recovery and verified GUI/CLI behaviour pass their checks. R1 then requires actual packaging and explicitly authorised target qualification.

**Next prompt:** [Slint implementation task](../.github/prompts/SLINT_CODEX_PROMPT.md)
for an explicitly requested card. [GUI_PROMPT](GUI_PROMPT.md) is the docs-audit brief,
not an instruction to repeat PLAN-001 or begin implementation.
