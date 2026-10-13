# Bug report card

**Short title:** Reopening resets the selected VM
**Date:** 13 October 2026
**Screen or action:** Close and reopen the installed application

## What I did

1. Selected Windows VM1 and exercised its normal GPU/VRAM workflow.
2. After Verify failed, closed the result and read-only recovery inspection.
3. Gracefully closed the application and reopened the same installed build.

## What I expected

The selected VM should remain VM1, as required by the QA discovery/reopen journey. Recovery warnings should remain visible.

## What actually happened

Reopening selected the first listed VM, VM2, instead. VM1's recovery warning persisted on its card; manually selecting VM1 reloaded its correct saved 6 GB setting. No saved configuration loss was observed. This was one reopen with a retained recovery hold; reopen selection without a hold remains untested.

## Does it happen again?

- [ ] Every time
- [ ] Sometimes
- [x] Once
- [x] I haven't tried again

## How badly does it affect me?

- [ ] Can't continue / might affect host, VM or data
- [ ] Main task doesn't work
- [x] Workaround exists, but confusing or broken
- [ ] Cosmetic / wording only
- [ ] Unsure

Suggested severity: P3 pending intake. The workaround is to select the intended VM again.

## Optional details

**VM/GPU starting state:** VM1 selected, Off, RTX 5060 attached, 6 GB saved; Verify recovery hold retained.
**App build/version:** 0.1.0; revision 8bf3201676ba379692cb3c256688b9b0e57afaee; installed exploratory Slint MCP feature build.
**Screenshot or local evidence path:** `local/evidence/qa/2026-10-13-codex-first-session/graceful-close.json`, `reopened-recovery.png`, `reopened-vm1.png`.
**Was a VM or host changed, interrupted or left in an uncertain state?:** Reopening introduced no VM/GPU effects. Existing recovery remained intact.
**Other notes:** Codex first-tester report. No fix or repeated reproduction claimed.
