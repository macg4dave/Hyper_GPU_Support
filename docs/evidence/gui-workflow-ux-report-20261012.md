# GUI workflow and usability report — 12 October 2026

The GUI successfully recovered VM1's partial GPU setup, deleted its inactive app profile and re-enrolled its selected GPU. The new setup was reviewed at **6 GB**, and the driver transfer reached **273 of 273 files, 3.4 GB copied**. At the last GUI observation, verification was still pending. Successful rendering, final shutdown, saved allocation and persistence after reopening **have not been confirmed**. The complete workflow remains open.

The largest usability problem is the amount of recovery, refresh and permission bookkeeping the user must understand before reaching a simple goal: enable GPU support and choose its memory. Progress feedback also becomes difficult to read precisely when the operation takes longest. The existing design can be improved without replacing the shared Rust core or its reviewed operation boundaries.

## Scope and evidence

- Target: `win_26300.9457 VM1`, selected NVIDIA GeForce RTX 5060.
- This report covers the recorded GUI sessions in this conversation, the resulting source investigation, and relevant earlier repository findings. It is not a complete audit of every screen or failure mode.
- VM recovery, profile deletion, enrollment and setup submission used the application GUI. Development tools were used for source inspection, builds and the authorized package installation.
- The initial current-source build and protected installation passed. Installation verified four artifact hashes and preservation of the existing pairing and three VM records. Raw installation evidence: `local/evidence/gui-workflow-install-20261012-183407/installation-result.json`.
- Screenshots were inspected in the computer-use session. They are conversation evidence; this report does not claim a separately saved screenshot collection.
- Control was returned before source investigation and for local credential entry. The user later stopped Computer Use with Escape. No subsequent GUI observation was made.
- Existing working-tree edits were present before this work. Their authorship and validation are not inferred from this session.

Earlier evidence is recorded in [live GUI findings](live-gui-check.md), [backlog clearance](backlog-clearance.md) and the [source interaction audit](../BASELINE.md#ui-interaction-audit---12-october-2026). Those records are historical context, not proof of current acceptance.

## Workflow actually exercised

| Step | Action and observed result | Assessment |
|---|---|---|
| Open installed app | Launch at the previous installation path failed with Windows error 2, executable not found. A fresh authorized build/install then allowed launch. | Installation availability issue observed; cause of the missing installation not established. |
| Identify broken state | Selected VM1. GUI showed stopped, GPU attached, saved support disabled and recovery required. A global recovery banner blocked new changes. | The mismatch was visible; the explanation was dense. |
| Check recorded outcome | Used Inspect recovery → Check recorded outcome. It refused the partial state and kept recovery. | Safe refusal; did not complete repair. |
| Recover partial setup | Followed Close → Refresh → Inspect recovery → Remove partial GPU setup → confirmation. GUI reported attachment removal and confirmed starting compatibility settings, with VM off. | Passed for this scenario. |
| Confirm recovered state | Explicit Refresh showed VM1 stopped, authorized, no GPU attached and saved support disabled. The recovery banner cleared. | Passed for this scenario. |
| Remove old app profile | Used More actions → Delete profile, reviewed scope, obtained action-time confirmation and confirmed. GUI reported verified deletion of app settings, authorization and managed history. | Passed for this inactive profile. The VM remained discoverable. |
| Re-enroll | Refreshed, enabled the GPU draft, selected 6 GB and used Apply → Allow GPU management. GUI returned to authorized status and retained the draft. | Passed for this pairing and draft. |
| Review setup | Used Apply again. Review showed VRAM minimum / optimal / maximum bytes **0 / 6,000,000,000 / 8,278,507,520**, driver preparation, GPU attachment, verification and final stopped state. | Exact requested value confirmed in review. This is not a completed allocation readback. |
| Submit setup | Confirmed Apply. User completed the local VM credential prompt. GUI reported VM start and driver preparation. | Submission and start observed. |
| Transfer and verification | Counters reached 100%, all 273 files copied. GUI continued to say verification pending. | Copy completion observed; setup success unconfirmed. |

The slider sets the optimal VRAM request while retaining the shown provider minimum and maximum. This session does not prove a hard 6 GB ceiling, reservation or isolation guarantee.

## Issues and recommended corrections

Priorities reflect impact on completing this user journey. An observed usability problem is distinct from a proven backend failure.

| Priority | Issue and evidence | Recommended user experience | Current disposition |
|---|---|---|---|
| High | **Recovery requires a long navigation chain.** A failed outcome check instructed Close, Refresh and Inspect recovery before partial removal; after removal, further refresh/profile/enrollment steps were required. | Keep a recovery page for the selected VM showing the failure, safe next action and current result. Offer the next review directly after successful recovery. | Partial removal passed. Guided continuation is a recommendation; APP-035 owns the recovery path. |
| High | **Status stays visibly stale after successful operations.** After partial removal, the old banner and card persisted until Refresh. After profile deletion, the list still showed Authorized while the details said Refresh needed. | Automatically perform readback after completion. While it runs, show “Updating VM status” consistently in both card and details; show a specific retry if readback fails. | Observed; no claim that the cached state authorized effects. |
| High | **Apply initially gives no stage feedback.** After credentials, the app spent a substantial interval displaying only an indeterminate bar before worker stages appeared. | Show “Checking reviewed VM and driver files” before those checks start, then the actual lifecycle stages. | Source fix added through the existing worker progress wrapper; focused regression passed. Fresh installed acceptance pending. |
| Medium | **Progress layout stretches controls excessively.** Once rows appeared, the heading had large gaps, the progress indicator became a thick block and the disabled button grew tall. | Keep headings, buttons and the indicator at normal sizes. Give remaining height to scrollable progress. | Source fix implemented in `dialogs.slint`; live retest pending. |
| Medium | **The indicator does not represent transfer percentage.** The animated bar continued to sweep while text reported 56%, 82% and 100%. | During copying, use a determinate bar based on acknowledged bytes. During checks, use a compact indeterminate indicator and name the check. | Recommendation. The existing text counters were useful and truthful. |
| Medium | **The post-copy phase lacks useful detail.** At 100%, the row remained “Copying files … verification pending” for multiple observations. | Change the current phase to “Checking copied driver files” when that phase actually begins. Distinguish copying, installation, file verification and graphics verification through real backend events. | Observed; no unsupported timeout or hung-operation diagnosis. |
| Medium | **Internal events clutter progress.** Repeated “Saving progress · In progress / Complete” rows preceded the user-visible VM/driver stages. | Present one current stage and a short checklist of meaningful steps. Keep detailed event history expandable. | Recommendation; preserve failure evidence and ordered event history. |
| Medium | **Status text demands technical interpretation.** The selected panel combined authorization, attachment, saved intent, historical graphics, host differences and a long GPU interface string. | Lead with a short result such as “Setup interrupted — repair needed,” then a clear next action. Put exact identities and detailed comparisons in Technical Details. | Observed readability issue; the underlying distinctions remain necessary. |
| Medium | **Action readiness changes without an explanation.** Remove partial GPU setup first appeared disabled while information was updating, then became available. | Show “Checking whether removal is safe…” with a visible result or specific unavailable reason beside the action. | Observed; no evidence that opening Technical Details caused readiness. |
| Medium | **Re-enrollment and setup feel disconnected.** Allow GPU management returned to the main screen and required another Apply. The wording explained this correctly, but the user still had to resume manually. | After permission succeeds, offer “Continue to setup review,” retaining the draft and requiring the separate effects confirmation. | Current two-review sequence passed; recommendation concerns continuity. |
| Low | **Setting an exact memory value depends on slider adjustment.** Pointer selection landed at 6.2 GB; two Left presses reached 6 GB. | Retain the slider and add a small numeric GB field with keyboard adjustment. Preserve exact bytes in review/readback. | Keyboard adjustment passed. Numeric input is a recommendation. |
| Low | **Inactive profile deletion is a prerequisite for this fresh enrollment journey.** The confirmation accurately explained scope, but deleting settings/history is a significant extra concept. | Explain why a fresh profile is needed in the guided recovery flow. Offer the deletion review there, with its own explicit confirmation. | Deletion passed; do not silently delete records or hide its scope. |

## What worked well

The application exposed the saved-versus-observed mismatch, retained recovery when outcome checks failed and offered a reviewed partial-removal path. Delete profile clearly distinguished app records from the VM and guest files. Re-enrollment preserved the 6 GB draft. The setup review disclosed the exact byte triple, planned steps and final power state. The local sign-in explanation named VM1, identified guest administrator credentials and made remembering optional. Transfer counters used readable MB/GB values and updated one row; 100% copying did not falsely claim verification success.

These strengths should remain in the simpler presentation.

## Proposed easier workflow

Keep the existing sidebar, VM list and selected-VM panel. Change the hierarchy and continuity of the existing controls:

1. **Show one primary state and action.** Examples: “GPU support is off — Enable,” “Setup interrupted — Review repair,” or “GPU support is on — Verify setup.” Follow with a short factual explanation and check freshness.
2. **Keep GPU and VRAM editing together.** Use the selected card name, toggle, slider and numeric GB value. Show a clear pending-change summary. Keep provider details in the review.
3. **Guide recovery without automatic effects.** Explain what failed and what safe repair can do. Each removal, deletion, permission or setup action still has its own bound review and confirmation.
4. **Continue after permission.** Confirm permission saved, retain edits and offer the setup review directly. Permission must not imply the guest is ready.
5. **Use task-focused progress.** Current stage at the top; copied bytes and determinate percentage during transfer; completed checklist below; detailed events expandable and scrollable. Never offer a cancellation promise the backend cannot fulfill.
6. **End with a verified result and refreshed state.** Successful setup should report rendering/check results, saved request and final VM power. Failure should name the failed stage and a reachable recovery action. Show “status unavailable” if readback failed instead of retaining a success-looking card.

Suggested ordinary summary after proven success: “GPU support enabled. VRAM request: 6 GB. Setup check passed. VM stopped.” Each sentence must be backed by the current operation result or clearly dated evidence.

## Testing approach

Use complete GUI journeys as the acceptance unit. Record starting state, exact actions, expected result, actual result, fresh installed version and retained non-secret evidence. Automated checks supplement these journeys.

- Complete this interrupted setup → recovery → profile deletion → re-enrollment → 6 GB setup → verification → reopen/persistence journey first.
- Retest the affected progress layout and pre-effect status against the rebuilt, reinstalled package during a real applicable operation. Source changes and screenshots of the older package cannot establish acceptance.
- Then exercise ordinary VRAM changes, toggle off/on, review cancellation, credential cancellation and recovery from a real failure within prepared authorization.
- Check keyboard-only navigation, focus return, progress scrolling, minimum supported size, scaling and both themes as distinct acceptance cases.
- Two-VM concurrency, driver throughput comparison and allocation isolation remain separate qualification work. This run does not close them.

## Fix and validation status at handoff

The initial build/install passed using `cargo build --locked --all-features --bins --jobs 2` and the protected installer, with matched built/installed/policy hashes. The subsequent progress changes are in the working tree. The focused test `worker::tests::admission_progress_is_visible_before_work_and_delivery_failure_prevents_work` passed; it checks that failed progress delivery prevents admission work and that failure is reported. The fresh build was last observed waiting for Cargo's artifact-directory lock. Its later outcome has not been inspected for this report. No fresh reinstall or GUI retest of those changes is recorded.

APP-035's partial-removal scenario and APP-024/APP-022's inactive deletion and re-enrollment continuation gained live evidence. That does not close their full matrices or the end-to-end task. APP-036's allocation correction, APP-011's live allocation contract, APP-032's full interaction coverage and APP-009/M2 remain unclosed here.

**Next required result:** observe the setup outcome through the GUI, resolve any reported failure through reviewed GUI actions, then confirm saved 6 GB request, enrollment, graphics checks and persistence after reopening. Finish the build/install/retest cycle for the progress fixes before claiming them accepted. Desktop control remains with the user; this report involved no further GUI input or VM effects.


## Source follow-up - 12 October 2026

At the user's request, the report's usability corrections are now implemented in source. This section supersedes the earlier recommendation/source dispositions for this follow-up; it does not alter the recorded GUI observations.

Completion now reads fresh host inventory and protected records automatically, while preserving the result dialog. Both card and selected details display Updating VM status during readback; failure shows unavailable status and a retry. Recovery failure retains a direct partial-removal review and explains readiness. Successful removal offers profile deletion review with its scope and a named draft-discard action where needed. Verified profile deletion offers a new setup review; confirmed permission offers Continue to setup review with the original draft. Each effects review and confirmation stays separate.

Progress uses a current stage, a short checklist and expandable ordered events. The indicator follows acknowledged copying percentage, then becomes indeterminate at the transport's real transfer-complete boundary while copied-file integrity checks and installation run. The guest integrity/installation phase is reported together because the current transport supervises one worker invocation; it does not invent separate internal timing or graphics success. Existing guest-file/setup-record and hardware-rendering verification follows as a separate backend stage. Internal recovery-save events stay in event details. The earlier layout and admission-stage fixes are preserved.

The selected panel leads with a short state/next action. Raw GPU identities, host comparisons, saved settings and dated graphics history remain in Technical Details. A decimal GB field accepts Enter or Set request alongside the existing keyboard-adjustable slider. Integer parsing preserves up to nine decimal places exactly, rejects overflow/out-of-range values and keeps the previous request on invalid input. Explicit edits retain the selected card's physical capacity as the maximum; review still shows exact bytes.

Regression source was added; **no builds, tests, installation, GUI launch or VM/guest effects ran for this follow-up**, as requested. Scoped Rust formatting and source/diff inspection were performed. Compilation, automated regression execution, visual/keyboard acceptance and the original end-to-end VM1 outcome remain pending. The missing executable's cause was not established by the report and has no inferred source fix. No task or milestone acceptance gate is closed.
