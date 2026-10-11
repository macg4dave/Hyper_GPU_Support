# Live GUI findings and handoff ? 11 October 2026

## Scope and stop point

The user authorized a full live check on **win_26300.9457 only**, small fixes with affected-scenario reruns, and backlog cards for larger changes. Other VMs, their disks and saved credentials were outside the cleanup/effect scope. The user later deferred live reruns for credential wording and transfer-display fixes.

At 19:03 BST the user requested documentation and an end to the chat while setup continues. The running application and operation were left untouched. The final read-only GUI observation showed all **273 files / 3,409,254,468 bytes** acknowledged, with verification still pending. This is a transfer observation, not setup success. At that handoff, preparation verification, GPU attachment, hardware rendering, publication and final shutdown had not yet been confirmed by the agent. No retry, forced shutdown, host restart or further test was started.

## User-reported completion

Later on 11 October 2026, the user confirmed that everything on the VM side worked and setup worked well apart from the recorded issues. The selected-VM setup outcome is now **accepted by the user**. This supersedes the pending setup outcome at handoff; no new independent terminal/readback inspection or repeated operation was performed for this confirmation. M1 remains closed. The noted UI copy/progress changes and APP-011 through APP-013 remain follow-up work; this acceptance does not claim explicit VRAM mapping or two-VM concurrency qualification.

## Findings and completed work

- **Stale test state:** deleted-VM enrollment/recovery records blocked the selected target. User-authorized cleanup removed stale protected records and old local test logs, preserving current VMs, disks and saved credentials. Package setup subsequently recreated the current selected-target authority. Raw former M1 gate logs were removed as requested; its tracked closure summary retains recorded outcomes.
- **Recurring historical mode:** an installed-only 15-second poll ran before enrollment, invalidated successful first-install discovery and disabled editing. Source now waits for protected enrollment before automatic polling. Real first-install and later installed polling remained usable beyond the previous failure interval.
- **Editing after refresh failure:** retained observations can support drafts and memory edits; fresh observation/review and protected admission still control Apply. Internal historical labels are replaced with unavailable-status/Refresh guidance. Isolated regressions checked that stale information cannot authorize Apply. No live failure injection was performed.
- **Sidebar:** removed the bottom-left ?Hyper-V virtual machines? label and Help button; actual live render confirmed their removal.
- **False VRAM ceiling:** removed wording that treated raw provider metadata as a physical GB ceiling, including the misleading default-request label. The underlying byte/provider mapping remains unchanged and unqualified, deliberately deferred under APP-011. No explicit VRAM allocation was used in this setup trial.
- **Guest sign-in copy:** after the user completed local credentials, source wording was shortened to explain the VM's Windows administrator account, setup purpose and optional Remember choice. Removed the VM ID and repeated technical wording; simplified the related operation header. This is a source-only follow-up, absent from the running build; the user waived its live rerun.
- **Transfer spam:** the running build appends acknowledged-byte/file counters as rows. Source now updates consecutive running transfer counters in place, preserving stage changes and terminal/failure rows. Regression coverage was added. Rebuild and live rerun are deferred at the user's request.
- **Recovery progress wording:** changed the user-facing “Saving recovery information” stage to “Saving progress.” Recovery behavior is unchanged. Text-only source follow-up; rebuild/live rerun deferred.

## Executed validation and live results

Before the later copy/progress follow-ups, **142 core tests passed** in the all-target run and **all 51 frontend tests passed** after correcting the new fixtures and enrollment-pair condition. Strict all-target Clippy, formatting, documentation/task/link checks and diff whitespace passed. The MCP-enabled real product GUI build passed. These results do not validate subsequent source-only copy/progress changes.

A focused credential test run had already started when the user waived rerunning the text fix: six tests passed and one old copy expectation failed. The assertion was corrected, then left unrun. The later progress regression is also unrun. Final documentation and whitespace checks passed; the latest source was not rebuilt or live-retested.

Live passes: actual rendering and inventory, selected-target navigation, editable toggle/draft, Apply becoming available, removal of sidebar items, retention beyond the first-install timer interval, package setup/enrollment, preservation of the draft after enrollment, subsequent installed polling, initial-setup review, local credential entry, confirmed VM start and transfer acknowledgements. The review showed guest preparation, graceful shutdown, compatibility/GPU attachment, start, graphics verification and graceful final-off sequence. It used provider defaults with no explicit VRAM request.

## Deferred work and next session

- **APP-011:** establish and correct the physical-memory/provider mapping, then qualify explicit VRAM allocation and independent readback. No 1 GB hardware-ceiling claim.
- **APP-012:** subsequently implemented in source: readable KB/MB/GB transfer sizes and percentage, preserving the distinction between acknowledgements and verification; one row updates in place. Build, tests and actual visual acceptance remain pending.
- **APP-013:** user change note to use **4 MiB chunks** instead of 1 MiB, updating the PowerShell sender and Rust acknowledgement bound together. Preserve bounded memory, ordering, integrity and deadline checks; measure actual throughput. This was note-only during the live run; subsequent APP-013 source implementation is recorded in the backlog. It has not been installed or throughput-tested.

The user has accepted the completed setup; do not repeat it solely to reconfirm that acceptance. Before any new effect, inspect current protected state and any pending recovery as required by normal admission. Do not replay setup or clear a hold. If it succeeded, confirm hardware rendering, saved settings and final Off before proceeding with the remaining authorized ordinary toggle/lifecycle and reopen/persistence checks. If it failed or remains uncertain, retain the evidence/hold and explain the specific result before any further effect. Read source and installed-artifact status before rebuilding; the running package predates the text/progress follow-ups. Two-VM concurrency is outside this one-VM run. Allocation tests await APP-011. **APP-009/M2 remains open.**
