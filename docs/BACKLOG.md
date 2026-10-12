# Product backlog

This queue replaces the previous task plan. [Roadmap](ROADMAP.md) owns requirements and milestones; [engineering](ENGINEERING.md#status-and-evidence) owns status conventions. Existing code is the starting point, not a request to rebuild the backend.

## Next implementation action

**Full source review (12 October 2026): APP-033 complete in source; automated validation pending.** The user authorized review of current code, small/medium fixes and tasks for larger defects. [Review findings](evidence/full-source-review.md) record the fixes and limits. APP-028 is now source implemented. Next safety work when requested: APP-034 (Apply drift checks) and existing APP-027 (failed verification-only Apply retirement). No live effects or new automated gate ran.

**UI usability work (12 October 2026): APP-030 source implemented; validation pending milestone gate and real interaction.** The [source audit](BASELINE.md#ui-interaction-audit---12-october-2026) records the initial gaps and subsequent source status. APP-024 is now source implemented with app-record-only deletion; APP-031 source now aligns operation wording; next UI action when requested: APP-025 (useful diagnostics), APP-032 (dialog interaction), then APP-026 (VRAM label). Existing recovery prerequisites and milestone gates remain in force. The user authorized APP-030, APP-024 and APP-031 source implementation; no further card or live GUI check is authorized.

Workflow source-review follow-up (12 October 2026): APP-027 (High) remains queued; APP-028 (Medium) was subsequently source implemented in APP-033. The user authorized only APP-029's compatibility-write improvement; its source and regression coverage are complete, with compilation/test execution pending.

**P0 workflow modularization: Implementation Done; validation Pending milestone gate (12 October 2026).** The user requested the [standalone plan](WORKFLOW_REFACTOR_PLAN.md) be implemented. Six private production modules now sit behind the existing workflow entry point; all 74 workflow tests and the current review fixes were preserved. Targeted formatting, structural comparison, documentation and diff checks passed. Compilation/test execution and the remaining automated gate stages have not run. No milestone closed and no live operations ran. This completed refactor does not authorize another backlog task.

**Completed UI source action: APP-024 — Add Delete profile under VM More actions, source implemented.** APP-016 through APP-022 are source implemented; APP-017 through APP-022 were reviewed and corrected as one related workstream on 12 October 2026. APP-024 now reuses APP-022's protected transition for app-record-only deletion; its separate source implementation is recorded below. The combined review ran static checks only; current compilation and regression execution remain pending under the milestone policy. APP-023 remains blocked by the agreed automated gate and separately authorized prepared live sequence. Allocation and concurrent-sharing qualification below remain open.

**APP-009 / APP-011 — Resolve the allocation contract and complete M2 preparation.** M1 is closed with [gate evidence](evidence/m1-closure.md). The [prepared sequence and source assessment](evidence/m2-preparation.md) record remaining environment facts and proposed effects. Selected-VM setup is already user-accepted; the remaining live sequence has not run. APP-011 source removes the metadata-derived VRAM ceiling; validate these changes at the agreed checkpoint before separately authorized live allocation checks.

## Task register

| ID | Milestone | Task | Depends on | Implementation | Validation |
|---|---|---|---|---|---|
| APP-001 | M1 | Remove retired mock GUI tooling | None | Done | Passed: [non-hardware gate](evidence/m1-closure.md) |
| APP-002 | M1 | Match the simple toggle and VRAM control | APP-001 | Done | Passed: non-hardware gate; hardware pending M2 |
| APP-003 | M1 | Complete review and confirmation | APP-002 | Done (R2 source; requested ahead of APP-002) | Passed: non-hardware gate; interaction pending M2 |
| APP-004 | M1 | Complete initial setup, verification, and final shutdown | APP-003 | Done | Passed: non-hardware gate; live pending M2 |
| APP-005 | M1 | Explain guest credentials before the local prompt | APP-004 | Done | Passed: non-hardware gate; native dialogs pending M2 |
| APP-006 | M1 | Separate ordinary changes from initial setup | APP-005 | Done | Passed: non-hardware gate; live pending M2 |
| APP-007 | M1 | Complete saved state, progress, and failure handling | APP-006 | Done | Passed: non-hardware gate; interaction pending M2 |
| APP-008 | M1 | Prepare and run the milestone's non-hardware gate | APP-007 | Done | Passed: [evidence](evidence/m1-closure.md) |
| APP-009 | M2 | Qualify the real GUI and two-VM workflow | APP-008 | In progress | Selected-VM setup accepted by user; remaining qualification pending |
| APP-010 | M3 | Package and document the qualified product | APP-009 | Blocked | Not run |
| APP-011 | M2 | Correct physical VRAM and provider mapping | APP-008 | In progress (ceiling removal source implemented; acceptance pending) | Not run |
| APP-012 | M2 | Make transfer progress human readable | APP-008 | In progress (source implemented; visual acceptance pending) | Pending milestone gate and live display check |
| APP-013 | M3 | Measure and improve guest transfer throughput | APP-008 | In progress (source implemented; measured acceptance pending) | Pending milestone gate and live throughput/memory comparison |
| APP-014 | M2 | Clarify slider-to-VRAM triple semantics | APP-011 | Ready (card-capacity maximum confirmed) | Not run |
| APP-015 | M2 | Prepare sustained concurrent-render evidence | APP-008 | Ready | Not run |
| APP-016 | M2 | Separate authorization from guest setup status | APP-008 | Done (source implemented) | Pending milestone gate and visual acceptance |
| APP-017 | M2 | Report saved-versus-observed host configuration differences | APP-016 | Done (source implemented) | Pending milestone gate and visual acceptance |
| APP-018 | M2 | Verify current guest payload after rollback | APP-016 | Done (source implemented) | Earlier compilation passed; combined review fixes await milestone gate; live acceptance pending |
| APP-019 | M2 | Invalidate stale success when verification fails | APP-016 | Done (source implemented) | Pending milestone gate; no compilation or tests run |
| APP-020 | M2 | Add explicitly reviewed driver reinstall | APP-019 | Done (source implemented; visual acceptance pending) | Pending milestone gate; no compilation or tests run |
| APP-021 | M2 | Make GPU removal usable after rollback | APP-016 | Done (source implemented; visual acceptance pending) | Pending milestone gate; no compilation or tests run |
| APP-022 | M2 | Remove an inactive pairing and its saved intent | APP-021 | Done (source implemented; visual acceptance pending) | Pending milestone gate; no compilation or tests run |
| APP-023 | M2 | Qualify checkpoint rollback detection and recovery | APP-017, APP-018, APP-019, APP-020, APP-022 | Blocked: automated gate and prepared live authorization | Not run |
| APP-024 | M2 | Add Delete profile under VM More actions | APP-022 | Done (source implemented; app records only) | Pending milestone gate and real interaction; regressions unrun |
| APP-025 | M2 | Keep Technical Details only where they add diagnostic value | APP-008 | Ready | Not run |
| APP-026 | M2 | Display the slider VRAM maximum as a whole GB | APP-002 | Ready | Not run |
| APP-027 | M2 | Retire a failed verification-only Apply without blocking repair | APP-019, APP-020 | Ready (High review finding) | Not run |
| APP-028 | M2 | Preserve the guest failure when restoration-proof persistence fails | APP-018, APP-019 | Done (source implemented in APP-033) | Pending milestone gate; regressions unrun |
| APP-029 | M2 | Skip unchanged compatibility writes during enabled Apply | APP-006 | Done (source implemented) | Pending milestone gate; regressions unrun |
| APP-030 | M2 | Explain selected-VM action availability before activation | APP-016, APP-021, APP-022 | Done (source implemented) | Pending milestone gate and real interaction; regressions unrun |
| APP-031 | M2 | Make action labels and review language match actual effects | APP-018, APP-020, APP-021 | Done (source implemented) | Pending milestone gate and real interaction; regressions unrun |
| APP-032 | M2 | Make dialogs usable with keyboard and constrained layouts | APP-003 | Ready (Medium interaction risk) | Not run |
| APP-033 | M2 | Review current source and fix bounded logic gaps | APP-008 | Done (source review and fixes) | Static checks only; automated gate pending |
| APP-034 | M2 | Recheck admitted Apply facts before delayed writes and power restoration | APP-006, APP-019 | Ready (High review finding) | Not run |

Current-source review (11 October 2026): [APP-001 through APP-008 assessment](evidence/m1-source-review.md) records the original five fixes and completed follow-up review. The [M1 closure](evidence/m1-closure.md) supersedes historical unrun/open statements in the implementation notes below for automated validation only; live qualification remains pending.

## Implementation tasks

### APP-001 — Remove retired mock GUI tooling

Remove the AI-era mock and snapshot GUI entrypoints, dedicated modules/scenarios, scenario controls, and dedicated mock-only checks. Trace dependencies through dispatch, GUI host, controller, and Slint bindings before deleting anything shared with the live application. Update help and source comments affected by the removal. No replacement simulation or preview framework.

Acceptance: production GUI/CLI and fixed internal entrypoints remain; removed modes are no longer advertised or accepted as supported workflows; shared code is preserved; no dedicated mock maintenance remains. Add no new mock scenarios. Meaningful backend tests using isolated test doubles remain valid. Validation waits for APP-008.

Implementation (11 October 2026): removed mock/snapshot dispatch, dedicated modules, timer scenarios, sample defaults, rehearsal controller/state branches, and dedicated mock checks. Shared live model/dialog helpers and protected operation paths remain. Added argument-free GUI dispatch regression coverage; retained live draft, stale-observation, publication, and recovery tests. Static reference/diff review and `git diff --check` completed; no formatting tool, build, test, GUI launch, or host/guest effect ran. Compilation and real UI acceptance remain pending.

### APP-002 — Match the simple toggle and VRAM control

Keep the existing design. Provide the GPU support toggle and card-dependent VRAM amount, from 1 GB to detected physical capacity. Only the toggle disables GPU support. Inspect current slider bounds/conversion and the native provider before changing mapping. Retain schema compatibility; do not rewrite existing settings silently.

Acceptance: range follows the selected card, minimum is 1 GB, saved/draft values remain distinct, and unsupported or unknown capacity produces a truthful unavailable state. Displayed units and provider writes agree for supported hardware. No balancing, new allocation policy, or full-capacity warning feature. Assess existing advanced controls against the simple product scope without redesigning the GUI.

Implementation (11 October 2026): slider starts at 1 decimal GB and ends at matched physical capacity, retaining exact non-grid endpoint bytes. The current conversion rejects some points by comparing byte requests with raw provider metadata. This is an unresolved mapping defect (APP-011), not evidence of a physical hardware ceiling. Existing requests below 1 GB are labeled exactly with an explanation, not silently rewritten. A single-value 1 GB range uses an explicit selection button. The native writer passes the same byte integers directly to Hyper-V's VRAM properties with fresh limits and readback; no capacity ratio or backend mapping change was introduced. Removed Compute/Encode/Decode editors from the GUI; existing schema-3 requests remain intact and visible in review. Advanced Allocation retains exact VRAM fields for compatibility/correction. Added boundary and controller tests covering saved schemas, other VMs/categories, toggle independence, and rejected/stale edits. Static source/reference review and `git diff --check` only; no formatter, build, tests, render, GUI launch, or live effects ran. Supported-hardware behavior still awaits APP-009 qualification; M1 remains open.

### APP-003 — Complete review and confirmation

Edits stay pending. First Apply opens a review naming the VM and describing actual changes, downtime, required start/shutdown/restart steps, expected final power state, and relevant time warnings. Second Apply confirms the fresh bound plan; cancel returns to editing.

Acceptance: review matches effects, target changes cannot redirect execution, stale inputs require renewed review, and no effects occur merely by editing. Keep protected setup/enrollment boundaries while making the flow understandable.

Implementation (11 October 2026): R2 source work was explicitly requested ahead of APP-002. First Apply opens the existing review dialog; second Apply confirms. Review names the VM/GPU, shows allocation and changed compatibility settings, ordered lifecycle/effect steps, the backend's expected final power, downtime, and preparation/boot/graphics time warnings. Cancel clears approval while retaining edits. Approval binds the selected VM, exact draft, and original committed revision; late plan replies and confirmation reject changed inputs. The protected worker continues to independently reject stale backend plans before journaling or effects. Added controller review/cancel/revision and presentation regressions; expanded backend stale-state coverage. Static source/diff review and `git diff --check` only; no formatter, build, tests, render, GUI launch, or live effects ran. APP-002 controls, APP-004 initial lifecycle, and APP-006 ordinary lifecycle remain separate; review truthfully describes the current backend until those tasks change it. M1 remains open.

### APP-004 — Complete initial setup, verification, and final shutdown

Reuse package setup, additive enrollment, shared preparation, and the existing guest graphics check. Distinguish first GPU setup from ordinary changes. Review the actual lifecycle sequence, prepare the guest, establish GPU support, check that it works, report pass/fail, and gracefully leave the VM off after completed setup.

Acceptance: setup on a second VM preserves the first; initial verification belongs to setup rather than an extra mandatory user action; success includes confirmed final shutdown. Failure or uncertain outcome remains honest and recoverable. Do not force power-off or blindly repeat interrupted setup.

Implementation (11 October 2026): the shared planner distinguishes initial setup using existing completed preparation/graphics receipts. Initial setup prepares as needed, establishes the selected GPU/resources, checks guest hardware rendering, and requires independently confirmed final-off state before returning success or recording successful verification. Review labels initial setup and includes its ordered final shutdown. Existing package setup and additive enrollment reviews remain separate authority steps; they do not authorize guest effects. Completed historical preparation/graphics receipts remain recognised; later-operation behavior is reserved for APP-006. Protected recovery records now persist the reviewed final power separately from the starting observation; older records retain their original-power contract. Reconciliation uses that intent without replaying setup, and save-only rejects an outcome in the wrong power state. GUI completion names the VM and reports rendering pass plus final shutdown; failures retain pending recovery and cannot report setup success. Added off/running setup, false/refused shutdown, preparation/graphics failure, no-replay, explicit reconciliation, two-VM isolation, receipt compatibility/publication, and review regressions. Static source/reference review and `git diff --check` only; no formatter, build, tests, render, GUI launch, guest access, protected write, or hardware operation ran. APP-008/APP-009 acceptance remains pending; M1 stays open.

### APP-005 — Explain guest credentials before the local prompt

Before opening a credential prompt, show the selected VM's friendly name and explain that the username/password are for Windows inside that VM, not the host computer. Offer explicit optional remembering through the existing local vault.

Acceptance: explanatory panel precedes the prompt, cancellation has a clear outcome, optional remembering is not pre-authorized, and credentials never enter logs/configuration/chat. Reuse existing local credential handling.

Implementation (11 October 2026): the shared native credential path now opens a blocking explanation panel naming the bound VM and distinguishing its Windows sign-in from the host account before opening credential entry. OK continues; Cancel at either panel stops before worker submission. Remember remains unchecked and only explicit selection writes to the existing per-user/per-VM vault. Already stored opt-in credentials remain reusable without prompting. GUI Settings, Apply, Verify, and recovery distinguish local cancellation from failure; pending edits remain, cancelled Apply/Verify reviews are cleared, and recovery cancellation retains its record/hold without reconciliation. Added isolated regressions for explanation/input order, both cancellation stages, explanation/vault failure, stored reuse, optional storage/write failure, and display-name sanitization. Static source/reference review and `git diff --check` only; no formatter, build, tests, render, GUI launch, credential prompt, guest access, or protected write ran. APP-008/APP-009 acceptance remains pending; M1 stays open.

### APP-006 — Separate ordinary changes from initial setup

Later enable/disable and VRAM updates use the same review and confirmation but do not repeat full preparation or initial rendering verification. Retain fresh authorization, provider validation, attachment/settings readback, and recovery.

Acceptance: ordinary changes avoid unnecessary guest access, preparation, and graphics checks; their real outcomes are reported. Required power transitions are managed by the app and shown in review. The user confirmed on 11 October 2026 that ordinary changes always finish off, as does completed initial setup. Changed drivers or invalid preparation must produce an explicit requirement for setup/update rather than silently hide expensive work in a toggle.

Implementation (11 October 2026): the user selected always-off final power for ordinary changes. Completed current preparation/graphics receipts allow enable/disable and VRAM changes without guest credentials, temporary boot, preparation, or graphics verification. Running guests are gracefully stopped; success requires independent attachment/settings/allocation and final-off readback. Missing/invalid receipts or a changed signed payload explicitly require setup/update in review, including its guest work and time warnings. Setup/update still checks hardware rendering and finishes off. Ordinary work preserves historical graphics timestamps. The protected receipt now retains whether graphics were reviewed; ordinary reconciliation requires achieved final power and readback without guest access or boot. Older receipts retain their original graphics requirement. Failures retain recovery and block new Apply admission. Added lifecycle, two-VM isolation, invalid-preparation review, failure/readback, no-guest reconciliation, receipt compatibility and presentation regressions; updated existing lifecycle expectations. Static source/diff review and `git diff --check` only; no formatter, build, test, render, GUI launch, credential prompt, guest access, protected write, or hardware operation ran. APP-008/APP-009 acceptance remains pending; M1 stays open.

### APP-007 — Complete saved state, progress, and failure handling

Audit current per-VM settings, refresh/draft conflicts, operation progress, protected publication, and recovery against the simplified workflow. Preserve the protected runner, typed worker modes, authenticated IPC, trust checks, locks, and durable receipts.

Acceptance: saved settings survive reopen; state labels distinguish pending, observed, applied, and initial verification; progress stays responsive and reports actual stages; another VM is unaffected. Missing terminal outcomes stay uncertain. A completed-but-unsaved operation offers a bound save-only retry without replaying GPU effects. Address concrete trust/recovery gaps found in these paths.

Implementation (11 October 2026): audited the existing committed source at `4c293ec`. The controller separates saved, pending, observed, applied and historical graphics state; completed effects clear only the matching draft. Protected reads and artifact availability run in background jobs. Lost replies/task panics require readback; terminal replies are checked against operation, target, lifecycle and publication. Completed publication receipts permit bound save-only retry; incomplete effects require reconciliation. Fresh readable store state is required to clear unsaved intent, and conflicts remain latched until discard. Existing isolated regressions cover reload, other-VM preservation, read failures, replaced receipts, conflicting/unreadable/deleted files, duplicate progress events and missing terminal outcomes. Follow-up source review found no further blocker; execution and real responsiveness/interaction acceptance remain pending.

## Validation and delivery tasks

### APP-008 — Prepare and run the milestone's non-hardware gate

Prepare a narrow non-elevated gate against current files; do not restore deleted tooling wholesale. Inspect selected tests for effects. At M1 completion run appropriate documentation, formatting, non-hardware tests, lint, product binary, and rustdoc checks described in [testing](TESTING.md#planned-automated-gate).

Acceptance: intended checks pass or failures keep M1 open; evidence states exact commands, revision, outcomes, and exclusions. Tests cover initial-versus-ordinary operations, final-off setup, credentials explanation, review binding, slider bounds, and meaningful failure boundaries. No GUI launch, VM access, privileged setup, or GPU probe in the automated gate.

Prerequisite from the user's review request: repeat the [current-source review](evidence/m1-source-review.md) after APP-006/APP-007 implementation, fix in-scope gaps, and present larger issues requiring separate tasks before their implementation. This review does not replace executed gate evidence.

Completion (11 October 2026): follow-up review complete; narrow non-elevated gate added and passed documentation, formatting, all-target non-hardware tests (142 core, 50 frontend), strict Clippy, product binary builds and rustdoc. Initial formatting failure was corrected; gate stderr reporting was corrected and final process exited 0. Exact revision, working-tree/source hashes, environment, commands, outcomes and exclusions are recorded in [M1 closure evidence](evidence/m1-closure.md). M1 is closed; APP-009 is ready for preparation, with live effects awaiting separate authorization.

### APP-009 — Qualify the real GUI and two-VM workflow

Prepare the exact environment and run sheet before asking for live authorization. Select host/guest builds, NVIDIA GPU/driver, two existing Generation 2 Windows 11 VMs, workloads, observation duration, effects, final power states, evidence paths, and stop conditions. Then run only the authorized sequence.

Acceptance: initial setup verifies each VM and leaves it off; later toggles and VRAM changes follow review without repeated initial verification; user-facing credential explanations and progress work. Saved settings survive reopen. Two guests produce checked hardware rendering concurrently, and operations on one preserve the other's functioning. No mock preview, automatic retry, host restart, or unrequested post-run investigation. Interrupted results keep M2 open.

Session handoff (11 October 2026): [live findings and changes](evidence/live-gui-check.md) record the real GUI/enrollment passes, the subsequent user-confirmed setup success, source-only follow-ups and remaining tests. The user initially requested an end to the chat while setup continued, then confirmed successful VM setup. Do not repeat accepted setup solely for qualification; remaining scenarios stay separate. APP-011 through APP-013 track the deferred mapping/readability/transfer changes.

### APP-010 — Package and document the qualified product

Prepare the required fixed package artifacts, hashes, notices, setup instructions, supported environment, limitations, and troubleshooting. Run final automated checks and obtain separate authorization for clean-install/upgrade trials.

Acceptance: package is complete, artifact trust and enrollment/settings preservation are checked, actual trials pass, and support claims match recorded evidence. Publishing or pushing requires a separate request.


### APP-011 ? Correct physical VRAM and provider mapping

The live GUI converts raw provider values into GB and backend admission compares physical byte requests directly against those values. The user confirmed that the resulting 1 GB ceiling claim is false. The user subsequently explicitly requested removal of any mistaken 1 GB hard ceiling. This authorizes source changes to VRAM admission, not live writes. Allocation qualification remains pending.

Establish and implement the correct physical-memory/provider contract in the shared Rust path and GUI. Preserve existing exact saved requests; do not invent a capacity ratio, reservation, or enforcement guarantee. Review source, provider semantics and independent readback before authorizing writes. Keep fresh protected admission and recovery boundaries.

Acceptance: the slider covers 1 GB through detected physical capacity with truthful values; GUI/backend agree on conversions and admission; regression tests cover the observed raw-metadata case, boundaries and invalid inputs; authorized live allocation and independent readback pass. Raw metadata must never be presented as a physical 1 GB ceiling.

Implementation (11 October 2026): removed reported VRAM partition range comparisons from the slider, shared GUI/planner admission and native writer. Requests use documented bytes unchanged; Hyper-V acceptance and exact independent readback decide the outcome. Fresh complete capability, request ordering, stale approval, enrollment, recovery and other-category range checks remain. The slider retains the requested 1 GB minimum and detected physical upper endpoint; saved intent is not rewritten. Apply review shows exact byte triples. Added regression source for multi-GB requests above metadata, retained requests, category isolation, malformed input, driver rejection and readback failure with recovery. [M2 preparation](evidence/m2-preparation.md) records the correction. No ratio or new backend physical-capacity limit was invented. Build, tests and live interaction remain unrun.


### APP-012 ? Make transfer progress human readable

Live transfer progress currently prints large raw byte counters. The user requested a follow-up note to use readable units such as KB, MB and GB, plus a percentage, while updating the transfer row in place. Keep file counts useful and distinguish copied/acknowledged progress from completed verification. Avoid one log-style line per chunk.

Acceptance: transfer sizes use consistent readable units and sensible precision; percentage follows acknowledged bytes without claiming verification; one row updates in place; real lifecycle changes, failures and final verification remain visible. Cover zero totals, boundary units, large transfers and failure display.

Implementation (11 October 2026): the worker uses a shared public formatter for acknowledged byte/file counters. Display uses decimal KB/MB/GB (and larger units), one decimal place, and an integer percentage that cannot round an incomplete transfer up to 100%. Rounded size boundaries promote to the next unit. Zero totals omit the undefined percentage; inconsistent counters show progress unavailable. Every transfer row says verification pending, including 100% copied. The GUI uses the shared fixed prefix to replace the current running transfer row without rebuilding its model; lifecycle, completion and failure rows remain separate. No protocol fields, effects, stage outcomes or verification logic changed.

Validation: formatter/unit-boundary/large-counter/zero-total/invalid-counter regressions added; existing in-place model/failure regression updated for readable text. Formatting, documentation/task/link checks and diff whitespace only. No build, tests, GUI launch, VM access or transfer rerun; visual and runtime acceptance remain pending. APP-013's separate throughput change is preserved.


### APP-013 ? Measure and improve guest transfer throughput

The live initial setup transfers roughly 3.4 GB across 273 signed driver files. The observed live build sent sequential 1 MiB chunks with a PowerShell Direct invocation and acknowledgement for each chunk. The user asked why this stage takes several minutes. The serial round trips add overhead; the precise bottleneck has not been measured.

Measure transport, serialization, guest disk and verification time before choosing an optimization. User change note: change transfer chunks from 1 MiB to 4 MiB (4 MB in the user request). Update the PowerShell sender buffer and Rust acknowledgement bound together. Preserve bounded memory and ordered acknowledgement/deadline checks; measure throughput and memory after implementation. The original request added a note only; the subsequent explicit APP-013 request authorizes the source implementation. Preserve bounded memory, deadlines, ordered acknowledgements, independent authentication and interruption/recovery behavior. Do not introduce whole-file buffering, blind retry or claim a faster UI reduces transfer time.

Acceptance: independently measured transfer improvement on the authorized environment, responsive progress, bounded host/guest memory, and failure/deadline/integrity regressions. Any live comparison requires its own prepared run; no repeated transfer is authorized by this backlog note.

Implementation (11 October 2026): source now uses one Rust-owned 4 MiB limit (4,194,304 bytes), injected into the fixed PowerShell sender and used for acknowledgement admission. The sender trims partial chunks before remoting so small files and final fragments do not serialize a full 4 MiB buffer; the receiver checks the exact positive chunk length. Streaming, serial acknowledgement, file-length/order/total validation, deadlines, independent guest authentication and recovery semantics remain intact. Added isolated regressions for multiple full 4 MiB chunks plus a tail, empty files, oversized/duplicate acknowledgements, idle/incomplete failure, and actual local copy-function execution with a remoting substitute to check hashes, exact tail sizes and stop-on-bad-ack. The local regression has no Hyper-V import, VM session or credential access.

Validation: formatting, documentation/task/link checks and diff whitespace only. No build, unit-test execution, installed-package update, GUI launch, guest access or repeated transfer. Throughput and process memory are not yet measured; no speedup is claimed. Source implementation is complete, but APP-013 acceptance remains pending its non-hardware gate and a separately prepared live comparison. Preserve the user-accepted setup; do not replay it merely to benchmark this change.

### APP-014 ? Use selected-card capacity for the VRAM maximum

User clarification (11 October 2026): 2 GB was an example, never a fixed cap. The maximum must follow the selected GPU's detected physical VRAM capacity. Do not introduce a fixed 1 GB, 2 GB or example-card maximum. The slider's chosen amount remains the optimal request; MaxPartitionVRAM should use the selected card's capacity rather than an inherited provider/default or old-card maximum.

Source review found that `MemorySlider::select` retains/expands the existing maximum, and omitted intent borrows provider defaults as its editing baseline. Replace that editing behavior with the confirmed selected-card maximum when the user explicitly edits the slider. Preserve exact saved triples on load; do not silently migrate settings. Handle GPU changes and unavailable capacity truthfully, retaining ordered triples and reviewed writes. No reservation or enforcement claim follows from setting the maximum.

Acceptance: slider bounds, reviewed maximum and written MaxPartitionVRAM follow the selected card; selected optimal remains within that range. New requests do not inherit an arbitrary metadata ceiling. Cover decreasing/increasing selection, exact physical endpoints, legacy triples, omitted requests and GPU switching. Source implementation and tests remain pending; this clarification updates the task only.

### APP-015 — Prepare sustained concurrent-render evidence

Source review (11 October 2026): the existing `d3d11-probe` performs a finite offscreen check and exits. Successful sequential probe results do not establish the proposed ten-minute overlap in APP-009. The preparation record still needs exact workload commands, hardware identity, overlapping timestamps and independent success criteria.

Prepare a bounded qualification workload/evidence method using existing components where suitable. Select sustained workloads for both recorded guest GUIDs, record their simultaneous rendering/results and observe the second workload through a selected-target change. Keep this outside ordinary product Apply; do not introduce mandatory repeated graphics checks, retired mock tooling or an alternate effect backend. Source/tooling preparation does not authorize guest execution.

Acceptance: the prepared run sheet contains exact commands, duration, evidence paths, success/stop criteria and final power policy; evidence distinguishes continuously overlapping work from sequential verification. Any new tooling requires meaningful isolated failure coverage. Execution remains separately authorized APP-009 work.

### APP-016 — Separate authorization from guest setup status

User report (11 October 2026): rolling a VM back to a checkpoint from before GPU driver installation leaves the application reporting enrollment and provides no clear recovery. Source investigation is recorded in [baseline](BASELINE.md#checkpoint-rollback-investigation). Host enrollment is permission to manage an exact pair; host preparation and graphics receipts are historical and survive guest rollback. Do not revoke authority or claim a detected rollback from those records alone.

Acceptance: the VM status labels permission as Authorized, separately from current attachment and graphics history. Previously prepared state explicitly says current guest files are unknown and checkpoint rollback can undo setup. Retain the past check timestamp as history, recovery precedence, unavailable-state handling, stable target identity and all other VMs. Add isolated presentation regressions for unchanged host receipts after guest rollback; no discovery-time guest access or automatic repair.

Implementation (11 October 2026): changed the enrolled VM row label to Authorized, explained authorization separately in the support summary, and made preparation history explicitly describe rollback uncertainty and the separate Verify check. Updated row/history coverage and added a regression asserting unchanged host receipts cannot certify current guest setup or mutate authority. Existing layout, journals and protected operation paths are retained. Source/diff and documentation checks only; regression execution, compilation and visual acceptance are pending. This fixes presentation only; detection and recovery remain the following cards.

### APP-017 — Report saved-versus-observed host configuration differences

Compare readable saved intent with fresh Hyper-V attachment, explicitly requested allocations and relevant managed compatibility settings. Reuse current discovery and configuration readers; enrollment settings are authority, not the expected applied configuration. Show specific differences without blocking the user's route to reviewed repair/removal merely because the expected GPU is absent.

Acceptance: report expected versus observed values for missing/wrong attachments and changed requested resources/settings; saved disabled state with no attachment is consistent. Missing/unreadable intent, unavailable observations and omitted allocations remain Unknown rather than an invented failure/default. Cover matching, missing, unrelated-GPU, changed-allocation, malformed/unavailable and stale-read cases using isolated controller/model tests. Refresh does not write state or access the guest; protected admission still rejects unrelated hardware and stale reviews.

Implementation (11 October 2026): the selected VM support summary now compares readable committed intent with the latest Hyper-V inventory. Differences name expected/observed attachments, explicit resource triples and compatibility values from the applicable managed host baseline. Drafts and enrollment allocation settings are never comparison intent. Missing/unreadable intent, incomplete or historical observations, omitted requests and unavailable managed baselines remain Unknown. Disabled intent with no attachment is consistent; old applied settings are not treated as its required baseline. Missing expected attachment retains the existing reviewed edit/removal route; unrelated hardware and stale execution remain blocked. Added isolated comparison/controller regressions; no provider writes, guest checks or authority changes are introduced.

Validation: targeted Rust formatting, documentation/link/task checks and diff whitespace only. Compilation, regression execution and visual acceptance are pending the agreed milestone gate. No GUI launch, guest access, VM/GPU effects or rollback qualification ran.

### APP-018 — Verify current guest payload after rollback

Extend explicit Verify through the existing protected worker and guest transport to check current guest preparation independently of the host digest receipt. Compare the current authenticated manifest with actual guest files and any matching guest receipt, alongside the existing selected-hardware PnP/rendering check. Guest checks require the existing reviewed lifecycle and local credentials. A matching host receipt must never bypass this explicit check.

Acceptance: removed, changed or partial guest driver files are detected even with unchanged host receipts and driver version; distinguish payload mismatch, graphics failure and unavailable/denied checks. An absent host GPU produces an actionable host discrepancy without falsely claiming guest verification. Cover complete, absent, partial, corrupted, wrong-pair, denied and timed-out guest observations with isolated doubles. Verify never reinstalls driver payloads or silently updates desired settings; disclose any existing helper bootstrap writes and power effects accurately. Unknown checkpoint origin is not reported as proven rollback.

Implementation (11 October 2026): explicit Verify now discovers/authenticates the current host manifest for its review and binds the digest into worker admission and recovery evidence. The native guest check compares every current destination's length/hash and the guest receipt's manifest digest, file count and current machine identity before checking PnP/rendering. It never accepts host preparation history as a substitute. Manifest membership is bound to the selected physical GPU, validated relative destinations, uniqueness and size limits; reads use bounded buffers/deadlines, protected paths and handles that deny writes/deletion. The existing guest preparation lock prevents overlapping product preparation. Missing/changed files, mismatched receipts, denied/unavailable/timed-out observations and graphics failures have separate safe component/cause diagnostics. A missing host attachment blocks Verify with a reviewed-change/removal instruction before guest work.

Review discloses temporary start/graceful restoration for an off guest, guest administrator credentials, refreshed protected helper executables, verification metadata/lock and optional diagnostics. Verify transfers no driver payload, rewrites no preparation receipt or saved intent, and does not infer a proven checkpoint rollback. The shared native verification path also checks payload parity during setup and explicitly reviewed recovery. Verify-only reconciliation checks the recorded current payload independently without requiring or upgrading an old host preparation digest; management/setup reconciliation still requires matching preparation. Failure retains recovery and approved lifecycle restoration; stale-success persistence and preparation eligibility remain APP-019.

Validation: `cargo check --locked --all-targets` passed, including compilation of isolated payload, review, failure/lifecycle and diagnostic regressions. Targeted formatting, PowerShell syntax parsing, documentation/link/task checks and diff whitespace passed. No tests were executed; no full binary build, GUI launch, credentials, guest access, protected writes or VM/GPU effects ran. Behavioral and live acceptance remain pending the agreed gate.

### APP-019 — Invalidate stale success when verification fails

The current Verify path retains the previous `last_verified` and `prepared` fields when a new check fails, alongside pending recovery. Record the latest check outcome separately from historical success so reopening cannot treat a failed/currently unknown guest check as valid setup. A confirmed payload mismatch must require preparation on a subsequent explicitly reviewed repair.

Acceptance: failed, interrupted, unavailable and successful checks remain distinguishable; historical timestamps stay labeled as history. Proven mismatch invalidates preparation eligibility before another Apply can skip setup. Credential cancellation before admission does not manufacture a guest failure. Persistence/readback failure, start-after-effect error and refused shutdown retain evidence and recovery holds. Add workflow/recovery regressions proving no failure clears a hold, replays work, or certifies old receipts; schema compatibility is explicit.

Implementation (11 October 2026): the protected per-VM journal now records `latest_check` independently of historical `last_verified`. Setup, explicit Verify and graphics reconciliation persist incomplete state before checking, then independently read back the fixed passed/failed/unavailable/payload-mismatch outcome. A proven missing/changed payload or mismatched guest receipt clears preparation eligibility, requiring the preparation sequence even with an unchanged host digest after recovery permits a reviewed operation. Unknown/failed checks cannot use old success to skip verification. Guest-check success remains separate from lifecycle completion; interrupted starts, refused restoration, persistence/readback errors and unfinished publication retain recovery. Read-only review and pre-admission credential cancellation write no outcome. The support summary shows the latest outcome and retains old successful timestamps as history. Added isolated outcome, reopen, same-digest preparation, persistence/readback, lifecycle and schema regressions; no new repair action is included.

Validation: source review, targeted Rust formatting and diff whitespace checks only. No compilation, build, test execution, GUI launch, credentials, guest access, protected writes or VM/GPU effects ran. Regression and live acceptance remain pending the agreed gate.

### APP-020 — Add explicitly reviewed driver reinstall

Expose a clear Repair / Reinstall action through the existing planner, protected worker and preparation backend. The current Reapply / Update action only stages enabled intent; matching historical receipts can omit preparation. Explicit repair must request the complete preparation/graphics sequence even when the host digest has not changed, without altering ordinary toggle/VRAM behavior.

Acceptance: review binds reinstall intent, exact VM/GPU, current payload, settings, credentials, power transitions and final-off state; a fresh worker plan independently checks that intent before effects. Unchanged host receipts cannot turn repair into a no-op. Complete preparation and selected-hardware verification are required for success. Cover stale approval, pending recovery rejection, partial preparation, failed graphics, shutdown failure and successful same-digest repair. Preserve other VMs and saved resource categories; cancellation has no effects. No automatic repair after reopen or failure.

Implementation (12 October 2026): More actions now offers Repair / Reinstall using the existing enabled draft, review dialog and Apply worker. An explicit `reinstall` flag belongs to the operation plan, not saved settings; the read-only protected planner and fresh worker admission independently derive complete preparation even with unchanged host receipts/digest. Review names the pair/current payload, retained category requests, compatibility settings, guest credentials, power transitions and final-off result. Ordinary toggle/VRAM Apply continues using the ordinary planner. Repair invalidates the preparation receipt before effects, retains historical rendering time, runs the existing authenticated transfer/preparation and current-payload/graphics check, then requires final-off readback. Independently matching destination files may be retained by the existing preparation backend; host history never bypasses the sequence. Recovery/publication holds and save-only handling are reused, with no automatic repair or replay. Added isolated workflow and controller regression source for success, rejection, partial failure, cancellation and other-VM preservation.

Validation: source review, targeted Rust formatting and diff whitespace checks only. No compilation, builds, tests, UI render/launch, credentials, guest operations, protected writes or VM/GPU effects ran. Real menu/review interaction and behavioral acceptance remain pending the agreed gate.

Review fixes (12 October 2026, APP-019/APP-020): explicit reconciliation now supports ending a terminal failed Verify after independently establishing its restored starting host state, retaining the failed/unknown check and invalid preparation instead of requiring guest success first. Setup/reinstall journals prove whether preparation-related effects could have started; refusal of the initial shutdown can end through independent unchanged-host readback and restoration of prior history. Both exits bind/archive the before/after journals before retirement, permit only matching interrupted transitions, and retain holds on unavailable/changed facts or persistence/readback failure. Verify revalidates host/GPU facts before restoration and never restarts an externally stopped running guest. Added isolated exit/repair, rejection, interrupted-transition and external-stop regressions. These fixes are source-only: formatting and whitespace checked; compilation, test execution, visual and live acceptance remain pending. At that review checkpoint APP-021 remained the next separate task.

### APP-021 — Make GPU removal usable after rollback

Reuse the GPU-off draft, review and shared disable workflow as a clearly available removal route after missing/invalid guest preparation. Removal must not require working guest GPU drivers or a successful graphics check. Distinguish removing host GPU support from deleting guest drivers or revoking product authority.

Acceptance: detach the exact managed GPU if present, restore only still-owned compatibility settings, publish disabled intent and confirm final-off state. Already absent attachment succeeds through independent readback without inventing guest verification. Cover absent driver receipts, missing adapter, externally changed settings, unrelated GPU refusal, recovery holds, stale review and partial failure. Preserve unrelated devices, guest files, other targets and uncertain-effect evidence. GUI wording and availability are tested separately from live acceptance.

Implementation (12 October 2026): More actions exposes Remove GPU support through the existing disabled draft, review and protected Apply/publication flow. It requires the exact authorized pair, preserves retained resource requests, and explains host removal separately from guest file deletion, authorization or history removal. Missing preparation or an already absent adapter does not require guest access, credentials, driver transfer or graphics verification. Compatibility restoration now compares each field with its recorded product write, preserves externally changed/unowned values, and rechecks settings after shutdown/detach before writing. Confirmed removal/reconciliation marks compatibility ownership released while preserving history; another removal cannot reclaim old settings, and later enable captures its new baseline. Existing fresh admission, recovery holds, exact native adapter binding, disabled publication and final-off readback remain. Added isolated controller/workflow source coverage for missing receipts/adapter, absent ownership, cancellation, preserved requests, settings changes, stale review, unrelated hardware, recovery, partial failure, repeat removal and other-VM preservation.

Validation: source review, targeted Rust formatting and diff whitespace checks only. No compilation, build, tests, GUI render/launch, credentials, guest access, protected writes or VM/GPU effects ran. Behavioral and visual acceptance remain pending; At that checkpoint APP-022 was the next separate source task.

### APP-022 — Remove an inactive pairing and its saved intent

Connect the currently unavailable Forget pairing workflow after successful GPU removal. Implement an exact-pair, revision-bound protected authority transition through existing enrollment/publication mechanisms. Review separately describes removing product authorization and its saved intent/managed history; it does not imply guest driver uninstall.

Acceptance: refuse active GPU support, pending operation/publication/enrollment recovery or conflicting authority/configuration revisions. Preserve every other pair and document, including another VM sharing the GPU. Independently read back authority and retained/deleted records; interruptions retain reconciliation evidence rather than leaving an unreported partial deletion. Cover missing/already-removed pairing, stale review, denied writes and failure at each publication boundary. Reopening permits a new reviewed enrollment/setup; historical receipts cannot skip that setup. Never delete shared package artifacts or unrelated guest files.

Implementation (12 October 2026): More actions > Forget pairing obtains a read-only protected removal review and a separate confirmation naming the exact inactive VM/GPU and the saved intent/managed history to delete. Existing worker authentication/elevation, operation lock, enrollment policy publication and enrollment-pending hold are reused. Fresh admission binds operator, full authority revision and complete saved/history revision sets; the VM must be off with no GPU adapter, saved intent disabled/absent and compatibility ownership released/absent. Active/unknown/pending/conflicting states are refused. A durable removal record precedes mutations; selected saved intent and its managed journal are removed/read back before exact authority revocation. Every retained record and artifact/operator policy remains bound. Explicit Recovery resumes only remaining approved deletions; completed deletions must remain absent, and unknown/new revisions retain the hold. The terminal audit precedes hold release. Other VMs, guest files, VM/disks/checkpoints, package artifacts and operational audits are retained. Last-pair removal keeps installation/operator/artifact policy; new enrollment is separate, and missing host history requires complete setup. Added isolated transition/failure-boundary, inactive-admission, stale/cancellation and reenrollment/setup regression source. APP-024's Delete profile presentation remains separate.

Validation: source review, targeted Rust formatting and diff whitespace checks only. No compilation, builds, tests, GUI render/launch, credentials, guest access, protected writes or VM/GPU effects ran. Visual and behavioral acceptance remain pending the agreed gate; APP-023 is not qualified.

#### Combined workstream review — APP-017 through APP-022 (12 October 2026)

Reviewed committed-versus-observed presentation, manifest/file/receipt comparison, durable check outcomes, planner and worker admission, lifecycle restoration, explicitly requested reinstall, settings ownership/removal, GUI action/review state, and exact profile deletion/publication/recovery. The existing shared workflow, native guest transport and protected authority/publication paths remain the implementation; no parallel deletion or repair backend was added.

Source-backed findings fixed:

| Severity | Items | Trigger and consequence | Correction |
|---|---|---|---|
| High | APP-018/APP-019/APP-020 | A failed Verify cannot restore shutdown; a later reconciliation establishes the same payload mismatch and restores power, but never marks the negative outcome restored. Repair remains blocked waiting for a guest check that cannot pass without repair. | Shared negative-outcome persistence now records independently confirmed restoration during verification-only reconciliation, retaining mismatch, invalid preparation and recovery until explicit bound retirement. |
| High | APP-018/APP-019 | Verification accepts new VM/GPU/journal facts between admission and execution without comparing them with the reviewed facts. Reconciliation can also restart an externally stopped running guest when graphics passed. | Fresh mutable observations must match the admitted review before writes/effects. Both passed and failed checks refuse an unapproved restart; full host/GPU readback follows restoration. |
| Medium | APP-018 | Verify's worker checks final VM/GPU facts but does not reauthenticate the current host payload before releasing recovery. A same-version payload change can leave a successful result bound to obsolete files. | Reauthenticate the final current payload and require its digest to match the review before recording worker success; changes retain the recovery hold. |
| Medium | APP-018/APP-019 | Retrying interrupted Verify from running state with recorded final-off intent compares restored state against the interrupted running state and reports failure despite correct restoration. | Compare against the recorded restoration power, preserving other observed facts and pending state on uncertainty. |
| Medium | APP-017/APP-019 | Invalidating preparation after payload mismatch hides the known managed host compatibility baseline. | Host baseline applicability depends on matching identity, recovery and settings ownership, independently of guest payload eligibility. |
| Medium | APP-020/APP-021 | Re-enable after external adapter removal replaces the original owned settings with the current applied settings. Later removal cannot restore the original baseline. Conversely, released ownership with an external attachment can reuse an obsolete baseline. | Retain the original baseline while ownership remains; capture current settings when acquiring released ownership, including an already-present adapter. |
| Medium | APP-018/APP-021/APP-022 | Verify with no management journal records compatibility ownership despite performing no settings write, obstructing inactive-profile removal and misrepresenting the host baseline. | A Verify-created journal carries no settings ownership. Existing owned journals remain owned until reviewed removal. |
| Medium | APP-019/APP-020 | Ordinary payload updates discard historical successful rendering time before preparation, whereas explicit reinstall preserves it. | Both preserve historical success while the latest incomplete/negative check and invalid preparation prevent stale eligibility. |

Consolidation: [workflow](../src/workflow.rs) provides consumed, in-process Verify admission used by the [worker](../src/worker.rs); the authenticated manifest is retained without duplicate pre-effect discovery, while mutable facts are rechecked and the final payload is independently authenticated. Initial Verify and verification-only reconciliation share restored-failure recording. [GUI review state](../src/gui/live_state.rs) uses the same [inactive-profile validator](../src/enrollment.rs) as protected deletion, while retaining its refresh/conflict/review checks.

Regression source covers admission drift and payload-authentication count, no-owner Verify followed by removal/forget admission, failed reconciliation followed by explicit retirement/reinstall/removal/forget admission, successful check with external stop, failed ordinary update preserving history, owned/released baseline capture, and host comparison after payload invalidation. Existing interrupted-Verify retry and profile deletion boundary coverage remains applicable. These are isolated backend/controller tests, not live evidence.

Validation: targeted Rust formatting/check, documentation/link/task consistency and diff whitespace checks passed. No compilation, builds, regression execution, GUI render/launch, credentials, guest access, protected writes or VM/GPU effects ran. Source corrections are implemented; current automated gate, visual interaction and prepared APP-023 live acceptance remain outstanding. APP-024's Delete profile presentation remains outside these six items. No additional unresolved functional requirement was identified in this source review; runtime correctness remains unqualified until those checks run.

### APP-023 — Qualify checkpoint rollback detection and recovery

After the source fixes, prepare a separately authorized APP-009 extension naming the exact disposable VM, checkpoint, rollback effects, driver payload, credentials, expected final power and evidence paths. The user report is not authorization to create/apply checkpoints or repeat setup on accepted VMs.

Acceptance: non-hardware regressions pass at the agreed gate. Authorized live evidence shows pre-driver rollback followed by reopen, clear host/guest discrepancy or unknown state, fresh Verify failure, same-digest reinstall success, GPU removal, inactive pairing removal and successful fresh setup. Check another VM's state is preserved. Record denial/cancellation/recovery behavior within the explicitly approved sequence; stop on uncertain effects. Capture actual Verify failure details because its reported runtime failure has not yet been reproduced here. Source implementation alone does not close R6 or M2.

### APP-024 — Add Delete profile under VM More actions

User request (11 October 2026): add a Delete profile button under the selected VM's More actions so users can remove the whole application profile when they choose. Reuse APP-022's protected removal transition rather than adding another deletion backend. The whole profile includes that VM's saved settings, enrolled VM/GPU pairing and managed setup/verification history. Deleting the application profile does not delete the Hyper-V VM, its disks/checkpoints, guest files or shared application artifacts.

Acceptance:

- More actions exposes a clearly labeled Delete profile button for the selected VM, using the existing Slint design.
- A confirmation names the VM and describes all profile records to be removed. Cancel preserves the profile and has no effects; changing selection or profile revisions invalidates the confirmation.
- Clarification (12 October 2026): Delete profile removes only the app's VM profile. It never stages GPU removal, changes VM power or chains into APP-021. The existing APP-022 contract requires an inactive profile and an off VM; active/unknown support remains blocked with an actionable explanation. Pending recovery or uncertain effects stay blocked; deletion never clears holds to bypass recovery.
- Confirmed deletion uses APP-022 to remove the complete selected profile, preserves all other VM profiles and verifies the result. The VM remains visible in discovery as unconfigured and can start a new reviewed setup; deleted history cannot skip setup.
- Report deletion failures and retained records honestly. Add isolated controller tests for confirmation/cancellation, changed selection, active support, recovery blocks, partial failure, complete deletion and other-profile preservation. Real menu interaction remains a separately authorized live check.

Implementation: Done (source implemented on 12 October 2026). More actions and the separate confirmation now say Delete profile, using APP-022's existing reviewed protected record deletion. Confirmation names the VM and lists saved settings, VM/GPU authorization and managed setup/verification history; it explicitly excludes Hyper-V VM/disks/checkpoints, guest files, other profiles and shared artifacts. No new host/guest operation or GPU-removal handoff was added. Cancel, changed selection/revisions, inactive admission and recovery checks reuse the existing controller/worker contracts.

Verified worker completion drops only that profile's cached settings, authority and history, keeps its VM visible and preserves other profiles. Current inventory is marked for Refresh before new setup; a named completion dialog is retained rather than overwritten by an automatic refresh. Failure leaves cached records unknown, keeps existing recovery, blocks another deletion until readback and explains that some protected records may already have been removed. Protected readback, durable partial-deletion holds and other-profile preservation remain APP-022's backend responsibility.

Regression source: three new isolated controller tests cover complete deletion/unconfigured rediscovery with unchanged VM and other profiles, partial failure retaining unknown cache/recovery and active-support refusal without a draft, GPU or power change. Existing confirmation/cancellation, changed selection/profile revision, pending/unknown/absent-authority and backend deletion-boundary tests remain applicable.

Validation: targeted Rust formatting/check, documentation/link/task consistency and diff whitespace only. Compilation, regression execution, Slint compilation/rendering and actual menu/confirmation acceptance remain pending the agreed checkpoint. No GUI launch, guest access, credentials, protected writes or VM/GPU effects ran; no milestone closed.

### APP-025 — Keep Technical Details only where they add diagnostic value

User request (11 October 2026): review where Technical Details buttons should be added, kept or removed. A button is useful only when it provides meaningful debug/technical information beyond what the normal UI or dialog already shows. Audit the existing VM panel and dialogs through their Rust presentation inputs, keeping the existing design and shared diagnostics. The current operation dialog falls back to displaying its normal body when separate details are empty; that repetition is not useful technical output.

Acceptance:

- Review each existing button and relevant missing location. Keep or add Technical Details where available diagnostic context helps explain or troubleshoot the current operation; remove or hide it where there is only duplicated normal text or no additional information.
- Useful details can include the actual failing stage/component, error code, exact VM/GPU identity, expected-versus-observed values, payload/configuration revisions, recovery state and available evidence references. Show only facts actually collected, with their source/freshness and explicit unknowns where relevant; do not invent diagnostics or present old results as current.
- Remove the dialog-body fallback. Button visibility follows explicitly available additional diagnostic content, rather than always exposing a second copy of the summary. Normal messages retain the outcome and actionable next step even without a technical panel.
- Opening details does not launch new guest checks, request credentials, alter VM/GPU state or clear recovery. Reuse existing reporting/redaction; never expose passwords, tokens or other secrets.
- Clear obsolete details when the dialog, operation or selected VM changes. Retain readable, scrollable/copyable output and keyboard access using existing components.
- Add focused presentation/controller coverage for useful details, absent/duplicate-only content, operation/selection changes, failure context, stale observations and secret redaction. Record real visual/interaction validation separately at the agreed milestone.

Status: task added only; no buttons, diagnostic collection or product behavior are changed by this request. Build, automated tests and live UI checks are not run.

### APP-026 — Display the slider VRAM maximum as a whole GB

User request (11 October 2026): round the slider's displayed maximum VRAM to a whole GB for readability and appearance. This is presentation only; retain the real detected capacity and exact byte values used by the slider and backend. Use the existing decimal GB convention and round the displayed maximum to the nearest whole GB, with half values rounding up (for example, 8.4 GB displays 8 GB and 8.6 GB displays 9 GB).

Acceptance:

- The maximum endpoint label displays a whole GB with no decimal fraction and follows the selected GPU. When the slider is at its exact maximum, its selected-value label uses the same rounded presentation so the labels agree.
- The numeric maximum, maximum selection's byte request, saved intent, provider writes and independent readback remain exact. Never feed the rounded label back into allocation or capacity calculations, clamp/rewrite saved settings, or change backend allocation behavior.
- Keep precise values available in the existing exact-value review/technical information, identifying the simple label as rounded where needed. Intermediate slider steps and non-maximum selections retain their existing behavior.
- Unknown/unavailable capacity remains unavailable; rounding does not create a default capacity or make an unsupported range selectable.
- Cover below/at/above half-GB boundaries, exact whole-GB capacities, non-grid maximum endpoints, GPU switching and unavailable capacity. Assert that selecting the maximum still produces the original exact bytes. Visual acceptance remains a separately authorized milestone check.

Status: task added only; no slider, backend or saved configuration changes are included in this request. Build, automated tests and live UI checks are not run.

### APP-027 — Retire a failed verification-only Apply without blocking repair

Priority: High. Source-review finding (12 October 2026); added at the user's request, not implemented.

When the current preparation digest matches but successful check evidence is absent/invalid, Apply can require verification without host changes or preparation. It records a pending verification-only journal. A proven guest mismatch invalidates preparation, but the Apply receipt requires publication: the existing negative-completion branch only admits standalone Verify receipts, and the preparation-not-started branch excludes verification-only journals. Further Apply/reinstall/removal rejects pending recovery; repeated reconciliation cannot fix mismatched guest files.

Acceptance: provide a narrowly bound, explicitly reconciled negative completion for this failed Apply case only after independent proof of unchanged host/GPU facts, committed revision and safely completed reviewed lifecycle. Preserve negative check evidence and preparation invalidation; cancel the failed operation's intent publication. Never fabricate success, replay effects, clear uncertain holds or automatically repair files. Add isolated regressions for matching-digest/missing-success Apply followed by payload mismatch, explicit retirement and reviewed reinstall/removal, plus changed/unavailable facts, uncertain outcomes, refused restoration and interrupted persistence. Reuse existing retirement/receipt/worker machinery.

Implementation: Ready. Validation: Not run. Builds/tests and live effects require their existing checkpoint/authorization; this card alone does not authorize execution.

### APP-028 — Preserve the guest failure when restoration-proof persistence fails

Priority: Medium. Source-review finding (12 October 2026); source implemented during the user-authorized APP-033 review.

`record_restored_check_failure` can fail while saving or independently reading back the restored-negative proof. Verify and verification-only reconciliation propagate that failure before combining their operation outcomes, hiding the original payload/graphics failure behind the persistence error.

Acceptance: retain both the original guest failure and restoration-proof save/readback failure with actionable, secret-free context. Preserve pending recovery and negative evidence; failed proof persistence must not certify restoration or grant negative retirement. Add isolated Verify/reconciliation regressions with a terminal payload or graphics failure followed by successful restoration and denied save or mismatching/unavailable readback. Preserve the existing primary/restoration error combinations and reporting boundaries.

Implementation: Done (source implemented, 12 October 2026). Both Verify and verification-only reconciliation retain the primary typed guest failure when restoration-proof save/readback fails. In-memory unconfirmed proof is reset; pending recovery and negative evidence remain. Shared public/technical reporting presents both contexts without raw adapter output. One isolated regression exercises twelve combinations of Verify/reconciliation, payload/graphics failure and denied save/unavailable or mismatching readback.

Validation: targeted formatting and documentation/diff checks only; compilation and regression execution pending the agreed automated gate. No live effects ran.

### APP-029 — Skip unchanged compatibility writes during enabled Apply

User request (12 October 2026): implement the workflow review's easy win. Write compatibility settings only when the freshly admitted initial settings differ from the reviewed desired settings, matching the planner's Settings action. Retain the applied-settings journal, ownership acquisition/release, original baseline and independent final readback. Required writes and their failure/recovery behavior remain intact.

Implementation: Done (source implemented). Enabled Apply now guards the settings backend call with the same comparison used by its preview. Test-only write counting and three focused regressions cover VRAM-only updates and same-digest reinstall with matching settings, re-enable after ownership release with a matching external baseline, and a required write failure retaining recovery without attachment or graphics. No authority, lifecycle, payload, allocation or publication policy changed.

Validation: targeted Rust formatting/check, documentation checks and diff whitespace checks only. Compilation, automated regressions, lint/build/API documentation and live acceptance remain pending the agreed checkpoint. No GUI launch, credentials, guest access, protected writes or VM/GPU effects ran. APP-027 remains unimplemented; APP-028 was subsequently source implemented in APP-033.

### APP-030 - Explain selected-VM action availability before activation

Priority: High. The [UI audit](BASELINE.md#ui-interaction-audit---12-october-2026) found More actions uses broad worker readiness while the existing controllers enforce different draft, pairing, attachment, power and recovery prerequisites. An enrolled GPU replacement is also selectable although that workflow is unavailable. Reuse existing eligibility and review contracts; presentation never grants effect authority.

Acceptance:

- For the selected VM, explain whether Verify, Repair / Reinstall, Remove GPU support and profile removal are available. An unavailable action has a visible specific reason and a reachable next step, including review/apply/discard of pending edits, Refresh or Review recovery as applicable. Do not hide the only route to inspect recovery behind a disabled operation.
- Cover no selection, unconfigured/unsupported VM, active versus inactive support, running/off power, missing attachment, unavailable observations, unreadable/conflicting records, pending edits and recovery. Avoid presenting a stale observation as permission; a fresh protected review remains required.
- Explain the enrolled GPU restriction before an unsupported replacement can become an apparently actionable draft. Preserve saved resource requests and stable GPU identity; do not implement replacement, auto-discard edits or silently choose another GPU.
- Refresh, VM selection and operation completion update reasons together with availability. Preserve drafts and unrelated VM profiles. Selection/revision changes still invalidate confirmation.
- Add focused controller/presentation regressions for these reasons, permitted recovery navigation and changed selection/observation. Actual mouse/keyboard interaction is checked under APP-009 at the agreed checkpoint.

Implementation: Done (source implemented at the user's request on 12 October 2026). The read-only [action presenter](../src/gui/actions.rs) reuses editor/draft/removal/profile eligibility and shared Verify power/assignment checks. More actions shows individual unavailable reasons alongside scrollable action choices, Refresh, draft review/discard and a separate Review recovery route. Busy/readiness state disables operation requests with an explanation; action callbacks recheck availability before changing drafts. Reasons refresh on selection, edits, observation and operation completion. Unknown managed history requests Refresh without inventing a hold. Inactive profile cleanup remains available without the physical GPU where the existing profile contract permits it.

The GPU selector is locked for an authorized VM, with a visible explanation. Identity-based lookup follows enumeration changes; a missing enrolled device stays unavailable instead of choosing another GPU. The edit callback rejects an attempted replacement without rewriting resource requests. Unconfigured setup retains GPU selection. Fresh protected reviews, revision bindings, effects and recovery authority are unchanged. APP-024 now supplies app-record-only Delete profile presentation; it never stages GPU removal.

Regression source: nine new isolated state tests cover active/inactive and running/off paths, missing attachment/device, invalid draft preservation and unrelated profiles, selection/transitional/unsupported states, failed refresh and unreadable intent, unavailable history, host-wide recovery, unconfigured setup, GPU reordering/replacement rejection, unsupported preparation and per-VM incomplete observations. The existing pending-journal regression additionally checks availability reasons and recovery navigation without a save record.

Validation: targeted Rust formatting/check, documentation/link/task consistency and diff whitespace only. Compilation, regression execution and Slint compilation/render/keyboard/mouse acceptance remain pending the agreed checkpoint. No GUI launch, credentials, guest access, protected write or VM/GPU effect ran. No milestone is closed.

### APP-031 - Make action labels and review language match actual effects

Priority: Medium. Verify now checks current guest payload and receipt as well as graphics, but its controls still label it as a graphics check. Shared shutdown consent refers to restoration even when Apply ends off. Other normal dialogs lead with internal authority/worker terminology rather than the user's task.

Acceptance:

- Use consistent, plain action names across More actions, reviews, progress and completion. Verify explains both current driver-file checks and hardware rendering; Repair / Reinstall explains driver preparation; removal explains what remains. Preserve the distinct reviews and APP-024's deletion scope.
- Each review leads with the VM name, requested changes, actual required power transitions, expected final power and relevant credentials/time warning. Shutdown consent describes the particular plan: ordinary Apply/setup ends off; Verify restores the reviewed initial state where supported. Never promise restoration or success before independent confirmation.
- Explain first setup's authorization review followed by the GPU-effect review as understandable steps. Cancellation retains drafts and grants no effects; credential cancellation is described without implying setup has begun or completed.
- Normal status/failure copy tells the user what happened and what to do next, distinguishing current observation, saved intent, pending edits and historical verification. Avoid a generic Ready status implying an unavailable action can run. Retain uncertainty and necessary warnings without burying them in diagnostics.
- Move implementation identifiers/details to existing diagnostic output where useful under APP-025. Keep truthful VRAM limitations; do not claim reserved memory, current guest health from history, or qualified hardware behaviour.
- Cover operation-specific labels, power consent, missing/stale facts and failure/next-step presentation with focused regressions. Check readability and actual dialogs separately under APP-009.

Implementation: Done (source implemented at the user's request on 12 October 2026). More actions, review, confirmation, progress and result now consistently say Verify GPU setup, covering current driver files/setup record and device health/rendering. The [wording presenter](../src/gui/wording.rs) derives lifecycle/consent from reviewed power facts; Apply describes its final state rather than initial-power restoration, Verify describes an off VM's temporary start/graceful shutdown or a running VM remaining running, and recovery uses the bound expected final state without promising an unconfirmed outcome. A changed consent label clears checkbox approval. Backend consent requirements and effects are unchanged.

Apply reviews lead with the VM, retain exact changes/steps and distinguish verification-only work from required driver preparation. Initial permission explains the two separate setup reviews. Verify IDs, driver/digest and helper details move into distinct Technical Details while normal copy retains the checks, power, guest credentials/time and unknown/history limits. Check recorded outcome replaces the recovery confirmation/progress wording; record identifiers remain in details. The footer distinguishes pending edits, blocked/setup/refresh/recovery conditions and whether an operation is running rather than displaying generic Ready.

Guest progress describes the actual requested operation; saved credential management never claims setup or a guest connection. The local prompt refers to approved guest steps instead of assuming every prompt starts setup. Cancellation preserves edits/holds and does not imply setup or verification completion. Verify result/failure names the VM and retains next steps/uncertainty. Delete profile remains app-record-only with no GPU or power effects. APP-025 still owns broader technical-button visibility/deduplication.

Regression source: six new isolated presentation tests cover Apply final-power consent, Verify off/running/unknown power and historical-health limits, technical identity separation, verification-only Apply without preparation, operation-specific guest progress, credential cancellation and recovery unknown-power wording. The existing local-prompt regression now asserts reviewed-purpose wording without an assumed setup claim.

Validation: targeted Rust formatting/check, documentation/link/task consistency and diff whitespace only. Compilation, regression execution, Slint compilation/rendering and real interaction remain pending the agreed checkpoint. No GUI launch, credentials, guest access, protected writes or VM/GPU effects ran; no milestone closed.

### APP-032 - Make dialogs usable with keyboard and constrained layouts

Priority: Medium. Source review identifies default focus on Technical Details and dense horizontal action rows; keyboard trapping/return, Escape behaviour and actual clipping remain untested interaction risks. Retain the existing design/components, and reproduce runtime failures before describing them as confirmed defects.

Acceptance:

- Initial focus reaches an appropriate safe action/content rather than always opening on Technical Details. Tab and Shift+Tab reach the complete dialog controls without activating the obscured underlying page; closing returns focus to the initiating control or a safe remaining control if it disappeared.
- Keyboard and pointer paths offer equivalent Stay, Cancel, Close, Review and explicit confirmation choices. Escape uses safe cancellation/close for idle reviews, retaining drafts and clearing approval; it never confirms effects or dismisses a running operation as though it stopped. Enter cannot bypass consent, disabled actions or explicit destructive confirmation.
- Check More actions, switch/exit, Apply, Verify, credentials handoff, profile removal, error, save-only and recovery dialogs. Buttons and consent remain readable/reachable at the supported minimum window size and recorded Windows scaling, with long VM names/messages and both themes.
- Keep review/diagnostic/progress text scrollable. User scrolling to earlier progress is retained until they return to the end; diagnostics and consent do not carry obsolete content/approval into a changed operation or VM. Coordinate hidden diagnostic-button focus with APP-025.
- Add focused controller coverage for cancellation, confirmation guards and operation changes; retain existing component keyboard behaviour where adequate. Real keyboard/focus/layout evidence belongs to the separately authorized APP-009 sequence, not a retired mock viewer.

Implementation: Ready. Validation: Not run. No current keyboard/layout failure was reproduced and no UI source changed.

### APP-033 - Review current source and fix bounded logic gaps

User request (12 October 2026): review accumulated changes for gaps and logic errors, fix small/medium findings, and create tasks for larger findings. Review spans GUI state/presentation, workflow planning/apply/verification/reconciliation, guest parity/transfer, protected worker completion, configuration publication, enrollment/removal and IPC/trust boundaries. [Findings and scope](evidence/full-source-review.md) record current evidence and exclusions.

Implementation: Done (source review and bounded fixes). Fix the binary/library diagnostic visibility mismatch; complete APP-028; preserve the original operation failure when GUI completion reads also fail; prefer newer adverse guest check facts over historical success in the compact VM summary; and request CLI reconciliation credentials from the durable graphics requirement. Existing receipt, enrollment, consent, publication and no-replay boundaries remain. Added regressions are isolated; existing graphics-requirement coverage is reused for the CLI change.

Validation: targeted Rust formatting/check, local documentation consistency and diff whitespace only. Compilation, tests, lint/build/API docs, rendering and real interaction are pending the agreed checkpoint. No GUI launch, credentials, protected writes, guest access or VM/GPU effects ran. Larger findings remain APP-027 and APP-034; no milestone closed.

### APP-034 - Recheck admitted Apply facts before delayed writes and power restoration

Priority: High. Confirmed source gap in `workflow/apply.rs`, found during APP-033. Admission checks initial VM/GPU/journal facts, but driver preparation and shutdown may take time. Enabled Apply compares its compatibility-write decision with the old initial settings, then writes the full desired settings without checking for an external change. Removal already refuses such a change. Final readback detects some drift only after overwriting it. Also, failed verification-only Apply uses `restore_guest_power` directly without the full host/GPU checks used by standalone Verify and reconciliation. Changed assignments/settings can therefore still be followed by a shutdown based on the earlier admission.

Acceptance:

- Reuse the existing admitted decision, protected worker/lock and backend interfaces; introduce no alternate effect path or new authority.
- Define and independently check stage-specific expected host/GPU/journal facts after slow steps and before delayed compatibility/resource/assignment writes and guest power changes. Account for effects this operation itself performed; reject external drift or unavailable facts before overwriting or changing power. Preserve the original compatibility baseline and setting ownership.
- Give verification-only Apply the same changed-fact and external-stop refusal as Verify/reconciliation. Restore only the reviewed supported final power when independently justified; retain primary and restoration failures and pending recovery on uncertainty.
- Keep previews accurate and ordinary operations guest-free. Never replay preparation, attachment or allocation to obtain proof or release recovery.
- Add isolated regression/failure coverage for external settings, GPU/driver, assignment/resource and power changes during preparation/checks, unavailable inspections, successful expected transitions, refused restoration, and interrupted proof persistence. Inspect native adapter safeguards and effect order together; source-only changes are not live acceptance.

Implementation: Ready. Validation: Not run. This review creates the task; the broader change requires a separate requested implementation and the existing automated/live gates.
