# Source baseline

Assessment: 11 October 2026, static source inspection during the product Q&A. APP-001 through APP-006 source work subsequently completed on the same date, with static reference/diff review only. No build, automated product test, render, GUI launch, credential prompt, guest operation, or hardware qualification ran. This page distinguishes current code from the requested workflow; [backlog](BACKLOG.md) owns fixes.

The subsequent [current-source review](evidence/m1-source-review.md) fixed the disconnected setup button, uncertain-start reconciliation restoration, pending-journal editor admission, concealed controller state errors, and UI-thread worker availability checks. Regression additions remain unrun. APP-007 is still open; follow-up review precedes APP-008.

## Source and requested changes

| Area | Current source finding | Required direction |
|---|---|---|
| Dispatch and GUI | [Main](../src/main.rs) selects only live GUI startup; [GUI host](../src/gui/mod.rs) retains shared model/dialog helpers after mock/snapshot removal | APP-001/APP-002 source implemented; compilation and UI acceptance pending |
| Discovery and review | [Live controller](../src/gui/live.rs) and [review state](../src/gui/live_state.rs) bind fresh review to VM/draft/committed revision; first Apply reviews, second confirms, Cancel retains edits; initial setup is labeled with its ordered final shutdown | APP-003/APP-004 source implemented, validation pending; APP-006 ordinary changes now skip guest work and finish off |
| VRAM | [Memory control](../src/gui/memory.rs) displays 1 decimal GB to matched physical capacity, preserves exact requests/endpoints, and separately rejects unsupported driver values; native writer passes byte integers without rescaling | APP-002 source implemented; compilation, UI acceptance, and supported-hardware behavior await APP-008/APP-009 |
| Initial and later effects | [Workflow](../src/workflow.rs) identifies initial setup from preparation/graphics receipts and includes guest verification before final shutdown | APP-004 source implemented; APP-006 ordinary changes skip preparation/graphics; invalid preparation is explicitly reviewed as setup/update |
| Final power | Initial setup requires verified final-off state; [protected receipt](../src/configuration_store.rs) persists reviewed final power for reconciliation/publication independently of the starting observation | APP-004 validation pending; APP-006 ordinary changes also require final-off readback; receipts retain the reviewed graphics requirement |
| Credentials | [Credential module](../src/credentials.rs) blocks on a named-VM explanation before native input; Remember stays opt-in; the live controller distinguishes cancellation before worker submission and preserves drafts/recovery | APP-005 source implemented; automated checks and real dialog/interaction acceptance await APP-008/APP-009 |
| State and recovery | [Model](../src/model.rs), [store](../src/configuration_store.rs), [worker](../src/worker.rs), and [enrollment](../src/enrollment.rs) support intent, publication, and protected operation state | Preserve boundaries and audit concrete workflow gaps under APP-007 |
| Hardware verification | [Guest module](../src/guest.rs) and [D3D11 probe](../src/bin/d3d11-probe.rs) provide hardware checks | Reuse for initial setup; real results require APP-009 authorization |
| Quality gate and release | Previous development tooling was deleted; existing inline tests and cached outputs do not establish a current pass | APP-008 prepares/runs the milestone gate; APP-009 qualifies; APP-010 packages |

Some table entries identify existing source paths, not a complete audit of every failure case. Inspect relevant code before implementation; source presence is not runtime acceptance.

## Open facts

Exact supported Windows builds, GPU/driver combination, live provider behavior, concurrent workloads/duration, package readiness, and current build health remain unestablished by this rewrite.

Existing advanced resource and CLI recovery/import capabilities are implementation context. They do not expand the agreed simple GUI requirements. Legacy task status and past milestone claims are not imported into this plan.

## Updating this page

Record actual source changes separately from executed checks. Link validation to its revision, environment, and evidence. Do not claim a slider reserves VRAM, that an operation succeeded from a simulation, or that a successful build proves GPU sharing.
