# Changelog

## 2026-10-10 - Reapply diagnostic instrumentation

- Added opt-in crash-persistent stage, driver lookup, hash/trust and file-copy
  tracing with durations, errors and unique retained logs. The one-shot disposable
  Reapply harness continuously records lightweight host telemetry and stops after
  the test result without retries or post-test investigation.

## 2026-10-10 - Reporting and application closure

- Completed shared CLI/Slint redacted diagnostics, partial inventory and truthful
  state provenance; retained safe refresh/consent/recovery guidance.
- Completed Windows GUI-subsystem packaging with parent-console/redirected CLI
  output and installed restricted-worker checks; isolated protected configuration
  from the older laboratory root without weakening trust admission.
- Fixed Review text clipping and qualified snapshot rehearsal in the running Slint
  app. Passed 123 tests, strict Clippy and Windows x64 build; independent review
  cleared CORE-012 and APP-001. See [closure evidence](evidence/closure-sprint.md).

## 2026-10-09 - GUI/configuration validation

- Passed 103 tests, strict Clippy, Windows x64 build and docs checks; added
  configuration byte-limit/encoding and allocation round-trip coverage.
- Qualified live discovery/System/Refresh and snapshot plan rehearsal/draft
  preservation through the actual Slint executable. Recorded Review text clipping;
  protected configuration saving and GUI effects remain open. See
  [testing results](evidence/GUI-002-testing.md).

## 2026-10-09 - GUI recorded-plan rehearsal

- Separated saved desired intent from enrollment in the live controller. Snapshot
  rehearsal can read a per-VM candidate file, validate drafts through the shared
  Rust planner using recorded payload context, and display simulated stages.
  External input changes preserve/block drafts; no effects or persistent writes.
  Focused test source added; tests, builds and GUI qualification remain deferred.

## 2026-10-09 - OPEN-01 configuration contract selected

- Selected schema-2 one-target GUID-keyed TOML and explicit import semantics.
  Added shared read-only bundle splitting with production serialization; protected
  publication/recovery remain on CFG-001/SEC-001. Test cases added, not run.

## 2026-10-09 - Per-VM configuration proposal

- Documented CFG-001 / OPEN-01's schema-2 TOML proposal, state separation,
  storage reuse and migration gaps before further GUI binding. Added GUID-keyed
  production-format samples and shared bounded parser/GUI-model read/display
  entry points. Proposal and implementation remain unvalidated; no live writes.

## 2026-10-09 - Main Slint application promotion

- Added explicit historical inventory snapshot input for no-write rehearsal,
  retaining fixture scenarios. Backend plans/credentials/effects are blocked;
  reader and UI qualification remain deferred to M3 validation.

- Connected live preparation/recovery/graphics history to shared Rust presentation
  logic; reject mismatched record identities and label retained observations
  historical after failed Refresh. Focused tests added; validation deferred to M3.

- Split normal live startup from explicit `--mock-gui` rehearsal. Live cards,
  Refresh and System use shared core discovery; unconnected effects are blocked
  and discovery errors never fall back to samples. Mock mode remains fixture-based
  and write-free; real-data no-write rehearsal is the documented next direction.

- Promoted the approved prototype into the main executable: no arguments launch
  Slint; explicit commands retain the shared Rust CLI/backend. Components/icons
  now live in `src/gui/ui/`, with controller/mock fixtures in `src/gui/`.
- Removed the standalone prototype Cargo package and obsolete Win32 GUI entry and
  presentation; preserved `gui_model`, protected runner and backend contracts.
- Made the completed interface the v1.0 specification, retired absent requirements
  and completed prototype tasks, retained IDs/history and scheduled actual bindings.
- Fixed Follow Windows surfaces to match Fluent's resolved theme for Unknown,
  retaining approved colors/layout. Validation performed no VM/GPU mutation.

## 2026-10-09 - Slint prototype refinement

- Simplified the VM header to its heading and Refresh row and added consistent
  layout gaps between cards. Dialog diagnostics now have a bounded, read-only
  monospaced black-on-white viewer with mouse/keyboard selection, Ctrl+C and
  two-axis scrolling, while preserving the bottom technical toggle.

- Contained and left-aligned VM names with ellipsis and full-name tooltips;
  removed VM search/filter controls and their mock state. Shared dialogs keep
  technical-details toggles below actions with stable sizing and a separately
  scrollable technical pane, including at the minimum window size.

- Polished the prototype with neutral Fluent-style surfaces, a heading-free
  sidebar, fixed-size technical-detail buttons, state-driven VM status dots and
  local metadata-driven Windows/Linux/unknown OS icons. No new dependencies or
  production GPU/Hyper-V changes.

- Refined the mock Slint GUI with a default-collapsed Advanced Allocation heading
  and chevron, an independent 1–8 GB visual memory slider, separate navigation/VM
  panel styling, and removal of Activity and its unused history state.

## 2026-10-09 - Slint documentation reconciliation

The subsequent user-authorized exploratory prototype is implemented in
`tools/gui-prototype/`: a mock-backed Rust/Slint Fluent interface with Dashboard,
Activity, System, Settings, About, adjustable scrolling panes, twelve allocation
fields, draft confirmations, review/progress/recovery and theme choices. It has
no real Hyper-V, GPU, driver, host configuration or protected-runner operations.
The existing CLI/backend remains intact; product integration is still planned.

- Reconciled single-executable GUI/CLI/restricted-worker direction, initial GPU
  selection and four allocation triples, per-VM worker-owned saving and manual
  recovery policies across current guides, roadmap and backlog.
- Recorded source-backed reuse of runner/Named Pipe/lock/journals/workflow and GUI
  state; assigned genuine gaps to existing cards and corrected prompt/link locations.
- Documentation only; no source/config/script edits, builds or hardware tests.

## 2026-10-08 - Native GUI foundation (GUI-001)

- Added sidebar/header/five-column VM dashboard, information panel/footer, real
  system/setup/about pages and DPI-scaled resizing over the existing Win32 core.
- Drafts survive navigation/refresh; discard has no effects; explicit reapply
  uses the shared preview. Protected enrollment and stale-readback gates constrain
  actions. Verification reports pending recovery power rather than assuming it.
- Busy/close/disconnected-response handling and separate configuration-save retry
  prevent blind operation replay. Credential vault actions remain explicit.
- Independent review cleared; 69 core and 3 CLI tests, strict gates/docs/release
  passed. Native navigation/draft/refresh/discard/resize/idle-close smoke passed.
  Row controls, broader DPI/accessibility and full GUI live acceptance remain.

## 2026-10-08 - Shared operator preview (CORE-006)

- Plan and apply now share validation/decisions. Preview reports ordered effects,
  preparation drift, compatibility settings, raw allocation writes, credentials,
  downtime, pending recovery and final power; running no-op avoids a restart claim.
- GUI Apply fetches the shared preview before confirmation/credentials/execution.
  Cancellation and preview/credential failures keep accurate status; apply still
  rechecks enrollment/state. Invalid pending restoration power fails before effects.
- Independent review cleared the allocation-refresh correction. Gates passed:
  60 core tests, 3 CLI tests, strict checks/build/docs and release x64 build.
  Installed-runner enable/disable previews passed without VM/guest/journal changes.
- Full GUI layout, draft/refresh/close and visual interaction remain GUI-001 work.

## 2026-10-08 - Observed current-build qualification

- Closed the hang investigation/reproduction/observation work by user direction;
  BLK-005 no longer blocks product delivery. Retained evidence and the unknown
  cause; next active product task is CORE-006 shared preview.

- Independently reviewed, rebuilt and installed revision `96152e7`; the bounded
  default attachment/PnP/D3D11/reapply/verify/disable/cleanup sequence passed with
  host resource observation and no symptoms reported by the user.
- Bounded the temporary contributor shutdown helper after its synchronous wait
  stalled; graceful reconciliation and settings/security preservation passed.
- Recorded the original hang as not reproduced, with cause unknown. Preparation
  was reused; fresh transfer/writing under the new limits remains unqualified.
- Product gates passed: 52 core and 3 CLI tests, strict checks, release build/docs.

## 2026-10-08 - Product and GUI plan reconciliation

- Reviewed proposed roadmaps against root Rust code and M1/M2 results; retained
  completed discovery, preparation, install/enrollment, workflow and recovery.
- Recorded BLK-005 host-stability qualification blocker; GUI layout/read-only work
  can proceed while disruptive testing stays paused.
- Aligned M2 core, M3 native GUI, R1 packaging and later allocation/vendors. Removed
  laboratory port/reset/CUDA gates from production dependencies while preserving
  task IDs and historical results. Concurrent sharing stays separately qualified.
- Added shared-preview, enrollment, partial-discovery, truthful status, GUI draft/
  persistence/close and accessibility steps; reuse Win32 and the current runner.
- Established written GUI layout requirements. No features or hardware tests were
  implemented/run in this documentation review.

## 2026-10-08 - M2 verification recovery

- Resolve the inbox Hyper-V module through its trusted module directory, including
  versioned layouts. Preserve bounded failed-child results when stdin closes early;
  successful children still require complete input delivery.
- Statically link the MSVC C runtime in product builds, quality checks and CI;
  guest workers no longer require a separately installed VC runtime.
- Record pending verification on running reapply without restarting or preparing
  the guest again. Retain failed verification intent for retry.
- Restore initial power after uncertain standalone-verification startup and report
  both check and graceful-restoration failures. Added failure/retry coverage;
  independent bounded review and product checks passed.
- Attach through the discovered GPU WMI object path and reconcile full/relative
  references against local inventory. Preserve nullable provider-default allocations
  and include terminal job details in errors.
- Retain completed preparation after later failure; invalidate stale receipts before
  refresh, including interrupted refresh followed by host-driver rollback.
- Live NVIDIA one-VM preparation/default attachment/checked rendering, Off/running
  reapply, verification and disable passed; [results](evidence/M2.md). Sharing remains
  unqualified.
- Subsequent reported host freeze/unclean restart blocks M2 qualification despite
  successful functional checks. Live testing stopped pending diagnosis; cause unknown.

## 2026-10-08 - M1 product boundary qualification

- Validate runtime intent from both TOML and native installation callers; reject
  unsupported VM generations and missing VM/GPU identities before installation effects.
- Added focused tests for enrollment independent of VM names, driver pins and
  laboratory inputs, with case-insensitive VM uniqueness and multiple selected VMs.
- Added protected durable runner admission/outcome records that exclude credentials,
  retain unfinished operations and fail explicitly if publication cannot complete.
- Closed M1 after independent full-boundary review, native enrollment and
  interrupted-install recovery, ordinary-token authorization/audit/write-denial
  qualification and separate laboratory build. Product checks pass 39 library
  and 3 CLI tests; [acceptance](evidence/M1.md) records the live environment and results.

## 2026-10-07 - Runtime GPU-PV architecture rebase

- Separated the previous fixed-slot application into standalone contributor tooling.
- Introduced runtime VM/GPU contracts, a shared management workflow and native GUI boundary.
- Rewrote roadmap/instructions around working core, GUI, allocation and incremental vendors.
- Rewritten privileged/native workflows require their own review and hardware qualification; prior laboratory passes are not reused as acceptance.

## 2026-10-07 - Public Rust plan and status

- Implemented read-only `plan` and `status` using bounded native inventory. The CLI shows the configured existing VM/GPU, desired settings and initial prerequisites, with denied/unavailable observations kept explicit.
- Guest assignment, staging and readiness remain unobserved until their readers are integrated. These commands do not inspect or hash VM disks; ordinary apply integration remains in progress.

## 2026-10-07 - Product boundary verification

- Completed module classification and documented preserved development reset helpers. Default builds deny reset while retaining legacy policy compatibility; quality gates exercise default and development feature sets.
- Added mode exclusion and pre-effect reset refusal tests. Disk-guard tests now run outside checkout junction ancestry and enforce directory rename exclusion with a read-access handle.
- `scripts/testing/check.ps1` passed formatting, strict Clippy/compiler warnings, default and development-feature workspace tests/doc-tests, build, rustdoc and configuration drift (`RUST_TEST_THREADS=4`). Documentation checks and independent boundary review passed. Enrolled-runner access with the stronger directory handle remains unqualified; no live GPU qualification is claimed.

## 2026-10-06 - Product and laboratory boundary

- Clarified the existing-VM v1 workflow, product recovery and separate contributor golden-image/disposable strategy across roadmap, backlog, engineering and prompts. Qualification gates remain; implementing laboratory management does not gate product delivery.
- Preserved disposable reset effects under `tools/test-harness/` behind the non-default `dev-harness` feature; default runner builds reject reset. Recorded remaining golden-parent/configuration coupling for CORE-021/027 and existing-VM recovery for CORE-010.


## 2026-10-06 - Native Hyper-V read slice

- CORE-025 adds exact VM identity/state/generation/version and host GPU capability reads through native WMI in the fixed Rust runner. The contained worker preserves unsigned resource values and rejects stale configurations, ambiguous identities and unsafe state.
- Runner inspection removes host GPU cmdlet discovery, retaining disk/snapshot/guest-adapter guards and comparing observations. All 18 read fields passed live cmdlet parity; the updated installed runner passed integrated inspection and audit/result publication. Rust quality checks and independent review passed. Remaining inspection/settings/mutations still require migration.

## 2026-10-06 - Native inventory

- CORE-024 replaced production PowerShell inventory with native registry/system/WMI queries and a fixed, contained Rust worker. Exact configured identities and configuration binding reject ambiguous targets and stale workers; filtered management access remains denied.
- Restricted detailed provider queries to the configured targets, preserving unrelated-device isolation. Host/GPU/VM parity passed for all 14 facts on the configured Windows x64 host; full Rust quality/documentation checks and independent implementation review passed.
- CORE-025 Hyper-V read/inspect is the next native slice. Inventory parity does not qualify guest workloads.

## 2026-10-06 - Dynamic driver payload contract

- Clarified dynamic discovery of the selected signed driver's complete package/associated payload, derived guest mapping and per-run integrity; no fixed file count or static NVIDIA list defines success.
- Updated agent/prompts, roadmap/backlog, architecture, configuration and script guidance. Preserved historical baseline results and explicitly labelled the NVIDIA 616.92 mapping fixture.
- Added variable-length apply/reapply, stale receipt extent and missing-source tests. Documented current baseline package-pin guards and CORE-015 regeneration work without weakening drift validation.

## 2026-10-06 - Native Rust production migration

- Classified all 14 maintained scripts and embedded PowerShell product adapters in the [migration audit](../scripts/PRODUCT-MIGRATION.md); runner setup/recovery and policy generation remain product debt.
- Prioritized CORE-024 native inventory, CORE-025 Hyper-V management, CORE-026 Rust guest writer/transport and CORE-027 runner setup. Existing CLI integration cards own workflow, lifecycle, removal, diagnostics and restaging.
- Made demonstrated native replacements and no manual PowerShell part of v1 acceptance. Preserved working adapters and current qualification work; historical adapter decisions no longer grant broad release exemptions. This change implements the audit and delivery plan, not the backend ports.
- Updated agent guidance and task/implementation/review/test/security prompts; added a focused native migration prompt to carry the replacement and qualification rules into future work.

## 2026-10-06 - Automated guest validation

- CORE-003 completed: public `validate` now verifies transferred runtime inputs, observes sustained Code 0 and runs fixed nvidia-smi/D3D11/D3D12/CUDA checks through a Rust guest worker, with per-check evidence and exit propagation.
- Added protected transfer/launch boundaries, independent deadlines and native child containment. Standalone CUDA safely selects the sole configured GPU without making CUDA/DXGI LUID equality a computation gate.
- Independent review and 166-test quality gate passed. Combined clean-child live qualification remains GPU-006.

## 2026-10-06 - Validated Hyper-V settings

- CORE-023 completed: typed VM profile and all GPU resource triples applied through the bounded, policy-pinned Rust runner; live apply and matching reapply passed on the existing staged child.
- Independent fresh-process readback verifies effective settings and retained Secure Boot/vTPM. Durable preimages and reconciliation markers preserve partial/uncertain outcomes; automatic checkpoints can be disabled while actual snapshots remain refused.
- Independent review and 151-test quality gate passed. Automated guest workloads and combined clean-child reproduction remain separate work.

## 2026-10-06 - Runner activity supervision

- CORE-022 live fresh apply and verified no-op reapply passed on a clean child: complete 271-file environment through PowerShell Direct. Settings and workload integration remain separate tasks.

- Inspection/reset now use Rust read-activity supervision instead of a single elapsed-time cutoff, with a finite task budget and reserved transition/publication time. Client waits cover that outer budget.
- Failure cleanup reaps the adapter and drains bounded output; watchdog diagnostics include phase and read activity. Independent hash/readback and reconciliation requirements remain authoritative.
- Interactive staging asks for guest credentials before the lengthy native driver scan, so the local test window displays its prompt immediately.

## 2026-10-05 - Complete Rust driver environment writer

- `hyper-gpu-stage` now discovers and stages the complete native manifest instead of the older package/CUDA-alias subset. Fresh staging requires an unattached guest; reapply verifies the full manifest receipt and every guest length/hash.
- Reused protected target, signature, servicing, credential and session boundaries; partial or interrupted writes retain the recovery lock. Added focused source/receipt and real local-copy tests, plus mapping coverage for the 271-file baseline.
- Independent review found and closed a volume-root preflight defect. Live apply/reapply was subsequently qualified on 2026-10-06; settings and workload integration remain separate tasks.

## 2026-10-05 â€” Engineering delivery plan

- Rebuilt M1â€“M3 around Rust reproduction, usable operation/maintenance and packaged delivery. Added CORE-022 full writer and CORE-023 settings; moved automated probes into M1. Retained GPU-006 as clean-child integration acceptance.
- Removed GPU-004, merged duplicate audit/CI/recovery/release cards, and moved resource/driver-transition research outside the v1 gate. Closed BLK-003 with the discarded comparison path and corrected CORE-002's stale result.
- Made the measured [full recipe and inventory](evidence/GPU-PV-BASELINE.md) self-contained; removed active reference-project links/lookup instructions and condensed obsolete documents/decision history. Simplified all agent prompts.
- Preserved target/parent/runner and physical-host lifecycle protections. This change edits documentation and source comments only; no new implementation or hardware run.

## 2026-10-05 â€” Complete working GPU-PV baseline

- GPU-009/GPU-005 established sustained Code 0, nvidia-smi, checked D3D11/D3D12 and CUDA on the complete provisioning/settings recipe.
- Native Rust discovery matched the full measured manifest. Full writer/settings integration and repeatable product automation remain delivery work.

Earlier completed changes are retained in the [historical changelog](evidence/CHANGELOG-HISTORY.md); read it only for a specific historical result.
