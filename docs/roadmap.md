# QA roadmap

**Status:** Planned; no tests run under this plan.

Test the installed Windows application through its normal GUI workflow. Reuse existing automated checks and the [QA report folder](../qa/README.md). Track planned work in the [QA backlog](QA_BACKLOG.md) and defects in the [case index](../qa/INDEX.md).

## Test stages

| Stage | Focus | Complete when |
|---|---|---|
| 1. Prepare | Record build, Windows/GPU versions, prepared VMs and authorised effects | Test targets, reset method and evidence location are known |
| 2. Core workflows | Discovery, initial setup, GPU toggle, VRAM, Apply and Verify | Main journeys pass on the installed candidate |
| 3. Failure and UI checks | Cancellation, recovery, stale state, responsiveness, layout and wording | Failures have usable next actions; controls remain usable |
| 4. Regression and acceptance | Retest fixes, run the existing automated gate, review open defects | Required tests pass and the user accepts remaining limitations |

## Report and retest

1. User submits a [questionnaire](../qa/templates/USER_QUESTIONNAIRE.md) or [bug report](../qa/templates/BUG_REPORT_CARD.md) in `qa/inbox/`.
2. On request, Codex triages reports into `qa/cases/` and updates `qa/INDEX.md`, preserving originals.
3. Fix authorised defects, record checks and build/install status, then provide GUI retest steps.
4. The user retests. Close a defect only after successful user acceptance.

**Case states:** New → Triaged → Fixing → Ready for user retest → User verified → Closed. Use Needs information, Blocked, Duplicate or Deferred where appropriate. Track implementation and validation separately.

**Severity:** P0 = unsafe effects, data loss, host instability or secret exposure; P1 = broken core workflow, false success or unusable recovery; P2 = secondary failure or confusing workaround; P3 = cosmetic issue.

## Validation boundaries

- Record each scenario as Passed, Failed, Blocked or Not tested, with build and evidence. Keep sensitive raw evidence out of Git; use `local/evidence/qa/` for suitable local evidence.
- Live GUI launches, installation, guest access and VM/GPU effects need explicit target/effect authorisation under [AGENTS.md](../AGENTS.md). Use prepared test VMs; stop on uncertain effects and preserve recovery evidence.
- At a checkpoint, use the existing non-hardware gate: `powershell -NoProfile -File scripts/testing/check.ps1`. Inspect effects first and follow [engineering validation](ENGINEERING.md#validation-and-live-acceptance). Automated checks do not prove live GUI or GPU behaviour.
- Acceptance requires no unresolved P0/P1 defects, successful core journeys and fix retests on the candidate, recorded automated results, and explicit disclosure of untested or deferred work.
