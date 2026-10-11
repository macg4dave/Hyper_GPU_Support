# Product roadmap

This page owns product scope and milestone gates. [Backlog](BACKLOG.md) owns the work queue; [baseline](BASELINE.md) records the gap between current source and the requested behavior. This plan replaces the previous roadmap following the user Q&A on 11 October 2026.

## Product

A simple GUI for enabling or disabling GPU support on supported existing Hyper-V VMs and selecting their GPU VRAM amount. Hyper-V remains responsible for creating VMs, general VM management, and system RAM.

Keep the existing Slint design and shared Rust backend. The initial target is NVIDIA hardware and existing Generation 2 Windows 11 guests, including at least two guests using one GPU concurrently. Exact supported host, guest, GPU, and driver versions are established through qualification.

## Required behavior

| Requirement | User-visible result | Tasks |
|---|---|---|
| R1: simple controls | GPU on/off toggle and VRAM range from 1 GB to the selected card's physical capacity; range changes with the card | APP-002 |
| R2: reviewed changes | Edits stay pending. Apply opens a panel naming the VM, actual changes, power transitions, and relevant time warnings. A second Apply confirms | APP-003 |
| R3: initial setup | App prepares GPU support, manages required starts/shutdowns, verifies it works in that VM, reports pass/fail, then leaves the VM off after completed setup | APP-004 |
| R4: clear credentials | Before a local prompt, explain which VM needs its Windows credentials and clearly distinguish them from host credentials; remembering is optional | APP-005 |
| R5: ordinary changes | Later toggle/VRAM changes use review and Apply without repeating full setup or initial rendering verification; report the real operation result | APP-006 |
| R6: useful, reliable product | Saved settings and current state remain truthful; selected-VM changes preserve other VMs; failures retain recovery; the package and instructions work on the qualified environment | APP-007, APP-008, APP-009, APP-010; rollback follow-ups APP-016 through APP-023 |

The VRAM control is a requested amount. Its displayed range alone does not establish reservation or enforced isolation. Do not invent provider mapping to make the slider appear functional. No automatic balancing, full-capacity warning policy, or additional allocation rules are required now.

VM creation, general VM controls, guest system RAM, new advanced Compute/Encode/Decode workflows, broad hardware support, and other management features are outside this delivery. Existing extra source paths do not automatically become release requirements.

Legacy mock/snapshot GUI tooling is retired and removed from source through APP-001; validation is pending. Do not use it for previews or maintain its scenarios.

## Milestones

| Milestone | Completion gate |
|---|---|
| M1: implement the agreed workflow | APP-001 through APP-007 are implemented; APP-008 runs the appropriate non-hardware checks at this milestone's completion and records their outcomes |
| M2: validate on real VMs | APP-009 passes the separately authorized setup, toggle, VRAM, lifecycle, and concurrent-sharing scenarios on the recorded environment; relevant live GUI interactions are checked |
| M3: deliver a usable package | APP-010 produces a complete package and accurate instructions; final automated checks and separately authorized installation trials pass |

Implementation and validation are recorded separately. No milestone is closed by this documentation rewrite. Do not run builds or tests after individual tasks unless explicitly requested. Mock GUI checks are not part of any gate.

M2 source preparation is recorded in the [qualification sequence](evidence/m2-preparation.md). APP-011 source now removes the metadata-derived 1 GB VRAM ceiling from the slider, shared admission and native writer; byte requests retain ordering, fresh capability and exact readback checks. Review shows exact byte triples. Non-hardware and authorized live allocation qualification remain pending.

## Current position

**M1 is closed** on 11 October 2026: APP-001 through APP-007 are implemented, follow-up source review is complete, and APP-008's non-hardware gate passed. See [closure evidence](evidence/m1-closure.md) for revision, commands, results and exclusions. Initial setup includes graphics verification and confirmed graceful final shutdown. Ordinary changes avoid guest work and require final-off readback; invalid preparation is reviewed as setup/update. Saved, pending, observed and applied state remain separate; recovery permits bound save-only retry without replay. M2/APP-009 live qualification began on one user-selected VM; the user subsequently confirmed successful VM setup, with the recorded issues accepted as follow-ups. See [live findings and handoff](evidence/live-gui-check.md). Allocation qualification is deferred pending APP-011; concurrent sharing remains untested under the one-VM authorization. No render, application launch, credential prompt, guest operation or hardware qualification ran for this milestone.

Final power policy confirmed on 11 October 2026: ordinary changes always leave the VM off after successful completion, regardless of its starting power. Initial setup and required setup/update also finish off. No automatic final restart is required; the user starts the VM through Hyper-V.

The checkpoint rollback report adds an R6/M2 follow-up: host authority and historical setup receipts must not certify current guest configuration. Explicit verification must report differences/unknown state, with separately reviewed reinstall, GPU removal and inactive-pairing removal paths. [Source investigation](BASELINE.md#checkpoint-rollback-investigation) and APP-016 through APP-023 track the incremental fixes. APP-016 presentation, APP-017 host comparison, APP-018 explicit guest-payload comparison, APP-019 durable latest-check/stale-success handling and APP-020 explicitly reviewed reinstall source are implemented; removal actions remain open. APP-018's all-target compilation check passed; APP-019 and APP-020 have not been compiled or tested; APP-020 visual acceptance is also pending. Tests and live acceptance remain pending. The earlier M1 gate does not establish rollback acceptance; live rollback qualification requires its own prepared authorization.
