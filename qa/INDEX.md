# QA case index

**Status:** Five reports have source patches reviewed and extended on 14 October 2026, including the user's choice of preliminary ordinary-change review. Original reports and live evidence are preserved. Static formatting/docs/copy checks ran; no builds, test execution or live operations ran, and no issues are closed. Source findings and implementation details are recorded in the [backlog](../docs/backlog.md#next-action).

Codex updates this file when explicitly asked to process `qa/inbox/`. Case IDs are stable and never recycled. User submissions remain in the inbox even after a case closes.

| Case | Severity | Status | Implementation | Validation | User retest | Source |
|---|---|---|---|---|---|---|
| BUG-0001 | P1 | Fixing | Done: source | Not run | Not tested | [Verify / Unknown](inbox/2026-10-13-codex-verify-unknown-recovery.md) |
| BUG-0002 | P2 | Fixing | Done: preliminary review and cancellation/progress source | Not run; timings unmeasured | Not tested | [Slow review](inbox/2026-10-13-codex-slow-vram-review.md) |
| BUG-0003 | P3 | Fixing | Done: source | Not run | Not tested | [Draft wording](inbox/2026-10-13-codex-vram-draft-wording.md) |
| BUG-0004 | P3 | Fixing | Done: source | Not run | Not tested | [Reopen selection](inbox/2026-10-13-codex-reopen-selection.md) |
| BUG-0005 | P2 | Fixing | Done: source | Not run | Not tested | [Recovery identity](inbox/2026-10-13-codex-recovery-wrong-vm-wording.md) |

## Intake notes

- New submissions: scan `qa/inbox/*.md` and compare with case `Source report(s)` before making new cases.
- Duplicate reports: add sources to the existing case; preserve original files.
- Release blocker BUG-0001 remains open until candidate-specific Verify and recovery results are recorded. The last QA readback observed VM1 Off with its protected recovery hold intact; this source batch did not change that state.
- Latest QA checkpoint: exploratory existing-VM session completed; candidate regression and final acceptance not run. Ordinary-feature automated gate passed separately; see [backlog](../docs/backlog.md#next-action).
