# Hyper GPU Support — QA and Testing Roadmap

**Status:** Proposed; no new QA tests or live VM operations have been performed by this plan.  
**Owner:** User for hands-on acceptance; Codex for intake, diagnosis and implementation.  
**Relationship:** Independent of `docs/roadmap.md` and `docs/backlog.md` (current language development). This QA plan does not reopen accepted APP work or claim that earlier test evidence proves current runtime behaviour.

## Goal

Qualify the *installed Windows application*, not just its Rust functions. Verify that an ordinary user can discover, configure, enrol, apply, verify, repair and remove GPU-PV for existing Hyper-V Windows 11 VMs, and can understand failures and recover without using developer commands.

The user conducts real GUI tests and submits observations. Codex reads those observations from `qa/inbox/`, creates tracked cases in `qa/cases/`, investigates and fixes confirmed defects, builds/installs on an authorised test machine, and returns precise retest steps. A fix is **not user-verified** until the user reports a successful retest.

**Non-goals:** New features unless needed to complete an existing user workflow, broad architecture rewrites, increasing test counts for their own sake, automated guest/host resets, or substituting headless tests for real UI acceptance.

## Roles and separation

| Role | Responsibility | Not permitted to claim |
|---|---|---|
| User tester | Run the actual GUI; complete the questionnaire; report failures/confusion; retest fixes | Source root cause without evidence |
| QA/triage AI | Read original reports; deduplicate; reproduce when authorised; classify and create cases | A user's experience was verified by unit tests |
| Developer AI | Fix assigned cases in existing Rust/Slint architecture; add focused regressions; build and install when authorised | That a fix works for the user before retest |
| Reviewer AI (optional) | Independently review consequential changes and release blockers | That code inspection proves VM/driver or host stability |

Keep `qa/inbox/` submissions unchanged. AI work belongs in `qa/cases/` and `qa/INDEX.md`. Separate **implementation** and **validation** outcomes, as required by `docs/ENGINEERING.md`.

## Ground rules

1. Test the GUI as a user: do not use PowerShell, configuration-file edits or internal APIs to bypass a failed GUI step. Developer diagnostics may be used **after** the user failure is recorded.
2. Fix the underlying workflow, not just the test expectation. Where a defect is directly related to another open item, fix it in the same coherent change instead of deferring it to create a loop.
3. A failed scenario is not silently recorded as passed because a unit test or another route succeeded.
4. Do not run full GUI/hardware tests after each line edit. Build/reinstall after each coherent fix or approved batch; do focused validation then. Run broad regression at checkpoints.
5. Check live-test authorisation for the precise host, VM, GPU, guest access, power effects and evidence. Never force host restart, clear uncertain recovery, blindly replay a protected operation, or collect passwords in reports.
6. Keep raw evidence and screenshots out of source control when sensitive. Use ignored `local/evidence/qa/`; link to relative evidence paths in cases. Do not commit credentials, tokens, VM disks or unredacted logs.
7. The existing non-hardware gate is `powershell -NoProfile -File scripts/testing/check.ps1` from the repository root. Inspect test effects before running it; it cannot prove hardware behaviour.
8. Do not close an issue for lack of reproduction alone. Mark it **Needs information** or **Blocked** and preserve the report.

## Milestone plan

Each row is an independently trackable task, **not** a claim that a test has already passed. The priority is real user workflows, not code coverage percentages.

| ID | Stage | Test focus | Exit evidence |
|---|---|---|---|
| QA-001 | Prepare | Record installed build/revision, Windows/Hyper-V/GPU versions, test VM inventory, backup/reset method and approved effects | Test environment and stop rules recorded; no live effects implied |
| QA-002 | Baseline | Start/close app, initial discovery, installation/enrolment state, main dashboard, selection and settings | User questionnaire and passed/failed case results |
| QA-003 | Initial setup | On a prepared clean VM: enable GPU, set VRAM, review plan, authorise, enrol, apply and verify | Real end-to-end result; actual guest health and final VM power state recorded |
| QA-004 | Everyday changes | Change VRAM, disable/re-enable GPU, verify, reinstall/repair if offered, remove managed state | Settings and observed state reconcile; no unintended fresh setup |
| QA-005 | Broken state | Missing guest driver after checkpoint restore, stale enrolment, malformed/outdated config, interrupted operation, missing VM/GPU | Accurate UI diagnosis and supported repair/reconciliation path; no false success |
| QA-006 | Cancellation/recovery | Cancel consent/credentials; reject or lose guest access; transient operation failure; restart app with an unresolved record | No unauthorised effects; truthful persistent warning and safe next action |
| QA-007 | Usability | UI language, terminology, buttons, focus, scrolling, progress, window off-screen, DPI/theme, long names and text | User understands actions/results; no blocking clipping or unreachable controls |
| QA-008 | Stability/security | Responsiveness during discovery/Apply, bounded waits/logging, repeated operations, protected-operation coordination, safe diagnostics | No unexplained lockup accepted; high-risk paths reviewed and evidence recorded |
| QA-009 | Regression | Retest fixed defects plus representative clean setup, existing setup, failed/recovery state and UI flows on installed candidate | All required cases with exact build, outcome and evidence; limitations explicit |
| QA-010 | Release decision | Review open P0/P1, deferred P2/P3, incomplete scenarios, automated gate and user's retest sign-off | Written release recommendation; blockers and untested risks listed |

### Test journey checklist

Use a new `qa/inbox/` questionnaire after each meaningful session, with one bug report per distinct defect. Mark tests **Passed / Failed / Blocked / Not tested** in the corresponding cases; *no result is assumed*.

| Journey | What to watch |
|---|---|
| Open dashboard | First-start state, active GPU/VM inventory, clear refresh/progress, usable controls |
| Select GPU and VM | Correct identity, preserved selection, card-specific VRAM maximum and advanced defaults |
| Apply review | Actual intended changes, current limits, VM power transitions, clear confirmation/cancellation |
| Enrol / initial Apply | Correct chosen target, credential explanation, bounded progress, truthful verified result |
| Adjust GPU VRAM | Save/effective state agree after refresh and reopening app |
| Disable, repair, remove | Available and discoverable actions, no stale enrolled/healthy labels |
| Verify | Checks actual current state, meaningful error information, no falsely healthy result |
| Recover interrupted work | On next launch, warning and permissible next action; no silent retry/clear |
| UI stress | Long names, minimum window size, high DPI, clipped text, auto-scroll, inaccessible controls |
| Responsiveness | No repeated greying, freezes, unbounded waits or off-screen repaint failures |

### Controlled negative scenarios

The user or developer prepares a known resettable test VM; do not corrupt a production VM. Record the starting state and reset method *before* creating a fault. Test one fault per case so the result is attributable. Candidate cases include driver removed by checkpoint restore while host enrolment remains; invalid VM configuration; powered-on VM where a change requires shutdown; unavailable GPU; rejected guest credentials; cancelled review; interrupted transfer; conflicting/stale plan; and save/publish failure after otherwise verified effects.

For crashes/hangs: record the last on-screen action, approximate elapsed time, whether the host or only the app stopped responding, whether diagnostics survived, and whether the operation's outcome is still uncertain. **Stop further modification tests** until state is safely reconciled.

## Severity, ownership and status

| Severity | Meaning | Gate |
|---|---|---|
| P0 Blocker | Host instability, destructive/unrecoverable operation, credential exposure, unsafe mutation or corrupting state | Block release and affected live tests |
| P1 High | Core user workflow fails, false success, misleading state, missing critical recovery action | Block release |
| P2 Medium | Workaround needed; confusing or unreliable UI; secondary function incorrect | Fix before release when practical or record explicit deferment |
| P3 Low | Cosmetic wording, spacing or minor nonblocking polish | May defer with recorded reason |

**Working-case states:** New → Triaged → Fixing → Ready for user retest → User verified → Closed. Also: Needs information, Blocked, Duplicate, Deferred. Do not automatically move from Ready for user retest to User verified. A developer may report `Implementation: Done` with `Validation: Awaiting user retest`.

## Intake-to-fix procedure

1. **User:** Copy `qa/templates/USER_QUESTIONNAIRE.md` or `qa/templates/BUG_REPORT_CARD.md` into `qa/inbox/` with a descriptive filename. Fill only what you know; uncertain fields may remain blank. File multiple observations separately where possible.
2. **AI triage:** On explicit request, scan unread `qa/inbox/*.md`; leave originals intact. Merge clear duplicates by linking their sources. For each unique defect, create `qa/cases/BUG-0001-short-title.md` using `qa/templates/CASE_CARD.md`; allocate IDs by scanning existing cases. Update `qa/INDEX.md`.
3. **AI developer:** When authorised to work on selected cases, trace the failure and implement the smallest sound product fix. Don't replace real implementation work with test-only changes. Include focused regression checks where useful.
4. **AI developer:** After the coherent fix, build the application. Reinstall/run live only within explicit authorisation; otherwise mark the case Awaiting install. Supply the build identifier, changed behaviour and GUI-only retest steps.
5. **User:** Retest from the recorded starting state and reply in the case or a new inbox report, giving **Pass / Still broken / Different result**, plus notes.
6. **AI triage:** Update validation/evidence. Close only after the user confirms the relevant acceptance outcome. Escalate release blockers and keep unrelated scope out of the fix.

Codex does **not automatically watch** the inbox just because these files exist. Run the instruction in `qa/prompts/PROCESS_QA_INBOX.md` when ready to process reports.

## Evidence and completion gates

Each case should record: source report, stable case ID, app commit/build, relevant Windows/Hyper-V/GPU and VM state, steps and expected/actual outcome, screenshots/log references, repair commit, checks actually run and not run, installation status and user retest response. Redact identifying and secret-bearing data.

**Release gate:** No unresolved P0/P1; key journeys verified on an installed release candidate; any incomplete live tests explicitly disclosed; the agreed non-hardware gate passed for that candidate or failures justified and resolved; a verified repair/recovery route for critical mismatch states; and an explicit user acceptance decision. A source-level fix or a previous milestone pass alone does not satisfy this gate.
