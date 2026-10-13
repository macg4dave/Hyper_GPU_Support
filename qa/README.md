# QA report inbox

This folder is the hand-off between **your GUI testing** and **Codex fixing defects**. Start with [the QA roadmap](../docs/QA_ROADMAP.md) and [test backlog](../docs/QA_BACKLOG.md). The product's existing [agent rules](../AGENTS.md) and [validation boundaries](../docs/ENGINEERING.md#validation-and-live-acceptance) continue to apply.

## The two forms

- [User questionnaire](templates/USER_QUESTIONNAIRE.md): one short report after a test session; also useful when the app *technically* worked but felt confusing.
- [Bug report card](templates/BUG_REPORT_CARD.md): one distinct bug, failure or misleading result. Fill as much or as little as you know.

## How to submit a report

1. Copy the appropriate template into **`qa/inbox/`**.
2. Rename it, e.g. `2026-10-13-vram-slider.md` or `2026-10-13-first-setup-session.md`.
3. Type answers in ordinary language. You don't need to diagnose the cause, know technical jargon or complete every field.
4. Add a screenshot path if available, but do not include passwords, guest secrets, tokens or private log dumps in tracked reports.
5. Tell Codex **"Process the QA inbox"**, using [the supplied prompt](prompts/PROCESS_QA_INBOX.md). Creating a file does not itself make Codex execute anything.

If you find several faults in one session, submit a questionnaire and one separate bug report for each fault. An AI can also split a questionnaire into bug cases without requiring you to fill a second form.

## Folder layout

```text
docs/QA_ROADMAP.md               Test stages and acceptance gates
docs/QA_BACKLOG.md               Planned test tasks and results
qa/README.md                    This guide
qa/INDEX.md                     AI-maintained status overview
qa/templates/USER_QUESTIONNAIRE.md  Your test-session report template
qa/templates/BUG_REPORT_CARD.md    Your quick bug template
qa/templates/CASE_CARD.md          AI's investigation/fix record
qa/inbox/                       Your original submissions (unchanged)
qa/cases/                       AI-managed BUG-0001-*.md files
qa/prompts/PROCESS_QA_INBOX.md  Ready-to-use Codex instruction
local/evidence/qa/              Local non-secret evidence, ignored by Git
```

**Never erase or silently rewrite an original user report.** `qa/INDEX.md` points to stable cases; each case references all relevant source reports and follows the state machine in the roadmap. Codex should fix only user-authorised cases, not unrelated backlog items. It must return GUI retest instructions; user confirmation is required for verification/closure.

### Quick acceptance reply

After Codex installs a fix, you can reply in plain text or submit another file:

```text
Case: BUG-0001
Retest: Pass / Still broken / Different result / Not tested
What happened: ...
Build (if known): ...
```
