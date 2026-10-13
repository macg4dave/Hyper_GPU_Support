# QA backlog

Follow the [QA roadmap](roadmap.md). Planned tasks are not test results or authorisation for live effects. Product defects use stable BUG IDs in [qa/INDEX.md](../qa/INDEX.md). Task dependencies mean the required scenario/environment is prepared; independent checks need not wait for an unrelated workflow to pass.

| ID | Stage | Task | Implementation | Dependencies | Validation |
|---|---|---|---|---|---|
| QA-001 | Prepare | Prepare test sessions | Done: existing-VM exploratory session | None | Passed: prepared target/effects/stop conditions recorded |
| QA-002 | Core | Discovery and selection | In progress | QA-001 | Failed: reopen selection; discovery/manual refresh passed |
| QA-003 | Core | Initial setup | Blocked: clean setup scenario not prepared | QA-002 | Not run |
| QA-004 | Core | Everyday changes | In progress | QA-002 | Passed: 6→4 GB, disable/re-enable at 6 GB; endpoint effects Not tested |
| QA-005 | Failure/UI | Verify, repair and profile removal | In progress | QA-002 | Failed: Verify returned Unknown and retained recovery; repair/removal Not tested |
| QA-006 | Failure/UI | Cancellation and recovery | In progress | QA-002 | Passed: review Cancel and recovery persistence; remaining scenarios Not tested |
| QA-007 | Failure/UI | Usability | In progress | QA-002 | Failed: misleading wording; inspected 1240×860, scale 1, dark only |
| QA-008 | Failure/UI | Responsiveness and diagnostics | In progress | QA-002 | Failed: long foreground waits/limited details; periodic draft samples passed |
| QA-009 | Acceptance | Candidate regression | Blocked: Verify failure and incomplete coverage | QA-003, QA-004, QA-005, QA-006, QA-007, QA-008 | Not run: exploratory build identity and ordinary-feature automated gate recorded separately |
| QA-010 | Acceptance | Acceptance decision | Blocked: QA-009 incomplete | QA-009 | Not run |

## Task cards

### QA-001 - Prepare test sessions

Dependencies: None.

Choose resettable test VMs, starting states, reset method, allowed effects, stop conditions and local evidence location. Record host/GPU details where useful. Exact candidate identity is deferred to QA-009.

**Complete when:** Prepared scenarios are safe to run within explicit authorisation; this task does not itself run live effects.

### QA-002 - Discovery and selection

Dependencies: QA-001.

Open/close the app, inspect discovery and installation/enrolment state, select the correct VM/GPU and refresh/reopen with selection and drafts preserved.

**Complete when:** Record the actual GUI outcome, identity and any lost state or misleading labels.

### QA-003 - Initial setup

Dependencies: QA-002.

On a prepared clean VM, enable GPU, choose VRAM, review changes/power transitions, consent/enrol, Apply and Verify.

**Complete when:** Record guest health, truthful completion and agreed final VM power state.

### QA-004 - Everyday changes

Dependencies: QA-002.

On a prepared configured VM, change VRAM within 1 GB to selected-card physical capacity, disable/re-enable GPU, refresh and reopen without repeating setup.

**Complete when:** Saved intent and observed state agree; record boundary rejection and final power state. Use QA-003 if a configured VM must first be prepared.

### QA-005 - Verify, repair and profile removal

Dependencies: QA-002.

Prepare individual missing-driver, stale enrolment/configuration and missing VM/GPU scenarios. Check Verify, available repair and profile removal explanations.

**Complete when:** Truthful results, clearly described removal scope and supported next steps; no false healthy/success labels.

### QA-006 - Cancellation and recovery

Dependencies: QA-002.

Cancel review and credentials, reject guest access and exercise separately authorised interrupted-work/restart scenarios.

**Complete when:** No unauthorised effects or silent replay; unresolved warnings persist with evidence and safe next actions.

### QA-007 - Usability

Dependencies: QA-002.

Check keyboard focus, scrolling, long names/text, minimum window size, DPI/themes, wording, progress and reachable actions.

**Complete when:** Record tested sizes/scaling/themes and any clipping, ambiguity or unreachable controls.

### QA-008 - Responsiveness and diagnostics

Dependencies: QA-002.

Observe several periodic refresh cycles while editing and during dialogs; check operation progress, repeated actions, bounded waits and repaint/diagnostics.

**Complete when:** Controls remain usable; no lost draft/selection or stale completion. Record timings and app versus host hangs; stop affected effects if uncertain.

### QA-009 - Candidate regression

Dependencies: QA-003, QA-004, QA-005, QA-006, QA-007, QA-008.

Now record exact installed candidate/build/revision and relevant environment. Retest fixes and representative clean/existing/failure/recovery/UI journeys on that candidate; inspect and run the existing non-hardware gate at this checkpoint.

**Complete when:** Candidate-specific outcomes, installation identity, gate evidence and coverage/exclusions recorded. Failed or blocked scenarios remain visible; prior exploratory passes are not candidate acceptance.

### QA-010 - Acceptance decision

Dependencies: QA-009.

Review candidate results, P0/P1 blockers, recovery usability, deferred P2/P3 and untested scenarios with the user.

**Complete when:** No unresolved P0/P1; required journeys/fix retests and automated checks recorded; explicit user acceptance or a blocked decision with reasons.

## Next action

The [first exploratory session](../qa/inbox/2026-10-13-codex-first-user-session.md) submitted a questionnaire and five bug reports. On request, process the inbox, prioritising the failed Verify result and retained recovery before further modification tests. No fixes or case closures are authorised by these reports alone. Exact candidate regression remains QA-009; automated passes remain separate from GUI/guest acceptance.

Session evidence: `local/evidence/qa/2026-10-13-codex-first-session/`. All four VMs were Off after Verify failed; VM1's original GPU and 6 GB saved request were restored, but its managed journal remains pending with latest_check=failed and the protected receipt remains EffectsStarted. The hold was preserved through graceful close/reopen. The ordinary-feature non-hardware gate passed at `local/evidence/m1-gate-20261013T175650800Z`; the installed exploratory build used Slint's MCP feature for real GUI inspection. User assistance was limited to local native guest sign-in. Clean setup, profile removal, credential cancellation, injected interruptions, recovery completion, other display settings and final acceptance remain unrun.
