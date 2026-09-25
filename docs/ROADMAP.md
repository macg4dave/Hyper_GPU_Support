# Roadmap to v1.0

**Current milestone: M1.** M0 exit evidence is complete; advance only on the M1
exit evidence below. The
[backlog register](BACKLOG.md#task-register) alone owns task status, priority,
milestone membership and task dependencies; follow its links for executable cards.

Build reproducible GPU-PV for Windows 11 x64 host/guest and one NVIDIA RTX 5060
8 GB. Start with one disposable Generation 2 VM based on an immutable clean parent, native Windows management
and a Rust CLI/library. AppSandbox is a pinned technical reference. There is no
GPU-PV implementation yet. The user's successful AppSandbox use on this host is
the known-working HCS reference; project tests must reproduce it but do not need to
re-prove general hardware feasibility. CORE-019 provides the hardware-independent
foundation and [build commands](../README.md#windows-development).

## Version 1.0 contract

The user selected D3D11/D3D12 plus CUDA and one guest on 2026-09-24; see
[DEC-006](DECISIONS.md#dec-006). A release describes an exact **project-tested
configuration**, not Microsoft/NVIDIA certification of client GPU-PV.

| Class | Deliverable / boundary |
|---|---|
| Essential | Actual D3D11 and D3D12 hardware rendering with checked output, plus CUDA allocation/transfer/kernel correctness in one guest. Explicit GPU selection; versioned configuration; preflight and reviewable plan; verified assignment and driver staging; status, diagnostics, start/shutdown/restart, removal and recoverable failed changes. Reproducible package/build, operator guide, limitations and release evidence. |
| Desirable if measured | Native OpenGL/Vulkan/OpenCL, DirectCompute/DirectML, hardware encode/decode per codec, graphics/compute interop, and improved presentation through existing Windows tools. Report each independently; none is an implicit v1.0 gate. |
| Experimental | Concurrent GPU-PV guests, resource enforcement/fairness, vendor-extension differences, NVAPI/NGX/DLSS, Optical Flow/OptiX, advanced ray-tracing/Tensor workloads and any required compatibility hooks. A shim may be essential only when necessary for an essential workload. |
| After v1.0 | GUI, general VM/OS installation, custom display driver/streaming, remote control plane/service, automatic host driver/Windows updates, live migration/HA, broad GPU/OS support and 32-bit application coverage. |

Release qualification covers one pinned host/guest edition/build pair, exact driver
and runtime/probe versions. Changed combinations are unvalidated until requalified.
No automatic promise of all VRAM, physical display outputs, fixed resource shares,
all NVIDIA APIs, live checkpoint/save/restore or vendor management access.
Optional failures stay visible in the compatibility report without holding the
release hostage. An essential failure stops the gate; narrowing that contract
requires the user, not a task-status edit.

## M0

**Objective:** prepare the real target and start the smallest Rust vertical slice.

- Entry: documentation-only workspace; preserve existing local changes.
- Work: hardware-independent scaffolding (CORE-019), environment/interface inventory,
  pinned reference and runnable artifact preparation, selected-driver manifest,
  probe definitions, golden-image/disposable-child procedure, controlled-elevation
  contract and read-only Rust inventory (CORE-001).
  Task cards are indexed under M0 in the [register](BACKLOG.md#task-register).
- Research gates: [technical gaps](ARCHITECTURE.md#technical-gaps-and-research-gates)
  G1-G6. Unknown native interface availability, guest access, driver provisioning
  or reference build/signing dependencies must be resolved or explicitly blocked;
  AppSandbox's working HCS result is the baseline, not a question to reopen.
- Validation: read-only queries distinguish denial from absence; inspect artifact
  provenance and setup side effects; define host controls and expected probe output.
- Exit: CORE-019/001 establish the build/test and read-only inventory foundation;
  HV-001/003, REF-001/002 and GPU-002/008/003 identify exact environment,
  workloads, inputs, parent/child targets, privilege boundary and recreation path.
  All M0 required cards completed; no protected setup implied by this gate.

## M1

**Objective:** produce the first Rust-driven GPU-PV demonstration on a disposable VM.

- Dependency: M0. Execute only specifically authorized guest/host changes.
- Work: immutable clean parent plus disposable child; AppSandbox reference capture;
  minimal configuration, native adapters and controlled privileged runner; guest
  transfer/runtime staging; explicit assignment; D3D11/D3D12/CUDA probes; native/HCS
  comparison only where behavior differs; resource headroom and child recreation.
- Risks: runner policy is too broad or agent-writable; parent image is accidentally
  targeted; upstream needs disallowed signing changes; native VMMS lacks a specific
  HCS capability; CUDA or D3D fails; resource fields do not enforce requested limits.
  Record actual impediments as blockers.
- Validation: same essential probes, files and builds for reference/native runs;
  distinguish management, staging, runtime, renderer/session and workload failures.
  Verify fixed privileged policy/identity checks, at least five clean stop/start
  cycles, guest reboot, deliberate child discard and recreation from unchanged parent.
- Exit: GPU-006 records essential workload passes through the Rust-driven path,
  reproducible preparation, conservative single-guest settings, tested disposable
  recreation and one justified backend/component choice. All required M1 cards
  complete. Optional API results may fail or remain untested with reasons.
- If the reference cannot run or an essential reference capability cannot be
  reproduced, stop and present evidence and options to the user. Do not silently
  switch to an untested backend or lower the essential gate; see DEC-007.

## M2

**Objective:** complete the proof into a usable, GUI-independent CLI/core.

- Dependency: M1, including the measured vertical slice and backend decision.
- Work: finish detach/disposable reset, lifecycle, diagnostics, CI coverage and
  operator-quality reporting around the M1 configuration, adapters, staging,
  assignment and probe path. CORE-013 extends the Windows checks established by
  CORE-019/001.
- Risks: credential handling, external state drift, privileged-runner version/policy
  drift, parent/child identity mistakes and conflicting operations.
- Validation: Windows CI for pure logic and fake adapters; native integration and
  the M1 probes on the authorized target. No hosted-CI claim of GPU coverage.
- Exit: all M2 cards completed; CLI follows a reviewed configuration through plan,
  apply, inspect, validate, detach and disposable reset. Reapplying the same state
  is a no-op; wrong VM/GPU/parent, stale plan or failed assignment cannot report
  success. Guest credentials never enter persisted configuration/logs.

## M3

**Objective:** make the single-guest path safe to operate repeatedly and maintain.

- Dependency: M2. Optional research can use the earlier dependencies in its card.
- Work: interrupted-operation recovery, compatibility drift and restaging,
  pressure/endurance qualification, controlled driver transition and security review.
  GPU-007 is a bounded, conditional improvement, not an undefined release gate;
  GPU-015 is the optional concurrent-guest experiment.
- Risks: host/guest driver skew, pending reboots, device loss, resource exhaustion,
  stale backups and manual changes while an operation is pending.
- Validation: failure injection at each mutation boundary; bounded workloads on
  real hardware, restarts and host reboot; fresh probes after authorized maintenance.
  Counts and failure thresholds live in the owning GPU/CORE cards.
- Exit: required M3 cards completed, all essential probes remain passing, interrupted
  changes have tested recovery, and incompatible/unknown versions cannot be silently
  treated as validated. No unresolved defect that risks data loss, wrong-target
  mutation, credential exposure, security weakening or essential workload failure.
  Optional research findings are documented, not release promises.

## M4

**Objective:** qualify a release candidate from distributable, documented inputs.

- Dependency: M3's required gate; experimental cards need not complete.
- Work: owner-selected release/license/signing policy, provenance/notice audit,
  stable configuration and reporting contract, packaging, operator guide and a
  fresh-guest rehearsal using only published candidate instructions.
- Risks: third-party redistribution terms, required unsigned privileged components,
  missing build inputs, machine-specific paths and unrepeatable guest preparation.
- Validation: clean Windows build/package smoke test; inspect payload and notices;
  independent fresh-guest apply/probe/remove/recovery run on the defined hardware.
- Exit: all M4 cards completed. Candidate is traceable to a project revision and
  pinned dependencies, contains no proprietary driver/OS payload, and has checksums,
  example configuration, tested operations guide and exact compatibility evidence.
  Required signing/redistribution decisions must be resolved before artifact creation.

## M5

**Objective:** deliver and close v1.0 against the agreed contract.

- Dependency: M4; candidate changes invalidate affected qualification evidence.
- Work: release gate audit, defect/limitation triage, final artifact verification,
  release notes and maintainer handover. Publication needs the owner's exact target
  and authorization; a draft package/report can be prepared first.
- Validation: DOC-005 traces every essential requirement to candidate evidence;
  DOC-006 verifies delivered artifacts, hashes and links against that audit.
- Exit: no open essential blocker; owner-approved distribution and artifacts match
  the audited revision; v1.0 notes identify measured configurations, unsupported and
  untested cases, known optional defects, recovery and requalification procedure.
  Delivering an artifact does not certify future driver/Windows versions.

## Execution guidance

The critical sequence is inventory/reference map -> golden image and controlled
runner -> thin Rust inventory/config/native adapter -> stage/assign/probe on a
disposable child -> isolate any VMMS/HCS difference -> harden -> candidate rehearsal
-> release audit. REF-001 can run alongside HV-001; CORE-001 starts as soon as
HV-001 supplies exact identities instead of waiting for the full hardware milestone.
Use task dependencies, not numeric ID order, to select the next session.

Every required card in a milestone contributes to its exit; GPU-007 and GPU-015 are
explicit optional lanes and never dependencies of a release gate. Split newly found
defects into permanent cards with a workload and acceptance criteria. Avoid calendar
estimates until M1 establishes whether the essential path is feasible. If an early
research gate fails, repair the evidence-backed cause or obtain a scope decision
before spending effort on dependent architecture.
