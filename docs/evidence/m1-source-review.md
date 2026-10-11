# APP-001 through APP-008: current-source review

Reviewed 11 October 2026 at `a62c174`, including the uncommitted APP-005 implementation and the fixes below. The user requested review and fixes to current source, followed by another review after APP-006/APP-007. This is static source assessment, not M1 acceptance or permission for live operations.

## Findings and fixes

All five findings are P2 functional or failure-handling defects. Source fixes are complete; automated and live validation remain pending.

| Finding | Trigger and consequence | Source fix and coverage |
|---|---|---|
| First-time setup action was disconnected | The setup card emitted `enroll`, which had no live-controller handler; clicking it reported an unavailable action instead of reviewing setup | [Setup card](../../src/gui/ui/details.slint) now uses the existing stage-enabled/review action and respects eligibility/recovery. Existing new-VM controller coverage preserves separate proposal/enrollment/effect authority; real button interaction remains untested |
| Reconciliation skipped restoration after an uncertain start | A start request could change the VM to running and then return an error. The early return bypassed safe restoration to the recorded off state | [Workflow](../../src/workflow.rs) now includes start in the checked stage. Fresh identity/settings/GPU/payload checks still precede restoration. New isolated regressions cover successful restoration, refused shutdown with both failures retained, and changed facts blocking restoration; no preparation, assignment, or allocation replay |
| Pending journals could reach new reviews without a save record | The selected VM's journal showed recovery, but controller eligibility checked only loaded host-wide records. Alternate actions could request a new plan; the worker rejected it later | [Controller state](../../src/gui/live_state.rs) now rejects pending/inconsistent selected-VM managed state before drafting/review. New regression covers management and verification journals without a save record, retained drafts, rejected approval, and a later completed journal readback |
| Completion dialogs could conceal local state-read failure | `with_state` displayed an error, but its caller continued and replaced it with a review/success dialog after a failed recovery read or state update | [Live controller](../../src/gui/live.rs) now returns success/failure from state updates, disables execution on failure, and stops affected enrollment/review/Apply/Verify/save-only/reconciliation callbacks. Failed Apply marks readback required and clears approval before attempting recovery reads. UI failure interaction remains untested |
| Worker artifact hashing ran on the UI event loop | Refresh and review completions called worker availability, which authenticates/hashes installed executables; slow disk or scanning could freeze interaction | [Live controller](../../src/gui/live.rs) now obtains availability inside the existing background jobs and returns the observation with their results. The protected worker still independently authenticates artifacts and revalidates admission. Responsiveness awaits real UI validation |

## Review scope and remaining work

| Task | Current-source assessment |
|---|---|
| APP-001 | Traced argument-free GUI dispatch, fixed internal modes, build inputs, live modules, and removed mock/snapshot references. No retired entrypoint found in current product source |
| APP-002 | Reviewed physical capacity matching, exact decimal-byte conversion/endpoints, provider admission, retained saved triples/categories, selected-GPU identity, and native allocation/readback. No additional source fix identified; provider semantics and hardware behavior remain unqualified |
| APP-003 | Reviewed target/draft/revision binding, cancellation, ordered lifecycle disclosure, stale admission, and execution readiness. The callback failure and background-availability fixes above apply |
| APP-004 | Reviewed setup/enrollment separation, preparation receipts, graphics check, independently confirmed final-off state, other-VM preservation, durable final-power intent, publication, and no-replay reconciliation. The setup button and uncertain-start fixes above apply |
| APP-005 | Reviewed explanation/input order, scoped vault reuse, unchecked remembering, cancellation before worker submission, secret-free diagnostics, and retained drafts/holds. Existing APP-005 regressions remain unrun |
| APP-006 | Unimplemented. Ordinary enabled changes still repeat graphics verification and can refresh stale preparation. Clarify final-power policy and implement the existing card; this review does not choose that policy |
| APP-007 | Unimplemented. Reviewed current saved/draft/observed state, refresh conflicts, progress delivery, worker process/SID/session/artifact binding, effect admission, audit/receipt ordering, save-only publication, and recovery. Fixed the concrete current-source findings above; the complete task audit remains required |
| APP-008 | Gate unprepared/unrun. Checked declared commands/toolchain, retained test modules and compile-time inputs. Inspected relevant non-hardware tests: isolated doubles, temporary filesystem fixtures, user-owned pipe fixtures, in-memory CIM objects, and fixed malformed-input/transport subprocesses. Gate preparation must finish its effect review and record actual command outcomes after APP-006/APP-007 and the follow-up source review |

No new standalone implementation task was created: these findings fit the existing APP scope. Larger findings needing separate tasks must be presented to the user before their implementation.

## Checks and limits

Completed targeted source/caller/diff/reference inspection and `git diff --check`. Preserved the pre-existing APP-005 working-tree changes. No formatter, compilation, automated test, rendering, GUI launch, credential dialog, installed IPC probe, guest connection, privileged setup, protected write, or GPU/VM effect was run. Regression additions are source only. This review cannot establish runtime correctness, trusted installation behavior, hardware sharing, or visual acceptance. M1 remains open; repeat the review after APP-006/APP-007 before APP-008 execution.
