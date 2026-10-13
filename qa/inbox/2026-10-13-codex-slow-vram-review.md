# Bug report card

**Short title:** A normal VRAM change spends minutes in preparation and Apply with limited progress detail
**Date:** 13 October 2026
**Screen or action:** Dashboard > configured VM > Set request > Apply

## What I did

1. Opened the installed application and selected the already configured, stopped Windows VM1.
2. Changed its GPU memory request from 6 GB to 4 GB and selected Apply.
3. Waited for review, cancelled it, reopened review and confirmed Apply.

## What I expected

A practical everyday change with useful progress information, including what a long wait is doing. I expected to be able to abandon read-only review preparation before confirming effects.

## What actually happened

The first review took approximately two minutes (17:56:34 UTC to between 17:58:26 and 17:58:36 UTC). Reopening after Cancel repeated the wait (17:58:58 to 18:00:43 UTC). The modal said "Building fresh plan", used an indeterminate progress indicator and exposed only a disabled "Operation in progress" button; there was no Cancel action during this read-only preparation.

After confirming the 4 GB change, Apply took roughly three minutes. Its resource update and driver-check rows showed Complete while the overall modal still said Operation in progress for over 40 seconds in subsequent observations. The eventual result was successful and saved; this is a responsiveness/progress concern, not evidence of a crash or false success. No fixed performance requirement or cause is established by this report. Automated gate work overlapped the first review; the second review and confirmed Apply ran after the gate had finished.

## Does it happen again?

- [x] Every time in the two enabled-target review attempts observed
- [ ] Sometimes
- [ ] Once
- [ ] I haven't tried again

## How badly does it affect me?

- [ ] Can't continue / might affect host, VM or data
- [ ] Main task doesn't work
- [x] Workaround exists, but confusing or broken
- [ ] Cosmetic / wording only
- [ ] Unsure

The workaround was to leave the app open and wait. Suggested severity for intake: P2; product performance expectations should be agreed during investigation.

## Optional details

**VM/GPU starting state:** Existing enrolled Windows Generation 2 VM1; Off; RTX 5060 attached; saved VRAM optimal 6,000,000,000 bytes. No pending managed operation.
**App build/version:** 0.1.0; revision 8bf3201676ba379692cb3c256688b9b0e57afaee. Exploratory Slint MCP feature build, with real installed protected artifacts and ordinary GUI/backend. Installed support executable SHA256 72278970a3ac0bee7aef41b1ab7f10c636cc57c1e72448c209905d6ed8f6a14b.
**Exact error wording:** No error. See quoted progress wording above.
**Screenshot or local evidence path:** `local/evidence/qa/2026-10-13-codex-first-session/review-after-wait.png`, `review-complete-4gb.png`, `apply-4gb-outcome.png`, `apply-4gb-success.png`; numbered MCP JSON observations in the same directory.
**Was a VM or host changed, interrupted or left in an uncertain state?:** Cancel did not change saved configuration; confirmed Apply changed VRAM to 4 GB and reported saved, final VM Off. No interruption or forced shutdown.
**Other notes:** Report written by Codex as the first exploratory tester, not by a human end user. No fix requested or implemented as part of this report. Native Windows Computer Use was unavailable; Slint GUI actions and real screenshots supplied evidence. Release acceptance remains pending.
