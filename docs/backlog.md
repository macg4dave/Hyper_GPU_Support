# QA backlog

Follow the [QA roadmap](roadmap.md). Planned tasks are not test results or authorisation for live effects. Product defects use stable BUG IDs in [qa/INDEX.md](../qa/INDEX.md). Task dependencies mean the required scenario/environment is prepared; independent checks need not wait for an unrelated workflow to pass.

| ID | Stage | Task | Implementation | Dependencies | Validation |
|---|---|---|---|---|---|
| QA-001 | Prepare | Prepare test sessions | Ready | None | Not run |
| QA-002 | Core | Discovery and selection | Blocked: prerequisites | QA-001 | Not run |
| QA-003 | Core | Initial setup | Blocked: prerequisites | QA-002 | Not run |
| QA-004 | Core | Everyday changes | Blocked: prerequisites | QA-002 | Not run |
| QA-005 | Failure/UI | Verify, repair and profile removal | Blocked: prerequisites | QA-002 | Not run |
| QA-006 | Failure/UI | Cancellation and recovery | Blocked: prerequisites | QA-002 | Not run |
| QA-007 | Failure/UI | Usability | Blocked: prerequisites | QA-002 | Not run |
| QA-008 | Failure/UI | Responsiveness and diagnostics | Blocked: prerequisites | QA-002 | Not run |
| QA-009 | Acceptance | Candidate regression | Blocked: prerequisites | QA-003, QA-004, QA-005, QA-006, QA-007, QA-008 | Not run |
| QA-010 | Acceptance | Acceptance decision | Blocked: prerequisites | QA-009 | Not run |

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

Start with **QA-001** when requested, then run the prepared GUI journeys. Exact candidate identity is not an early-testing prerequisite; capture it at QA-009. After each session, submit observations to `qa/inbox/` and update task validation with actual outcomes/evidence. Fix authorised BUG cases through the repository workflow; source/automated passes remain separate from user GUI acceptance.
