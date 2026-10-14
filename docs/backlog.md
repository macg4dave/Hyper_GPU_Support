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

The five reports from the [first exploratory session](../qa/inbox/2026-10-13-codex-first-user-session.md) have source patches reviewed and extended on 14 October 2026. The user requested source work and chose faster preliminary review with full authentication before Apply. Regression test source was added/extended, but no builds, test execution, installation or live operations ran. Static source/documentation checks are recorded below. All bugs remain open pending validation; the retained VM1 hold was not changed. Next: run the non-hardware gate at the agreed checkpoint, then prepare installed-candidate retests. QA-009/QA-010 remain blocked; historical passes do not validate these patches.

| Bug | Root cause and source change | Implementation | Validation |
|---|---|---|---|
| BUG-0001: [Verify / Unknown](../qa/inbox/2026-10-13-codex-verify-unknown-recovery.md) | Source defect: guest phase parser rejected CRLF-terminated records. Strip framing while retaining token/order checks. Preserve typed failures before adding dialog context; classify malformed records, deadlines and supervision errors, with usable transport explanations. Recovery review and credential decisions agree, with no fallback guest start when record-only proof changes. | Done: source | Not run; exact live cause, guest success and recovery completion unproven |
| BUG-0002: [Slow review / progress](../qa/inbox/2026-10-13-codex-slow-vram-review.md) | Review previously performed full driver discovery/authentication; final independent authentication and readback were invisible after effect rows completed. Ordinary preliminary reviews now use recorded setup evidence with fresh host facts. Apply authenticates the full current payload before effects, rejects any changed plan and requires a full next review. Preparation can be dismissed without publishing late results; final checks show progress. Review memory uses GB once, with exact byte triples in Technical Details. | Done: preliminary review, cancellation/progress source | Not run; elapsed-time improvement and Apply latency unmeasured |
| BUG-0003: [Draft wording](../qa/inbox/2026-10-13-codex-vram-draft-wording.md) | Summary used the desired enabled flag alone. Compare with baseline and say support stays enabled/disabled for memory-only changes. | Done: source | Not run; GUI retest pending |
| BUG-0004: [Reopen selection](../qa/inbox/2026-10-13-codex-reopen-selection.md) | Selection existed only in memory. Save a bounded VM GUID in per-user preferences on graceful close and restore it before discovery; normal missing-VM fallback remains. Preferences grant no authority. | Done: source | Not run; reopen/missing-VM retests pending |
| BUG-0005: [Recovery identity](../qa/inbox/2026-10-13-codex-recovery-wrong-vm-wording.md) | A global hold selected a setup-specific message in every VM panel. Name the affected VM and operation in the panel/banner, distinguish Verify from setup, and describe the specific Verify recovery action. | Done: source | Not run; retained-hold GUI retest pending |

Static checks on this source batch: targeted Rust formatting completed; `python -B scripts/testing/check-docs.py` passed (45 Markdown files, 10 task cards); `python -B scripts/testing/check-ui-copy.py` passed (28 paths, zero enforced baseline-copy errors); `git diff --check` passed. Copy scanning is lexical and new candidates remain advisory. These checks do not establish compilation, test behavior, elapsed-time improvement, rendering, installed integration or live recovery.

Session evidence: `local/evidence/qa/2026-10-13-codex-first-session/`. All four VMs were Off after Verify failed; VM1's original GPU and 6 GB saved request were restored, but its managed journal remains pending with latest_check=failed and the protected receipt remains EffectsStarted. The hold was preserved through graceful close/reopen. The ordinary-feature non-hardware gate passed at `local/evidence/m1-gate-20261013T175650800Z`; the installed exploratory build used Slint's MCP feature for real GUI inspection. User assistance was limited to local native guest sign-in. Clean setup, profile removal, credential cancellation, injected interruptions, recovery completion, other display settings and final acceptance remain unrun.
