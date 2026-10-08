# GPU-PV product roadmap

**Reviewed:** 8 October 2026 against the root Rust code, architecture and M1/M2
results. [BACKLOG.md](BACKLOG.md) owns status/dependencies; the [GUI plan](gui_roadmap.md)
owns presentation slices. Findings below are inspection results, not fresh hardware tests.

## Product outcome

Deliver a reusable Rust core, thin CLI and native Windows GUI for existing Hyper-V
Generation 2 VMs. Prepare a selected Windows guest from the current signed NVIDIA
driver, attach one GPU with provider defaults, verify graphics, and disable safely.

- Windows 11 x64/NVIDIA first; RTX 5060 is a test baseline, not a universal claim.
- Multiple VM enrollment and isolated state; simultaneous same-GPU sharing requires
  separate two-VM qualification. No scheduler or fairness promise.
- Preserve disks, CPU/RAM quantities, Secure Boot and security devices. Preview
  required MMIO/cache/checkpoint-policy changes; restore attributable settings.
- Graceful guest lifecycle is part of apply/verify; preserve initial power where
  feasible. Failed graceful shutdown stops mutation. Host lifecycle needs immediate
  explicit permission; never automate host restart.
- No VM creation/reset, golden-parent requirement, fixed driver counts/hashes/versions,
  HCS work or new resident management service in production.
- Reuse root Rust code. DEC-028 permits only the fixed PowerShell Direct transport/
  bootstrap bridge; preparation and application logic remain Rust.
- Advanced allocation and additional vendors follow the defaults-based release.
  Provider values do not establish GiB, percentages, fairness or hard limits.

## Existing implementation: reuse rather than rebuild

| Area | Implemented | Remaining gap |
|---|---|---|
| M1 boundary | Separate root workspace, schema 2, native install/enrollment, protected runner; qualified | M1 stays complete |
| Discovery/selection | Native VM/GPU inventory, multiple targets and exact VM/GPU enrollment | Partial-access reporting; guided selection/enrollment UX |
| NVIDIA preparation | Current-driver discovery, signature/catalog trust, complete payload and Rust guest writer | Explain drift/receipt state; affected qualification |
| Apply/disable | Minimal compatibility changes, defaults, initial-power handling, detach/restoration; current-build repeat passed | Useful operator preview |
| Recovery | Per-VM journals, pending intent, stale-digest invalidation, retry/reconciliation and audit | Clear next actions; no new rollback/reset subsystem |
| Verification | PnP and checked D3D11; running no-op retains uptime | Current versus last verified state in presentation |
| CLI | inventory/install/plan/apply/enable/disable/status/verify/credentials/forget | Readable effects/errors; no new validate command needed |
| GUI | Win32 controls, runner discovery, selection, background apply, credentials/details/config save | Layout, draft/observed split, fresh preview, busy/close/setup handling |
| Allocation | Optional raw VRAM triple, range validation and native setter | Units/enforcement unqualified; no ordinary slider |

Sources: root `src/model.rs`, `workflow.rs`, `runner.rs`, `windows_hyperv.rs`,
`windows_driver.rs`, `guest.rs`, `main.rs` and `windows_gui.rs`.
The standalone lab is historical/contributor tooling, not a product backend.

**Important limits:** status returns observed Hyper-V state and the journal, not a
fresh guest receipt/graphics check. Enable-plan currently discovers/hashes the full
signed payload; it is read-only for VM/guest effects, but is not a cheap dashboard
poll. Runner requests also write protected audit records. Inventory currently fails
the entire request if an individual VM inspection fails; per-row denied/unavailable
state is still work. Do not label these capabilities implemented.

## Milestones

| Milestone | Exit | Current state / owner |
|---|---|---|
| M1: product boundary | Native install/enrollment reviewed and qualified; no lab dependency | Complete; ARCH-001 |
| M2: reliable NVIDIA core | One-VM apply/render/reapply/disable, recovery and stable host | Current-build observed repeat passed; fresh preparation under new limits remains; ARCH-001 |
| M3: native GUI | Written layout, responsive operations, truthful state and CLI parity | Prototype in progress; GUI-001 |
| R1: packaged v1.0 | CLI/GUI/runner/guest/probe candidate, tested guide, no essential blocker | CORE-017, DOC-003, GPU-014 |
| M4: allocation/vendors | Qualified units/readback/enforcement and incremental vendor preparation | Later; GPU-010 and separately scheduled vendor work |

Visual/read-only GUI work can proceed during M2 diagnosis. Live GUI acceptance and
release depend on M2 qualification. Multi-target isolation is required; simultaneous
sharing is a conditional capability, not a defaults-based v1 release gate.

## Small core steps

### C1 — Host-hang investigation closed (BLK-005)

**8 October result:** reviewed revision `96152e7` was rebuilt, installed and passed
an observed default-attach/PnP/D3D11/Off-and-running-reapply/verify/disable sequence.
Graceful cleanup and preservation passed; the user confirmed no slowdown or beeps.
This bounded run did not reproduce the earlier hang or establish its cause. It
reused preparation, so fresh transfer/writing under the new child limits remains
an affected qualification gap. See [M2 evidence](evidence/M2.md).

**Closed by user direction, 8 October:** no further hang investigation, reproduction
campaign or resource-observation requirement is scheduled. Preserve the evidence;
the cause is unknown, not claimed fixed. BLK-005 no longer blocks product work.
Normal development/test authorization and existing operation guards still apply.
Next product step: C2 / CORE-006 shared preview.

### C2 — Useful shared preview (CORE-006, P1)

**Completed 8 October:** typed plan and apply share their validated decisions.
CLI JSON and GUI's fresh pre-apply confirmation use the same effect summary.
Focused parity/read-only/recovery tests and live enable/disable previews passed;
the latter preserved observed/journal state. Full GUI interaction/layout acceptance
remains GUI-001. See [CORE-006 results](BACKLOG.md#core-006).

Reuse target/observed/journal/preparation results to describe attach/detach, settings,
driver drift, credential need, downtime and initial-power restoration. Remove the
unconditional restart implication for running unchanged targets. Keep lightweight
inventory/status separate from full payload validation.

**Acceptance:** enable/disable/running-no-op/pending-recovery previews match workflow
decisions with no VM/guest effects. CLI and GUI use the same summary. Apply rechecks
identity/state; preview cannot broaden protected enrollment.

### C3 — Truthful operator state and errors (CORE-012 / CORE-021, P1)

Distinguish requested state, observed attachment, pending recovery, recorded prepared
digest and last successful graphics timestamp. A past check is not fresh health.
Separate missing/denied/unsupported/unobserved; isolate individual inventory failures
where practical. Never turn failed discovery into an empty successful list.

Explain schema-2 intent versus protected enrollment: changing the VM/GPU pair needs
administrator re-enrollment. Preserve actual commands, improve errors/next actions
and redacted reports. Configuration-save failure after successful apply must not
cause a blind replay. Do not add a second configuration authority.

**Acceptance:** focused partial-access, stale-state, enrollment mismatch and save-failure
checks; help/examples match shipped behavior. No broad event-collection service.

### C4 — Close M2 proportionally (GPU-012)

Reuse M1 review/M2 passes and existing recovery/drift tests. Qualify affected
behavior with one representative bounded retry and current-driver refresh when that
path changed. No manufactured driver upgrade, two recreated children, long stress
campaign or mandatory CUDA/D3D12 gate.

**Acceptance:** stable host, PnP/checked D3D11, no-op uptime, clean disable/settings/power
restoration and understood recovery. Independent review applies to materially changed
privileged boundaries, not repeated review of unchanged code.

## M3: native GUI

Follow [gui_roadmap.md](gui_roadmap.md) under GUI-001:

1. Reuse Win32 and background runner calls; build sidebar/header/five-column table,
   information panel and Apply/Discard footer.
2. Bind inventory/journals; preserve selection, expose eligibility/enrollment and
   unknown state; place GPU selection in details.
3. Stage one VM draft, show C2 preview, execute through the existing runner; handle
   busy/close, credentials, uncertain response and config persistence.
4. Check keyboard/accessibility/DPI/resize and the qualified operator journey.

**Acceptance:** native UI with CLI-equivalent effective results. No unqualified
GiB/percentage slider, implicit sharing, toolkit migration or separate backend.
Use the concrete written design requirements.

## R1: package and accept

- CORE-017: clean locked Windows x64 candidate containing all product binaries;
  unpack/help/setup/missing-prerequisite checks. Reuse native installation and
  document update/re-enrollment/interrupted-install recovery. Removal instructions
  preserve unrelated state; no general installer framework required.
- DOC-003: actual install/config/enrollment, plan/apply/status/verify/disable,
  credentials, drift refresh and pending recovery. No developer paths.
- GPU-014: candidate-only journey on the existing designated disposable VM, including
  no-op and representative recovery. Repeat affected package failures only.
- Include revision/checksums, runtime requirements and notices. Product MSVC CRT is
  statically linked; verify prerequisites rather than require a separate VC runtime.
  Exclude proprietary drivers, media, disks and secrets.

**Acceptance:** M2 + M3, tested guide/useful diagnostics and no essential blocker.
Packaging does not authorize publication.

## M4 and conditional sharing

GPU-010 starts from the existing raw VRAM API: establish units, bounds, readback and
enforcement before exposing ordinary controls. Defaults remain the normal path;
no host partition-count tuning.

GPU-015 qualifies two explicitly designated VMs: independent attach/render/no-op and
disable of one without disturbing the other. Reuse serialization; no scheduler.
Before advertising concurrent sharing, document the core's admission/support policy;
UI-only warnings cannot enforce safety.

Additional vendors need chosen hardware and a concrete signed-payload recipe, then
small adapters and affected qualification. Optional API probes, HCS and recipe
minimization remain separate from production delivery.

## Significant corrections, 8 October 2026

- Replaced repeated implemented work with explicit gaps and reuse points.
- Corrected plan cost, status freshness, GUI toolkit and enrollment assumptions.
- Recorded the host incident as active without inventing its cause.
- Removed old lab-port/reset/CUDA release dependencies; aligned M2/M3/R1.
- Kept multiple-target isolation and made simultaneous sharing conditional.
- Established a concrete written GUI layout and actionable integration steps.
