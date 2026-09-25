# Backlog

## Resume

Overwrite this small note at handover; keep durable evidence on the task card.
Treat it as a pointer, not another task-status or authorization store.

- Last session: 2026-09-25, target inventory and the Rust read-only inventory
  slice completed ([HV-001](#hv-001), [CORE-001](#core-001)).
- Changed: added exact host evidence plus typed/versioned Rust inventory, bounded
  Windows query transport, deterministic selection/error tests and DEC-013.
- Outstanding: no candidate Hyper-V guest, parent/child roots or guest evidence;
  the pending host reboot and unknown host Secure Boot state remain recorded.
- Next recommended: [HV-003](#hv-003); [GPU-002](#gpu-002) and
  [REF-001](#ref-001) are also ready.
- Milestone: read the single current-milestone pointer in [ROADMAP.md](ROADMAP.md).
- Blockers: see the register below; this note authorizes no protected operation.

## Task register

This table is the **only source** of task status, priority, milestone and
dependencies. A task consists of its row plus its ID card below.
Dependencies list required completed tasks; `-` means none.
P0 precedes P1/P2; respect dependencies before priority. Milestone exits are
additional readiness gates. GPU-007 and GPU-015 are optional lanes; other rows
contribute to their milestone's required exit.

| ID | Milestone | Priority | Status | Depends on |
|---|---|---|---|---|
| [DOC-001](#doc-001) | M0 | P0 | completed | - |
| [DOC-002](#doc-002) | M0 | P0 | completed | DOC-001, GPU-001 |
| [DOC-007](#doc-007) | M0 | P0 | completed | DOC-002 |
| [DOC-008](#doc-008) | M0 | P0 | completed | DOC-007, CORE-019 |
| [DOC-009](#doc-009) | M0 | P1 | completed | DOC-007 |
| [DOC-010](#doc-010) | M0 | P0 | completed | DOC-007 |
| [CORE-019](#core-019) | M0 | P1 | completed | DOC-007 |
| [GPU-001](#gpu-001) | M0 | P0 | completed | - |
| [HV-001](#hv-001) | M0 | P0 | completed | - |
| [REF-001](#ref-001) | M0 | P1 | ready | - |
| [REF-004](#ref-004) | M0 | P0 | completed | GPU-001 |
| [HV-003](#hv-003) | M0 | P0 | ready | HV-001 |
| [GPU-002](#gpu-002) | M0 | P0 | ready | GPU-001, HV-001 |
| [REF-002](#ref-002) | M0 | P0 | planned | REF-001, HV-001, GPU-002 |
| [GPU-008](#gpu-008) | M0 | P0 | planned | HV-001, GPU-002 |
| [GPU-003](#gpu-003) | M0 | P0 | planned | GPU-002, HV-003, REF-002, GPU-008 |
| [HV-002](#hv-002) | M1 | P0 | planned | DOC-002, GPU-003, CORE-005 |
| [GPU-004](#gpu-004) | M1 | P0 | planned | HV-002 |
| [GPU-009](#gpu-009) | M1 | P0 | planned | HV-002, CORE-009 |
| [GPU-005](#gpu-005) | M1 | P0 | planned | GPU-004, GPU-009, CORE-003 |
| [GPU-010](#gpu-010) | M1 | P0 | planned | GPU-005 |
| [GPU-011](#gpu-011) | M1 | P0 | planned | GPU-005 |
| [GPU-006](#gpu-006) | M1 | P0 | planned | GPU-010, GPU-011 |
| [CORE-001](#core-001) | M0 | P0 | completed | HV-001, CORE-019 |
| [CORE-004](#core-004) | M1 | P0 | planned | CORE-001, GPU-003 |
| [CORE-005](#core-005) | M1 | P0 | planned | CORE-001, HV-003, REF-001, GPU-003 |
| [CORE-006](#core-006) | M1 | P0 | planned | CORE-004, CORE-005 |
| [CORE-007](#core-007) | M1 | P1 | planned | CORE-006 |
| [CORE-008](#core-008) | M1 | P0 | planned | CORE-005, HV-002 |
| [CORE-009](#core-009) | M1 | P0 | planned | CORE-007, CORE-008, GPU-002 |
| [CORE-002](#core-002) | M1 | P0 | planned | CORE-006, CORE-009 |
| [CORE-010](#core-010) | M2 | P1 | planned | CORE-002 |
| [CORE-011](#core-011) | M2 | P1 | planned | CORE-002, CORE-010 |
| [CORE-012](#core-012) | M2 | P1 | planned | CORE-002 |
| [CORE-003](#core-003) | M1 | P0 | planned | CORE-002, GPU-008 |
| [CORE-013](#core-013) | M2 | P1 | planned | CORE-001 |
| [CORE-014](#core-014) | M3 | P1 | planned | CORE-003, CORE-013 |
| [CORE-015](#core-015) | M3 | P1 | planned | CORE-003 |
| [GPU-012](#gpu-012) | M3 | P1 | planned | CORE-014, CORE-015 |
| [GPU-013](#gpu-013) | M3 | P1 | planned | CORE-015, GPU-012 |
| [CORE-016](#core-016) | M3 | P1 | planned | CORE-014, CORE-015, GPU-012, GPU-013 |
| [GPU-007](#gpu-007) | M3 | P2 | planned | CORE-003 |
| [GPU-015](#gpu-015) | M3 | P2 | planned | GPU-010, CORE-003 |
| [DOC-004](#doc-004) | M4 | P1 | planned | CORE-016 |
| [REF-003](#ref-003) | M4 | P1 | planned | DOC-004 |
| [CORE-018](#core-018) | M4 | P1 | planned | CORE-016 |
| [CORE-017](#core-017) | M4 | P1 | planned | REF-003, CORE-018 |
| [DOC-003](#doc-003) | M4 | P1 | planned | CORE-017 |
| [GPU-014](#gpu-014) | M4 | P1 | planned | DOC-003 |
| [DOC-005](#doc-005) | M5 | P1 | planned | GPU-014 |
| [DOC-006](#doc-006) | M5 | P1 | planned | DOC-005 |

## Tracking rules

- Permanent `<area>-NNN` IDs: DOC, REF, HV, GPU, CORE. Allocate the next unused
  number in that area; never renumber/reuse when moving or splitting scope.
  Keep completed rows/cards. Dependencies, not ID order, determine sequence.
- Exact statuses: `planned` (not actionable yet), `ready` (dependencies complete,
  scope/acceptance known), `in progress` (claimed by named session/agent),
  `blocked` (specific impediment), `completed` (acceptance met with results).
  Owner appears only on an actively claimed card.
- Normal flow: planned -> ready -> in progress -> completed. Incomplete
  dependencies remain planned; actual impediments get linked permanent BLK IDs.
  Verify milestone/context gates before marking a task ready.
- An experiment may complete with a failure report. This does not establish
  capability or satisfy a milestone that requires a passing workload.
- Before claiming, read this register, dependency result pointers, the relevant
  card and blockers. Release unfinished ownership to ready/blocked at handover.
- Task status tracks ownership; the assigned task authorizes routine repository
  implementation under the root permission boundary. It does not authorize a
  protected mutation. Request scoped approval for a new protected target/effect/
  recovery only when concrete steps are prepared; preserve prior authorization.
- Proposed evidence/code/document paths in cards do not exist yet. Create them
  only when actual procedures/results/work exist. Later code tasks locate modules
  via dependency results and record exact paths before editing.
- For every hardware run use the [validation contract](ARCHITECTURE.md#validation-contract).
  Apply root security/provenance/protected-operation rules to every task without
  copying them into each card. New essential defects receive bounded permanent
  cards and must close before the affected gate; optional work cannot mask them.
- All authored code and tests follow [ENGINEERING.md](ENGINEERING.md); task cards
  add acceptance criteria, not exceptions to the Rust or quality standards.

## Blocker register

No execution blocker has been established: target inventory and experiments
have not run. Missing evidence is planned work, not a discovered hardware failure.
The documented client/desktop vendor-support exclusion is a known project
constraint, not proof that measured GPU-PV execution is impossible.
Anticipated risks/questions are owned by the
[gap index](ARCHITECTURE.md#technical-gaps-and-research-gates); conditional product
choices are in DEC-007/008. Promote an actual impediment here with evidence.

| ID | State | Affected task IDs | Problem and evidence | Unblock condition / next action |
|---|---|---|---|---|
| BLK-001 | resolved 2026-09-25 | HV-001; CORE-001; GPU-002; HV-003 | The initial non-elevated inventory identified the host and RTX 5060 but Hyper-V denied partitionable-GPU, VM and supported-version queries; see [`docs/evidence/HV-001.md`](evidence/HV-001.md). | User approved the bounded administrator read-only rerun. It captured the GPU-P interface/ranges, supported versions and zero registered VMs without mutation. HV-001 completed; dependent cards became ready. |

Use permanent BLK-NNN IDs; retain resolved entries and link the resolution.
A failed essential experiment blocks GPU-006 or the relevant release gate even
when the measurement task itself completes. Authorization not yet requested
for future work is not an invented current blocker.

## DOC-001

**Establish the documentation and task system**

- Context/scope: migrate the agreed GPU-PV plan into five owned documents;
  align root/Copilot navigation. No application or host setup.
- Read: the user's documentation request and the migrated [decision](DECISIONS.md#dec-005).
- Acceptance: stable IDs, dependencies, exact statuses, linked blockers,
  acceptance/result fields and compact resume workflow; source research preserved;
  no conflicting planning authority, broken internal links or dependency cycles.
- Result: documentation structure established on 2026-09-24. Validation:
  local Markdown links/anchors, task IDs/dependencies, status/gate consistency,
  source-reference preservation and legacy-pointer checks; no GPU/build tests.
  See [change entry](CHANGELOG.md#2026-09-24).

## DOC-002

**Expand and critically review the implementation plan through v1.0**

- Context/scope: Documentation and primary-source research only; preserve IDs and define release scope, early experiments and session-sized work.
- Read: Current five planning documents, repository state and pinned upstream sources.
- Acceptance: Complete milestone/task sequence through release; measurable gates, failure/recovery validation, explicit research/user decisions; independent second review and mechanical link/dependency checks. No application implementation.
- Files/output: Five existing docs only.
- Result: 2026-09-24: populated M0-M5 and 45 permanent task cards across the five
  planning documents; user selected D3D11/D3D12 + CUDA and one guest. Rechecked
  primary Windows/NVIDIA documentation and selected files at the pinned AppSandbox
  commit; recorded gaps, recovery boundaries and unresolved owner decisions.
  Independent second review corrected cross-VM GPU locking, release self-gating,
  historical pointer guidance and probe precision. Validation: local Markdown
  links/anchors/reference definitions, unique row/card IDs, dependency existence
  and acyclicity, status readiness, all required tasks reaching delivery and
  optional tasks excluded from that path; `git diff --check`. No application build,
  target inventory, GPU workload or protected mutation. See [change entry](CHANGELOG.md#2026-09-24).

## DOC-007

**Establish consistent Rust development and AI instruction standards**

- Context/scope: review every repository AI instruction and prompt; establish one
  detailed engineering standard with concise entry points. Documentation only;
  preserve existing planning changes and the user deletion of FORK_PLAN.md.
- Read: user request, AGENTS.md, all `.github` instructions/prompts, and relevant
  planning sections whose language or validation requirements need alignment.
- Acceptance: Rust-first exceptions, idiomatic modular code, incremental work,
  meaningful testing, strict linting, documentation, diagnostics, dependencies,
  reproducibility and CI standards; no stale fork instructions or conflicting
  authorities. Independent review plus link and scope checks; no implementation.
- Result: 2026-09-25: reviewed and updated all 13 AI instruction/prompt files
  (AGENTS.md, Copilot instructions and 11 prompts). Added the authoritative
  [engineering standards](ENGINEERING.md) and [DEC-009](DECISIONS.md#dec-009);
  aligned architecture and CORE-001/005/013 so Rust is the implementation default
  and tests/basic Windows PR checks begin with the first code. Removed obsolete
  fork-pointer, convenience-PowerShell and C/C++ retention guidance; retained
  attribution, hardware evidence and protected-operation boundaries.
  Validation: independent final review against all 12 requested areas found no
  remaining defects; checked all 19 Markdown files for local links/anchors and
  reference definitions, 11 prompt frontmatter blocks, 46 unique task row/card
  pairs, dependencies, acyclicity and status readiness; `git diff --check` passed.
  No Cargo project, application implementation, CI execution or hardware test;
  existing planning edits and the FORK_PLAN.md deletion preserved. See the
  [change entry](CHANGELOG.md#2026-09-25).

## DOC-008

**Refine the single-VM, single-GPU configuration architecture**

- Context/scope: apply the user's narrowed existing-VM scope to the roadmap,
  backlog, architecture, decisions and AI instructions. Preserve IDs, completed
  work and useful upstream research; no application implementation or hardware changes.
- Acceptance: a small configuration/CLI path, explicit native Windows ownership,
  immutable golden parent plus disposable differencing child, minimal presentation,
  constrained repeatable privilege boundary and a native-first Rust GPU experiment.
  Consistent task gates and links; independent review.
- Files/output: affected planning/instruction documents and README navigation.
- Result: 2026-09-25: accepted [DEC-011](DECISIONS.md#dec-011), treating the
  user's successful AppSandbox HCS run as the working reference while retaining
  project-specific D3D11/D3D12/CUDA proof. Replaced guest rollback machinery with
  an immutable parent VHDX, disposable differencing child and runner-owned logical
  VM slot/GUID enrollment. Kept VMConnect/Enhanced Session/RDP separate from GPU
  evidence and excluded custom display infrastructure absent an essential failure.
  Defined an on-demand, administrator-installed Rust runner that cannot execute
  repository commands, with fixed operations/targets, protected policy/enrollment
  and audited results; installation or policy expansion still needs approval.
  Moved CORE-001 and the Rust config/adapter/staging/assignment/probe slice ahead
  of GPU-006 without adding task IDs; removed semantic dependency cycles and made
  CORE-005 own the minimal runner before HV-002 uses it. Validation: `git diff
  --check`; 49 unique task row/card pairs with existing acyclic dependencies;
  local file links across 20 Markdown files; targeted stale-policy scan. Independent
  architecture review found and then verified fixes for dependency, VM-identity and
  runner-ordering defects; no blocking findings remain. Reviewer runtime model ID
  was unavailable. No Rust/hardware test or protected mutation was performed.

## DOC-009

**Configure independent Astra review for implementation milestones**

- Context/scope: add a project-scoped GPT-5.6 Sol default, read-only GPT-6
  Astra reviewer and Codex handoff instruction. Do not advance a milestone.
- Acceptance: configuration pins reviewer model and reasoning effort; the
  implementation workflow invokes review after checks, addresses blocking
  findings and records actual model evidence when available; links validate.
- Files/output: `.codex/config.toml`, `.codex/agents/architecture_reviewer.toml`,
  `AGENTS.md`, this card.
- Result: 2026-09-25: added project Sol default and a project-scoped
  `architecture_reviewer` pinned to GPT-6 Astra/high with a read-only sandbox
  default. AGENTS now requires review after implementation milestone checks,
  response to blocking findings and repeat review after material design fixes.
  A runtime without named-agent selection must pass the Astra model explicitly.
  Python `tomllib` parsed both TOML files; `git diff --check` passed and
  DOC-009 links/anchors were checked. An independent reviewer found the
  named-agent selection gap; the explicit-model fallback resolved it. This
  runtime did not expose a verified subagent model ID or enforceable sandbox
  metadata, so an actual Sol-to-Astra handoff remains unproven. No milestone
  implementation, Cargo check or hardware test was performed.

## DOC-010

**Clarify autonomous development and protected-operation permissions**

- Context/scope: audit repository, Copilot and Codex instructions so ordinary
  repository development proceeds without approval while new privileged,
  destructive or host-wide effects remain user-controlled. Documentation and
  project-local Codex configuration only; do not weaken platform safeguards.
- Acceptance: one concise authoritative three-category permission policy;
  other agent instructions reference it without conflicting approval rules;
  project Codex defaults permit workspace development while retaining approval
  for escalation; explain platform/configuration boundaries and verify syntax,
  links and instruction consistency.
- Files/output: `AGENTS.md`, affected instruction/configuration files, decisions,
  changelog and this card.
- Result: 2026-09-25: audited AGENTS, ENGINEERING, Copilot instructions, all 11
  task prompts, both project Codex TOML files and the absence of repository VS
  Code settings. Replaced the ambiguous backlog statement that task status
  authorized no implementation, made AGENTS the explicit effect-based authority,
  and linked ENGINEERING/Copilot to it. Set future project Codex sessions to
  `workspace-write`, `on-request` approval and workspace network access; retained
  the read-only reviewer. Read-only client inspection found this trusted project,
  no user-level approval/sandbox key, no managed `requirements.toml` and no
  matching VS Code user setting. Platform policy still takes precedence and no
  Windows elevation is implied. Validation: Python `tomllib` parsed both configs;
  local links passed across 20 Markdown files; 11 prompt frontmatter blocks and
  50 unique task row/card pairs matched; targeted approval-language scan and
  `git diff --check` passed (line-ending notice only). No Cargo/hardware test or
  protected mutation was needed.

## GPU-001

**Map the reference GPU-PV responsibilities**

- Context/scope: initial source investigation at the pinned upstream revision,
  separating Windows assignment, file staging, compatibility shims and display.
  The exact target manifest remains GPU-002.
- Read: [source map](ARCHITECTURE.md#upstream-reference-map).
- Acceptance: responsible files/functions, Windows facilities, conditional reuse
  choices and pinned links identified; unknown hardware behavior clearly labelled.
- Result: source inspection completed 2026-09-24; findings migrated from
  FORK_PLAN.md into the source map. No imported code or target hardware validation.

## HV-001

**Inventory the target without changing it**

- Context/scope: Discover exact Windows 11 host, RTX 5060 and candidate guest prerequisites without installing/enabling anything.
- Read: [native boundaries](ARCHITECTURE.md#native-windows-boundaries).
- Acceptance: Record host edition/build/x64, firmware virtualization/SLAT evidence, Hyper-V feature/service/module versions, elevation context, GPU PCI/interface identity and driver package. Record guest edition/build/x64, VM GUID (redacted externally), configuration version, Gen 2, Secure Boot/vTPM, CPU/RAM/storage and pending reboot information, or no guest. Distinguish unavailable data, permission denial and absent facility.
- Files/output: `docs/evidence/HV-001.md`: commands, date, results, unknowns and redaction.
- Result: 2026-09-25 inventory recorded Windows 11 Pro 25H2
  `26200.9457` x64, active Hyper-V facilities, firmware virtualization, pending
  host reboot, and a healthy RTX 5060 at PCI `10DE:2D05` with NVIDIA driver
  `616.92` / package `32.0.16.1692`. SLAT fields are inconclusive under the active
  hypervisor. An explicitly approved administrator read-only rerun captured one
  RTX 5060 GPU-P interface, raw resource ranges, configuration versions through
  default `12.0`, and zero registered Hyper-V VMs. Candidate-guest fields are
  therefore absent rather than permission-blocked. [BLK-001](#blocker-register)
  is resolved. Evidence: [`docs/evidence/HV-001.md`](evidence/HV-001.md). No
  protected mutation occurred.

## REF-001

**Establish the independent Git reference**

- Context/scope: DOC-002 observed existing commit dd95e58 and origin https://github.com/macg4dave/Hyper_GPU_Support.git. Recheck before work; preserve history and the user's FORK_PLAN.md deletion.
- Read: [DEC-004](DECISIONS.md#dec-004).
- Acceptance: Verify origin and pinned upstream URL; fetch non-shallow AppSandbox history without importing/merging its tree; preserve reviewed commit under a durable ref. Record actual remotes/refs and ancestry. No push, repository publication or global identity changes.
- Files/output: Result on this card; update upstream review log only for newly inspected source.
- Result: pending.

## REF-004

**Establish the unmodified-media and disposable-image baseline**

- Context/scope: Determine whether pinned AppSandbox installation-media changes
  are GPU-PV requirements or product automation, establish ignored local artifact
  roots, and document the smallest native golden-parent/differencing-child workflow.
  No ISO/VHD servicing, VM/host/driver mutation, or new VM-management framework.
- Read: pinned [upstream reference map](ARCHITECTURE.md#upstream-reference-map),
  [DEC-011](DECISIONS.md#dec-011), and official Windows/Hyper-V documentation.
- Acceptance: Account for AppSandbox ISO/disk preparation effects and separate
  installation automation, guest services, networking/display, and GPU runtime
  staging from GPU-PV requirements. Decide whether an official unmodified Windows
  11 ISO is the baseline and identify any evidence-triggered exception. Add a
  configurable, reproducible ignored `data/` layout; verify no prohibited large
  artifacts are tracked. Document activation, identity, parent-chain protection,
  reset, and external-root considerations. Validate links/consistency and Git
  ignore behavior; identify the next practical implementation task.
- Files/output: repository artifact skeleton, ignore rules, concise architecture/
  operator guidance, source references, result and changelog entry.
- Result: completed 2026-09-25. Pinned source inspection found that the legacy
  patcher only substitutes no-prompt UEFI boot files in a new output ISO, while
  the current path reads the official ISO and applies/stages a new VHDX. Neither
  is a GPU-PV requirement. Established an official unmodified ISO baseline,
  post-install child provisioning, native generalized parent/differencing-child
  workflow, and ignored trackable `data/` layout. Verified all intended sample
  artifacts ignored, skeleton files trackable, no prohibited large types tracked,
  unique task entry/reference definitions and `git diff --check`; no ISO/VHD/VM,
  host, driver or hardware action. Next: HV-001 read-only target inventory.

## HV-003

**Record the installed management and privilege contract**

- Context/scope: Read-only validation of candidate APIs before assuming Server documentation applies to the installed client build.
- Read: [native boundaries](ARCHITECTURE.md#native-windows-boundaries) and gaps G1/G3/G7/G9.
- Acceptance: Capture installed Add/Get/Set/Remove GPU adapter parameter sets, WMI/HCS availability as relevant, explicit GPU selection syntax, reported resource ranges/units/counts, current assignments and query rights. Map each proposed query/mutation to required host/guest rights and allowed VM states; mark unproven state rules for GPU-011. Do not change host partition count or infer percentages from raw values.
- Files/output: `docs/evidence/HV-003.md`: interface/privilege matrix and exact unresolved experiments.
- Result: pending.

## GPU-002

**Derive the selected driver and runtime manifest**

- Context/scope: Inspect the installed package and pinned reference for only the x64 target and selected APIs.
- Read: [source map](ARCHITECTURE.md#upstream-reference-map), [recovery contract](ARCHITECTURE.md#configuration-and-recovery-contract), DEC-004.
- Acceptance: List source/destination, version/hash, package/license origin, required/optional classification and copy/transform step. Include ICD/registry/junction/ACL/owner effects needed to reproduce staging in a clean disposable child. Separate Windows, NVIDIA and AppSandbox files; inspect NVAPI dependencies of compute hooks. Identify drift/reboot conditions and exact package terms; no copying, binary modification or driver distribution.
- Files/output: `docs/evidence/GPU-002.md`: reviewable manifest specification and dependency closure.
- Result: pending.

## REF-002

**Prepare a safe runnable reference artifact**

- Context/scope: Select the smallest provenance-verified AppSandbox artifact/build path for the GPU experiment, not a full product import.
- Read: [reference hazards](ARCHITECTURE.md#reference-implementation-hazards), DEC-004 and DEC-007.
- Acceptance: Pin source/artifact hash, inspect actual setup/build side effects and selected project dependencies (MSVC/SDK, WDK only if needed, exports, shims and notices). Verify signing/isolation/Secure Boot preservation and document a usable build or artifact-acquisition recipe. If building is needed, report only commands actually run; inspect before running scripts. No signing-key access, test-signing, permission broadening or host installs. If no safe route exists, record evidence/options and block the reference path for user decision.
- Files/output: `docs/evidence/REF-002.md`: artifact identity, dependency list, exact recipe and safety review.
- Result: pending.

## GPU-008

**Specify the reproducible probe kit**

- Context/scope: Choose existing small workloads before designing product probes; define essential and optional cases separately.
- Read: [validation contract](ARCHITECTURE.md#validation-contract), [v1.0 contract](ROADMAP.md#version-10-contract).
- Acceptance: Pin obtainable source/binary/runtime versions, license and build/run recipe for D3D11, D3D12 offscreen checked rendering and CUDA allocation/transfer/kernel with CPU-checked output. Record requested/negotiated D3D feature level and shader model, plus CUDA compilation target architecture. Choose Blackwell-capable CUDA compiler/runtime against the selected driver; define hardware identity/no-software-fallback checks and host control runs. Define representative optional API/video/interop tests or a specific unavailable-dependency reason. Record expected outputs/tolerances, duration, timeout, session, inputs and failure interpretation; do not claim execution before M1.
- Files/output: `docs/evidence/GPU-008.md`: probe manifest and expected-result table; no application skeleton.
- Result: pending.

## GPU-003

**Prepare the baseline procedure**

- Context/scope: Turn real inventory, artifact and probe specifications into an executable comparison and recovery plan.
- Read: Dependency results and root protected-operation rules.
- Acceptance: Specify exact parent/child roots, logical disposable slot and initial enrolled VM identity, host/guest versions, artifact inputs, settings, guest access, probe order and host control. Define immutable-parent checks, runner-owned GUID enrollment, differencing-child creation/recreation, stop/start/staging/assignment operations and abort thresholds. Define the privileged runner's fixed operation allowlist, log/result path, installation approval and revocation. Explain reference/native and presentation-session differences. Prepare exact target/effect/recovery scopes before requesting any protected action; no generic approval request or setup execution.
- Files/output: `docs/evidence/GPU-003.md`: reviewable ordered procedure and authorization scopes.
- Result: pending.

## HV-002

**Prepare the golden image and disposable guest**

- Context/scope: Execute the reviewed golden-image setup only under explicit scoped authorization.
- Read: GPU-003 result and root protected-operation rules.
- Acceptance: Prepare and shut down a clean updated Win11 x64 Generation 2 parent with legitimate OS inputs, normal Secure Boot/signing/isolation, integration support and no experimental GPU/runtime changes. Protect and fingerprint the parent; never boot it for experiments. Through the approved runner, create one named disposable slot on a differencing VHDX with its own VM identity/security state and atomically enroll the generated GUID. Prove discard/recreation from the same parent, enrollment rotation and guest reachability. Check storage and differencing-chain identity. Any host feature, driver or network change and initial privileged-runner installation needs its own exact authorization.
- Files/output: `docs/evidence/HV-002.md`: actual targets, authorizations, preparation and clean-state recovery.
- Result: pending.

## GPU-004

**Measure the AppSandbox reference baseline**

- Context/scope: Run the reviewed reference GPU path and chosen host controls.
- Read: GPU-003/008 and REF-002 results; [validation contract](ARCHITECTURE.md#validation-contract).
- Acceptance: Capture actual adapter, staged hashes, enabled shims, session, artifact and exact commands. Run all essential workloads and available optional probes; separate assignment, device readiness, runtime load and checked execution results. Include raw output and missing-dependency reasons. A failed run can complete measurement, but cannot satisfy GPU-006 or M1; no 'copy done' or desktop-only GPU claim.
- Files/output: `docs/evidence/GPU-004.md`: environment and per-workload result matrix.
- Result: pending.

## GPU-009

**Prove native guest staging and disposable recovery**

- Context/scope: Validate minimum unmodified runtime provisioning in the disposable native guest before compatibility hooks.
- Read: GPU-002/003 manifests and [recovery contract](ARCHITECTURE.md#configuration-and-recovery-contract).
- Acceptance: Under scoped authorization, prove native transfer/execution, hash-verified staged driver/runtime/registry inputs and guest readiness. Test interrupted staging, mark uncertain state unusable, discard the child and recreate a clean child from the protected parent. Repeat staging successfully on the replacement. Use offline servicing only if proven necessary and separately approved. Any added shim dependency requires reproduced failure evidence and a bounded manifest delta.
- Files/output: `docs/evidence/GPU-009.md`: staging/recreation recipe, transport choice and exact manifest.
- Result: pending.

## GPU-005

**Reproduce through native Hyper-V and isolate differences**

- Context/scope: Compare explicit VMMS assignment with the reference using matched builds, files and workloads.
- Read: GPU-004/009 results, HV-003 and [DEC-007](DECISIONS.md#dec-007).
- Acceptance: Run essential probes and compare available optional probes. Verify actual adapter/effective resources and hardware renderer; isolate assignment, runtime, vendor extension and presentation failures. Introduce one justified shim dependency set at a time with before/after and clean-child recreation checks. Only a demonstrated gap permits a bounded HCS experiment; broader backend/scope changes use DEC-007. No silent default GPU or success after failed attachment.
- Files/output: `docs/evidence/GPU-005.md`: native/reference comparison and minimum component candidates.
- Result: pending.

## GPU-010

**Measure one-guest allocation and host headroom**

- Context/scope: Determine a conservative usable resource envelope on the 8 GB GPU before exposing presets.
- Read: HV-003 values and GPU-005 result.
- Acceptance: Record requested/reported effective VRAM/compute/encode/decode fields, partition counts and host/guest memory/load observations. Exercise small and increasing bounded allocation plus concurrent host rendering; define abort threshold before execution. Identify invalid-request behavior and whether any limit is actually enforced. Choose one measured safe preset or retain measured defaults; document unknown units/oversubscription and no hard-quota promise. No host-wide partition-count mutation without separate authorization.
- Files/output: `docs/evidence/GPU-010.md`: workload, resource observations, headroom and proposed preset.
- Result: pending.

## GPU-011

**Verify lifecycle and disposable recreation**

- Context/scope: Establish legal operation states and recovery without assuming checkpoint support.
- Read: GPU-003 recovery plan, HV-003 state questions and GPU-009 preimages.
- Acceptance: Under authorization run at least five clean shutdown/start cycles, one guest reboot, attach/remove/reapply and a controlled invalid assignment. Check essential probes after each stable state and adapter removal. Record failed boot/device-not-ready diagnosis, discard the affected child and prove recreation from the unchanged parent. Saved state/checkpoints/host sleep remain explicitly unvalidated unless separately tested; never force-stop by default.
- Files/output: `docs/evidence/GPU-011.md`: state transitions, outcomes and verified recovery.
- Result: pending.

## GPU-006

**Verify reproducibility and choose the minimum path**

- Context/scope: Close the first Rust GPU-PV demonstration and select its minimum proven path.
- Read: GPU-004/005/009/010/011 results, [M1](ROADMAP.md#m1), DEC-002/006/007.
- Acceptance: Essential D3D11/D3D12/CUDA workloads pass through the Rust-driven native/minimum path and reproduce the user's known-working reference behavior. Record backend/component decision, exact tested versions, manifest, resource envelope and disposable-child recreation. Optional comparisons keep individual limitations. A specific essential gap is diagnosed against AppSandbox/HCS evidence and repaired or presented to the user; failed experiments do not prove success.
- Files/output: `docs/evidence/GPU-006.md`, new accepted backend decision and affected architecture sections.
- Result: pending.

## CORE-001

**Extend the Rust CLI/library with inventory**

- Context/scope: Extend the CORE-019 foundation with inventory in one Windows x64 package; no GUI/daemon or general VM manager.
- Read: HV-001 result, CORE-019 result and [proposed components](ARCHITECTURE.md#proposed-components).
- Apply: [toolchain policy](ENGINEERING.md#toolchain-dependencies-and-features) and
  [required checks/CI](ENGINEERING.md#required-checks-and-ci). Extend CORE-019's
  tests and Windows PR checks with focused inventory coverage and strict warnings.
- Acceptance: Extend the existing CLI/library boundary with read-only inventory of selected VM/GPU/driver facts and unknowns. Report missing facility/denial distinctly. Preserve pinned tools and locked builds; document inventory commands and focused Windows validation; keep backend access replaceable for tests. Record exact introduced paths before follow-up work.
- Files/output: Existing `src/`, build instructions and proposed `docs/evidence/CORE-001.md`.
- Result: 2026-09-25 added `inventory` through `src/inventory.rs` and
  `src/windows_inventory.rs`, with a typed replaceable source, versioned report,
  exact RTX 5060/GPU-P correlation, unambiguous VM selection and distinct
  known/missing/denied/unavailable outcomes. The fixed query-only Windows child
  is bounded to 15 seconds and 64 KiB per stream; Rust kills/reaps on timeout,
  validates its hex protocol and preserves bounded structured launch/exit/native
  diagnostics. [DEC-013](DECISIONS.md#dec-013) records the native investigation,
  narrow transport rationale and HV-003 revisit gate. On Windows 11 Pro 25H2
  `26200.9457` x64, fmt, strict locked Clippy, 16 library + 2 binary + 4
  integration + 1 doc tests, locked build and rustdoc passed with warnings denied.
  The real unelevated command returned exact host/RTX facts and structured GPU-P/
  VM denial. Independent architecture review drove fixes for process bounds,
  transport ownership, identity correlation, classifications, diagnostics and a
  malformed-index panic; final recheck found no blocking issue and independently
  passed all eight adapter tests plus `git diff --check`. Reviewer runtime model
  metadata was unavailable. Evidence: [`docs/evidence/CORE-001.md`](evidence/CORE-001.md).
  No elevated project binary, host/VM mutation or workload test; positive VM/GPU-P
  behavior remains unverified and M0 remains current.

## CORE-004

**Define configuration and CLI data contracts**

- Context/scope: Small versioned configuration, plan and report types; choose serialization as an implementation detail.
- Read: [configuration contract](ARCHITECTURE.md#configuration-and-recovery-contract) and GPU-003.
- Acceptance: Specify VM GUID, GPU identity, measured resource values, manifest identity and CLI operations inventory/plan/apply/status/validate/remove/recover/lifecycle. Reject unknown schema/fields, ambiguous targets, invalid ranges and credentials. Define stable error categories/exit behavior and examples; test round-trip, invalid input and defaults without hardware. No invented percentage abstraction.
- Files/output: Configuration/types/parser/help modules introduced here; examples and `docs/evidence/CORE-004.md`.
- Result: pending.

## CORE-005

**Implement fixed native adapters and the minimal privileged runner**

- Context/scope: Implement Rust adapters and a stable privileged broker for only
  the measured Windows operations needed by the first vertical slice. Prefer native Rust/API access; justify any
  necessary command/script boundary through [the exception policy](ENGINEERING.md#rust-and-native-windows).
- Read: HV-003, REF-001 and GPU-003; [configuration contract](ARCHITECTURE.md#configuration-and-recovery-contract).
- Acceptance: Build and test the minimal runner plus unelevated client adapter. Use fixed operations and typed parameters, structured results and explicit VM/GPU identifiers. Implement administrator-owned installed code/policy/enrollment, one pinned disposable slot/GPU/roots, runner-owned differencing-child and VM create/reset, atomic generated-GUID rotation, operation allowlist, request/result audit, replay protection and fail-closed validation. Never execute repository binaries or arbitrary commands elevated on the host. Preserve native error codes and handle missing rights and timeout/cancellation. Test invalid/stale identities, attempted agent enrollment/path selection, parent writes, query denial, policy tampering, partial reset and native failures using fakes; then run read-only target queries. Retain detailed native errors without secrets. Installation/execution on the target occurs only through HV-002's scoped authorization.
- Files/output: Native adapter modules and `docs/evidence/CORE-005.md`.
- Result: pending.

## CORE-006

**Implement read-only preflight and change planning**

- Context/scope: Convert desired configuration and observed state into a concrete diff before mutation.
- Read: GPU-003 and [configuration contract](ARCHITECTURE.md#configuration-and-recovery-contract).
- Acceptance: Check edition/interfaces, rights, VM state/security, guest access prerequisites, driver/build fingerprints, resource ranges and backup capacity. Emit named targets, ordered changes, warnings, recovery and plan fingerprint. Reject wrong GPU, unsupported states and missing essential input; unchanged configuration yields empty plan. Test stale identities and malformed resource values; planning must not mutate a guest/host.
- Files/output: Planner/preflight modules and `docs/evidence/CORE-006.md`.
- Result: pending.

## CORE-007

**Implement operation audit and target locking**

- Context/scope: Bounded coordination and audit for one disposable VM, not a general transaction engine.
- Read: [configuration contract](ARCHITECTURE.md#configuration-and-recovery-contract).
- Acceptance: Persist restricted-access request ID, plan fingerprint, exact target, intent and verified step outcomes. Acquire VM and host/physical-GPU locks by stable identity in a fixed order; revalidate plan/state and assignments under lock. Refuse conflicting, duplicate, stale or out-of-policy operations. Test a wrong VM GUID, simultaneous requests, interrupted audit writes, replay and external state changes. Retain only host-setting preimages needed for safe detach; uncertain guest state produces an explicit discard/recreate result.
- Files/output: Journal/state/lock modules and `docs/evidence/CORE-007.md`.
- Result: pending.

## CORE-008

**Implement native guest sessions and verified file transfer**

- Context/scope: Implement the PowerShell Direct/native transport specified by GPU-003 and validate it here; no custom network agent/protocol.
- Read: HV-002/GPU-003 and [configuration contract](ARCHITECTURE.md#configuration-and-recovery-contract).
- Acceptance: Acquire guest credentials ephemerally and keep secrets out of CLI arguments/logs/configuration. Verify guest identity/readiness, restrict transfer paths, checksum contents and time out stalled operations. Test denied credentials, unavailable integration, interrupted transfer, path traversal/reparse points and retry cleanup; perform an authorized harmless transfer on the dedicated guest. No network/security configuration side effects.
- Files/output: Guest session/transfer modules and `docs/evidence/CORE-008.md`.
- Result: pending.

## CORE-009

**Implement manifest-based runtime staging**

- Context/scope: Encode the minimum driver/runtime manifest derived by GPU-002; add a conditional shim only after a reproduced failure.
- Read: GPU-002/REF-002, [reference hazards](ARCHITECTURE.md#reference-implementation-hazards), DEC-004.
- Acceptance: Validate origin/hash/version; apply audited steps in the disposable child and verify files/settings/permissions after each. Detect missing required payload, identical-size/different-content files and changed source driver. Repeat staging is a no-op; a controlled partial-write failure marks the child unusable and recreation proves recovery. If a shim is required, retain source commit/path, authorship, ABI/build dependency and version-range tests; adapt only the measured dependency closure.
- Files/output: Manifest/staging modules, conditional attributed shim sources if proven necessary, and `docs/evidence/CORE-009.md`.
- Result: pending.

## CORE-002

**Encode validated GPU assignment and apply orchestration**

- Context/scope: Retained task ID; staging, audit/locking and disposable reset are split into dedicated cards.
- Read: CORE-006/007/009, HV-003 and GPU-003 results.
- Acceptance: Apply only a reviewed, current plan with exact selected adapter/resources; verify actual attachment and staged readiness separately. Journal all native settings changed. Refuse duplicate/foreign assignments or an unvalidated second-guest configuration by default. Test native assignment failure, wrong returned adapter, state drift and no-op reapply; run essential probes after authorized target apply. Failure must not be reported as success.
- Files/output: Assignment/apply modules and `docs/evidence/CORE-002.md`.
- Result: pending.

## CORE-010

**Complete detach/reset CLI integration and hardening**

- Context/scope: Integrate and harden the CORE-005 runner's existing detach/reset operations in the completed CLI workflow; never touch the parent image.
- Read: GPU-009/011 and CORE-007/009 preimage formats.
- Acceptance: Plan and execute adapter removal in measured safe states, detect external changes/conflicting assignments and verify host state. For uncertain guest files/settings, stop and delete only the policy-pinned disposable VM/child after explicit operation authorization, then recreate it from the fingerprinted parent. Test remove twice, wrong identity, partial assignment, missing child and parent mismatch. Never delete or attach the master for writes.
- Files/output: Detach/reset modules and `docs/evidence/CORE-010.md`.
- Result: pending.

## CORE-011

**Implement bounded VM lifecycle operations**

- Context/scope: Delegate start, graceful shutdown and restart to native Windows for the configured existing VM.
- Read: GPU-011 legal states and CORE-005/007 results.
- Acceptance: Preflight security/configuration/driver drift before start; wait with bounded readiness/shutdown timeouts and report VM state separately from GPU health. Refuse unvalidated saved-state recovery and conflicting mutation; no forced stop or automatic host reboot. Test stopped/running/unresponsive guest states and guest reboot, then repeat essential probes after authorized lifecycle runs.
- Files/output: Lifecycle module and `docs/evidence/CORE-011.md`.
- Result: pending.

## CORE-012

**Implement layered diagnostics and a redacted report**

- Context/scope: Explain failures at inventory, assignment, PnP, runtime, workload or presentation layer.
- Read: [validation contract](ARCHITECTURE.md#validation-contract), GPU-004/005/011 failure evidence.
- Acceptance: Collect relevant native errors/events, environment, effective state, manifest hashes and operation audit references. Emit readable and versioned machine-readable reports with tested/unknown distinctions and actionable next checks; never auto-restart devices. Test partial access, missing logs, code-43-like status and redaction of credentials/sensitive identifiers. Confirm a real guest failure/success can be diagnosed without leaking binary payloads.
- Files/output: Diagnostics/report modules and `docs/evidence/CORE-012.md`.
- Result: pending.

## CORE-003

**Make the baseline probes repeatable through the core**

- Context/scope: Invoke the existing pinned probe kit, not a new general benchmarking framework.
- Read: GPU-008, CORE-002 and [validation contract](ARCHITECTURE.md#validation-contract).
- Acceptance: Produce per-workload pass/fail/blocked/untested/unsupported-with-evidence outcomes with exact environment, hardware adapter, runtime/probe version, inputs, checked outputs and logs. Handle timeouts and software fallback correctly; an absent probe is never pass. Run the essential M1 baseline through the CLI on the target and compare outputs; preserve optional failures distinctly.
- Files/output: Probe runner/report integration and `docs/evidence/CORE-003.md`.
- Result: pending.

## CORE-013

**Extend and verify hardware-free Windows CI**

- Context/scope: Extend CORE-001's initial PR checks for supported feature sets and
  reliable failure/artifact handling without the physical GPU/Hyper-V guest.
- Read: CORE-001 actual commands and [test lanes](ARCHITECTURE.md#test-lanes).
- Acceptance: Pin dependency lockfile/toolchain and CI actions; run formatting/lints/build and focused logic/adapter-fake tests on Windows x64. Keep privileged/GPU suites explicit manual/self-hosted jobs requiring the authorized target, never default on untrusted PRs. Verify failed assertions/nonzero adapters fail CI and artifacts contain no secrets/driver binaries. Later tasks add meaningful tests to this lane.
- Files/output: Proposed CI workflow and `docs/evidence/CORE-013.md`.
- Result: pending.

## CORE-014

**Verify interrupted apply and disposable recovery**

- Context/scope: Ensure interruption cannot target the wrong VM/parent or report success; recover uncertain guest state by recreation.
- Read: CORE-007/009/010 and [configuration contract](ARCHITECTURE.md#configuration-and-recovery-contract).
- Acceptance: Inject process interruption before/after every mutation boundary using fake adapters, then selected authorized guest interruptions. On restart, reconcile observed host state to the audit record: safely detach a verified project assignment or stop and require child recreation. Test concurrent/replayed requests, wrong identities, parent-write attempts, denied cleanup and storage exhaustion. No false success or wrong-target change; a manual stop includes the exact discard/recreate or host-cleanup action.
- Files/output: Recovery implementation fixes as needed and `docs/evidence/CORE-014.md` failure matrix.
- Result: pending.

## CORE-015

**Detect compatibility drift and plan driver restaging**

- Context/scope: Maintain the tested combination without automating host driver/OS installation.
- Read: GPU-002/006, CORE-006/009/012 and [configuration contract](ARCHITECTURE.md#configuration-and-recovery-contract).
- Acceptance: Compare current host/guest build, driver/package hashes and essential runtimes to last-validated record at plan/start/validate. Mark unknown or mismatched versions unvalidated and refuse stale apply; explain requalification. Generate an explicit new-child restaging plan with pending-reboot handling; no silent DLL refresh, host downgrade or parent mutation. Test host-only update, guest-only update, changed GPU path and partial staging using fixtures.
- Files/output: Compatibility/restage modules and `docs/evidence/CORE-015.md`.
- Result: pending.

## GPU-012

**Qualify sustained load and lifecycle endurance**

- Context/scope: Measure the single-guest release envelope, including host responsiveness.
- Read: GPU-010 thresholds, GPU-011 and CORE-003 probe kit.
- Acceptance: Run 20 clean guest shutdown/start cycles, two separately authorized host reboots and four hours of mixed essential graphics/CUDA load including bounded VRAM pressure. Sample host responsiveness and memory/use against predeclared GPU-010 abort limits. Require correct outputs, no unexplained device loss/hang and post-run essential passes; check event logs and recovery. Compare three timed runs to M1 and investigate >10% median regression before acceptance; publish conditions, not a universal performance SLA.
- Files/output: `docs/evidence/GPU-012.md`: raw measurements, outcomes and regressions.
- Result: pending.

## GPU-013

**Rehearse driver mismatch, restaging and recovery**

- Context/scope: One controlled compatible driver transition; Windows build drift is separately detected, not a promise to support every update.
- Read: GPU-002 package terms, CORE-015 plan, GPU-011 cold recovery.
- Acceptance: Select an available signed driver pair that supports the hardware and prepare exact host/guest targets, expected changes and viable recovery before approval. Exercise mismatch detection, operator-controlled host servicing/reboot, guest restage, essential probes and recovery to the prior validated combination where supported. Do not force a host downgrade or remove working packages to manufacture a test. If no safe transition exists, record blocker and obtain a release-contract decision; simulated drift alone is not hardware transition evidence.
- Files/output: `docs/evidence/GPU-013.md`: old/new hashes/builds, authorization, outcomes and recovery.
- Result: pending.

## CORE-016

**Review privilege boundaries and release-blocking defects**

- Context/scope: Independent code/security review of the implemented local CLI, not a general penetration-testing product.
- Read: Native adapter, privileged runner, audit, transfer and staging modules; [reference hazards](ARCHITECTURE.md#reference-implementation-hazards).
- Acceptance: Review injection/path/reparse attacks, forged plans/requests/audits, runner replacement/policy tampering, parent-image protection, secret handling, binary provenance, ACLs, dependency risks and untrusted guest/probe output. Verify no isolation/signing/network boundary weakened and no automatic privileged maintenance outside the allowlist. Reproduce and fix findings threatening wrong-target changes, data loss, credentials or essential workloads, allocating permanent defect cards if needed; run affected tests and target regressions. Essential findings keep this gate open.
- Files/output: `docs/evidence/CORE-016.md`: reviewer findings, fixes and residual limitations.
- Result: pending.

## GPU-007

**Resolve one measured optional capability gap**

- Context/scope: Optional improvement lane retained from the original roadmap; not a required release dependency.
- Read: One concrete CORE-003/GPU-005 failure and only its relevant source/architecture sections.
- Acceptance: Before ready, name exactly one optional workload, input, expected output/threshold and reproduced cause on this card. Implement the minimum attributed fix or conclude unsupported-with-evidence; record before/after, essential regression results and recovery. Split other findings into new IDs. This card must not authorize speculative 'GPU support' work or absorb essential release defects.
- Files/output: `docs/evidence/GPU-007.md` only after a specific workload is selected.
- Result: pending.

## GPU-015

**Measure two-guest contention experimentally**

- Context/scope: Optional research outside the one-guest v1.0 guarantee; no scheduler implementation.
- Read: GPU-010 resource evidence, CORE-003 probe kit and DEC-006.
- Acceptance: With separately authorized second guest/resources, run isolated and simultaneous essential probes under bounded aggregate memory/load. Record per-guest correctness, host headroom, start-order/stop recovery and observed resource enforcement; abort at declared thresholds. Publish measured failures/limits without a fairness or hard-quota claim. If unsafe or unavailable, leave untested with reason; no impact on the one-guest release gate.
- Files/output: `docs/evidence/GPU-015.md`: concurrent experiment and compatibility limitations.
- Result: pending.

## DOC-004

**Resolve owner release, license and signing choices**

- Context/scope: Obtain actual owner choices once the component set is known; recommendations are not authorization.
- Read: [DEC-008](DECISIONS.md#dec-008) and selected component provenance.
- Acceptance: Present concrete payload/license obligations, source versus portable archive options, signature identity/cost and intended delivery channel. Record owner's project license/distribution/signing selection and any publication constraints in an accepted decision. No certificate purchase, signing-state changes, public release or proprietary payload implied. If unanswered when packaging needs it, record exact blocked choice.
- Files/output: Accepted replacement decision for DEC-008 and result on this card.
- Result: pending.

## REF-003

**Audit release provenance and third-party notices**

- Context/scope: Verify actual included source/binaries against the selected release policy.
- Read: DEC-004, DOC-004 and every adapted component's source pointer.
- Acceptance: Inventory direct/transitive dependencies, licenses, upstream commits/paths, build inputs and applicable MIT/third-party notices. Inspect exact NVIDIA/Windows package terms for local provisioning instructions; exclude unlicensed redistribution and proprietary payload. Check shim export/build dependencies and authorship. Release payload contains no drivers, OS images, disks, production data, keys or credentials; unresolved essential rights block packaging.
- Files/output: Proposed license/notice inventory and `docs/evidence/REF-003.md`.
- Result: pending.

## CORE-018

**Freeze v1 configuration and reporting compatibility**

- Context/scope: Stabilize the small public CLI/configuration/report contract before the candidate.
- Read: CORE-004/012/015 and actual operator workflows.
- Acceptance: Document schema version, exit codes, command help and unknown-version behavior. Test v1 example round trips, machine-report parsing, upgrade from any shipped preview fixture (or record none), invalid config rejection and report redaction. No promise of AppSandbox API compatibility. Changes after freeze require affected tests/evidence to rerun.
- Files/output: Versioned schema/examples/help and `docs/evidence/CORE-018.md`.
- Result: pending.

## CORE-017

**Build the reproducible release candidate package**

- Context/scope: Package the owner's selected source/archive form without an installer dependency.
- Read: DOC-004, REF-003, CORE-001/013 commands and CORE-018 contract.
- Acceptance: Build from a clean Windows x64 checkout using pinned toolchain/lockfile and only documented inputs. Include needed project runtime components, notices, examples, version/revision metadata and checksums; inspect exclusions and signing identity per decision. Test unpack/run/help and missing prerequisites on clean Windows. Reproducible means repeatable recipe and traceable contents; claim byte identity only if measured. No upload/publish or signing-key changes.
- Files/output: Proposed release recipe/workflow, local candidate artifact and `docs/evidence/CORE-017.md`.
- Result: pending.

## DOC-003

**Write the operator and recovery guide from tested workflows**

- Context/scope: Explain using the candidate with the measured golden-parent/disposable-VM configuration.
- Read: GPU-006, CORE-003/010/011/015, GPU-012/013 and CORE-017.
- Acceptance: Document prerequisites and legitimate local driver inputs, rights/credential handling, exact config examples, parent protection, child creation, plan/apply/validate/lifecycle/detach/discard/recreate steps, logs and maintenance. State exact compatibility matrix and optional/untested features, resource limits, no automatic host updates and project versus vendor support boundary. Every command matches actual candidate help; guide requires no hidden machine-specific paths.
- Files/output: Proposed `README.md`, `docs/OPERATIONS.md` and `docs/evidence/DOC-003.md`; technical results stay in existing task evidence.
- Result: pending.

## GPU-014

**Rehearse the candidate from a fresh guest**

- Context/scope: Fresh developer/operator review using only candidate artifacts and the guide.
- Read: CORE-017 artifact hashes, DOC-003 and [validation contract](ARCHITECTURE.md#validation-contract).
- Acceptance: On an authorized clean guest with the pinned release environment, follow documented prerequisites and config through inventory/plan/apply/essential validation, restart, removal and recovery. Require D3D11/D3D12/CUDA checked output and hardware identity; no undocumented setup or stale driver inputs. Verify the package runs without developer tools unless the selected source-only policy explicitly requires them. Fix guide/package defects and repeat affected steps; record observer and raw evidence.
- Files/output: `docs/evidence/GPU-014.md`: candidate hash, fresh-state procedure and results.
- Result: pending.

## DOC-005

**Audit the v1.0 release gate**

- Context/scope: Independent requirement-to-evidence review of the exact candidate revision.
- Read: [v1.0 contract](ROADMAP.md#version-10-contract), all required milestone results and blocker register.
- Acceptance: Trace each essential capability and operational requirement to candidate-compatible tests, provenance, docs and recovery. All earlier required cards complete; no open essential/security/data-loss blocker. Triage optional defects explicitly and check links/config examples. If candidate code/dependencies change, rerun affected checks; essential failures require fixes, not lowered thresholds.
- Files/output: `docs/evidence/DOC-005.md`: signed-off-by-reviewer checklist, candidate identity and known limitations.
- Result: pending.

## DOC-006

**Deliver v1.0 and maintainer handover**

- Context/scope: Finish concrete artifacts/notes first; perform only owner-authorized publication/delivery.
- Read: DOC-004 distribution decision and DOC-005 gate audit.
- Acceptance: Verify final hashes/revision match audited candidate; provide release notes, exact measured compatibility, limitations, checksums, recovery and requalification instructions. Obtain exact publication target authorization only after artifacts are reviewable, if publishing is selected. Verify delivery/download and links; record final version/artifact location, refresh Resume/current milestone and retain evidence. No credentials, drivers or guest disks in delivered artifacts.
- Files/output: Final release artifacts/notes and `docs/evidence/DOC-006.md`.
- Result: pending.

## Task and handover templates

Add one register row and one card; never repeat row fields in the card.

```markdown
## <AREA-NNN>
**<Action and outcome>**
- Context/scope: <bounded deliverable>
- Read: <dependency result pointers and relevant section/source links>
- Acceptance: <observable criteria and validation>
- Files/output: <existing files or clearly proposed outputs>
- Owner: <only while claimed>
- Result: <pending, or exact checks/results and evidence links>
```

For unfinished work record the exact next action and linked blocker if any,
then refresh Resume with completed IDs, changed files and next recommended task.
Do not append a transcript or duplicate the task-status register.

## CORE-019

**Establish the hardware-independent Rust project foundation**

- Context/scope: User-requested early scaffolding split from CORE-001; single
  CLI/library package, help/version, errors, tests, pinned tools and Windows CI.
  No inventory, configuration schema, native adapters or GPU-PV implementation.
- Read: DOC-007 result, engineering standards, architecture components and M0/M2 gates.
- Acceptance: Windows x64 build/run; meaningful unit/integration/doc tests; fmt,
  strict Clippy, locked tests/build and rustdoc; clean-clone setup documentation;
  preserve existing edits and pinned upstream references. No hardware requirement.
- Result: 2026-09-25: created one dependency-free Rust 2024 package/workspace,
  exact Rust/MSRV 1.94.0 and x64 MSVC target, lockfile, repository hygiene,
  CLI/library with help/version and consistent errors, and SHA-pinned Windows CI.
  Paths and clean-machine commands: [README.md](../README.md); boundaries and
  future module locations: [architecture](ARCHITECTURE.md#foundation-source-layout).
  Scope/toolchain/cache decisions: [DEC-010](DECISIONS.md#dec-010).
  No pre-existing Cargo project/tests to baseline. Actual Windows 25H2 build
  26200.9457 x64 (legacy ProductName reports Windows 10 Pro), MSVC 14.51.36231,
  SDK 10.0.26100.0, rustc 1.94.0 (4a4ef493e): fmt check, strict Clippy across
  all targets/features, locked tests/build and rustdoc passed with warnings denied.
  Six unit tests, three executable integration tests and one doc test passed;
  CLI help/version launched successfully. Final run used repository-local target/;
  incremental-cache error 5 was resolved as described in DEC-010, with no lint
  suppression. Git whitespace and local Markdown/task consistency checks passed.
  cargo-audit/cargo-deny are unavailable; no third-party crates/advisory database
  check. Hosted CI and a separate clean Windows installation have not been run.
  No GPU/driver/API-runtime workload or host/guest mutation; M0 remains current.
