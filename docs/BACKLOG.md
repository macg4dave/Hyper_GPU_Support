# Hyper GPU Support — active implementation backlog

**Updated:** 10 October 2026. [ROADMAP](ROADMAP.md) owns milestones; this file owns task selection and status. Scope is the approved Slint GUI over the shared Rust core. Source observations below refer to the current working tree, including uncommitted changes, inspected statically on this date. No validation was run for this review.

## Next implementation action

**Start `SEC-001.3 / CORE-028`: reviewed physical GPU replacement through the existing selector.** CORE-028.1 recovery presentation and GPU-010.1–2 / GUI-002.3 four-category allocation are implemented pending validation. The allocation boundary received independent static review with no remaining blocker. The latest Windows x64 debug build succeeded and was separately authorized for protected installation: four installed/policy hashes match the build, and existing operator/enrollment were preserved. Tests/UI/provider acceptance remain pending; earlier GUI startup/discovery checks apply to the previous enrollment build.

Continue **`SEC-001.4` / `GUI-002.4` (Forget pairing)** after reviewed replacement. Source implementation is not authority to build, launch, install or exercise a runner. Inspect named modules and reuse working contracts; do not repeat a project audit.

If replacement work is blocked by a concrete external prerequisite, record it once and take **`SEC-001.4`** or **`GUI-003.1`**, which are independent ready source tasks. Do not substitute qualification or test infrastructure work.

## Status and selection rules

**Authorized BUGS-001 / GPU-010.3 build and installation (10 October):** fixed the Slint duplicate binding in modal availability by deriving its value from dialog/operation state and synchronizing the global from that value. `cargo build --locked --all-features --bins --jobs 2` succeeded for Windows x64/MSVC debug with the configured static CRT. The existing development wrapper then invoked native Rust `install` using the full existing enrollment. All four built/installed/policy SHA-256 hashes match; the original operator and one enrolled pair were preserved. Evidence: `local/gui-update/0852293a78f94462a5a03e3ab3462074/result.json`. No tests, GUI launch or VM/GPU/guest effect exercise was run; behavior acceptance remains pending.

**Requested bug repair — [BUGS-001](evidence/BUGS-001.md) (10 October):** GUI-002/GUI-003 source changes implement concise GPU status, before/after Review summaries with expandable technical output, fixed downtime/Apply footer, reactive modal availability, baseline-based dirty tracking and installed read-only status polling every 15 seconds while dialogs/operations are idle. Navigation does not stage intent; reverted edits clear the draft and invalid edits remain visible with their validation reason. Tests/builds/render/GUI acceptance are deferred; BUG-05's reported interaction still needs reproduction/acceptance. The subsequent user request implemented BUG-06 / GPU-010.3 native byte-to-GB request conversion (DEC-034), pending acceptance. Individual bugs are not marked completed.

- **ready:** implementation can start with present dependencies.
- **in progress:** scoped coding is underway; aggregate parent cards may contain delivered slices.
- **blocked:** a named prerequisite or safety gate prevents this task.
- **implemented pending validation:** scoped source exists; specified checks remain unrun/unaccepted. Continue independent implementation.
- **completed:** scoped implementation and required evidence are established. Historical completion does not qualify later changes.

Suffixes below are executable slices of existing IDs, not replacement projects. Parent IDs remain the tracking owners. Follow the order table; skip blocked tasks to the first ready task. Update the owning slice with changed behavior, exact blocker and actual validation status. Write meaningful tests for new logic, but execution follows AGENTS and ROADMAP: no automatic tests/builds/UI launches after a task; M3/release validation needs explicit authorization. Material privilege/recovery changes require independent architecture review at the implementation milestone or before live use.

## Ordered source queue

**BUGS-002 source repair (10 October):** non-modal polling and in-place row updates,
presentation edit guards, reported allocation default suggestions, full provider-range
slider with explicit draft-bound expansion, clearer enrollment/planning/guest credential
and actual-stage Apply text, failure context and safe native-code diagnostics implemented.
Physical-card VRAM request mapping (BUG-02) and structured safe component error messages
(BUG-10) remain open for the exact reasons in [BUGS-002](evidence/BUGS-002.md).
GUI-002/GUI-003/GPU-010 own these repairs; tests/builds/render/live acceptance are deferred.

| Order | Slice / owner | Status | Depends on |
|---|---|---|---|
| 1 | SEC-001.1 — incremental pair enrollment | implemented pending validation | Existing installer, security, IPC and common lock |
| 2 | SEC-001.2 — fixed first-install bootstrap | implemented pending validation | SEC-001.1 source implemented |
| 3 | GUI-002.1 — first-time Details enrollment | implemented pending validation | SEC-001.1–2 source implemented |
| 4 | GUI-002.2 — truthful action availability/text | implemented pending validation | Existing live action handlers |
| 5 | CORE-028.1 — synchronize GUI recovery state | implemented pending validation | Existing recovery/store reads |
| 6 | GPU-010.1 — shared full-resource capability/schema | implemented pending validation | Existing raw VRAM model/native provider adapter |
| 7 | GPU-010.2 — resource planning/write/readback | implemented pending validation | GPU-010.1 source implemented |
| 8 | GUI-002.3 — bind supported allocation fields | implemented pending validation | GPU-010.2 source implemented |
| 9 | SEC-001.3 / CORE-028 — reviewed GPU replacement | ready | SEC-001.1, GUI-002.1 source implemented |
| 10 | SEC-001.4 — protected Forget pairing backend | ready | SEC-001.1 source implemented |
| 11 | GUI-002.4 — bind Forget pairing dialog | blocked | SEC-001.4 |
| 12 | GUI-002.5 / CFG-001 — production-format fixture intent | ready | Shared parser/model; extend with GPU-010 when available |
| 13 | GPU-010.3 — truthful GPU Memory slider mapping | implemented pending validation | Documented native VRAM bytes; GPU-010.1–2 |
| 14 | GUI-003.1 — existing dialog/focus/close defects | ready | Approved current Slint components |
| 15 | CORE-017.1 — package manifest/lifecycle preparation | ready | Existing installer/artifact layout; no build execution |
| 16 | CORE-017.2 — native removal/update completion | blocked | CORE-017.1 |
| 17 | DOC-003 — candidate operator guide | blocked | Stable implemented GUI/core and CORE-017 lifecycle |

Ready does not mean qualified. Provider-dependent fields may remain unavailable with an exact blocker while other work proceeds; that does not satisfy the four-category initial-release requirement. No M2 safety pass is required to start these safe source tasks.

## Coding cards

**Authorized protected update (10 October):** elevated development wrapper invoked the fixed native Rust `install` with the complete existing enrollment. Exit code 0; all four installed executables and enrollment artifact pins match source SHA-256 hashes, existing operator and one enrolled pair preserved, no installation-pending record remains. Evidence/logs: `local/gui-update/37784041f41a4e2ba34f5e9a638c366b/`. Installer performed its required fresh native identity discovery; no tests, GUI launch or GPU/guest/power effect operation requested. New runtime acceptance remains pending.

**Authorized latest build (10 October):** `cargo build --locked` succeeded using Rust 1.94.0 for `x86_64-pc-windows-msvc`, debug profile. All four product executables are present under the inherited `CARGO_TARGET_DIR` output directory (`C:\Users\dave\.cargo-target\text-game\x86_64-pc-windows-msvc\debug`). No tests, Clippy, UI launches, protected installation or live provider operations were run. Earlier unbuilt results below describe their implementation-time state; this build does not establish runtime acceptance.

**Authorized validation/update (10 October, SEC-001.1–2 and GUI-002.1–2):** fixed missing `Inventory` serialization for the setup reply, then `scripts/testing/check.ps1` passed formatting, strict Clippy, 145 tests, Windows x64/MSVC debug all-features build, rustdoc and documentation checks. Four opt-in tests remained ignored (layout and privileged enrollment/M2 checks). With separate user approval, updated the four protected artifacts through native Rust `install` using all existing enrolled targets; installed/source/policy SHA-256 hashes matched and operator/pair intent was preserved. The installed no-argument GUI is running/responding and its startup Discover audit succeeded. No enrollment/setup, crash/cancellation or VM/GPU/guest effect operation was exercised; cards remain pending that acceptance.

### SEC-001.1 — incremental exact-pair enrollment

- **What / Why:** add a fixed reviewed operation to add one VM/GPU pair without reinstalling/replacing the whole enrollment set. Current worker commands cannot authorize a new pair.
- **Where:** `src/runner.rs::{Enrollment,install,validate_enrollment}`, `src/worker.rs::{Command,execute}`, `src/windows_pipe/`, `src/security.rs`, `src/model.rs`; share the core contract with CLI where appropriate.
- **Depends on / Status:** existing protected artifacts/IPC/operation lock; **implemented pending validation**.
- **Result (10 October):** `enrollment::PairReview`, fixed worker EnrollPair/ReconcilePair, revision/operator binding, independent native discovery and shared admission lock. Other enrolled VMs/artifact pins remain intact. A write-ahead pair record holds GPU work until terminal audit; explicit old/new-policy readback resolves interruptions without replay. Authored additive/conflict/publication-failure/readback tests; unrun. Independent static boundary review found no remaining blocker. Adding another VM is supported in source; changing an already enrolled GPU remains SEC-001.3 and is refused here.
- **Done when:** administrator approval is bound to exact VM GUID, GPU interface, authenticated operator and expected enrollment revision; elevated code rediscovers and independently validates them under common admission/recovery rules. Preserve other targets, SID, artifact pins, audits and journals; conflict/interruption cannot silently replace authority. Enrollment alone performs no GPU/guest/power effects or verified-config publication. Config/draft text never grants authority. Add focused contract/failure tests; review the changed boundary before live use.

### SEC-001.2 — bounded first-install bootstrap

- **What / Why:** enable setup when neither protected runner nor worker is installed. `worker::trusted_executable` currently needs enrollment/artifact pins before UAC launch, and ordinary-token discovery uses the installed runner.
- **Where:** `src/main.rs`, `src/runner.rs::{install,enrollment}`, `src/worker.rs::{trusted_executable,launch}`, `src/process.rs`, `src/windows_pipe/`, `src/security.rs`.
- **Depends on / Status:** SEC-001.1; **implemented pending validation**.
- **Result (10 October):** fixed same-package `--internal-setup` discovers inventory via UAC and installs reviewed identity-only scope from four fixed sibling artifacts. Same initiating SID, retained process/session identity, authenticated bounded IPC and source-file locks bind package hashes. Existing policy is never replaced; bootstrap locks/checks absence before scheduler changes and preserves existing ACLs. Original setup scope/pins survive interruption; pre-publication retry retains scope, post-publication Resume validates full authority/artifacts and completes task/audit registration without overwriting them. Recovery requires the original package and a discoverable eligible original VM/GPU; other cases remain explicitly held for administrator inspection. Contract tests authored; all validation deferred. Independent review blockers fixed and re-reviewed.
- **Done when:** a fixed Rust setup route can obtain eligible discovery and install/enroll the reviewed pair with explicit UAC from the approved GUI journey. Preserve the initiating user's SID across elevation, validate trusted source artifacts/paths and bounded typed inputs, and reuse native installation/IPC machinery. Existing installation takes the incremental route; it is never replaced from a one-pair draft. Cancellation and interrupted install remain explicit/recoverable, with no effects or invented saved intent. No arbitrary elevated executable/command/path channel; maintain mandatory audit and independent boundary review.

### GUI-002.1 — first-time enrollment through Details

- **What / Why:** let an unenrolled eligible VM/GPU reach review and enrollment from existing controls; `State::target` and editor eligibility currently stop at CLI guidance.
- **Where:** `src/gui/live_state.rs::{target,editor_eligibility,draft}`, `src/gui/live.rs::{present,review,run}`, `src/gui_model.rs`, `src/gui/ui/{details,dialogs,app}.slint`.
- **Depends on / Status:** SEC-001.1–2; **implemented pending validation**.
- **Result (10 October):** eligible unenrolled VMs can draft a discovered physical GPU through existing Details controls. Review presents separate exact enrollment/setup scope; confirmation enrolls only, reloads protected authority and retains raw edits. A subsequent Review obtains the separate shared GPU plan before Apply. Failed/cancelled enrollment retains its visible error and requires Refresh; lost authority latches a draft conflict. Snapshot proposals cannot enroll. Authored proposal/authority and historical-mode regressions; no test/build/render/live validation run. No new pages or controls.
- **Done when:** proposal eligibility is separate from execution authority; discovered VM/GPU can be drafted and exact setup scope reviewed through Details/Review without extra controls/pages. Confirmation uses fixed setup/enrollment; then reloads protected authority and committed intent. GPU Apply requires its own fresh reviewed plan, credentials and downtime consent. Cancel/stale/unsupported/conflicting states preserve raw edits and exact identity; no CLI needed for supported first-time setup. Snapshot/fixture confirmation never enrolls.

### GUI-002.2 — truthful live controls and messages

- **What / Why:** repair concrete stale claims and readiness messaging around already implemented actions.
- **Where:** `src/gui/ui/app.slint` live footer (`Apply unavailable`), `dialogs.slint` (`Forget sample pairing`), `src/gui/live.rs::{present,refresh,run}` credential/help/action messages.
- **Depends on / Status:** existing handlers; **implemented pending validation**.
- **Result (10 October):** footer reflects protected worker/recovery readiness; enrollment review uses mode-specific confirmation in the existing dialog, live Forget text is accurate and remains disabled, credential cancellation/non-retention explains local prompts for Apply/Verify, and help describes separate enrollment/Apply review. Diagnostics use actual protected enrollment rather than synthesized intent. Source formatted; UI/build validation deferred.
- **Done when:** live footer reflects actual worker/action readiness; credentials cancellation/non-retention no longer says no guest operation is connected. Existing Verify/Apply/save/reconcile controls show action-specific reasons for unavailable authority/recovery/stale inputs. Forget remains unavailable until its real backend is connected and has correct mode-specific text. Reapply still means review, never an automatic effect/retry. No new controls or redesign.

### CORE-028.1 — recovery state after external completion

- **What / Why:** synchronize GUI state with durable recovery across Refresh. Current `refresh` sets `view.unsaved`/`verified_unsaved` only when a recovery record exists; after CLI save/reconciliation removes it, stale in-memory save-only state can remain.
- **Where:** `src/gui/live.rs::{refresh,save_only_live,reconcile_live}`, `src/gui/live_state.rs`, `src/gui_model.rs::{View,apply_target,published}`.
- **Depends on / Status:** existing durable records/committed reads; **implemented pending validation**.
- **Result (10 October):** Refresh derives save-only presentation from fresh durable records. An absent record clears retained verified-unsaved intent only after matching readable committed intent; missing/unreadable/mismatched publication blocks execution. External completion preserves drafts and existing revision conflicts. Standalone Verify never offers configuration publication; import/enrollment records retain their separate holds. Failed reads retain loaded recovery and invalidate readiness. Four focused regressions authored; no tests, builds, GUI launches or live validation run.
- **Done when:** successful fresh reads derive unsaved/recovery state for record-present, record-absent, standalone Verify and import cases. Confirm committed/readback facts before releasing any hold; read errors never imply absence. Preserve or explicitly conflict an existing draft when another frontend completes publication. Clear stale review/execution readiness after failed/changed reads. Add regression coverage for external save/reconcile and unreadable records; never replay effects or automatically clear durable recovery.

### GPU-010.1 — shared full-resource schema and capabilities

- **What / Why:** represent supported VRAM/Compute/Encode/Decode triples and observed limits through the shared model.
- **Where:** `src/model.rs`, `src/windows_hyperv.rs::{discover_gpus,inspect,read_optional_resource}`, `src/configuration_store.rs`, `src/gui_model.rs`; affected schema contract in `docs/CONFIGURATION.md`.
- **Depends on / Status:** existing native WMI/model/parser; **implemented pending validation**.
- **Result (10 October):** schema 3 adds optional raw Compute/Encode/Decode triples; schema 2 remains readable and rejects new intent. Parsing/splitting preserve versions; typed upgrade is explicit, and new reviewed publication writes schema 3. All four native capabilities isolate missing/malformed ranges; legacy VRAM snapshot ranges remain readable and missing capabilities stay unknown. Official Microsoft property contracts checked. Parser/round-trip/u64/legacy-capture regressions authored; no tests/builds/live validation run. DEC-033 records the contract.
- **Done when:** verify relevant official provider property contracts as part of implementation; typed capabilities distinguish supported, unavailable/unknown and unset per category. Version the intent extension explicitly, retaining schema-2 input support and deliberate upgrade/serialization rules. Preserve omission as no write, reject partial/out-of-order/out-of-range/unknown input, and use the same parser for CLI/GUI/fixtures. No guessed bytes/percentages or enforcement promises. Unknown provider evidence blocks only the affected category, recorded once.

### GPU-010.2 — plan, apply and read back resource triples

- **What / Why:** complete native backend behavior for all four categories through the existing operation boundary.
- **Where:** `src/workflow.rs::{Backend,decision,Plan,apply_approved,reconcile_observed}`, `src/windows_hyperv.rs::allocation`, `src/runner.rs::NativeBackend`, `src/worker.rs`, `src/configuration_store.rs` verified receipts.
- **Depends on / Status:** GPU-010.1; **implemented pending validation**.
- **Result (10 October):** typed category writes recheck fresh native ranges and independent category readback. Shared review enumerates exact triples; admission retains mutable-state checks; final verification/save-only/reconciliation compare every request. Omission performs no explicit write; new/recreated adapters retain provider-default behavior. Authored ordering/omission/unknown-capability/drift/partial-failure/readback/receipt regressions. Independent static boundary review findings fixed and re-reviewed; no remaining blocker. Tests/builds/provider behavior remain unverified; no protected installation/effect exercised.
- **Done when:** reviews enumerate exact per-category writes; fresh limits/identity are checked before effects, each requested triple has independent readback, and save-only/reconciliation compares all requested categories. Omitted resources are retained/no-write, not reset to zero. Preserve default behavior, separate consent and recovery on partial writes. Author focused fake-backend ordering/drift/failure tests; hardware enforcement remains unclaimed.

### GUI-002.3 — supported allocation editor bindings

- **What / Why:** connect the existing twelve allocation fields to the shared category capabilities and backend.
- **Where:** `src/gui/live_state.rs::{draft,allocation_values,review_text}`, `src/gui/live.rs::{present,refresh}`, `src/gui/ui/{details,resources}.slint`, `src/gui/snapshot.rs`.
- **Depends on / Status:** GPU-010.2; **implemented pending validation**.
- **Result (10 October):** Details binds each supported raw triple, preserves invalid text, allows clearing existing unsupported requests, and distinguishes capability absence from omitted intent. Review, observed/saved labels and System display all categories. Disabled targets perform no resource writes. Snapshot rehearsal requires recorded positive capability and retains no-write behavior; old snapshots cannot invent new support. Source formatted and focused GUI/snapshot regressions authored; no builds/tests/render/UI launch/live acceptance run. Approved controls/layout preserved.
- **Done when:** each category's editability, raw values, bounds, validation, review and readback use shared capability/types; unsupported/unknown stays truthful. Retain invalid raw text, one selected-VM draft and stale/conflict behavior. Snapshot rehearsal accepts only recorded supported capabilities, with no live discovery/effects or new mock format.

### SEC-001.3 / CORE-028 — reviewed physical GPU replacement

- **What / Why:** make the existing selector useful for an enrolled VM. A different interface currently fails `View::eligibility`; another attached GPU and old journal identity also block it.
- **Where:** `src/gui/live_state.rs`, `src/gui_model.rs::eligibility`, `src/runner.rs` enrollment, `src/worker.rs`, `src/workflow.rs` and protected configuration receipts.
- **Depends on / Status:** SEC-001.1, GUI-002.1 source implemented; **ready**.
- **Done when:** separately review new-pair authorization and the exact old/new GPU transition through existing controls. Use shared verified disable/enable/settings-restoration paths; preserve attributable old-pair journal/recovery until resolved, independently check the new pair/payload, and publish only verified intent. Drift/partial transition keeps a durable host-wide hold. Never silently broaden authority, overwrite the old journal, detach an unrelated adapter or change disks/CPU/RAM/Secure Boot. Boundary review required.

### SEC-001.4 — protected Forget pairing

- **What / Why:** implement pairing removal; no such worker operation exists, and CLI credential `forget` is a different action.
- **Where:** `src/runner.rs` enrollment, `src/worker.rs`, `src/configuration_store/protected.rs`, `src/workflow.rs` recorded state; preserve `src/credentials.rs` semantics.
- **Depends on / Status:** SEC-001.1 source implemented; **ready**.
- **Done when:** exact reviewed VM/pair scope is removed under authenticated elevation, expected revisions, common lock and durable recovery. Require independently verified disabled/detached state and no unresolved recovery; otherwise require the existing Disable/reconcile journey first. Review includes deletion of that pair's committed intent; retain attributable audit/history and prepared guest files. Partial metadata removal is recoverable, other pairs are preserved, and no implicit credential deletion, guest cleanup or GPU effect occurs. CLI `forget` retains its credential-only contract.

### GUI-002.4 — live Forget pairing dialog

- **What / Why:** connect the existing mock-only More actions/Forget dialog.
- **Where:** `src/gui/live.rs::run`, `src/gui/live_state.rs`, `src/gui/ui/dialogs.slint`.
- **Depends on / Status:** SEC-001.4; **blocked on backend**.
- **Done when:** dialog shows exact pairing/config deletion and retained guest/history/credential state; confirmation invokes the protected operation and reloads inventory/intent. Dirty draft, attachment, revisions and recovery block unsafe removal with an actionable reason. Cancel makes no changes; fixture/snapshot modes remain no-write. No new page/control.

### GUI-002.5 / CFG-001 — reuse production intent in fixtures

- **What / Why:** remove the separate saved-configuration semantics in `src/gui/mod.rs::Saved` while preserving approved simulated scenarios. Snapshot already uses production `Configuration`/`Target`; fixture saved intent does not.
- **Where:** `src/gui/mod.rs::{Saved,Session,draft}`, `src/gui/mock.rs`, `src/model.rs`, `src/gui_model.rs`, existing `config/samples/`.
- **Depends on / Status:** production parser/model present; **ready**.
- **Done when:** saved/candidate fixture intent uses the exact production schema/parser/types in memory. Raw invalid editor text, illustrative slider position and simulated stage outcomes remain presentation state, never a second config contract or committed success. Unsupported categories cannot masquerade as production supported intent; adapt with GPU-010 when implemented. Retain useful scenarios and strict no-write/no-effect behavior; no new rehearsal/test infrastructure.

### GPU-010.3 — GPU Memory slider

- **What / Why:** settle the existing illustrative GB slider without inventing real allocation semantics.
- **Where:** `src/gui/ui/details.slint`, `src/gui/live_state.rs`, shared GPU capability/model.
- **Depends on / Status:** GPU-010.1–2 and native byte contract; **implemented pending validation**.
- **Result (10 October):** user requested source mapping after the earlier BUG-06 hold. Microsoft's current Set-VMGpuPartitionAdapter reference explicitly defines Min/Optimal/MaxPartitionVRAM in bytes. DEC-034 records decimal GB conversion without physical-card ratios or enforcement claims. `src/gui/memory.rs` uses checked integer 0.1 GB ticks within fresh native bounds and existing Minimum/Maximum; omitted triples remain omitted until an edit initializes provider bounds. Slider and raw fields share the same typed draft/review path. Raw non-grid values are displayed exactly, not rewritten on refresh. Invalid/stale/unknown/historical/narrow/imprecise ranges remain unavailable. Focused conversion/draft regressions authored; no tests/builds/UI renders/launches/live provider checks executed. Source evidence establishes the documented request conversion; actual provider/guest semantics and GUI acceptance remain pending.
- **Done when:** the documented native-byte conversion yields the same validated triple as Advanced Allocation, with truthful units/readback and no contradictory settings. Accept the actual GUI/provider behavior at the authorized gate. Unknown/historical unit provenance stays unavailable. Other raw categories are not GB/GiB/percentages; byte conversion does not demonstrate reservation or enforcement.

### GUI-003.1 — restore focus after existing dialogs

- **What / Why:** restore keyboard focus to the invoking control when an existing modal closes. `DemoDialog` sets initial focus to technical details, but the Rust close/completion paths only clear `dialog-kind`; no explicit focus restoration is connected.
- **Where:** `src/gui/live.rs::{background,run}`, `src/gui/mod.rs`, `src/gui/ui/{app,dialogs,details,resources,technical-output}.slint`.
- **Depends on / Status:** current UI; **ready** for source-level fixes.
- **Done when:** cancellation/completion returns focus to the invoking control, or a safe existing control if selection changed; keyboard focus stays usable through Review, dirty switch/close and progress transitions. Preserve busy close deferral, callback lifetime, draft and worker outcome. Keep split/independent scrolling/selectable output and approved pages. DPI/text scaling, accessibility and renderer observations wait for the M3 authorized acceptance pass; repair additional defects only when identified. Earlier scoped mock checks do not qualify new live callbacks.

### CORE-017.1 — candidate package/lifecycle preparation

- **What / Why:** define a usable candidate from actual fixed artifacts, without building it yet.
- **Where:** `Cargo.toml`, `src/main.rs`, `src/runner.rs::install`, `src/bin/`, existing distribution/build configuration and notices.
- **Depends on / Status:** current installer/layout; **ready**, preparation may overlap M3.
- **Done when:** package preparation includes `hyper-gpu-support.exe`, `hyper-gpu-runner.exe`, `hyper-gpu-guest.exe`, `d3d11-probe.exe`, version/revision/checksums/notices and real runtime requirements, with no lab or developer paths. Inventory actual install/update/removal gaps for CORE-017.2 in this card. No driver/media/secret bundling without rights, new installer framework, build, publication or installation implied.

### CORE-017.2 — native installation lifecycle gaps

- **What / Why:** finish concrete update/removal behavior required by distribution. Native install/interrupted-install marker handling exists; there is no implemented product uninstaller to reuse.
- **Where:** `src/runner.rs::{install,enrollment,operation_lock}`, `src/main.rs`, `src/security.rs`, `src/configuration_store/protected.rs`.
- **Depends on / Status:** CORE-017.1; **blocked on concrete package scope**.
- **Done when:** bounded Rust update/removal preserves operator authority and recoverable intent/history according to explicit reviewed scope, refuses active work/unresolved recovery, and manages fixed artifacts/task registration safely. Installation failure remains explicit and cannot enable half-updated artifacts. Removal neither detaches GPUs nor deletes guest files/disks implicitly. Reuse current native installer; independently review changed privileged lifecycle before use.

### DOC-003 — candidate operator guide

- **What / Why:** document the implemented product journey rather than another internal plan.
- **Where:** `README.md`, existing operator/configuration documentation, candidate package instructions.
- **Depends on / Status:** stable M3 functionality and CORE-017 lifecycle; **blocked**.
- **Done when:** instructions cover GUI startup/setup/edit/review/disable/Verify, CLI equivalents, credentials, import, save-only/manual recovery and install/update/removal with actual supported limits. No hidden lab paths, credential/pairing confusion, reset/CUDA requirements or untested qualification claims. Candidate instruction execution is GPU-014's gate.

## Existing implementations — reuse, do not reschedule

| Owner | Source implementation | Status / remaining scope |
|---|---|---|
| GUI-001 | Approved Slint prototype promoted to `src/gui/ui/` | completed design scope; defects belong to GUI-003 |
| APP-001 | `main.rs`, `console.rs`, same-executable restricted mode/dispatch | completed reported scope; package and expanded boundaries still need acceptance |
| CORE-012 | Partial inventory, `diagnostics.rs`, `reporting.rs`, truthful recorded-state presentation | completed reported scope; no new diagnostics initiative |
| CORE-006 | `workflow.rs` shared typed decision/effect preview | completed existing scope; extend for resource/replacement tasks only |
| GUI-002 / CFG-001 | Live inventory/System; selected-VM draft/review; bounded snapshot; committed reads; unreadable-vs-absent and latched conflict behavior in `live_state.rs` | implemented pending validation for current slices; do not schedule parser/store/Refresh creation again |
| CFG-001 | `configuration_store{,/protected}.rs`: GUID documents, revisions, protected atomic per-file publication; worker import/import-resume | implemented pending validation; trust/race/crash/installation ownership acceptance remains. Import is CLI-supported, not a missing GUI import page |
| SEC-001 | Trusted artifacts, authenticated pipes, `runner::operation_lock`, `require_no_recovery`, worker commands | implemented pending validation for expanded scope; add named enrollment/removal boundaries only |
| CORE-028 | `live.rs` Apply/Verify/progress/save-only/reconcile; `worker.rs` durable phases and shared admitted execution/readback | implemented pending validation; named state/transfer/replacement gaps remain |
| CORE-021 | Shared configuration/parser/enrollment rules and CLI operation paths | existing core implemented; aggregate GUI parity closes with owning slices, no parallel parser/backend project |
| GPU-010 | Shared four-category schema/capabilities/plan/native writes/readback and existing editor bindings; exact GPU discovery; native VRAM GB request mapping | implemented pending validation for GPU-010.1–3 / GUI-002.3; replacement and actual provider acceptance remain |
| GUI-003 | Reported scoped mock/software-renderer sizing/page/Review checks | scoped historical result; current live/UI milestone acceptance pending |
| PLAN-001 | Previous scoped architecture/source/document review | completed; no recurring audit prerequisite |

Parent GUI-002/CFG-001/SEC-001/CORE-028/CORE-021/GPU-010/GUI-003 remain **in progress** as aggregate delivery owners. This review does not promote unexecuted source changes to completed/qualified.

## Separate M2 containment lane

### CORE-028.2 / ARCH-001 — bounded preparation transfer

- **What / Why:** finish the already recorded Rust-supervised per-file/no-progress transfer bounds and byte/file progress, reusing the existing contained-process supervisor.
- **Where:** `src/guest.rs::bridge`, `src/process.rs`, `src/guest_transport.ps1` only within DEC-028's fixed bridge, `src/worker.rs::ObservedBackend` progress.
- **Depends on / Status:** **completed, 10 October** with M2 acceptance; no further M2 live validation required.
- **Acceptance:** Rust supervises bounded transfer activity, reports real bounded progress without secrets, and retains durable uncertainty after interrupted/partial transfer. No unchecked cache, omitted payload members, automatic retry/rollback or product logic expansion in PowerShell. Existing admission reuse, WMI/hash limits and protected bootstrap publication are reused. Independent boundary review and user-confirmed live acceptance are complete.
- **Result (10 October):** fixed 1 MiB transport chunks acknowledge remote stream writes; Rust checks ordered exact-file byte totals, 300-second file budgets, 60-second stalls and 240-second setup/inter-file gaps. Existing local kill-job and 3,600-second outer budget remain. Progress IPC can delay local termination by up to 20 seconds beyond those budgets; termination does not prove guest cancellation. Protected bootstrap hashes and complete guest payload/receipt checks remain independent. Worker progress includes bootstrap files; the GUI coalesces transfer rows and retains later outcomes. Monitor/supervisor regression tests and the repository quality gate passed. Independent static boundary review completed. The user confirmed completed live testing and working behavior, and closed M2; no further live qualification is outstanding.

## Validation / qualification / release gates — not coding tasks

| Owner | Status | Depends on | Done when |
|---|---|---|---|
| ARCH-001 / GPU-012 — M2 stability | completed 2026-10-10 | Source/review complete; quality gate passed; user-confirmed live tests accepted | Known-good baseline accepted by the user; no further M2 live tests or investigation required. See [acceptance](evidence/M2.md). |
| GUI-002 / GUI-003 / CFG-001 / SEC-001 / CORE-028 — M3 | blocked | All planned M3 coding slices and applicable independent reviews; explicit validation authorization | Proportionate existing Rust/build/UI checks, shared CLI/GUI behavior and relevant security/failure/recovery cases accepted. Live work needs separate authorization; M2 is complete. |
| CORE-017 — built candidate | blocked | Implemented package/lifecycle, authorization under release validation policy | Actual Windows package built/inspected with fixed artifacts, provenance/notices/prerequisites and documented lifecycle; no publication implied |
| GPU-014 — R1 acceptance | blocked | M2/M3 acceptance, candidate, DOC-003 and separate live authorization | Candidate GUI/CLI setup/enable/Verify/disable/save/recovery and changed lifecycle paths accepted on designated targets; exact environment/outcomes recorded, stable host and no essential/security blocker |

**M2 and its investigation/qualification lane are closed by user direction on 10 October.** The user confirmed completed live tests and working behavior; no further M2 live tests are required. [Incident](evidence/reapply-investigation-20261010.md) and [source containment](evidence/m2-source-containment-20261010.md) retain historical evidence, without claiming an established lockup cause. New M3/release live work still requires its own authorization. Never restart/shut down/log out the host without immediate explicit permission.

## Retained IDs and history

[BACKLOG_HISTORY](BACKLOG_HISTORY.md) and existing evidence preserve original results and meanings. OPEN-01's per-VM schema decision and OPEN-07's draft-navigation decision are resolved. OPEN-03/05/08/12 map to existing security/recovery/persistence acceptance; OPEN-04/10 to GPU-010; OPEN-06/11 to GUI-003; OPEN-09 to packaging acceptance. They are not separate research tasks.

GPU-010 is promoted from the historical optional/M4 placement to the stated initial-release four-category requirement; raw VRAM is already implemented. GPU-015 simultaneous sharing, additional vendors, HCS, optional API/CUDA/stress and laboratory/native-port history stay outside the active queue. CORE-021 is integrated acceptance across the named slices, not duplicate coding. Historical CORE-017 packaging scope is retained without inventing a new installer framework or assuming an existing uninstaller.
