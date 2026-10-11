# First-release roadmap

This document owns release requirements and milestone gates. [Backlog](BACKLOG.md) owns task order and status; [baseline](BASELINE.md) records source findings. A milestone closes only when its implementation and relevant validation criteria are satisfied. Finishing one task does not close a milestone.

## Release target

A usable GUI for existing Generation 2 Windows 11 VMs sharing an NVIDIA GPU. At least two guests must run hardware-rendered workloads concurrently. The existing Slint design and shared Rust backend remain the foundation. Exact Windows versions are selected before qualification; the user's current NVIDIA hardware is the initial target, not a claim of support for every NVIDIA GPU.

| Requirement | Acceptance | Owning tasks |
|---|---|---|
| R1: setup and pairing | A complete package enables reviewed setup and additive enrollment of two existing VMs without replacing the first VM's authority | V1-006, V1-015, V1-016 |
| R2: independent GPU control | Enable, disable, and Reapply/Update operate on the selected VM; another enrolled/rendering VM keeps working | V1-007, V1-014, V1-015 |
| R3: saved settings and truthful state | Saved intent survives reopen; drafts, observed attachment, verification, and enrollment stay distinct; external changes block stale approval | V1-008, V1-009, V1-015 |
| R4: hardware verification and sharing | Both guests identify the selected hardware and produce checked hardware-rendered results during an overlapping workload interval | V1-007, V1-014, V1-015 |
| R5: safe failure and recovery | Uncertain work blocks new effects; readback/reconciliation resolves only established outcomes; save-only retry does not replay GPU work | V1-010, V1-011, V1-012, V1-013, V1-015 |
| R6: usable release | Existing GUI remains responsive and understandable; a reproducible gate, complete package, support statement, and troubleshooting instructions are available | V1-002, V1-005, V1-009, V1-013, V1-016, V1-017 |

Resource isolation, fairness, non-NVIDIA preparation, Linux guests, VM creation, GPU replacement, pairing removal, and GUI import/export are outside first-release acceptance. Current unsupported controls stay unavailable. Future work requires a new agreed requirement.

## Milestones

| Milestone | Work | Completion gate |
|---|---|---|
| B1: trustworthy baseline | Fresh guidance, reproducible non-hardware gate, qualification environment selection, accurate help/source comments | Documentation checks pass; exact environment proposal is recorded; baseline automated gate runs successfully and its limits are recorded |
| G1: complete GUI workflow | Existing setup/enrollment, review/apply/verify, saved settings, drafts, and allocation presentation | Relevant automated checks pass at milestone completion; explicit mock preview confirms appearance/interactions; live correctness remains a separate qualification claim |
| S1: reliable operations and recovery | Trust/IPC review, interruption and save-failure coverage, bounded transport/progress diagnostics | Non-hardware failure scenarios and gate pass; unresolved security/recovery findings block qualification; live containment claims remain unproven until authorized tests |
| Q1: concurrent sharing qualification | Reviewed two-VM tooling and authorized NVIDIA/Windows guest trials | R1-R5 pass on the recorded environment, including independent operation while the other guest renders; evidence records stable terminal outcomes and no observed host lockup during the defined runs |
| P1: release readiness | Package, notices, clean setup/upgrade checks, final support and troubleshooting documentation | Package hashes and required files are checked; final automated gate and separately authorized package trials pass; R1-R6 have evidence and all release blockers are resolved |

Delivery order is B1 → G1 → S1 → Q1 → P1. Each milestone has its own gate; validation is normally deferred until that milestone is ready to close. Packaging work may be prepared earlier when needed for setup trials, without claiming P1 completion.

## Current position

The documentation reset (V1-001) is complete with static documentation validation only. B1 remains open; its product gate has not run in this reset. All later milestones remain open. Existing implemented code reduces the amount of new implementation needed, but is not carried forward as completed milestone evidence.

## Release evidence

[Testing](TESTING.md) defines evidence and scenarios. Qualification establishes the recorded hardware/software/workload combination and observation interval; it cannot establish universal stability or resource enforcement. Failed or interrupted runs retain their evidence and require an explicit next action rather than automatic retry.
