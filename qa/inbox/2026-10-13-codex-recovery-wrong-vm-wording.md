# Bug report card

**Short title:** An unrelated VM's detail panel says Setup interrupted for VM1's recovery hold
**Date:** 13 October 2026
**Screen or action:** Dashboard after reopening with a retained Verify recovery hold

## What I did

1. Ran Verify on existing configured VM1; it failed with Unknown and retained recovery.
2. Gracefully closed and reopened the application.
3. Read the initially selected VM2's GPU support panel.

## What I expected

Explain that new changes are blocked by VM1's host-wide recovery hold. Do not imply VM2 had an interrupted setup. Also distinguish a failed verification from interrupted first setup.

## What actually happened

VM2's own status was Stopped · Unknown, with No GPU attached. Its GPU support panel nevertheless said "Setup interrupted - review recovery before changing GPU support." VM2 was not enrolled or modified during this session. VM1's card correctly said Recovery required, and Inspect recovery correctly named VM1. The top banner described a protected recovery record but did not name the affected VM.

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

Suggested severity: P2 pending intake, because recovery identity and next steps are confusing. Host-wide blocking itself is expected and was preserved.

## Optional details

**VM/GPU starting state:** Retained failed Verify hold belongs to VM1; all VMs Off; VM2 had no GPU attached and was untouched.
**App build/version:** 0.1.0; revision 8bf3201676ba379692cb3c256688b9b0e57afaee; installed exploratory Slint MCP feature build.
**Exact error wording:** "Setup interrupted - review recovery before changing GPU support."
**Screenshot or local evidence path:** `local/evidence/qa/2026-10-13-codex-first-session/reopened-recovery.png`, `inspect-recovery.png`, `verify-failed-recovery.json`.
**Was a VM or host changed, interrupted or left in an uncertain state?:** No additional effects were attempted. VM1's original failure hold remains.
**Other notes:** Codex exploratory observation. No root-cause claim or fix; keep the durable hold while investigating.
