# Bug report card

**Short title:** VRAM-only draft says it will enable already enabled GPU support
**Date:** 13 October 2026
**Screen or action:** Dashboard > configured VM > GPU Memory request

## What I did

1. Selected VM1 with GPU support already enabled and the GPU attached.
2. Changed its VRAM request from 6 GB to 4 GB using Set request.
3. Read the blue pending summary and then opened review.

## What I expected

The pending summary should explain that GPU support stays enabled and that I am changing its memory request.

## What actually happened

The pending summary said "Pending: enable GPU support. VRAM request: 4 GB. Review before applying." The actual review correctly said Enabled → Enabled and listed only the resource update. The wording therefore describes an enable action that is not in the plan. A similar "disable" summary appeared when changing only memory in an already disabled draft.

## Does it happen again?

- [x] Every time in the draft-only edits observed
- [ ] Sometimes
- [ ] Once
- [ ] I haven't tried again

## How badly does it affect me?

- [ ] Can't continue / might affect host, VM or data
- [ ] Main task doesn't work
- [ ] Workaround exists, but confusing or broken
- [x] Cosmetic / wording only
- [ ] Unsure

Suggested severity for intake: P3. The actual reviewed plan and final operation result were truthful.

## Optional details

**VM/GPU starting state:** Existing enrolled Windows Generation 2 VM1; Off; RTX 5060 attached; GPU support enabled; saved optimal VRAM 6 GB.
**App build/version:** 0.1.0; revision 8bf3201676ba379692cb3c256688b9b0e57afaee; exploratory Slint MCP feature build. Full identity in the session evidence.
**Exact error wording:** "Pending: enable GPU support. VRAM request: 4 GB. Review before applying."
**Screenshot or local evidence path:** `local/evidence/qa/2026-10-13-codex-first-session/draft-4gb.png` and `review-complete-4gb.png`.
**Was a VM or host changed, interrupted or left in an uncertain state?:** Editing alone changed only the draft. The later confirmed 4 GB operation succeeded and left VM1 off.
**Other notes:** Codex exploratory observation. A limited source check found `pending_summary` in `src/gui/live/presentation.rs` chooses enable/disable from the desired enabled flag alone; no fix was made. Preserve this report when processing the inbox.
