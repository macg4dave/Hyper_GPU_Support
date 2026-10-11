# APP-001 through APP-008: current-source review

Reviewed 11 October 2026 at `a62c174`, including the uncommitted APP-005 implementation and the fixes below. The user requested review and fixes to current source, followed by another review after APP-006/APP-007. This is static source assessment, not M1 acceptance or permission for live operations.

## Findings and fixes

All five findings are P2 functional or failure-handling defects. Source fixes and follow-up review are complete. The [M1 gate](m1-closure.md) passed automated validation; live validation remains pending. The original findings below retain their review-time limits.

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
| APP-006 | Follow-up at `4c293ec`: ordinary changes with matching completed preparation skip credentials, preparation and graphics checks; fresh host readback and confirmed Off remain mandatory. Invalid preparation/payload is reviewed as setup/update. Receipts persist final-power and graphics requirements; ordinary reconciliation avoids guest access. Isolated lifecycle/failure/two-VM regressions cover these paths |
| APP-007 | Follow-up at `4c293ec`: traced saved/draft/observed/applied/historical graphics labels, refresh/conflict latching, background protected reads, actual progress events, terminal operation binding, authenticated IPC, admission locks, audit/receipt ordering and save-only. Reload, other-VM isolation, file conflicts, read failures, replaced receipts and missing-terminal regressions exist. No further source blocker found; real responsiveness and interaction remain APP-009 acceptance |
| APP-008 | Narrow gate prepared in `scripts/testing/check.ps1`; documentation checker in `scripts/testing/check-docs.py`. No retired tooling restored. Test-effect review completed as described below; all six stages passed with exit 0, recorded in [closure evidence](m1-closure.md) |

No new standalone implementation task was created: these findings fit the existing APP scope. The follow-up found no larger issue requiring a separate task.

## Follow-up test-effect review

The current root workspace contains inline tests only, with no integration-test directory or ignored live suites. `cargo test --locked --all-targets` exercises isolated workflow doubles, configuration parsing/store snapshots, controller state/presentation helpers, enrollment validation, audit callbacks, credential closures (no vault/native prompt), progress and transfer budgets. Filesystem tests use temporary user-owned fixtures. Windows tests use anonymous/user-owned test pipes, current-token queries, empty process jobs, in-memory CIM objects without provider connections, and signature/ACL refusal against temporary unsigned files. Fixed PowerShell subprocess tests cover stdin, exit/diagnostic/transfer records and malformed guest-transport JSON; the latter rejects input before Hyper-V import or guest connection. Probe tests use synthetic adapter records and never run rendering. No test invokes installed runner submission, setup/enrollment, protected publication, credential prompts, real discovery, VM operations or GPU workloads. Cargo builds binaries but does not launch the application or graphics probe.

The gate refuses elevated execution, runs stages sequentially, stops on failure and retains incremental local evidence. No GUI render/interaction is included; the user's milestone contract assigns those checks to separately authorized APP-009.

## Checks and limits

The original review completed targeted source/caller/diff/reference inspection and `git diff --check`, preserving the then-existing APP-005 edits. Follow-up review on 11 October 2026 inspected current source at `4c293ec`, including APP-006/APP-007 and retained test effects, and preserved the user's testing-document edit. This static review cannot establish runtime correctness, trusted installation behavior, hardware sharing or visual acceptance. APP-008 records its passing automated gate separately in [M1 closure](m1-closure.md); M1 is closed within that scope.
