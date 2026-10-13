# QA case index

**Status:** First Codex exploratory session submitted on 13 October 2026; no cases registered or inbox intake performed yet. Live results and exclusions are in the [questionnaire](inbox/2026-10-13-codex-first-user-session.md); five separate bug reports await requested intake.

Codex updates this file when explicitly asked to process `qa/inbox/`. Case IDs are stable and never recycled. User submissions remain in the inbox even after a case closes.

| Case | Severity | Status | Implementation | Validation | User retest | Source |
|---|---|---|---|---|---|---|
| — | — | — | — | — | — | — |

## Intake notes

- New submissions: scan `qa/inbox/*.md` and compare with case `Source report(s)` before making new cases.
- Duplicate reports: add sources to the existing case; preserve original files.
- Observed release blocker awaiting triage: [Verify failed with Unknown and retained recovery](inbox/2026-10-13-codex-verify-unknown-recovery.md), suggested P1. VM1 is Off; its protected recovery hold remains intact. Further modification tests stopped.
- Latest QA checkpoint: exploratory existing-VM session completed; candidate regression and final acceptance not run. Ordinary-feature automated gate passed separately; see [backlog](../docs/backlog.md#next-action).
