# Source baseline

Assessment date: 2026-10-11. Method: read-only source inspection followed by documentation checks. No builds, Rust tests, GUI launches, guest access, elevated product operations, or hardware qualification were performed. This page owns source findings; the [backlog](BACKLOG.md) owns work status.

## Reading the assessment

**Implemented** means a concrete code path exists. **Incomplete** means a necessary part of that capability is absent or blocked. **Unknown** means static inspection cannot establish it. **Validated** requires relevant recorded checks with revision, environment, and outcomes. Nothing in the table is newly runtime-validated by this reset. Existing test functions are coverage candidates, not passing results.

Old milestone IDs and completion claims are not carried into the new plan. Git history can answer a specific question, but does not establish the current checkout's build or release readiness.

Documentation reset checks passed: 20 maintained documents, 190 local links/anchors,
19 task cards with matching register entries and acyclic dependencies, six release
requirements with owning tasks, six prompt metadata/authority links, and
`git diff --check`. Hash comparison confirmed all 79 pre-existing tracked
non-documentation files were unchanged. All 141 deletions outside the nine
deliberately rewritten README/instruction/handbook files remain intact. These are
documentation/change-boundary checks, not product validation.

## Capability inventory

| Capability | Source assessment | Evidence | Validation in this reset |
|---|---|---|---|
| GUI/CLI dispatch and explicit simulation | Implemented | [main](../src/main.rs), [GUI host](../src/gui/mod.rs), [snapshot](../src/gui/snapshot.rs) | Not run |
| Live VM/GPU discovery and periodic status reads | Implemented; installed inventory polls every 15 seconds without initiating setup | [live controller](../src/gui/live.rs), [Hyper-V adapter](../src/windows_hyperv.rs) | Not run |
| First-install discovery and fixed-package setup | Implemented; UAC path and pending setup handling exist | [setup](../src/setup.rs), [runner](../src/runner.rs) | Not run |
| Additive exact-pair enrollment | Implemented; appends one reviewed pair and preserves other pairs; replacement is refused | [enrollment](../src/enrollment.rs), [worker](../src/worker.rs) | Not run |
| Review, enable/disable, and Reapply/Update | Implemented GUI callbacks reuse shared plans and the protected worker | [live controller](../src/gui/live.rs), [workflow](../src/workflow.rs) | Not run |
| Separate guest Verify | Implemented; device health and selected-adapter D3D11 report are checked | [guest](../src/guest.rs), [probe](../src/bin/d3d11-probe.rs), [workflow](../src/workflow.rs) | Not run |
| Saved intent, draft conflict handling, protected publication | Implemented; per-VM files, expected revisions, and recovery receipts exist | [model](../src/model.rs), [store](../src/configuration_store.rs), [protected store](../src/configuration_store/protected.rs), [live state](../src/gui/live_state.rs) | Not run |
| VRAM/Compute/Encode/Decode requests | Implemented schema-3 category paths; schema 2 retained | [model](../src/model.rs), [Hyper-V adapter](../src/windows_hyperv.rs), [workflow](../src/workflow.rs) | Provider behavior not measured |
| Memory slider and physical-capacity display | Implemented native VRAM byte-to-GB request conversion; historical snapshots fall back to raw fields | [memory](../src/gui/memory.rs), [live state](../src/gui/live_state.rs), [adapter identity](../src/windows_probe.rs) | Unit/display behavior and enforcement not qualified |
| Credential prompting and optional storage | Implemented local Windows prompt/vault path; CLI deletion exists | [credentials](../src/credentials.rs), [main](../src/main.rs) | Not run |
| Save-only and explicit GPU/enrollment reconciliation | Implemented; bound records and independent readback paths exist | [worker](../src/worker.rs), [enrollment](../src/enrollment.rs), [live controller](../src/gui/live.rs) | Fault behavior not exercised |
| Explicit configuration import/resume | Implemented CLI paths; GUI reports pending import for CLI resolution | [main](../src/main.rs), [store](../src/configuration_store.rs), [worker](../src/worker.rs) | Not run |
| GPU replacement / Forget pairing | Incomplete; explicitly unavailable in live GUI; additive enrollment refuses replacement | [live state](../src/gui/live_state.rs), [live controller](../src/gui/live.rs), [enrollment](../src/enrollment.rs) | No support claim |
| Guest payload trust and bounded transfer | Implemented checks, byte budget, transfer monitoring, and process deadlines | [trust](../src/trust.rs), [payload](../src/payload.rs), [transfer](../src/guest_transfer.rs), [transport](../src/guest_transport.ps1) | Containment and host stability not qualified |
| Two guests rendering concurrently on one GPU | Unknown; multiple target records and additive enrollment do not prove simultaneous operation | [model](../src/model.rs), [enrollment](../src/enrollment.rs) | No concurrent run performed |
| Resource isolation, fairness, broad vendor support | Unknown / outside initial release target; preparation is NVIDIA-first | [workflow](../src/workflow.rs), [model](../src/model.rs) | No guarantees |
| Reproducible gate, integration suites, CI, qualification tooling | Incomplete working-tree tooling; inline tests survive | [manifest](../Cargo.toml), [testing plan](TESTING.md) | Deleted tools not run or restored |
| Installable release package | Unknown; fixed four-file setup contract exists, release packaging not established | [setup](../src/setup.rs) | No clean-install or package check |

## Discrepancies and missing pieces

- `src/gui/live.rs` says effects remain behind unfinished worker/save contracts, but the controller calls Apply, Verify, save-only, and reconciliation. Correct that comment under V1-004 after verifying exact behavior.
- `src/main.rs` help describes the GUI mainly as live inventory and omits import/recovery commands accepted by its dispatcher. V1-004 owns accurate help text without changing operations.
- The sample guide previously described pending GUI binding and named deleted integration tests. This reset replaces those assertions with current schema/loading information.
- The probe manifest previously instructed builds of deleted laboratory binaries and linked deleted evidence. This reset preserves historical artifact metadata while marking it as unverified context and removes executable instructions for absent tools.
- `config/project.toml` contains a disposable slot, old task IDs, environment pins, and machine paths. Source search found no direct references to this file, `config/artifact-pins.toml`, or `config/runner-policy-v1.json` in the product modules/build script. Their continuing development role needs review under V1-003; do not treat them as current product input.
- External `tests/`, `scripts/`, `.github/workflows/ci.yml`, `.codex/` configuration, and laboratory/reset tooling were deleted before this reset. They remain deleted unless a named tooling task justifies an individual replacement or restoration.
- No current support matrix or live evidence for the two-VM release requirement is established. Exact guest versions, driver, workloads, and run duration must be recorded before qualification.

## Refreshing this page

Update affected rows when implementation changes. Record validation with a revision and evidence reference; do not turn every row into a duplicate backlog card. A passing automated gate establishes its own coverage, not live host stability or concurrent GPU sharing.
