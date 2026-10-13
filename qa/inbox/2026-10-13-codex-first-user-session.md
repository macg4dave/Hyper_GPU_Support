# User testing questionnaire

**Date:** 13 October 2026
**Screen or feature tested:** Installed GUI: discovery, draft memory editing, refresh, review/cancel, VRAM Apply, GPU disable/re-enable, Verify, recovery inspection, close/reopen.
**App build/version:** 0.1.0; revision 8bf3201676ba379692cb3c256688b9b0e57afaee. Clean working tree at start. Exploratory build with Slint MCP inspection enabled; real installed protected artifacts and ordinary winit/software GUI, no fixtures/headless mock data.
**Test VM/GPU:** Existing enrolled Windows Generation 2 VM1 (`9aca1d8e-6e4e-44ea-ab6d-3776c66e005c`), NVIDIA GeForce RTX 5060. Started Off with GPU attached and 6 GB saved. Other VMs were untouched.

### 1. What were you trying to do?

**Answer:** Use the application as an ordinary user to select an existing configured VM, change its memory request, turn GPU support off and on, check the guest GPU, and confirm settings and warnings after reopening. Codex acted as the first exploratory tester; the user entered guest sign-in credentials locally when requested.

### 2. Did it work?

- [ ] Yes, completed successfully
- [x] Partly — I was able to do some of it
- [ ] No — blocked or failed
- [ ] Unsure whether it succeeded

**What happened on screen?** Discovery showed the four actual VMs and RTX 5060. VM1 loaded 6 GB. The 4 GB draft survived manual Refresh and ten samples across 45 seconds; the input and Apply were enabled in those periodic samples. Cancel retained the draft and left saved settings unchanged. Confirmed 6→4 GB, GPU disable, and re-enable at 6 GB each succeeded and saved without repeating first setup. The editor rejected 0.9 GB and 9 GB while keeping the previous request, and accepted 1 GB and exact capacity 8.27850752 GB as drafts; endpoint values were not applied.

Verify's reviewed start/check/shutdown journey ran after the user's local sign-in, but failed with Unknown. It returned VM1 to Off and retained a recovery hold. I inspected the result/details/recovery without replaying the check, clearing the hold or running further modification tests. The warning survived graceful close/reopen. Saved 6 GB reloaded after selecting VM1 again.

### 3. Was anything difficult to understand?

- [ ] No
- [x] Labels or instructions were confusing
- [ ] I couldn't find the right action
- [x] A button or setting behaved unexpectedly
- [x] I wasn't sure what the application was doing

**Where and why?** VRAM-only editing said Pending: enable GPU support even though it stayed enabled. Review itself was accurate but repeated long raw byte triples. Verify's Unknown result and stage-only Technical Details did not explain the cause. The recovery dialog offered several alternative lifecycle descriptions. On reopen selection reset to VM2; its panel said Setup interrupted for VM1's retained failed Verify hold.

### 4. Was the interface responsive and readable?

- [ ] Yes
- [x] Slow, froze or repeatedly greyed out
- [ ] Text clipped, overlapped or did not wrap
- [ ] Progress or scrolling did not work properly
- [ ] A control became unreachable or failed to update

**Details:** Slow is the applicable part of the checked answer, not a claimed host freeze. Normal enabled-target review preparation took approximately two minutes and had no Cancel action during preparation. Confirmed 4 GB Apply took roughly three minutes, including a period with displayed stages Complete but the overall operation still in progress. It eventually succeeded. Periodic draft samples stayed usable; manual Refresh and foreground operations intentionally disabled controls. Observed display: 1240×860, scale factor 1, dark theme; screenshots showed readable wrapping and reachable review actions. Minimum size, other DPI/themes, screen-reader output and comprehensive keyboard navigation were not tested.

### 5. Did the final result make sense?

- [ ] Yes, I trusted the reported state
- [ ] No, it appeared to disagree with the VM/GPU state
- [ ] It gave an error, but I understood what to do
- [x] It gave an error, and I didn't know what to do
- [ ] Not applicable

**What did you expect it to tell you?** A useful reason for the failed guest check, clearly separate from historical success, and a recovery action scoped to the affected VM. Apply results were truthful and independent readback confirmed the original GPU/6 GB state restored. Verify did not claim success. All four VMs were independently observed Off after failure; VM1's journal has latest_check=failed and pending=true. The operation receipt remains EffectsStarted. Do not treat the older last_verified record as current healthy evidence.

### 6. What would you change first?

Investigate Verify's Unknown failure and make its failure/recovery explanation useful before exposing this journey to real users. Then address the long everyday-change waits and misleading draft/recovery wording.

### Optional evidence

**Screenshot path / error text:** `local/evidence/qa/2026-10-13-codex-first-session/` contains screenshots, numbered real-GUI MCP observations, installation artifact hashes, saved-state readbacks and retained failure/recovery evidence. Separate bug cards in this inbox cover Verify, performance, draft wording, reopen selection and recovery wording.
**Any action interrupted, cancelled or followed by a hang?** Cancelled the first review normally. No forced process termination, VM power-off, checkpoint restoration, host restart or manual recovery deletion. No host hang observed. Verify failed normally and retained recovery; application close/reopen was graceful.
**Anything else Codex should know?** Both builds passed. Protected installation was independently verified against all four enrollment artifact pins. The full non-hardware gate passed at `local/evidence/m1-gate-20261013T175650800Z` (317 product tests plus 12 catalogue tests, lint/build/docs); its ordinary feature set is distinct from the installed exploratory MCP build. Windows Computer Use's native pipe was unavailable, so Slint's supported real-GUI interface was used. User assistance was limited to local native guest sign-in. Clean first-install/setup, credential cancellation, destructive fault injection, profile removal, interrupted-work recovery completion, full endpoint effects and final release acceptance remain untested. No bugs were fixed or cases closed in this session.
