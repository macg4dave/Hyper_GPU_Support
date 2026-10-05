# Historical changelog

Archived completed changes; [current delivery plan](../ROADMAP.md) supersedes old research and approval workflows.

# Changelog

## 2026-10-05 — Delivery roadmap after proven GPU-PV baseline

- Rebased M1 on full Rust reproduction of the complete 271-file normal-Hyper-V
  recipe; merged CLI/reliability into M2 and candidate/delivery into M3.
- Completed GPU-005 baseline workload tracking; made GPU-006 ready for writer and
  VM/resource integration. Deferred AppSandbox comparison and optional research;
  closed BLK-003 for v1 scope without claiming its transport was repaired.
- Added post-v1 GPU-016 CUDA identity compatibility and GPU-017 recipe minimisation;
  preserved historical evidence outside the current architecture reading path.
- Updated decisions, agent guidance and implementation/review prompts to assume
  proven GPU-PV feasibility. No implementation or hardware experiments performed.

Record meaningful completed changes in one short entry per change, with task IDs
and evidence links when applicable. Task statuses remain in BACKLOG; partial
handover notes remain in its Resume block. Do not duplicate detailed test output.

## 2026-10-05

- [GPU-009](../BACKLOG.md#gpu-009): reproduced Easy-GPU-PV's complete 271-file copy
  closure and relevant VM/resource settings on a clean normal Hyper-V guest.
  Sustained Code 0, nvidia-smi, checked D3D11/D3D12 frames and CUDA vector addition
  passed. Added exact D3DKMT physical identity selection for GPU-PV probes and native
  Rust read-only driver discovery/manifest construction. CUDA LUID correlation and
  full Rust guest-writer integration remain open. [Evidence and inventory](GPU-PV-BASELINE.md).

## 2026-10-04

- [GPU-009](../BACKLOG.md#gpu-009): changed the guest CUDA runtime alias from a hard
  link to a normal verified copy. A clean disposable-VM restage passed package
  and alias checks; Code 43 persisted. A copied NVML alias and AppSandbox-style
  device cycle narrowed the remaining failure without adding filesystem tricks.
  Evidence: [upstream comparison](GPU-009-upstream-comparison.md).

- [CORE-002](../BACKLOG.md#core-002): added strict idempotent `ensure-gpu`, proved
  matching reapply and zero-to-one RTX 5060 attachment, and fixed a live one-shot
  Task Scheduler teardown race by waiting for native `Ready` state before another
  bounded runner trigger. Evidence: [CORE-002](CORE-002.md).

- Established one authoritative Windows elevation/UAC policy: determine privilege
  before execution, keep ordinary development unelevated, route known privileged
  GPU-PV/Hyper-V work through the bounded runner, classify maintained scripts, and
  retain explicit approval only for physical-host lifecycle operations.

- [CORE-009](../BACKLOG.md#core-009): completed live NVIDIA guest staging, verified
  full-hash no-op reapply and partial-state discard/recreate on the fixed disposable
  VM. Added masked credential feedback and credential-safe fixed preflight phase
  diagnostics after live testing exposed an unelevated Hyper-V query failure.
  Evidence: [CORE-009](CORE-009.md).

- Began CORE-021's configuration/discovery cleanup without adding it to the M1
  gate: inbox Windows PowerShell, System32 tools and the trusted Hyper-V module
  root now derive from the OS-reported Windows directory instead of `C:` or
  `PATH`; the backlog and agent standards now preserve the same boundary.

- Refined CORE-009 build-drift handling: staging now records the qualification,
  measured host and authenticated-guest builds and warns on inequality instead of
  rejecting it. Exact driver, package, signature, target and stable-servicing checks
  remain mandatory.

- Refined CORE-009 servicing qualification after a controlled reboot: active or
  pending replacement servicing and unknown deletions still fail closed, while exact
  reviewed delete-only records are receipt-bound warnings without clearing
  Windows-owned state.

## 2026-10-03

- Extended Windows CI with the existing generated configuration/policy drift and
  documentation consistency checks; corrected stale dependency guidance and added
  hardware-independent staging/guest-copy usage coverage and manifest-hash
  coverage that prevents success-shaped output before the configured digest is
  verified.

- [CORE-009](../BACKLOG.md#core-009): implemented bounded manifest-level NVIDIA guest
  staging with signature/host-drift preflight, protected atomic publication,
  per-file verification, idempotent reapply and disposable-recovery classification.
  Hardware-independent gates pass; live apply stopped safely on BLK-004's host/
  guest Windows-build drift and pending rename marker without mutating the guest.

- [CORE-008](../BACKLOG.md#core-008): completed the bounded PowerShell Direct
  single-file transfer path with ephemeral zeroized credentials, exact host/guest
  identity and hash receipts, protected flat staging destinations, pinned system
  PowerShell/Hyper-V module resolution, strict process status and uncertain-state
  handling. Passed the hardware-independent security/error coverage and proved one
  verified transfer to the designated disposable guest before a graceful shutdown.

## 2026-10-01

- [CORE-005](../BACKLOG.md#core-005): made disposable-child reset fail closed with
  the same durable operation ID and reconciliation boundary used by lifecycle and
  GPU assignment, including timeout, result-publication and final-audit failures;
  exact inspection now also rejects a mismatched attached GPU. Fixed installer
  verification to compare canonicalized Task Scheduler durations, installed the
  pinned runner, and proved exact GPU attach/detach on the disposable VM. Evidence:
  [CORE-005](CORE-005.md).

- Consolidated development authorization in `AGENTS.md`: normal runner, Hyper-V,
  GPU-PV and designated-disposable-VM work proceeds autonomously, while every
  physical-host restart, shutdown, logout or session termination still requires
  explicit permission immediately beforehand. Removed conflicting approval gates
  from engineering, prompt and active execution guidance.

## 2026-09-30

- Centralized mutable machine, disposable-slot, image, runner, tool and test values
  in validated `config/project.toml`. Rust and maintained scripts now consume the
  shared settings, the exact privileged policy is generated and drift-checked, and
  a maintained command refreshes release binary/policy hashes without weakening
  integrity checks. Engineering and agent guidance now enforces the configuration
  versus implementation-constant boundary.

## 2026-09-29

- [CORE-005](../BACKLOG.md#core-005): reconciled retained target evidence showing the
  corrected limited-principal runner passed fixed start and graceful shutdown, then
  added repository-only fixed RTX 5060 attach/detach operations with exact off-state,
  VM/disk/GPU identity and single-assignment checks, durable uncertain-state blocking,
  bounded output/deadlines and typed fake coverage. Installing and exercising the
  broadened runner remains protected. Evidence: [CORE-005](CORE-005.md).

- Simplified the project workflow around the first GPU-PV vertical slice. M1 now
  gates only the fixed runner, guest transfer/staging, exact GPU attachment and
  checked D3D11/D3D12/CUDA demonstration; planning, generalized audit/locking,
  repeated lifecycle qualification, resource tuning and CLI probe integration move
  to M2. Agent and prompt guidance now treats documentation as a record of product
  work, not a prerequisite, while preserving protected-operation approval and
  host/VM safety checks.

## 2026-09-28

- [CORE-005](../BACKLOG.md#core-005): an authorized start trial failed closed before
  `Start-VM` because the measured parent hash exceeded the whole-script deadline;
  read-only reconciliation proved the VM remained off and unchanged. Lifecycle now
  uses separate 300-second inspection and 180/120-second transition limits, aligned
  client deadlines, and a verified ten-minute scheduler cap. Corrected reinstall
  and retry remain protected. Evidence: [CORE-005](CORE-005.md).

- [CORE-005](../BACKLOG.md#core-005): installed the exact reviewed limited-principal
  candidate and passed read-only target inspection, including the 24.8 GB parent
  hash in about 222 seconds, durable audit/result publication, pinned VM/GPU checks
  and the enforced response path. The candidate remains installed; lifecycle and
  GPU mutation were not authorized or run. Evidence: [CORE-005](CORE-005.md).

- [CORE-005](../BACKLOG.md#core-005): an approved limited-principal install proved
  cross-account authenticated request exchange; read-only inspection exposed the
  parent-hash and ineffective client-deadline bounds, then recovery restored the
  original reset-only state. The corrected candidate uses a measured hash bound
  and an enforced bounded-frame response deadline. Evidence:
  [CORE-005](CORE-005.md).

- [CORE-005](../BACKLOG.md#core-005): added the repository-only fixed `start-slot`
  and graceful `shutdown-slot` runner slice with exact enrolled-state and disk-chain
  checks, pinned/single-guest GPU and memory admission, replay/audit/result
  integration, durable uncertain-state blocking, bounded execution and no
  force-off/save fallback. Hardware-free checks passed; installation and VM execution remain
  protected and unperformed. Evidence: [CORE-005](CORE-005.md).

- [CORE-005](../BACKLOG.md#core-005): diagnosed the limited S4U runner's missing
  batch-logon right, safely restored the prior installation, and replaced the
  unsafe whole-template policy proposal with a fixed Rust exact-SID LSA delta and
  symmetric SID-authoritative recovery. A second trial proved limited task launch,
  exposed the cross-account process-token assumption and a helper cleanup defect,
  and drove exact pipe-owner/client-token authentication plus checked recovery.
  Protected reinstallation remains pending.
  Evidence: [CORE-005](CORE-005.md).

## 2026-09-27

- [CORE-020](../BACKLOG.md#core-020): implemented the standalone hardware-only Rust
  D3D11/D3D12 probes, CUDA/DXGI identity companion and unchanged pinned NVIDIA
  CUDA workload with retained hashes/licenses. Bounded failure self-tests and
  the host warm-up plus three measured runs per API passed; independent re-review
  accepted the fixes. Evidence: [CORE-020](CORE-020.md).

- [CORE-004](../BACKLOG.md#core-004): added strict version-one configuration,
  plan/report and CLI operation/error contracts with exact VM/GPU/manifest
  identities and opaque provider resource ranges. Unknown, credential/path,
  duplicate, ambiguous and invalid inputs fail closed; unimplemented commands
  return exit 70 explicitly. Evidence: [CORE-004](CORE-004.md).

## 2026-09-26

- [HV-002](../BACKLOG.md#hv-002): converted the completed Windows 11 VM into a
  hash-protected read-only parent plus fixed-identity disposable child, preserving
  the local account without Sysprep under DEC-015. Retained a full pre-merge export,
  matching parent backup and original merged source; disabled checkpoints, proved
  initial and recreated-child boots, and left the VM off with no GPU adapter.
  Evidence: [HV-002](HV-002.md).
- [CORE-005](../BACKLOG.md#core-005): implemented and installed the first Rust runner
  slice with immutable policy, fixed `reset-slot`, lock/audit/atomic result and a
  highest-privilege on-demand task triggerable from the normal token. Full runner
  operations remain in progress. Evidence: [CORE-005](CORE-005.md).

- [DOC-011](../BACKLOG.md#doc-011): added maintained setup/diagnostics/Hyper-V/testing
  script categories and reusable Rust/documentation check scripts. Centralized the
  shell standard in ENGINEERING and made each host restart, shutdown, logout or
  session termination require explicit permission while retaining normal development
  and verified disposable-guest lifecycle autonomy. Both scripts and all normal
  Rust checks passed; no host or guest lifecycle operation occurred.

## 2026-09-25

- [GPU-003](../BACKLOG.md#gpu-003): completed the reviewable M1 baseline procedure
  with exact roots/slot/settings, immutable-parent and runner boundaries, ordered
  comparison/recovery steps, abort limits and separately scoped protected actions.
  No setup was executed. Evidence: [GPU-003](GPU-003.md).

- [GPU-008](../BACKLOG.md#gpu-008): pinned the essential D3D11/D3D12/CUDA probe
  inputs and defined hardware/LUID identity, exact offscreen image and CUDA CPU
  correctness oracles, toolchains, timeouts and host/reference/native run matrix.
  Optional candidates remain separate and no probe was executed. Evidence:
  [GPU-008](GPU-008.md).

- [REF-002](../BACKLOG.md#ref-002): pinned and verified the signed AppSandbox 0.1.9
  x64 release, its driver catalogs, shims/exports, notices and acquisition recipe.
  Startup certificate/network side effects and the existing untouched reference
  VM candidate are now explicit; no upstream binary or script ran. Evidence:
  [REF-002](REF-002.md).

- [GPU-002](../BACKLOG.md#gpu-002): pinned the exact signed 616.92 DriverStore
  package and x64 runtime dependency closure, defined the byte-preserving native
  staging baseline and separated optional AppSandbox transforms/hooks. Added
  drift, reboot, recovery and no-redistribution rules. Evidence:
  [GPU-002](GPU-002.md).

- [REF-001](../BACKLOG.md#ref-001): added the independent AppSandbox upstream,
  fetched complete history/tags without importing its tree and preserved reviewed
  0.1.9 commit `6f3adb6` under `refs/upstream-reviewed/appsandbox/0.1.9`.
  Independent ancestry and origin were verified; no source review or push occurred.

- [HV-003](../BACKLOG.md#hv-003): captured the installed GPU-P cmdlet/WMI/HCS
  surface, exact RTX selection syntax, raw resource ranges and query-rights
  matrix. Native WMI inventory works unelevated; Hyper-V cmdlets require broader
  rights on this host. No VM exists and no mutation/state rule was inferred.
  Evidence: [HV-003](HV-003.md).

- [CORE-001](../BACKLOG.md#core-001): added versioned read-only Rust inventory with
  exact RTX/GPU-P correlation, typed availability states, bounded/cancellable
  Windows query transport and deterministic failure/selection coverage. Strict
  build/lint/test/doc checks and independent architecture review passed; positive
  VM/GPU-P behavior remains untested. Evidence: [CORE-001](CORE-001.md).

- [HV-001](../BACKLOG.md#hv-001): inventoried Windows 11 Pro 25H2 build
  `26200.9457`, active Hyper-V facilities and the RTX 5060/616.92 driver and
  GPU-P interface. An explicitly approved administrator read-only rerun found
  zero registered Hyper-V VMs and resolved the initial permission blocker without
  changing the host. Evidence: [target inventory](HV-001.md).

- [REF-004](../BACKLOG.md#ref-004): established an official unmodified Windows 11
  ISO baseline after separating AppSandbox's no-prompt ISO rebuild and VHDX
  provisioning from GPU-PV requirements. Added the ignored reproducible `data/`
  artifact tree and documented the native generalized-parent/differencing-child
  workflow, activation/licensing boundary and external-root contract. Ignore,
  tracked-artifact, link/reference, task and diff checks passed; no host/VM change.

- [DOC-010](../BACKLOG.md#doc-010): made routine repository development explicitly
  autonomous, retained scoped approval for privileged/destructive/host-wide work,
  and configured future Codex sessions for workspace writes with on-request
  escalation and dependency network access. Consolidated the detailed policy in
  AGENTS and linked engineering/Copilot guidance to it.

- [DOC-008](../BACKLOG.md#doc-008): changed development to an immutable Windows 11
  parent plus disposable differencing child, separated presentation from GPU proof,
  and defined a constrained on-demand privileged Rust runner. Moved the thin Rust
  inventory/config/adapter/staging/assignment/probe slice before the final hardware
  gate while preserving task IDs. Independent review closed dependency, identity
  rotation and runner-ordering issues; documentation consistency checks passed.

- [DOC-009](../BACKLOG.md#doc-009): configured a project Sol default and an
  Astra/high read-only reviewer with an explicit-model fallback for clients
  without named-agent selection. Added the implementation handoff and verified
  TOML syntax, links and diff whitespace. Runtime model identity was unavailable;
  the first cross-model milestone run remains to be verified.

- [CORE-019](../BACKLOG.md#core-019): added a single Rust CLI/library workspace,
  pinned toolchain and lockfile, help/version and explicit errors, ten passing
  tests, Windows CI and clean-machine development instructions. Formatting,
  strict Clippy, locked build/tests, rustdoc and CLI smoke checks passed locally.
  Recorded the incremental-cache workaround in DEC-010. Hardware gates unchanged.

- [DOC-007](../BACKLOG.md#doc-007): aligned all 13 AI instruction/prompt files with
  shared [Rust engineering standards](../ENGINEERING.md). Documented narrow language
  exceptions, meaningful testing, strict checks, reproducible tooling and native
  operation cleanup; corrected conflicting architecture/decision guidance and
  required initial tests/PR checks with the first Rust code. Independent review
  and Markdown/task consistency checks passed. Documentation only; no application
  implementation, CI execution or hardware validation.

## 2026-09-24

- [DOC-002](../BACKLOG.md#doc-002): expanded the plan to v1.0 across M0-M5 and 45
  permanent task cards, preserving existing IDs. Recorded user-selected essential
  D3D11/D3D12/CUDA and single-guest scope; added research gates, transactional
  configuration/recovery, hardening, qualification and release work. Rechecked
  selected upstream/official sources and completed independent plan review plus
  link/dependency/gate checks. Planning only; no application or hardware changes.
- [DOC-001](../BACKLOG.md#doc-001): established five documentation files, one task
  register, linked blockers, bounded context loading and compact handover.
  Migrated FORK_PLAN.md into the owned documents and aligned agent prompts.
- [GPU-001](../BACKLOG.md#gpu-001): preserved the pinned AppSandbox GPU-PV source
  map and Windows facility research from the earlier planning pass. Corrected
  project direction to a Windows-native Rust project with selective upstream
  adaptation. Research only; no target GPU capability verified.


