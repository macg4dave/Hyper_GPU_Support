# Source baseline

Assessment: 11 October 2026, static source inspection during the product Q&A. APP-001 source removal subsequently completed on the same date, with static reference/diff review only. No build, automated product test, GUI launch, guest operation, or hardware qualification ran. This page distinguishes current code from the requested workflow; [backlog](BACKLOG.md) owns fixes.

## Source and requested changes

| Area | Current source finding | Required direction |
|---|---|---|
| Dispatch and GUI | [Main](../src/main.rs) selects only live GUI startup; [GUI host](../src/gui/mod.rs) retains shared model/dialog helpers after mock/snapshot removal | APP-001 source implemented; compilation and UI acceptance pending; APP-002 controls remain incomplete |
| Discovery and review | [Live controller](../src/gui/live.rs) contains protected inventory, draft, review, Apply, Verify, and recovery paths | Keep existing design and complete the simple reviewed workflow under APP-002/APP-003 |
| VRAM | [Memory control](../src/gui/memory.rs) converts byte requests and uses provider-derived lower bounds with matched physical capacity | Enforce the agreed 1 GB displayed minimum and card-dependent upper bound; verify provider mapping under APP-002 |
| Initial and later effects | [Workflow](../src/workflow.rs) supports preparation, GPU/resource effects, guest verification, and recorded power restoration | Distinguish initial setup from ordinary toggle/VRAM changes under APP-004/APP-006; no repeated initial verification during ordinary changes |
| Final power | Shared workflow retains original/recorded restore power | Completed initial setup must instead end off; ordinary-operation final power policy remains to be clarified |
| Credentials | Existing [credential module](../src/credentials.rs) and live controller supply local prompting/storage paths | Audit and add the friendly named-VM explanation before the prompt under APP-005 |
| State and recovery | [Model](../src/model.rs), [store](../src/configuration_store.rs), [worker](../src/worker.rs), and [enrollment](../src/enrollment.rs) support intent, publication, and protected operation state | Preserve boundaries and audit concrete workflow gaps under APP-007 |
| Hardware verification | [Guest module](../src/guest.rs) and [D3D11 probe](../src/bin/d3d11-probe.rs) provide hardware checks | Reuse for initial setup; real results require APP-009 authorization |
| Quality gate and release | Previous development tooling was deleted; existing inline tests and cached outputs do not establish a current pass | APP-008 prepares/runs the milestone gate; APP-009 qualifies; APP-010 packages |

Some table entries identify existing source paths, not a complete audit of every failure case. Inspect relevant code before implementation; source presence is not runtime acceptance.

## Open facts

Exact supported Windows builds, GPU/driver combination, live provider behavior, concurrent workloads/duration, package readiness, and current build health remain unestablished by this rewrite. Clarify ordinary-operation final power behavior in APP-006.

Existing advanced resource and CLI recovery/import capabilities are implementation context. They do not expand the agreed simple GUI requirements. Legacy task status and past milestone claims are not imported into this plan.

## Updating this page

Record actual source changes separately from executed checks. Link validation to its revision, environment, and evidence. Do not claim a slider reserves VRAM, that an operation succeeded from a simulation, or that a successful build proves GPU sharing.
