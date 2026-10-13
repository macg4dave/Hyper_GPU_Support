# Bug report card

**Short title:** Verify fails with Unknown and leaves an operation recovery hold
**Date:** 13 October 2026
**Screen or action:** Dashboard > VM1 > Verify GPU setup

## What I did

1. Used the normal GUI to change configured VM1 from 6 GB to 4 GB, disable GPU support, then re-enable it at the original 6 GB. All three confirmed changes succeeded, saved and left the VM off.
2. Selected Verify GPU setup, reviewed the current guest file/device/rendering check and agreed to the temporary start followed by graceful shutdown to Off.
3. Confirmed Verify. The user completed the native guest sign-in locally; Codex did not receive or enter credentials.
4. Observed the check's failure, opened Technical Details, then inspected recovery without confirming another check or clearing anything.

## What I expected

A current guest GPU health/rendering result, or a specific understandable failure and safe next action. A previous successful check should remain historical only.

## What actually happened

The progress reported starting the VM complete, guest health/hardware rendering in progress, and copying 2.3 MB of 2.3 MB (100%), 2 of 2 files, verification pending. It then displayed:

"Verify GPU setup did not complete: Unknown. Refresh read-only state and inspect recovery before retrying an uncertain operation."

The VM was stopped again, but its status became Recovery required and new edits were disabled. The dashboard called this "Setup interrupted" even though this was Verify on an existing configured VM. Technical Details listed stages but did not reveal a specific cause or failed guest test. The recovery dialog named VM1 and described several alternative paths, offering Check recorded outcome behind power consent; partial GPU removal was unavailable. I did not run the offered check because its description permits another guest check/power lifecycle and the recorded effects were uncertain.

Independent protected-state and Hyper-V readback confirmed all four VMs Off, VM1's original GPU attached and optimal VRAM 6,000,000,000 bytes; saved settings unchanged from the original state. Its managed journal has pending=true, verification_only=true, latest_check=failed, and retains the older last_verified timestamp. The protected operation receipt remains EffectsStarted. Failure/recovery evidence is retained.

## Does it happen again?

- [ ] Every time
- [ ] Sometimes
- [x] Once
- [x] I haven't retried the failed check

## How badly does it affect me?

- [x] Can't continue / might affect host, VM or data
- [x] Main task doesn't work
- [ ] Workaround exists, but confusing or broken
- [ ] Cosmetic / wording only
- [ ] Unsure

Suggested severity for intake: P1, pending investigation. The core Verify journey failed and blocks further changes. This does not establish that the GPU hardware is unhealthy, that the provided credential was wrong, or that the retained hold is itself a defect. The missing useful failure explanation is directly observed.

## Optional details

**VM/GPU starting state:** Existing enrolled Windows Generation 2 VM1, Off, RTX 5060 attached, saved optimal VRAM 6 GB, no pending operation before Verify.
**App build/version:** 0.1.0; revision 8bf3201676ba379692cb3c256688b9b0e57afaee. Exploratory installed Slint MCP feature build; installed four-artifact hash/pin readback recorded locally.
**Exact error wording:** Quoted above.
**Screenshot or local evidence path:** `local/evidence/qa/2026-10-13-codex-first-session/verify-review.png`, `verify-failed.png`, `verify-failure-details.png`, `inspect-recovery.png`, `verify-failed-recovery.json`, `journal-final.json`, `vms-final.json`, `gpu-final.json`.
**Was a VM or host changed, interrupted or left in an uncertain state?:** VM1 was temporarily started by Verify and returned to Off. A durable EffectsStarted recovery hold remains; no new effects were attempted after failure. No forced power-off, checkpoint rollback, direct hold deletion or manual GPU repair.
**Other notes:** Codex exploratory report with the user's local sign-in assistance. Automated checks passed but do not prove this GUI/guest path. No product fix or root-cause claim is made. Investigate the failed check and safe reconciliation before asking real users to test this path.
