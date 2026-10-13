# Bug report card

> Copy into `qa/inbox/YYYY-MM-DD-brief-title.md`. **Describe the symptom, not a guessed solution.** You only need a title, what you did, and what happened. The rest is optional.

**Short title:**  
**Date:**  
**Screen or action:**  

## What I did

1. 
2. 
3. 

## What I expected


## What actually happened


## Does it happen again?

- [ ] Every time
- [ ] Sometimes
- [ ] Once
- [ ] I haven't tried again

## How badly does it affect me?

- [ ] Can't continue / might affect host, VM or data
- [ ] Main task doesn't work
- [ ] Workaround exists, but confusing or broken
- [ ] Cosmetic / wording only
- [ ] Unsure

## Optional details

**VM/GPU starting state:**  
**App build/version (if known):**  
**Exact error wording (if safe to share):**  
**Screenshot or local evidence path:**  
**Was a VM or host changed, interrupted or left in an uncertain state?:**  
**Other notes:**  

---

**For the AI:** Leave the original in `qa/inbox/` unchanged. Create/link a separate case using `qa/templates/CASE_CARD.md`, determine severity from evidence, investigate the real cause, and return short GUI-only user retest steps. Do not fabricate results or request secrets.
