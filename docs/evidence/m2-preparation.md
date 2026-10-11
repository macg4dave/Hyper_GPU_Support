# M2 preparation — 11 October 2026

Status: source preparation started; no new live sequence authorized or executed. The current [roadmap](../ROADMAP.md) defines M2 as APP-009 real-VM qualification, including APP-011 allocation qualification. This is distinct from historical milestones with the same name.

The subsequent [source review](m2-source-review.md) fixed allocation-order diagnostics and fresh adapter binding, and recorded APP-014 slider/triple semantics and APP-015 sustained concurrency evidence as larger follow-ups. The user deferred tests; added regressions remain unrun.

## Source assessment

M1's non-hardware gate passed. The selected-VM setup is [accepted by the user](live-gui-check.md#user-reported-completion); do not repeat setup merely to reconfirm it. APP-012 readable progress and APP-013 bounded 4 MiB transfer changes exist in source but their subsequent runtime checks remain pending.

APP-011 ceiling-removal source is implemented; acceptance remains pending. The user explicitly requested removing the mistaken 1 GB restriction. The slider, shared GUI/planner admission and native writer no longer compare VRAM byte requests against reported partition range bounds. Requests are sent unchanged. Fresh complete capability, ordered triples, target binding, protected admission and exact independent readback remain required. Compute/Encode/Decode keep their provider range checks. The requested 1 GB slider minimum and independently detected physical upper endpoint remain; saved requests are not rewritten.

The earlier preparation assessment overstated uncertainty about units. Microsoft's [Set-VMGpuPartitionAdapter reference](https://learn.microsoft.com/en-us/powershell/module/hyper-v/set-vmgpupartitionadapter?view=windowsserver2025-ps#-maxpartitionvram) explicitly documents VRAM in bytes. Its [GPU-PV example](https://learn.microsoft.com/en-us/windows-hardware/drivers/display/gpu-paravirtualization) reports TotalVRAM and MaxPartitionVRAM of 1,000,000,000; this metadata is no longer imposed as a physical-card ceiling by the product. This source policy follows the user's explicit request; actual driver acceptance and allocation behavior still require live qualification. No ratio or enforcement guarantee is inferred.

Apply review displays exact observed/planned byte triples. Regression source covers multi-GB requests above reported metadata, exact endpoints, retained requests, unknown observations, omission/no-write intent, invalid ordering, other-category range rejection, driver rejection and independent readback failure retaining recovery. These tests have not run.

## Prepared sequence

Complete and review the following environment record before requesting execution. Candidate guests from [testing](../TESTING.md#available-hyper-v-test-vms) are `win_26300.9457  VM1` and `win_26200.9457 VM2`; friendly names are candidates only. Bind the user-accepted setup to its actual GUID before selecting it as the first target. No candidate name authorizes access or effects.

| Required fact | Evidence to record before effects |
|---|---|
| Source and installed artifacts | Revision, working-tree diff, binary hashes, package/build identity; identify which source follow-ups the installed build contains |
| Host | Windows build, Hyper-V capability and host responsiveness baseline |
| GPU | Exact interface, PCI identity, NVIDIA model, signed driver version, independently matched physical bytes, raw TotalVRAM and category triples |
| Guest A and B | Stable GUID, friendly name, Windows build, Generation 2, current power, attachment, saved intent, preparation/graphics receipts, enrollment and recovery state |
| Allocation requests | Concrete requested bytes/triples within the selected physical slider range; record the driver result and independent readback without claiming reservation or enforcement |
| Workloads | Fixed guest hardware-render workload/command, hardware adapter identity and success criterion; confirm how it stays active for simultaneous observation |

Use the existing inventory/state paths for authorized read-only collection; do not add a parallel allocation backend. Record ordinary-operation baselines independently for both guests. If a recovery hold exists, retain it and stop the proposed sequence; reconciliation needs its own reviewed action.

| Step | Proposed effects and acceptance |
|---|---|
| 1: source gate | At the agreed checkpoint, inspect and run the existing non-hardware checks against the latest source, including progress/transfer and new review regressions. Record actual outcomes before selecting the live binary |
| 2: GUI and persistence | Separately authorize launch/reopen; check selected names, review/cancel, scrolling, pending/saved/observed labels and settings persistence without submitting operations |
| 3: ordinary target A | With valid receipts, separately review disable and enable; confirm each operation. Permit graceful shutdown and allocation/attachment changes only as reviewed; require exact readback and final Off. No guest prompt/preparation/initial verification should occur |
| 4: second target setup | If B lacks valid preparation, separately review additive enrollment and then initial setup. Permit local guest credentials, bounded transfer, starts, graceful shutdown, attachment/settings and graphics verification. Success includes B Off; preserve A's intent/enrollment/state |
| 5: explicit allocation | After the source gate, review concrete GB requests and their exact byte triples before separate authorization; record Hyper-V acceptance/rejection and independent settings readback. No automatic retry |
| 6: simultaneous rendering | Separately authorize starting both guests and the agreed checked workloads. Proposed overlap is 10 minutes, with timestamped hardware/results/host responsiveness observations at start, each minute and end. This interval needs approval before execution |
| 7: isolation | During the authorized overlap, review one ordinary change on A, including its shutdown. Check B's attachment/settings and continuing hardware workload against baseline; B is outside A's operation target |
| 8: finish | Separately authorize graceful shutdown of the running qualification guests; proposed final state is both Off. Record terminal outcomes, independent power/settings readback and saved intent |

Guest inspection/workload commands, exact GUIDs, requests and installed binary must be filled in from the authorized environment assessment before the effect sequence is reviewable. Preparation alone does not authorize any step above. Do not run APP-013's repeated setup/transfer benchmark as part of M2; accepted setup remains accepted.

## Evidence and stop conditions

Use ignored `local/evidence/m2-<timestamp>/` for non-secret inventory, artifact hashes, approved run sheet, timestamped per-step outcomes, independent readback and workload results. Write and flush stage boundaries incrementally. Never record credentials. Promote only a concise reviewed result to tracked evidence.

Stop at the first missing/contradictory terminal result, unexpected lifecycle, failed readback, workload failure, host responsiveness failure, changed approval inputs or recovery hold. Retain evidence and the actual state. No automatic retry, forced power-off, host restart, hold clearing or unrequested post-run investigation. A stopped sequence leaves its uncompleted scenarios pending.

Validation of this preparation and ceiling removal: targeted Rust formatting, static source/diff review, `python scripts/testing/check-docs.py` (28 Markdown files, 13 task cards, zero errors), and `git diff --check` passed. No build, test execution, render, GUI launch, credential prompt, guest/hardware access, enrollment or protected write was performed. M2 remains open.
