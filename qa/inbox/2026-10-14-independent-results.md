# Independent installed-GUI QA results — 14 October 2026

Exploratory black-box session using the installed Windows application and its normal GUI controls. No source investigation, builds, tests, fixes, enrollment, guest access or VM/GPU effects were performed. This is not release acceptance. Existing reports/case status were not changed.

## Environment and evidence

- Installed executable: `C:\Program Files\HyperGpuSupportProduct\hyper-gpu-support.exe`.
- About version: 0.1.0. SHA-256 captured after testing in `local/evidence/qa/2026-10-14-independent/installed-app-hash.txt`; installation-to-source revision correspondence was not independently established.
- Workspace reference only: `d21efbd7538182c017fbdf0fa21ef1da329bc320`, with pre-existing edits. This is not asserted as the installed revision.
- Slint inspection listener was unavailable on port 9315. Used the Computer Use skill's Windows GUI and accessibility APIs successfully; did not replace the installed binary.
- Screenshots and timestamped accessibility observations: `local/evidence/qa/2026-10-14-independent/`. Numbered filenames below identify evidence. Screenshots show a 1242 × 892 capture; desktop DPI was not independently measured.
- Starting GUI: VM2 selected; all four VMs Stopped. VM1 showed NVIDIA GeForce RTX 5060, a 6 GB request and a retained GPU-verification recovery hold. Other VMs showed no GPU attached. Status: No operation running.
- No live VM/GPU target/effect authorization was supplied. The retained hold was never cleared or retried.

## Results register

| ID | Workflow / check | Outcome | Actual evidence and limits |
|---|---|---|---|
| R01 | Launch installed app and inspect dashboard | Passed | Four VMs listed; app responsive. `01-start.json`, `01-start-0.png`. Discovery accuracy against Hyper-V was not checked outside the GUI. |
| R02 | Review affected-VM recovery from unrelated VM, inspect details, close | Passed | VM2 selected; banner, panel and dialog identify VM1 and GPU verification. Details identify the same VM and Off final power. Confirmation disabled without consent; Close retains hold and no operation running. `01`–`04`. Positive bounded BUG-0005 retest evidence; no case closure claimed. |
| R03 | About → application help → Escape | Passed | Version 0.1.0 visible; help opens; Escape dismisses it and restores navigation. `05`–`07`. Help copy was visible, not verified against effects. |
| R04 | Navigate to System | Passed | GPU and driver information renders. `08-system`, `09-system-settled`. Hardware claims not independently validated. Immediate navigation highlight settled on the subsequent observation; no persistent rendering defect confirmed. |
| R05 | Settings → Light → Follow Windows | Passed | Light palette rendered and original Follow Windows setting restored. `10`–`12`; final dark palette in `17`. Other themes/settings and credential dialog not exercised. |
| R06 | Select non-first VM3 → graceful close → reopen | Passed | VM3 remained selected; VM1 hold still identified correctly; all four VMs still Stopped. `14-vm3-before-reopen`, `15-reopened-vm3`. One reopen cycle with a retained hold; bounded BUG-0004 retest evidence only. Missing/renamed VM and hold-free scenarios remain untested. |
| R07 | Compare visible memory request with accessible slider value | Failed | VM1 shows 6 GB and edit value 6; slider labelled in GB exposes 60, twice across restart. `13-vm1-selected`, `16-vm1-slider-repeat`. See separate defect report. |
| R08 | Edit/review/apply VRAM, progress/cancel, disable/re-enable GPU | Blocked | Existing host-wide recovery hold disables changes; no authorized live targets/effects. BUG-0002 and BUG-0003 complete retests not performed. No bypass attempted. |
| R09 | Verify/reconcile retained recovery | Blocked | Requires authorized guest access/power consent. Opened read-only review only. BUG-0001 execution and root cause not retested. |
| R10 | Enrollment, first setup, driver transfer, credentials, profile deletion, interruption recovery | Not Tested | Outside this bounded session; no authorized setup/destructive effects. |
| R11 | Minimum size, DPI changes, screen-reader speech and enabled slider keyboard operation | Not Tested | Only existing window size and Windows accessibility observations captured. |
| R12 | Restore session preferences/selection and close | Passed | VM2 restored, Follow Windows restored, all VMs Stopped and hold visible; no operation running before close. No remaining app windows after graceful close. `17-final-restored-vm2`, `18-closed.json`. Final VM/power state is GUI readback, not an independent host query. |

## Development handoff

One new reproducible P3 accessibility defect is recorded in [the slider report](2026-10-14-independent-slider-accessible-units.md). No fix was implemented and no existing case was closed. The handoff is this repository report; no external message was sent.

Next live session requires explicit target/effect authorization for the retained VM1 verification recovery, guest credentials supplied locally, evidence capture and stop conditions. Preserve the hold until that authorized sequence is prepared. Then independently retest full workflows and remaining installed fixes.
