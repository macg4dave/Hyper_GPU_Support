# Roadmap to v1.0

**Current milestone: M1.** M0 exit evidence is complete; advance only on the M1
exit evidence below. The
[backlog register](BACKLOG.md#task-register) alone owns task status, priority,
milestone membership and task dependencies; follow its links for executable cards.

Build reproducible GPU-PV for Windows 11 x64 host/guest and one NVIDIA RTX 5060
8 GB. Start with one disposable Generation 2 VM based on an immutable clean parent, native Windows management
and a Rust CLI/library. Easy-GPU-PV is the primary implementation reference for
normal Hyper-V configuration and driver provisioning; AppSandbox is secondary.
The Rust foundation implements lifecycle, attachment and package staging. The
full-copy Easy-GPU-PV experiment sustained Code 0 and passed NVIDIA, D3D11/D3D12
and CUDA computation; complete Rust provisioning/recreation and CUDA identity
correlation remain to be qualified. HCS-owned
guest work is paused. CORE-019 provides the hardware-independent
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
  Easy-GPU-PV's normal Hyper-V configuration and provisioning are the primary
  reproduction baseline; AppSandbox is a secondary technical reference.
- Validation: read-only queries distinguish denial from absence; inspect artifact
  provenance and setup side effects; define host controls and expected probe output.
- Exit: CORE-019/001 establish the build/test and read-only inventory foundation;
  HV-001/003, REF-001/002 and GPU-002/008/003 identify exact environment,
  workloads, inputs, parent/child targets, privilege boundary and recreation path.
  All M0 required cards completed; no protected setup implied by this gate.

## M1

**Objective:** produce the first Rust-driven GPU-PV demonstration on the pinned
disposable VM through autonomous implementation and target testing.

- Dependency: M0. Normal runner, Hyper-V, GPU-PV and guest changes on the designated
  disposable VM proceed without another permission gate.
- Critical path: the fixed runner, guest transfer/package staging and exact GPU
  attachment foundation (CORE-005/008/009/002) is complete. GPU-009 now owns
  Easy-GPU-PV discovery -> full manifest/destination comparison -> complete ordinary
  file copies on a clean disposable child -> matching relevant Hyper-V settings ->
  sustained NVIDIA Code 0. Immediately run nvidia-smi and the checked standalone
  D3D11/D3D12/CUDA probes (GPU-005), then refine the proven Rust provisioning path
  and record reproducibility (GPU-006). Do not minimize files before readiness.
- Validation: verify the pinned parent/child/GPU identities immediately before a
  mutation, verify attachment and guest device/runtime readiness, reject software
  rendering, check probe outputs, and leave the VM in a known recoverable state.
  GPU-009 has passed the sustained Code 0 gate on the full-copy baseline. Preserve
  that working guest; the next gate is complete Rust provisioning, explicit CUDA
  identity correlation and attach/start/probe/graceful-shutdown reproducibility.
  Transient Code 0 is insufficient.
- Exit: GPU-006 records passing D3D11, D3D12 and CUDA workloads through the
  Rust-driven path with exact versions, manifest, configuration and checked output.
  Only the critical-path cards above gate M1.
- Diagnostic/reference work: GPU-004 is useful when a native failure needs comparison
  with AppSandbox, but its current guest-access blocker does not hold up the native
  vertical slice. HCS-owned-guest work is paused and outside the critical path.
  Use AppSandbox selectively after the normal Hyper-V baseline is reproduced.
- Deferred hardening: generalized planning, broader audit/locking, probe integration,
  repeated lifecycle cycles, resource tuning and extended diagnostics belong in M2.

## M2

**Objective:** turn the proven path into a usable, repeatable GUI-independent
CLI/core and harden the boundaries exercised by M1.

- Dependency: M1, including the measured vertical slice and backend decision.
- Work: add reviewable planning/preflight (CORE-006), generalized audit/locking
  (CORE-007), CLI probe integration (CORE-003), repeated lifecycle/resource
  qualification (GPU-010/011), detach/reset, diagnostics, CI coverage and
  operator-quality reporting around the proven M1 path. CORE-013 extends the
  Windows checks established by CORE-019/001. CORE-021 incrementally removes
  remaining environment assumptions without gating the vertical slice.
- Risks: credential handling, external state drift, privileged-runner version/policy
  drift, parent/child identity mistakes and conflicting operations.
- Validation: Windows CI for pure logic and fake adapters; native integration and
  the M1 probes on the designated target. No hosted-CI claim of GPU coverage.
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

The completed preparation has already identified the RTX 5060, pinned the disposable
VM/parent, built the probes and established the Rust configuration/runner foundation.
The discovery, comparison and clean-child full-copy experiment below passed on
2026-10-05. Preserve the working guest while completing the remaining Rust path:

1. GPU-009: use the completed Rust runner/transfer/package/attachment foundation;
   discover the full Easy-GPU-PV file closure and compare every logical destination.
2. GPU-009: recreate the disposable child, provision all discovered files with
   ordinary copies, match relevant VM/resource settings, and verify filesystem
   parity plus sustained NVIDIA Code 0. Keep HCS paused.
3. GPU-005: immediately test nvidia-smi and checked D3D11/D3D12/CUDA through the
   RTX 5060. Use AppSandbox as a secondary reference for a measured API failure.
4. GPU-006: integrate the full discovered manifest into Rust provisioning,
   reproduce the passing configuration with checked lifecycle evidence, and only
   then evaluate payload reduction. GPU-005 retains the CUDA LUID correlation gap;
   its vector-add workload passed independently.

After that proof, complete M2 hardening and then the release milestones. Do not split
minor fixes, documentation drift or implementation choices into new cards. Create a
task or blocker only for schedulable product work or a concrete technical/safety
impediment. Use task dependencies, not numeric ID order, and keep checks proportional
to the operation's risk.
