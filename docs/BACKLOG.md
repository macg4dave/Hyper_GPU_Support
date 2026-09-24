# Backlog

## Resume

Overwrite this small note at handover; keep durable evidence on the task card.
Treat it as a pointer, not another task-status or authorization store.

- Last session: 2026-09-24, documentation setup (DOC-001); no task currently owned.
- Changed: root guidance/legacy pointer, five `docs/` files and affected Copilot prompts.
- Outstanding: environment, driver and guest capability evidence has not been collected.
- Next recommended: [HV-001](#hv-001); [REF-001](#ref-001) can run independently.
- Milestone: read the single current-milestone pointer in [ROADMAP.md](ROADMAP.md).
- Blockers: check the [blocker register](#blocker-register); no host changes authorized by this note.

## Task register

This table is the **only source** of task status, priority, milestone and
dependencies. A task consists of its row plus its ID card below.
Dependencies list required completed tasks; `-` means none.
P0 precedes P1/P2; respect dependencies before priority.

| ID | Milestone | Priority | Status | Depends on |
|---|---|---|---|---|
| [DOC-001](#doc-001) | M0 | P0 | completed | - |
| [GPU-001](#gpu-001) | M0 | P0 | completed | - |
| [HV-001](#hv-001) | M0 | P0 | ready | - |
| [REF-001](#ref-001) | M0 | P1 | ready | - |
| [GPU-002](#gpu-002) | M0 | P0 | planned | GPU-001, HV-001 |
| [GPU-003](#gpu-003) | M0 | P0 | planned | GPU-002 |
| [HV-002](#hv-002) | M1 | P0 | planned | DOC-001, REF-001, GPU-003 |
| [GPU-004](#gpu-004) | M1 | P0 | planned | HV-002 |
| [GPU-005](#gpu-005) | M1 | P0 | planned | GPU-004 |
| [GPU-006](#gpu-006) | M1 | P0 | planned | GPU-005 |
| [CORE-001](#core-001) | M2 | P1 | planned | GPU-006, REF-001 |
| [CORE-002](#core-002) | M2 | P1 | planned | CORE-001 |
| [CORE-003](#core-003) | M2 | P1 | planned | CORE-002 |
| [GPU-007](#gpu-007) | M3 | P2 | planned | CORE-003 |

## Tracking rules

- Permanent `<area>-NNN` IDs: DOC, REF, HV, GPU, CORE. Allocate the next unused
  number in that area; never renumber/reuse when moving, completing or reprioritising.
  Keep completed rows/cards. If later archived, retain an ID link to the card.
- Exact statuses: `planned` (not actionable yet), `ready` (dependencies complete,
  scope/acceptance known), `in progress` (claimed by a named session/agent),
  `blocked` (specific impediment prevents progress), `completed` (acceptance met
  with linked results). Record owner on the card only while claimed.
- Normal flow: planned -> ready -> in progress -> completed. Move to blocked
  with a blocker entry; return to ready/in progress when the unblock condition
  is met. Incomplete dependencies remain planned, not automatically blocked.
- Dependencies being completed does not automatically make a task ready:
  verify context and milestone gates first. Completion of an experiment may
  record failure; it never establishes a working GPU capability.
- A future protected operation is an execution gate, not an invented blocker.
  Obtain scoped authorization only after the operation is concrete. If awaiting
  a required decision prevents further work, record the exact unresolved action.
- Before claiming, check the row, dependencies' result pointers and blocker
  register. Respect another agent's claim. At handover release ownership to ready,
  or mark blocked with evidence; leave in progress only for a still-active owner.
- No implementation is authorized merely by a ready row. Follow the user task
  and root [AGENTS.md](../AGENTS.md); selecting a next task does not require
  reading the entire backlog, all history or the full source tree.

## Blocker register

No active blockers have been established. Missing inventory is planned work,
not a discovered hardware limitation.

Add `BLK-NNN` records here only for concrete impediments; IDs are permanent.
Resolved entries remain with a concise resolution/evidence link.

| ID | State | Affected task IDs | Problem and evidence | Unblock condition / next action |
|---|---|---|---|---|
| - | - | - | No records yet | - |

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

- Context/scope: Windows 11 x64 host/guest, RTX 5060 8 GB; exact edition/build,
  installed driver and available Hyper-V facilities are unknown.
- Read: [native boundaries](ARCHITECTURE.md#native-windows-boundaries).
- Acceptance: capture host edition/build/x64, virtualization prerequisites,
  Hyper-V feature/service/cmdlet availability, GPU PCI/interface identity and
  driver; identify any candidate guest and its build, Gen 2, Secure Boot/vTPM,
  CPU/RAM/storage, or record that none exists. Distinguish query denial from
  absence. No enable/install/restart or VM/disk mutation.
- Output: create `docs/evidence/HV-001.md` only when findings exist; redact
  sensitive identifiers and record commands, date, results and unknowns.
- Result: pending.

## REF-001

**Establish the independent Git reference**

- Context/scope: the setup workspace has no Git metadata. Preserve current
  files and upstream authorship; no upstream working-tree import or merge.
- Read: [Git decision and last review](DECISIONS.md#dec-004).
- Acceptance: inspect current Git state first; initialise local history if
  still absent; set verified upstream URL and fetch non-shallow history;
  retain the reviewed commit under a durable ref. Record actual remotes/refs.
  Set origin only if its real destination is known; no fabricated URL,
  repository publication, pushes or unrelated identity/global-config changes.
- Output: result on this card; update the upstream review log only if new
  commits were actually reviewed. An absent origin is not a completion blocker.
- Result: pending.

## GPU-002

**Derive the selected driver and runtime manifest**

- Context/scope: use the HV-001 evidence and GPU-001 map to identify only
  relevant RTX 5060 files, registry/ICD settings and conditional shim dependencies.
- Read: dependencies' results; [source map](ARCHITECTURE.md#upstream-reference-map)
  and [provenance decision](DECISIONS.md#dec-004).
- Acceptance: list source/destination, file version/hash collection method,
  ownership/notice, copy-versus-transform step and restoration for proposed
  changes; distinguish NVIDIA, Windows and AppSandbox material. Include NVAPI
  dependencies of chosen compute shims. No driver copying or guest mutation.
- Output: `docs/evidence/GPU-002.md`, a reviewable manifest, not bundled binaries.
- Result: pending.

## GPU-003

**Prepare the baseline procedure**

- Context/scope: turn inventory and manifest into an executable experiment plan
  before any protected setup. Record actual build/driver, not generic defaults.
- Read: HV-001 and GPU-002 results; [validation contract](ARCHITECTURE.md#validation-contract).
- Acceptance: pin reference artifact/source and provenance, inspect setup side
  effects, select real graphics/compute/video/interop probes, state exact steps,
  named VM/disk targets, required settings and recovery. Separate VMMS/HCS
  vendor-extension and presentation questions. Explain any untestable capability.
  Request authorization only when exact protected actions are ready for review.
- Output: `docs/evidence/GPU-003.md`; no full upstream build or product scaffold
  required merely to prepare this procedure.
- Result: pending.

## HV-002

**Prepare dedicated baseline guest states**

- Context/scope: perform the GPU-003 setup using normal Windows facilities;
  preserve clean states for comparable reference/native runs.
- Read: GPU-003 result and root protected-operation rules.
- Acceptance: obtain/verify scoped authorization, prepare named Win11 x64
  guests with matching builds/resources, record driver inputs and recovery
  points. Verify the stopped-guest backup/recreation path; do not assume live
  GPU checkpoints work. Preserve normal Secure Boot, signing and isolation.
- Output: execution evidence under GPU-003's procedure, linked from this card.
- Result: pending.

## GPU-004

**Measure the AppSandbox reference baseline**

- Context/scope: run only the reviewed reference GPU path on the dedicated
  guest; capture actual outcomes even when upstream fails.
- Read: HV-002/GPU-003 results; [validation contract](ARCHITECTURE.md#validation-contract).
- Acceptance: run the selected workloads; capture artifact identity, enabled
  shims/settings, environment, hardware adapter identity, outputs and per-API
  pass/fail/blocked/untested evidence. DLL presence/desktop appearance is insufficient.
- Output: `docs/evidence/GPU-004.md` with exact steps and result matrix.
  Failed runs can complete this measurement task, but do not complete M1.
- Result: pending.

## GPU-005

**Reproduce through native Hyper-V and isolate differences**

- Context/scope: compare clean native VMMS assignment and matching unmodified
  NVIDIA runtimes against GPU-004 with the same builds and workloads.
- Read: GPU-004 result, GPU-002 manifest and
  [native boundaries](ARCHITECTURE.md#native-windows-boundaries).
- Acceptance: record native per-API results alongside reference evidence;
  add shims individually only for reproduced failures. Diagnose assignment,
  runtime, adapter identity, resource and presentation differences separately.
  Use a minimal HCS experiment only for a demonstrated management/capability gap.
  No speculative second backend or general VM product.
- Output: `docs/evidence/GPU-005.md`, linking reference results without copying
  them; record proposed minimum components and unresolved gaps.
- Result: pending.

## GPU-006

**Verify reproducibility and choose the minimum path**

- Context/scope: close the reproduction gate before product implementation.
- Read: GPU-004/GPU-005 results; [M1 exit criteria](ROADMAP.md#m1) and DEC-002.
- Acceptance: actual accelerated workloads pass; upstream-passing probed
  capabilities are reproduced; repeat guest restart/cold start and verify
  removal/restoration under authorization. Record component/backend choice
  with evidence in a decision and update architecture. Remaining reproduction
  gaps keep this task blocked with affected probes linked; partial is not completed.
- Output: `docs/evidence/GPU-006.md` and a decision linked from this card.
- Result: pending.

## CORE-001

**Create the minimal Rust CLI/library and inventory**

- Context/scope: only after M1, create one Windows x64 Rust package with a CLI,
  library and read-only inventory using the measured native interfaces.
- Read: GPU-006 result; [components](ARCHITECTURE.md#proposed-components).
- Acceptance: report explicit adapter/driver/capability facts and unknowns;
  useful errors for missing facilities; no GUI or always-running service.
  Document real build/run/check commands and run focused validation.
- Files: `Cargo.toml`, `src/` and relevant checks (planned, not present).
  Put command/evidence pointers in this result; update architecture for actual modules.
- Result: pending.

## CORE-002

**Encode validated assignment and driver preparation**

- Context/scope: target an existing VM ID and the proven component set.
- Read: CORE-001/GPU-006 results and GPU-002's manifest.
- Acceptance: explicit selected adapter; reviewable change plan; effective
  assignment verified; driver versions/manifests validated; repeatable staging
  and restoration; no silent default-GPU substitution or success after assignment
  failure. Test meaningful failure paths and the baseline on target hardware.
- Files: modules introduced by CORE-001; list exact paths before editing.
- Result: pending.

## CORE-003

**Make the baseline checks repeatable through the core**

- Context/scope: invoke the already chosen workloads, keeping renderer/runtime
  loading distinct from successful execution.
- Read: CORE-002 result and [validation contract](ARCHITECTURE.md#validation-contract).
- Acceptance: readable and machine-readable results include exact environment,
  workload and outcome; same target baseline verified through the CLI;
  absent hardware/software is explicit. No invented CI hardware coverage.
- Files: existing probe/reporting modules; retain raw-output links in task evidence.
- Result: pending.

## GPU-007

**Resolve the first measured capability or reliability gap**

- Context/scope: select one failure from baseline/core evidence; name the exact
  graphics, compute, video, interop or stability workload before marking ready.
- Read: CORE-003 result and only the owning source-map/architecture sections.
- Acceptance: define expected output and pass/fail threshold from that workload;
  implement the smallest change; record before/after, target environment,
  regression checks and recovery. Do not promise physical-GPU features from specs.
- Files: identify after the failure is selected. Split other findings into new
  permanent IDs; advanced APIs, updates, multi-guest and display work remain candidates.
- Result: pending.

## Task and handover templates

Add one register row and one card; never repeat row fields in the card.

```markdown
## <AREA-NNN>
**<Action and outcome>**
- Context/scope: <enough to start; explicit exclusions when needed>
- Read: <dependency result pointers and relevant file/section/source links>
- Acceptance: <observable criteria and validation; protected actions if any>
- Files/output: <existing files or clearly labelled proposed outputs>
- Owner: <session/agent while claimed; omit when unclaimed>
- Result: <pending, or concise checks/results plus evidence links>
```

For an unfinished task, put the exact next action and any required authorization
scope in its result/notes, without secrets. Refresh the Resume block with
completed task links, changed files, unresolved problem/blocker IDs and next
recommended task. It is a short overwrite, not an accumulating session log.
