# Product backlog

This queue replaces the previous task plan. [Roadmap](ROADMAP.md) owns requirements and milestones; [engineering](ENGINEERING.md#status-and-evidence) owns status conventions. Existing code is the starting point, not a request to rebuild the backend.

## Next implementation action

**APP-009 — Prepare real GUI and two-VM qualification (M2).** M1 is closed with [gate evidence](evidence/m1-closure.md). Prepare the target/effect/evidence sequence before requesting live authorization; no live sequence has run.

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
| APP-011 | M2 | Correct physical VRAM and provider mapping | APP-008 | Ready | Not run |
| APP-012 | M2 | Make transfer progress human readable | APP-008 | In progress (source implemented; visual acceptance pending) | Pending milestone gate and live display check |
| APP-013 | M3 | Measure and improve guest transfer throughput | APP-008 | In progress (source implemented; measured acceptance pending) | Pending milestone gate and live throughput/memory comparison |

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

The live GUI converts raw provider values into GB and backend admission compares physical byte requests directly against those values. The user confirmed that the resulting 1 GB ceiling claim is false. Remove that claim now; mapping implementation is deferred at the user's request. Allocation qualification remains pending.

Establish and implement the correct physical-memory/provider contract in the shared Rust path and GUI. Preserve existing exact saved requests; do not invent a capacity ratio, reservation, or enforcement guarantee. Review source, provider semantics and independent readback before authorizing writes. Keep fresh protected admission and recovery boundaries.

Acceptance: the slider covers 1 GB through detected physical capacity with truthful values; GUI/backend agree on conversions and admission; regression tests cover the observed raw-metadata case, boundaries and invalid inputs; authorized live allocation and independent readback pass. Raw metadata must never be presented as a physical 1 GB ceiling.

Implementation: backlog only. False ceiling wording removed during APP-009; underlying mapping remains unchanged and unqualified.


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
