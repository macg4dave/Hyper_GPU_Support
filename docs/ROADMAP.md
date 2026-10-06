# Roadmap to v1.0

**Current milestone: M1 â€” Rust reproduces the proven baseline.**

Normal Generation 2 Hyper-V GPU-PV already works on the Windows 11 RTX 5060 target.
Sustained Code 0, nvidia-smi, checked D3D11/D3D12 and CUDA computation, complete
driver/runtime provisioning and native Rust manifest discovery are established.
The [project baseline](evidence/GPU-PV-BASELINE.md) records measured facts; the
[architecture](ARCHITECTURE.md) specifies our behavior. No external project lookup
or new feasibility proof is required.

[BACKLOG](BACKLOG.md#task-register) owns task status/dependencies. Completed foundation
work is retained as compact results; its old research dependencies do not gate delivery.

## Version 1.0 contract

One TOML-configured Windows 11 x64 Generation 2 Hyper-V VM, one RTX 5060 8 GB,
Rust CLI, native Hyper-V GPU-PV and the complete discovered driver/runtime recipe.
Configure and validate Windows facilities; do not recreate a VM platform.
Discover the complete associated driver/runtime payload for the selected GPU and
installed signed driver, derive guest destinations and verify every copy. Manifest
size is discovered data; no baseline file count, static NVIDIA list or historical
per-file hash table defines product success. Driver changes require rediscovery
and a new manifest, with explicit restaging and requalification under CORE-015.

Normal product operation, installation and recovery belong in Rust. No manual
PowerShell sequence is required for v1. Development/diagnostic scripts remain
optional. Existing embedded cmdlet/file adapters are migration debt, not a native
implementation merely because Rust launches them. Any retained external interface
needs a narrow technical exception with investigated alternatives, exact calls,
typed validation, bounded execution and error/recovery behavior.
The [script/backend audit](../scripts/PRODUCT-MIGRATION.md) maps every maintained
script and remaining production PowerShell operation to its owner.

Required behavior: runtime configuration and clear preflight/plan/apply/status;
verified staging/settings/assignment, start/shutdown/restart, stable GPU readiness,
automated checked D3D11/D3D12/CUDA, safe removal and recovery of product-owned changes,
explicit driver re-stage, useful bounded diagnostics, repeatability and a traceable
package with tested instructions. The approved on-demand runner is reused;
no GUI, resident application service, multi-VM scheduler or multi-GPU orchestration.

Report exact project-tested host/guest builds, driver/runtime/probe versions and
limitations. Build differences alone are qualification context, not proof of failure.
An essential failure or unsafe operation blocks release; optional capability failures
do not. No promise of vendor certification, resource fairness, every NVIDIA API,
live migration, checkpoints/saved state or graphics/compute interoperability.

| Class | Scope |
|---|---|
| Required for v1 | M1â€“M3 below: full recipe, essential workloads, usable safe operation/maintenance and delivery. |
| Useful if cheap | A specifically requested optional API check (GPU-007), or qualification during an actual safe driver update (GPU-013). Neither gates v1. |
| Post-v1 | Recipe/settings minimisation (GPU-017), resource enforcement/presets (GPU-010), CUDA/D3D LUID correlation/interop (GPU-016), broader GPU/OS support. |
| Experimental | Multi-guest contention (GPU-015), advanced optional GPU workloads and any future compatibility hooks, only for an explicit measured requirement. |

GUI, automatic host updates, general OS installation, custom display/streaming,
remote control plane and an HCS VM platform are outside this product plan.

## Development/test support (outside shipped functionality)

The user workflow is **existing Hyper-V VM -> configuration -> inspect/plan ->
apply GPU-PV and driver provisioning -> verify**. No golden image is required.
Golden-parent copies, disposable VM creation, test-disk preparation/reset, clean
environment rebuilding and cleanup support contributors only. Reuse the existing
fixed laboratory and acceptance harness; do not implement laboratory management
as a v1 product task. Test tooling may use the product, never the reverse.

GPU-006/012/014 remain product qualification gates. Preparing their clean targets
is external test support, not a CLI capability or a native product migration gate.
CORE-025 covers product management; optional native reset helpers are test-only.
CORE-010 covers safe removal and reconciliation on an existing VM, not disk replacement.

## M0 â€” Established foundation

Inventory, protected golden parent/disposable child, strict types, bounded guest
transfer, fixed runner, exact attachment, standalone probes and the complete working
recipe are available. This is completed history, not an active investigation milestone.
Read a specific foundation result only when the selected implementation needs it.

## M1 â€” Rust reproduces the baseline

Independent implementation tasks:

- CORE-022 completed: complete native-manifest staging and verified reapply passed on a clean child; guest file operations still use embedded PowerShell (CORE-026).
- CORE-023 completed: validated VM settings and explicit GPU resources passed live apply and independent verified no-op reapply; Hyper-V read/mutation still uses cmdlets (CORE-025).
- CORE-003 completed: public validate and its Rust worker own readiness and essential workload checks; transfer/launch debt remains CORE-026 and combined live qualification remains GPU-006.
- CORE-024 completed: native registry/system/WMI inventory, exact-target/configuration-bound Rust worker and all 14 read-only parity facts passed. CORE-025's first exact-VM/host-GPU native read slice passed 18-field parity; guest-adapter/profile reads and disk guards continue before mutations.
- CORE-025 ports fixed Hyper-V attachment/resources/settings/lifecycle incrementally while preserving enrollment, readback and reconciliation.
- CORE-026 moves guest file/security/hash/receipt operations to a Rust writer and establishes the smallest justified session/transfer interface, if one is necessary.

GPU-006 runs the combined current workflow on a clean child; its in-progress
qualification remains useful while backend migration proceeds.
**Exit:** verified full files/settings/adapter, sustained Code 0, nvidia-smi,
checked D3D11/D3D12/CUDA, verified reapply and graceful shutdown. Preserve exact
inputs/results in one concise report and independently review implementation.
CORE-024/025/026 must also demonstrate their replacements and remove production
PowerShell logic; repeat affected qualification after changed boundaries. A
successful cmdlet-backed GPU-006 run alone does not close the native migration gate.
Read-only discovery parity and experimental shell provisioning do not complete M1.

Do not reduce the working recipe or demand CUDA/graphics LUID equality.
Changed privileged writer/settings boundaries receive review before deployment;
reuse existing identity, credential, receipt and recovery safeguards.

## M2 â€” Usable operation and maintenance

CORE-006 connects public plan/apply/status; CORE-011 lifecycle and CORE-010 removal/
recovery expose existing operations; CORE-012 gives useful diagnostics; CORE-021
finishes runtime TOML/help/report contracts; CORE-015 regenerates/restages after drift.
CORE-027 supplies native Rust runner installation, enrollment and recovery,
replacing product setup scripts while retaining the existing privilege boundary.
Tasks with independent foundations can start alongside M1; no blanket milestone
dependency postpones work that does not need the clean-child result.

GPU-012 qualifies two independently recreated children, verified no-op, a few guest
lifecycle cycles, representative interruption/recovery, explicit re-stage and bounded
sustained checked workloads. Reuse the existing Windows CI and foundation results.
**Exit:** the integrated CLI performs these workflows repeatably; no unresolved
essential failure, wrong-target/data-loss/credential defect or blocking review finding.
Include demonstrated native inventory/Hyper-V/guest writer and setup replacements;
optional test harnesses cannot provide missing public command behavior.

There is no mandatory host reboot, manufactured host-driver transition, general
transaction engine, resource-fairness investigation or review-of-review task.

## M3 â€” Package and tested delivery

CORE-017 builds a traceable Windows x64 candidate including actual notices and
runtime prerequisites. DOC-003 writes tested operating/recovery instructions;
package and guide work can proceed together after commands stabilize.
GPU-014 follows only those artifacts/instructions on a fresh child, checks required
behavior, incorporates final review and provides release artifacts/handover.

**Exit:** candidate hashes/revision, tested clean workflow, actual compatibility/
limitations and recovery instructions agree; required checks pass and no essential
blocker remains. Exclude proprietary drivers, OS images/disks, secrets and keys.
Resolve actual license/signing/distribution choices at packaging; publish only to an
authorized destination. Local reviewable packaging does not need publication approval.

## Critical path

```text
proven baseline + qualified current staging/settings/worker
  -> native inventory + Hyper-V adapters + Rust guest writer (CORE-024/025/026)
  -> clean-child reproduction + affected replacement qualification (GPU-006 / M1)
  -> one CLI/config + lifecycle/recovery + diagnostics/re-stage + Rust setup
  -> integrated repeatability and implementation review (GPU-012 / M2)
  -> package and guide in parallel
  -> packaged clean-child acceptance, final review and handover (GPU-014 / M3)
  -> v1.0
```

Public CLI integration and the current GPU-006 run can proceed alongside backend
ports. Remove each production script dependency only after its replacement is
demonstrated. Final packaged acceptance must use the Rust CLI with no manual
PowerShell and only specifically justified external Windows interface exceptions.

Ordinary tasks follow read â†’ implement â†’ test â†’ update status. Keep documentation
changes proportional to behavior. Immediately verify target identities before effects;
protect the parent, signing, Secure Boot and isolation. Disposable guest development/
testing is authorized; physical-host restart/shutdown/logout/session termination always
requires explicit permission immediately beforehand.
