# QA report inbox

This folder is the hand-off between **your GUI testing** and **Codex fixing defects**. Start with the current user task and its acceptance criteria. The product's existing [agent rules](../AGENTS.md) and validation boundaries continue to apply.

## The two forms

- [User questionnaire](templates/USER_QUESTIONNAIRE.md): one short report after a test session; also useful when the app *technically* worked but felt confusing.
- [Bug report card](templates/BUG_REPORT_CARD.md): one distinct bug, failure or misleading result. Fill as much or as little as you know.

## How to submit a report

1. Copy the appropriate template into **`qa/inbox/`**.
2. Rename it, e.g. `2026-10-13-vram-slider.md` or `2026-10-13-first-setup-session.md`.
3. Type answers in ordinary language. You don't need to diagnose the cause, know technical jargon or complete every field.
4. Add a screenshot path if available, but do not include passwords, guest secrets, tokens or private log dumps in tracked reports.
5. Tell Codex **"Process the QA inbox"**, using [the supplied prompt](prompts/PROCESS_QA_INBOX.md). Creating a file does not itself make Codex execute anything.

If you find several faults in one session, submit a questionnaire and one separate bug report for each fault. Codex can investigate each symptom directly without requiring separate case files.

## Folder layout

```text
docs/roadmap.md                 Test stages and acceptance gates
docs/backlog.md                 Planned test tasks and results
qa/README.md                    This guide
qa/INDEX.md                     Optional status overview, updated only on request
qa/templates/USER_QUESTIONNAIRE.md  Your test-session report template
qa/templates/BUG_REPORT_CARD.md    Your quick bug template
qa/templates/CASE_CARD.md          Optional investigation template, used only on request
qa/inbox/                       Your original submissions (unchanged)
qa/cases/                       Optional case files, created only on request
qa/prompts/PROCESS_QA_INBOX.md  Ready-to-use Codex instruction
local/evidence/qa/              Local non-secret evidence, ignored by Git
```

**Reports are temporary inputs.** You may delete them when finished. Codex leaves current submissions unchanged unless asked, respects your deletions and does not recreate or archive removed reports. Processing reports does not require separate cases, index updates or permanent report history; those are optional and need an explicit request. Fix/check results and GUI retest instructions are returned in chat. See the report lifecycle. User confirmation is still required for user acceptance.

### Quick acceptance reply

After Codex installs a fix, you can reply in plain text or submit another file:

```text
Case: BUG-0001
Retest: Pass / Still broken / Different result / Not tested
What happened: ...
Build (if known): ...
```
