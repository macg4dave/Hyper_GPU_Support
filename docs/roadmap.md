# QA roadmap

**Completed, 15 October 2026:** the user confirmed that all tests are complete and requested closure and removal of every backlog task. The [backlog](backlog.md#next-action) is empty. This records the user's completion confirmation; the workflow guidance below remains available for future changes.

**Build and installation checkpoint:** revision `d4565b9a7dbc8c5dc28269d4683d4fc36586229c` passed the full non-hardware gate (367 product tests, 12 catalogue tests, copy/docs/format checks, strict Clippy, binary builds and API documentation). The protected installation was repeated successfully, with all four artifact hashes matching, both enrolled targets and the operator preserved, persistent settings/history/recovery records unchanged, and the runner enabled and Ready. Evidence: `local/evidence/m1-gate-20261015T100454631Z/` and `local/evidence/install-20261015T100906705Z/verification.json`.

Qualify the installed Windows application through the [agreed user workflow](../README.md#agreed-workflow): discover an existing VM/GPU, configure, review, enrol, Apply, Verify and make everyday changes. Preserve the existing Slint UI, shared Rust core and protected operation architecture. Use the [backlog](backlog.md) for any future requested tasks, the [QA guide](../qa/README.md) for observations, and the [case index](../qa/INDEX.md) only when explicitly requested.

This page owns the QA scope and acceptance gates. [Engineering](ENGINEERING.md) owns repository status/evidence conventions and check timing; [AGENTS.md](../AGENTS.md) owns authorisation. Earlier APP work and language-extraction records remain historical context, not proof of current behaviour. Address defects in existing workflows; new features and broad refactors need their own requested scope.

## Test stages

| Stage | Focus | Complete when |
|---|---|---|
| 1. Prepare | Choose resettable test VMs, starting states, allowed effects, stop conditions and evidence location | Sessions can be run safely; an exact release candidate is not required |
| 2. Core workflows | Discovery, initial setup, GPU toggle, VRAM, Apply and Verify | Actual GUI outcomes and guest health/final power state are recorded |
| 3. Failure and UI checks | Cancellation, recovery, stale state, responsiveness, layout, keyboard access and wording | Failures have supported next actions; actions remain readable and reachable |
| 4. Regression and acceptance | Identify the installed candidate, retest fixes and core journeys, run the existing automated gate and review open defects | Candidate-specific results and limitations support the user's acceptance decision |

Early exploratory sessions may use the current installed app. Record the version/build if readily available; unknown identity does not block early testing. Exact installed build/revision and automated evidence become required at candidate regression, before final acceptance. Early passes do not transfer automatically to that candidate.

## Test journeys

| Journey | What to check |
|---|---|
| Open/close and discovery | Correct VM/GPU identity, installation/enrolment state, preserved selection and drafts across refresh/reopening |
| Initial setup | Enable GPU, choose VRAM, review actual changes and power transitions, consent/enrolment, Apply and Verify; confirm guest health and agreed final power state |
| Everyday changes | VRAM from 1 GB to selected-card physical capacity; disable/re-enable without repeating setup; saved and observed state agree after refresh/reopening |
| Verify, repair and profile removal | Missing drivers, stale enrolment/state, missing VM/GPU and configuration mismatch produce truthful results and supported next steps; profile removal is clearly distinguished from GPU/guest effects |
| Cancellation and recovery | Cancel review/credentials, reject guest access, interrupt authorised work, reopen with unresolved records; retain warnings and evidence without silent retry or clearing holds |
| Usability | Minimum window size, long names/text, DPI/themes, keyboard focus, scrolling, reachable actions, progress and useful error wording |
| Responsiveness | Observe several periodic refresh cycles while editing and during dialogs; check progress, repeated actions, bounded waits, diagnostics and repaint behaviour |

Use known resettable VMs for negative scenarios. Record the starting state/reset method before introducing one fault per scenario. Candidate faults include checkpoint-restored missing drivers, unavailable GPU, stale plan, rejected credentials, interrupted transfer and publication/readback failure. Prepare each scenario and obtain the applicable authorisation before running it; the checklist is not authorisation.

For crashes/hangs, record the last action, elapsed time, whether the app or host stopped responding, and whether the outcome is uncertain. Stop further affected modification tests until state is safely reconciled; preserve recovery evidence.

## Repository report and fix workflow

1. The user tests through the normal GUI and submits a [questionnaire](../qa/templates/USER_QUESTIONNAIRE.md) or [bug report](../qa/templates/BUG_REPORT_CARD.md) in `qa/inbox/`. Record the failed GUI path before using developer diagnostics; shell/configuration workarounds do not establish GUI acceptance.
2. On request, Codex processes current reports using the [intake prompt](../qa/prompts/PROCESS_QA_INBOX.md) and returns concise fix/check results in chat. Reports are temporary inputs that the user deletes when finished. Separate reports, case files and index updates require an explicit request; do not recreate or archive deleted reports. There is no automatic inbox watcher.
3. Fix only authorised defects, one requested case or scoped batch at a time. Include tightly related corrections and meaningful failure regressions; preserve unrelated edits. Use existing components and do not expand into unrelated backlog work or delegate unless requested.
4. At the coherent fix/checkpoint, record source changes, focused checks and build outcome. Installation/live testing require separate applicable authorisation. Record installed/not-installed status and return short GUI-only retest steps with the starting state and expected result.
5. The user retests. Summarise Pass, Still broken, Different result or Not tested in chat; update optional case tracking only if requested. Positive user acceptance is required for acceptance; report deletion alone does not establish it.

**Case states:** New -> Triaged -> Fixing -> Ready for user retest -> User verified -> Closed. Also use Needs information, Blocked, Duplicate or Deferred. Keep implementation, automated validation, installation and user retest distinct. The backlog tracks test tasks; optional BUG cases track product defects only when requested. Report actual evidence without marking unrun journeys passed. Follow the [report lifecycle](ENGINEERING.md#status-and-evidence); permanent QA reports are not required.

## Severity and acceptance

| Severity | Meaning | Gate |
|---|---|---|
| P0 | Unsafe effects, data loss, host instability or secret exposure | Stop affected live tests; block release |
| P1 | Broken core workflow, false success or unusable critical recovery | Block release |
| P2 | Secondary failure or confusing workaround | Fix when practical or explicitly defer |
| P3 | Cosmetic or nonblocking wording/layout | May defer with a recorded reason |

Record each scenario as Passed, Failed, Blocked or Not tested, with evidence and build identity where known. Keep secrets out of tracked reports; use ignored `local/evidence/qa/` for suitable local evidence. Live GUI launches, installation, credentials/guest access, protected writes and VM/GPU/driver effects follow [AGENTS.md](../AGENTS.md) and [security](SECURITY.md). Never request passwords in chat, blindly replay effects, clear uncertain recovery or force power-off/restart.

At an agreed checkpoint, inspect effects and run the existing non-hardware gate: `powershell -NoProfile -File scripts/testing/check.ps1`, following [engineering validation](ENGINEERING.md#validation-and-live-acceptance). Do not run full tests after individual edits. Automated checks do not prove live GUI, guest or GPU behaviour.

Final acceptance requires the exact installed candidate recorded during candidate regression, successful core journeys and fix retests on that candidate, recorded automated results, no unresolved P0/P1, a usable critical recovery route, disclosed untested/deferred scenarios and the user's explicit acceptance decision.

## Plan maintenance validation

Consolidated 13 October 2026. Documentation/tooling checks passed: `python -B scripts/testing/test_check_docs.py` (14 isolated tests), `python -B scripts/testing/check-docs.py` (39 Markdown files, 10 QA cards, zero errors), and `git diff --check`. No product build, full automated gate, installation or live QA ran for that historical consolidation. Current completion is recorded above.
