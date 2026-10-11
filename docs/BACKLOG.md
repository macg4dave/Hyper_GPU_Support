# Product backlog

This queue replaces the previous task plan. [Roadmap](ROADMAP.md) owns requirements and milestones; [engineering](ENGINEERING.md#status-and-evidence) owns status conventions. Existing code is the starting point, not a request to rebuild the backend.

## Next implementation action

**APP-005 — Explain guest credentials before the local prompt (R4).** Ready after APP-004 source implementation. Name the selected VM and explain its Windows sign-in before prompting; preserve optional vault opt-in and cancellation. Validation waits for APP-008; live work requires separate authorization.

## Task register

| ID | Milestone | Task | Depends on | Implementation | Validation |
|---|---|---|---|---|---|
| APP-001 | M1 | Remove retired mock GUI tooling | None | Done | Pending milestone gate |
| APP-002 | M1 | Match the simple toggle and VRAM control | APP-001 | Done | Pending milestone gate |
| APP-003 | M1 | Complete review and confirmation | APP-002 | Done (R2 source; requested ahead of APP-002) | Pending milestone gate |
| APP-004 | M1 | Complete initial setup, verification, and final shutdown | APP-003 | Done | Pending milestone gate |
| APP-005 | M1 | Explain guest credentials before the local prompt | APP-004 | Ready | Not run |
| APP-006 | M1 | Separate ordinary changes from initial setup | APP-005 | Blocked | Not run |
| APP-007 | M1 | Complete saved state, progress, and failure handling | APP-006 | Blocked | Not run |
| APP-008 | M1 | Prepare and run the milestone's non-hardware gate | APP-007 | Blocked | Not run |
| APP-009 | M2 | Qualify the real GUI and two-VM workflow | APP-008 | Blocked | Not run |
| APP-010 | M3 | Package and document the qualified product | APP-009 | Blocked | Not run |

## Implementation tasks

### APP-001 — Remove retired mock GUI tooling

Remove the AI-era mock and snapshot GUI entrypoints, dedicated modules/scenarios, scenario controls, and dedicated mock-only checks. Trace dependencies through dispatch, GUI host, controller, and Slint bindings before deleting anything shared with the live application. Update help and source comments affected by the removal. No replacement simulation or preview framework.

Acceptance: production GUI/CLI and fixed internal entrypoints remain; removed modes are no longer advertised or accepted as supported workflows; shared code is preserved; no dedicated mock maintenance remains. Add no new mock scenarios. Meaningful backend tests using isolated test doubles remain valid. Validation waits for APP-008.

Implementation (11 October 2026): removed mock/snapshot dispatch, dedicated modules, timer scenarios, sample defaults, rehearsal controller/state branches, and dedicated mock checks. Shared live model/dialog helpers and protected operation paths remain. Added argument-free GUI dispatch regression coverage; retained live draft, stale-observation, publication, and recovery tests. Static reference/diff review and `git diff --check` completed; no formatting tool, build, test, GUI launch, or host/guest effect ran. Compilation and real UI acceptance remain pending.

### APP-002 — Match the simple toggle and VRAM control

Keep the existing design. Provide the GPU support toggle and card-dependent VRAM amount, from 1 GB to detected physical capacity. Only the toggle disables GPU support. Inspect current slider bounds/conversion and the native provider before changing mapping. Retain schema compatibility; do not rewrite existing settings silently.

Acceptance: range follows the selected card, minimum is 1 GB, saved/draft values remain distinct, and unsupported or unknown capacity produces a truthful unavailable state. Displayed units and provider writes agree for supported hardware. No balancing, new allocation policy, or full-capacity warning feature. Assess existing advanced controls against the simple product scope without redesigning the GUI.

Implementation (11 October 2026): slider starts at 1 decimal GB and ends at matched physical capacity, retaining exact non-grid endpoint bytes. Provider admission remains separate: unsupported points are rejected, and ranges with no supported selection or unknown/sub-1 GB capacity are unavailable. Existing requests below 1 GB are labeled exactly with an explanation, not silently rewritten. A single-value 1 GB range uses an explicit selection button. The native writer passes the same byte integers directly to Hyper-V's VRAM properties with fresh limits and readback; no capacity ratio or backend mapping change was introduced. Removed Compute/Encode/Decode editors from the GUI; existing schema-3 requests remain intact and visible in review. Advanced Allocation retains exact VRAM fields for compatibility/correction. Added boundary and controller tests covering saved schemas, other VMs/categories, toggle independence, and rejected/stale edits. Static source/reference review and `git diff --check` only; no formatter, build, tests, render, GUI launch, or live effects ran. Supported-hardware behavior still awaits APP-009 qualification; M1 remains open.

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

### APP-006 — Separate ordinary changes from initial setup

Later enable/disable and VRAM updates use the same review and confirmation but do not repeat full preparation or initial rendering verification. Retain fresh authorization, provider validation, attachment/settings readback, and recovery.

Acceptance: ordinary changes avoid unnecessary guest access, preparation, and graphics checks; their real outcomes are reported. Required power transitions are managed by the app and shown in review. Clarify the exact final power policy for ordinary changes before altering it; completed initial setup must end off. Changed drivers or invalid preparation must produce an explicit requirement for setup/update rather than silently hide expensive work in a toggle.

### APP-007 — Complete saved state, progress, and failure handling

Audit current per-VM settings, refresh/draft conflicts, operation progress, protected publication, and recovery against the simplified workflow. Preserve the protected runner, typed worker modes, authenticated IPC, trust checks, locks, and durable receipts.

Acceptance: saved settings survive reopen; state labels distinguish pending, observed, applied, and initial verification; progress stays responsive and reports actual stages; another VM is unaffected. Missing terminal outcomes stay uncertain. A completed-but-unsaved operation offers a bound save-only retry without replaying GPU effects. Address concrete trust/recovery gaps found in these paths.

## Validation and delivery tasks

### APP-008 — Prepare and run the milestone's non-hardware gate

Prepare a narrow non-elevated gate against current files; do not restore deleted tooling wholesale. Inspect selected tests for effects. At M1 completion run appropriate documentation, formatting, non-hardware tests, lint, product binary, and rustdoc checks described in [testing](TESTING.md#planned-automated-gate).

Acceptance: intended checks pass or failures keep M1 open; evidence states exact commands, revision, outcomes, and exclusions. Tests cover initial-versus-ordinary operations, final-off setup, credentials explanation, review binding, slider bounds, and meaningful failure boundaries. No GUI launch, VM access, privileged setup, or GPU probe in the automated gate.

### APP-009 — Qualify the real GUI and two-VM workflow

Prepare the exact environment and run sheet before asking for live authorization. Select host/guest builds, NVIDIA GPU/driver, two existing Generation 2 Windows 11 VMs, workloads, observation duration, effects, final power states, evidence paths, and stop conditions. Then run only the authorized sequence.

Acceptance: initial setup verifies each VM and leaves it off; later toggles and VRAM changes follow review without repeated initial verification; user-facing credential explanations and progress work. Saved settings survive reopen. Two guests produce checked hardware rendering concurrently, and operations on one preserve the other's functioning. No mock preview, automatic retry, host restart, or unrequested post-run investigation. Interrupted results keep M2 open.

### APP-010 — Package and document the qualified product

Prepare the required fixed package artifacts, hashes, notices, setup instructions, supported environment, limitations, and troubleshooting. Run final automated checks and obtain separate authorization for clean-install/upgrade trials.

Acceptance: package is complete, artifact trust and enrollment/settings preservation are checked, actual trials pass, and support claims match recorded evidence. Publishing or pushing requires a separate request.
