# Roadmap to v1.0

**Current milestone: M0.** Advance only on the exit evidence below. The
[backlog register](BACKLOG.md#task-register) alone owns task status, priority,
milestone membership and task dependencies; follow its links for executable cards.

Build reproducible GPU-PV for Windows 11 x64 host/guest and one NVIDIA RTX 5060
8 GB. Start with an existing dedicated Generation 2 VM, native Windows management
and a Rust CLI/library. AppSandbox is a pinned technical reference. There is no
GPU-PV implementation yet. CORE-019 provides the hardware-independent foundation
and [build commands](../README.md#windows-development). Research is not hardware proof.

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

**Objective:** prepare a source-grounded, executable experiment on the real target.

- Entry: documentation-only workspace; preserve existing local changes.
- Work: hardware-independent scaffolding (CORE-019), environment/interface inventory, pinned reference and runnable artifact
  preparation, selected-driver manifest, probe definitions and recovery procedure.
  Task cards are indexed under M0 in the [register](BACKLOG.md#task-register).
- Research gates: [technical gaps](ARCHITECTURE.md#technical-gaps-and-research-gates)
  G1-G6. Unknown edition, interface availability, guest access, driver provisioning
  or reference build/signing dependencies must be resolved or explicitly blocked.
- Validation: read-only queries distinguish denial from absence; inspect artifact
  provenance and setup side effects; define host controls and expected probe output.
- Exit: CORE-019 establishes the build/test foundation; HV-001, HV-003, REF-001/002, GPU-002/008/003 results identify exact
  environment, workloads, inputs, named targets, privileges and restoration.
  All M0 required cards completed; no protected setup implied by this gate.

## M1

**Objective:** reproduce the essential GPU-PV path before choosing implementation.

- Dependency: M0. Execute only specifically authorized guest/host changes.
- Work: clean reference/native guest states; reference results; minimal guest
  provisioning and restoration; native comparison; resource headroom; lifecycle
  and recovery experiments. M1 cards are in the register.
- Risks: upstream needs disallowed signing changes; reference artifact cannot run;
  CUDA or D3D fails; vendor-extension or runtime hooks differ by backend; resource
  fields do not provide reliable enforcement. Record actual impediments as blockers.
- Validation: same essential probes, files and builds for reference/native runs;
  distinguish management, staging, runtime, renderer/session and workload failures.
  At least five clean stop/start cycles plus guest reboot and restoration checks.
- Exit: GPU-006 records essential workload passes, reproducible preparation,
  conservative single-guest resource settings, tested recovery and one justified
  backend/component choice. All required M1 cards complete. Optional API comparison
  results may fail or remain untested with reasons and do not establish support.
- If the reference cannot run or an essential reference capability cannot be
  reproduced, stop and present evidence and options to the user. Do not silently
  switch to an untested backend or lower the essential gate; see DEC-007.

## M2

**Objective:** encode the measured procedure in a usable, GUI-independent CLI/core.

- Dependency: M1, including the recorded backend decision.
- Work: small Rust package, structured native adapters, configuration contract,
  preflight/planning, durable operation journal, guest transfer, manifest staging,
  assignment, restore, lifecycle, diagnostics and probe reporting. CORE-001/002/003
  retain their IDs; new CORE cards split their formerly broad implementation scope.
  CORE-001 extends the initial tests and Windows PR checks from CORE-019 required by
  [engineering standards](ENGINEERING.md#required-checks-and-ci); CORE-013 extends them.
- Risks: elevation and credential boundaries, PowerShell marshalling, external
  state drift, partial file/registry writes and conflicting operations.
- Validation: Windows CI for pure logic and fake adapters; native integration and
  the M1 probes on the authorized target. No hosted-CI claim of GPU coverage.
- Exit: all M2 cards completed; CLI follows a reviewed configuration through plan,
  apply, inspect, validate and remove/restore. Reapplying the same state is a no-op;
  wrong GPU, stale plan or failed assignment cannot report success. Guest credentials
  never enter persisted configuration/logs. Actual build/test commands are documented.

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

The critical sequence is inventory -> exact reference/probe procedure -> hardware
proof -> backend decision -> configuration/transaction core -> hardening -> candidate
rehearsal -> release audit. REF-001 can run alongside HV-001; once the core foundation
exists, configuration, adapters and hardware-free CI can proceed independently.
Use task dependencies, not numeric ID order, to select the next session.

Every required card in a milestone contributes to its exit; GPU-007 and GPU-015 are
explicit optional lanes and never dependencies of a release gate. Split newly found
defects into permanent cards with a workload and acceptance criteria. Avoid calendar
estimates until M1 establishes whether the essential path is feasible. If an early
research gate fails, repair the evidence-backed cause or obtain a scope decision
before spending effort on dependent architecture.
