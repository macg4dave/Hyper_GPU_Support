# Engineering workflow

This page owns task/status conventions, documentation maintenance and [validation boundaries](#validation-and-live-acceptance). [AGENTS.md](../AGENTS.md) owns agent behavior. Use [backlog](backlog.md#next-action) for work selection within the current user request.

## One task at a time

Start by inspecting current source and user changes. Confirm the requested task's objective and dependencies, then make the smallest coherent change that satisfies its acceptance criteria. Existing implemented paths should be audited and corrected where needed, not rewritten to match an old proposal.

Do not automatically continue into another backlog card. Directly related fixes needed to complete the requested task belong in that task. Scope changes affecting product direction, authority, interface design, or release requirements require discussion and a recorded decision.

Update affected documentation and backlog status as part of the change. Give concise progress updates during longer work. Do not create a new planning file for each conversation or duplicate the same invariant across several prompt files.

## Status and evidence

Use separate implementation and validation fields:

| Implementation status | Meaning |
|---|---|
| Ready | Dependencies satisfied; work may begin when requested |
| In progress | Current requested task is being handled |
| Blocked | A named prerequisite or necessary decision is missing |
| Done | Task deliverables and its stated acceptance criteria are met; validation field still states exactly what ran |
| Deferred | Outside the agreed release or waiting for a new requirement |

Validation is recorded as **Not run**, **Pending milestone gate**, **Passed** with evidence/scope, or **Failed** with the failing stage. Use **Unknown** for capability behavior that cannot be established from existing evidence. Describe source coverage separately as Implemented, Incomplete or Unknown.

A code task can be implementation Done with validation Pending milestone gate when its criteria permit deferred checks. It cannot close a milestone whose gate has not passed. A documentation task can complete through static checks without a build. Never label a failure, an existing test function, or a user-facing simulation as a passing runtime result.

Evidence for a gate/run records revision and working-tree context, exact commands/scenarios, environment, timestamps, outcomes, and exclusions. Use ignored `local/evidence/` for raw non-secret output. Only promote concise reviewed summaries into tracked documentation when necessary. QA reports are temporary user-owned inputs: the user deletes them when finished. Do not create or maintain QA reports, case files, indexes, report archives or duplicate report histories unless explicitly requested. Give concise fix/check/retest summaries in chat by default. Respect deleted reports; remove or repair surviving stale references without restoring reports or treating their absence as unfinished work. Report deletion does not establish a passing test or release acceptance. Automated gate output may remain in ignored `local/evidence/`; protected operational journals and unresolved recovery records follow the existing security/recovery rules and are separate from disposable QA reports. Do not fabricate an evidence link before its file exists.

## Documentation ownership

| File | Authority |
|---|---|
| [README](../README.md) | Product overview and navigation |
| [AGENTS.md](../AGENTS.md) | Codex behavior and authorization |
| [Rust guidance](../.github/prompts/rust.prompt.md), [Slint guidance](../.github/prompts/slint.prompt.md) | Focused coding practices, loaded for applicable source work |
| [Roadmap](roadmap.md) | Release requirements, sequence, milestone gates |
| [Backlog](backlog.md) | Task IDs, dependencies, implementation/validation status, next action |
| [UI text guide](UI_TEXT_GUIDE.md) | Catalogue schema, formatting/generation contract and editor handoff |
| [Architecture](ARCHITECTURE.md) | Components, state separation, data flow |
| [Configuration](CONFIGURATION.md) | Production intent schemas, storage, compatibility |
| [Agreed workflow](../README.md#agreed-workflow) | Existing user workflow |
| [Security](SECURITY.md) | Authority, trust, recovery semantics |
| [Validation](#validation-and-live-acceptance) | Check effects, gate evidence and live acceptance boundaries |
| `.github/prompts/*.prompt.md` | Focused task/coding guidance routed by AGENTS.md; no duplicated product requirements |

Change the authoritative page, then update links/summaries that need it. Source behavior outranks stale descriptions; current user decisions outrank historical plans. Keep machine-specific values in explicit local configuration/evidence and general policy in the handbook. Remove completed task cards and superseded plans; keep IDs stable for remaining work.

The root agent file routes Codex to only the applicable specialist/task guidance. Supporting prompt files are read through that explicit routing; their filenames are not an automatic Codex loading mechanism. Keep operational guidance concise and product requirements in their owning pages.

## Tooling and change hygiene

Rust is product code. Narrow PowerShell development scripts may orchestrate Cargo, documentation checks, and explicitly authorized diagnostics. They must not become alternate product logic. Keep the default quality gate non-elevated and non-hardware; do not restore old scripts, CI, agents, or laboratory code wholesale.

Preserve user edits/deletions; do not reset/clean the working tree or stage unrelated changes. Keep dependency upgrades, source formatting and generated artifact changes out of documentation cleanup. Current toolchain versions come from manifests, not memory. Reassess removed CI and helpers individually with a concrete purpose and effect review.

## Size accounting

The [simplification roadmap](roadmap.md) sets a **10,000-line goal for `src/` only, excluding Rust tests**. The user clarified this boundary after the first GUI checkpoint. Count maintained program code under `src/`, including Rust, Slint, guest PowerShell, comments and blank lines. Repository tools, developer scripts, integration tests, root build code and probes outside `src/` do not consume this budget and are outside this simplification task. Use the union of tracked and non-ignored untracked files, deduplicate paths, and omit intentionally deleted files. Count a final unterminated line once. Normal formatting remains required.

Exclude inline Rust test items/modules, Rust integration tests and helpers/doubles compiled only for tests. Report those lines separately; Rust tests have no size budget. Shared code compiled into the program remains counted even when tests also use it. Code outside `src/` is not counted. The revised baseline must separate inline test items correctly rather than subtracting everything after the first `#[cfg(test)]` or relying only on filenames.

Tests follow the program's design. Remove unused, obsolete or redundant tests and test-only helpers as their purpose disappears; update useful tests to the simplified interfaces. Existing test structures do not justify retaining unused production APIs, compatibility layers or indirection. Keep meaningful checks of retained product behavior and failure contracts, without imposing a test-driven architecture on the program.

Exclude third-party `vendor/` and `probes/cuda/upstream/`, build output, documentation and inert assets/configuration/fixtures. Disclose excluded data/generated/third-party source separately. Generated build output is excluded. Do not move product logic out of `src/`, into data or into generated output to satisfy the goal; the scope clarification is not permission to displace the program. Source-bearing fixture/template extensions must be reviewed even when outside the extension list.

The corrected `src/` baseline is **29,141 counted lines**, excluding **17,160 Rust test/helper lines**, from **46,301 physical lines**. The first core slice is 28,938 counted lines. The first GUI checkpoint is 28,627 counted lines. The subsequent request-capture cleanup is **28,541 counted lines**, excluding **17,129 Rust test/helper lines**, from **45,670 physical lines**. Its latest validation is recorded below.

Earlier wider inventory on 18 October 2026 at `843997179407e727a7f2cb5edc587d9df74a8e27`: **50,595 lines in 171 code files**, including Rust tests. That earlier syntax-based count was **31,131 program/tool lines**, with **19,464 Rust test/helper lines** excluded. It includes **1,990 program/tool and 2,304 Rust test/helper lines outside `src/`**; those are now outside the goal. Wider totals are retained only for historical comparison. Source was unchanged during planning; the pre-existing Rust prompt edit and untracked complexity review were preserved.

| Gross inventory area, including Rust tests | Lines |
| --- | ---: |
| `src/`, including inline/separate tests, Slint and guest PowerShell | 46,301 |
| Integration tests in `tests/` | 1,642 |
| Text-generation/validation code and its tests in `tools/` | 1,979 |
| Developer scripts | 604 |
| `build.rs` and project-owned shader | 69 |
| Total | **50,595** |

Separate excluded maintenance inputs: active catalogue TOML **5,497 lines**, text contract TOML **5,605 lines**; third-party vendored text **10,322 lines** and CUDA upstream text **390 lines**. These are selected text categories, not totals for all repository documentation, assets, fixtures or generated build output. The vendored backend includes a local fix and must still be maintained/accounted for if changed. Inert text is not executable source, but its duplicated declarations are part of the simplification work.

Gross inventory command, run from the repository root; this deliberately includes Rust tests and does not evaluate the revised goal:

```powershell
@'
from collections import defaultdict
from pathlib import Path
import subprocess
names = subprocess.check_output([
    'git', 'ls-files', '-z', '--cached', '--others', '--exclude-standard'
]).decode('utf-8').split('\0')
extensions = set('.rs .slint .ps1 .py .hlsl .cu .cpp .c .h .hpp .js .ts .mjs .cjs .sh .cmd .bat'.split())
totals = defaultdict(lambda: [0, 0])
for name in sorted(set(filter(None, names))):
    path = Path(name)
    if not path.is_file() or name.startswith(('vendor/', 'probes/cuda/upstream/')):
        continue
    if path.suffix.lower() in extensions:
        group = name.split('/')[0] if '/' in name else 'root'
        totals[group][0] += len(path.read_text(encoding='utf-8-sig').splitlines())
        totals[group][1] += 1
for group, (lines, files) in sorted(totals.items()):
    print(f'{group}: {lines:,} lines / {files} files')
print(f'Total: {sum(v[0] for v in totals.values()):,} lines / {sum(v[1] for v in totals.values())} files')
'@ | python -B -
```

This command reports the earlier wider gross inventory only; select its `src` row for the scoped physical total. OPT-009 counted Rust with the cached `syn 2.0.119` parser and source spans, excluded `cfg(test)` items/methods and their external module trees plus Rust integration tests, and retained mixed runtime/test items. Its temporary audit tool and per-file tables are in ignored `local/evidence/core-slice-count/`; no maintained size tool or dependency was added. OPT-011 adds count enforcement and accounting regressions. The roadmap budget is unproven until implementation fits it while preserving retained product behavior.

## Validation and live acceptance

Run `powershell -NoProfile -File scripts/testing/check.ps1` from the repository root at integration/migration checkpoints. Inspect the script and test effects first. The gate requires a non-elevated shell and records revision, working tree, input hashes, commands and stage results in `local/evidence/`. It runs catalogue failure tests, resource validation, copy scanning, documentation, formatting, non-hardware tests, lint, binary builds and API documentation. Stop on a failing stage; remaining stages are Not run. The explicit resource rebuild proof is `python -B scripts/testing/test-ui-text-rebuild.py`; it temporarily changes prepared resources and restores them in `finally`.

These checks do not authorize a live application launch, native credential prompt, guest access, installed runner/setup, protected writes, VM power transitions or GPU/driver effects. Prepare exact targets, effects, evidence destination and stop conditions before requesting explicit authorization. Use local credential prompts and opt-in storage. Retain uncertainty and recovery evidence; do not blindly replay, clear holds or force power-off.

For UI text acceptance, prepare representative labels, dialogs/consent, progress, accessibility and long content at the supported minimum size, recorded scaling and themes. Record wrapping/clipping, reachable actions, keyboard focus and scrolling, with revision and actual coverage. Rendering does not establish guest or VM behavior. Keep text migration acceptance open while required visual/interaction checks are unrun. Existing source and historical acceptance are separate from new evidence.

## Optimization measurements

Work measurement uses the existing protected diagnostics opt-in. It adds no configuration switch, telemetry service or caller-selected log path. [Work counters](../src/work_counts.rs) are a fixed array of saturating unsigned integers, local to the calling thread. Incrementing a counter performs no I/O, privilege check, allocation or fallible operation. Disabled diagnostics retain only that bounded in-memory accounting. Counters cannot authorize admission, skip authentication, certify completion or retire recovery.

Terminal `runner-request`, `worker-command`, `guest-worker`, `payload-processing`, `guest-transport`, `artifact-authentication` and `worker-availability` spans carry `work_schema: 1` and a `work` object. Existing per-file records receive no counter fields or additional events. Each summary is an inclusive interval on its PID/thread; nested summaries overlap. Sum disjoint root intervals for a journey, never a root plus its nested summaries. Include standalone artifact/availability summaries during elevated bootstrap before the request/command span starts. Guest processes have separate roots. The transport monitor explicitly contributes its validated acknowledgements to the supervising thread once supervision returns; exceptional supervision failure can leave partial counts.

| Counters | Attribution |
| --- | --- |
| `provider_connections`, `provider_queries`, `provider_gets` | Native ConnectServer boundary, ExecQuery submissions (including association WQL), and GetObject submissions in Hyper-V and driver discovery. Attempts include native rejection; enumeration waits/properties and guest PowerShell CIM calls are not additional queries in this schema. |
| `file_hashes`, `hashed_bytes`, `catalog_hashes` | File-hash attempts and bytes actually fed into Rust SHA-256; data-bearing catalog-hash submissions are separate. Includes installed artifact checks and payload/guest hashes, excludes small in-memory revision/manifest hashes and Windows-internal signature hashing. |
| `payload_passes` | Entries into the native backend's complete discover/hash/trust pipeline. A failed pass remains an attempt, not authenticated evidence. Preparation using an already admitted manifest is not another host pass. |
| `vm_reads`, `gpu_reads`, `journal_reads`, `journal_writes` | Calls at the workflow Backend boundary, including failed reads/writes. Nested native guard reads remain visible through provider counters rather than additional Backend calls. |
| `power_starts`, `power_shutdowns`, `attach_writes`, `detach_writes`, `settings_writes`, `vram_writes`, `compute_writes`, `encode_writes`, `decode_writes` | Native mutation submissions immediately after durable provider admission. No-op native guards are excluded. These are attempts, not completed transitions; readback, terminal outcome and recovery evidence establish what actually happened. Isolated doubles count Backend requests instead. |
| `process_starts` | Supervised child and fixed elevated-worker launch attempts in this process. PowerShell-internal remote worker starts are outside this counter. |
| `helper_bytes`, `payload_bytes`, `transfer_chunks` | Validated incremental guest transfer acknowledgements, including partial files. Helpers precede payload files in the existing transfer order. Expected totals, duplicate/invalid messages, terminal replies and graphics success add no bytes. Counts do not prove installation. |
| `completion_reads` | Explicit independent completion checkpoints begun: core Apply/Verify final VM read, worker final confirmation/save-only admission, and configuration publication readback. A checkpoint may perform several reads; individual Backend/provider counters describe those calls. |
| `diagnostic_flushes`, `diagnostic_flush_ns` | Attempted durable diagnostic writes and time spent locking, serializing, writing and syncing those records. A span includes its entry write and nested records, but excludes its own terminal write because that sync happens after serialization. Record-building/logger-initialization overhead and PowerShell trace flushes are not included in this counter. |

Durations are monotonic elapsed milliseconds, including entry and nested diagnostic writes. A terminal `done` means the measured function returned `Ok`; it does not imply a saved/finished/verified product outcome. `failed` records failure with a bounded typed failure envelope when available, otherwise fixed `operation failed` text; aggregate measurement records do not copy arbitrary adapter errors. `incomplete` means the span dropped without a confirmed terminal record. Missing terminal logs, failed supervision, saturation or unavailable frontend counters make a comparison incomplete, never zero work. Ordinary frontends still create no protected traces: their hash calls are instrumented in memory but not exported by this opt-in. Disclose this exclusion rather than infer frontend runtime totals from runner logs.

### Count expectations

[Workflow count fixtures](../src/workflow/baseline_tests.rs) assert current candidate reads, writes, payload calls and power transitions. [Prepared-session regressions](../src/workflow/optimization_tests.rs) cover retained admission and independent final authentication. These isolated counts describe backend requests, not measured native queries or installed timings. An expectation change must explain removed calls and preserve permitted/forbidden effects.

For successful batched discovery with V VMs and G partitionable GPUs, source expectations are 5+G queries for Poll and 6+2G for Full when V is positive, or 2+G and 2+2G without VMs. Relationship fallback changes totals. Reads authenticate all enrolled artifacts once under the common lock; effect admission remains independent. Record actual membership, fallback and invocations during qualification.

### Comparison and acceptance procedure

1. Retain an unoptimized source/build identity before changing each affected path. At the coherent checkpoint, run the complete non-hardware gate; `cargo test --lib workflow::baseline_tests -- --nocapture` runs that checkout's comparison expectations when test execution is requested. Compare the identified unoptimized and candidate checkouts; do not infer original measurements from current expectations. Preserve failures rather than automatically replacing fixture expectations. An optimized expectation change must explain the removed calls and unchanged safety checks.
2. Prepare exact candidate identities, VM/GPU targets, initial power/attachment/settings, management and recovery receipts, operator/enrollment/configuration revisions, driver/manifest membership and helper/payload file counts/bytes. Record host/guest versions, inventory size, cache/warm-up conditions, concurrent workload and diagnostics settings. Never clear recovery or reset a VM merely to make measurements comparable.
3. Under separate live authorization, compare open/idle polling/Refresh, setup from Off and Running (attached/unattached where eligible), ordinary VRAM/toggle changes, disable, Verify, repair, enrollment continuation, recovery, save-only and profile deletion. Capture disjoint root counters and stage durations, external whole-journey elapsed time, acknowledged bytes, actual transitions, terminal product result, independent saved/current readback, final power and retained recovery. Keep raw non-secret output in ignored `local/evidence/` only when collected.
4. Match diagnostics settings before/after. Compare several equivalently prepared runs and report range/variability, including a separately authorized diagnostics-disabled timing pair where practical. Do not subtract an assumed constant flush cost or invent percentage targets. Disclose the excluded terminal flush and unexported frontend/PowerShell work; use external elapsed time for whole-operation cost.
5. Cover failed required/optional reads, same-version changed payload/helper, stale review/revision/topology, foreground/poll races, read cancellation/deadline expiry, progress disconnect, partial mutation/preparation, failed final readback/publication and interrupted cleanup in the owning source packages. Isolated failure fixtures run without real VMs; unsafe live fault scenarios stay unrun unless specifically prepared and authorized. No cancellation or retry of effects is introduced by measurement.

### Helper and transport measurement preparation

The sequential transport uses a Rust-owned 16 MiB chunk limit; the same constant supplies the PowerShell buffer and Rust acknowledgement bound. Timeouts, hashes, recovery and remoting resource caps are unchanged. Helper selection precedes all copies; successful matching validation removes that helper from acknowledged file/byte totals. Warm Verify can therefore have zero transfer while current payload and graphics checks remain mandatory. No native speedup is established by the source changes.

For the separately authorized candidate acceptance session, prepare the same target/manifest and diagnostics setting for cold missing helpers, two matching helpers, one changed helper and changed payload. Preserve hashes, lengths, membership, initial power/attachment and terminal readback for each comparison. Use existing `session-connect`, `bootstrap-leaves`, `bootstrap-publication`, `file-copy`, `chunk-round-trip`, `chunk-file-write`, guest payload hashing and `guest-d3d11-probe` spans, plus external journey elapsed time. Chunk round trip includes serialization, guest writing and reply; guest file-write timing is a nested component and must not be added to the inclusive round trip. Final durable flush and stream-open/close remain part of file-copy time.

Prepare a separate isolated PowerShell serialization microbenchmark with the same typed byte arrays (small helper, full 16 MiB chunk and exact final tail, with the prior 4 MiB size as a comparison), using `System.Management.Automation.PSSerializer.Serialize` and elapsed time/output length. Prepare a guest-local scratch-file write/flush comparison under that session's explicitly authorized staging scope. These measurements are not production-path timings and do not replace actual remoting round trips; do not subtract them as an exact estimate of remoting overhead. Keep session setup, serialization, file writes, hashing and graphics results separate, including warm-up and diagnostic-flush variability. Benchmark execution and guest scratch writes require their own prepared scope.

Real-session throughput, peak host/guest memory and slow-guest/transport-limit acceptance remain pending. Compare 4 MiB if qualification finds problems; changing the size never authorizes automatic retry or effect replay. Broader transport redesign requires a separate measured justification.

### Optimization validation checkpoint

The recorded 18 October 2026 non-hardware gate passed all stages: catalogue tests/validation, copy checks, documentation, formatting, product tests, strict Clippy, four binary builds and API documentation. It identifies revision `ce963ac9e38b44d2d917c0a3077549924ef7f3cf` plus its captured working tree and input hashes. Evidence: `local/evidence/m1-gate-20261018T034306748Z/report.json`.

The corresponding independent installation verification passed with matching build/installed/policy hashes for all four artifacts, preserved enrollment and 661 persistent files across three targets, and an enabled Ready runner. Evidence: `local/evidence/install-20261018T033924335Z/independent-verification.json`. These are recorded results, not checks rerun by documentation maintenance; compare candidate inputs before reusing them.

Installed GUI journeys, native work-count/timing comparisons, live guest cleanup, VM/GPU/guest acceptance and 16 MiB throughput/peak-memory/slow-guest qualification remain pending. Gate/install evidence does not establish a performance gain or final acceptance.

## Core simplification checkpoint

OPT-009 has an implemented source slice, not a complete feasibility result. On 18 October 2026, `powershell -NoProfile -File scripts/testing/check.ps1` passed every stage for revision `843997179407e727a7f2cb5edc587d9df74a8e27` plus the captured working tree/input hashes. Evidence: `local/evidence/m1-gate-20261018T051126336Z/report.json`. The gate covered catalogue validation/failure checks, copy scanning, docs, formatting, 477 product Rust tests, strict Clippy, four binary builds and API documentation. Separate catalogue tests also passed. No installed application, credential/vault, protected installation, guest, VM or GPU acceptance was run.

The slice removes the ordinary CLI prediction/readmission detour and worker optional backend branches, shares the typed Configure steps between preview/execution, and shares completion merging across Configure/save-only/Verify/workflow reconciliation. A newer same-VM draft survives completion, including failed readback. Added regressions cover ordinary/no-op/omitted-category behavior, power approval, credential-free preparation, direct-plan rejection and retained drafts. The obsolete ordinary-prediction test was replaced. Existing stage/fault, mutable-drift, cancellation/release, independent final authentication and save-only tests passed.

The first gate stopped on two pre-existing missing fixed catalogue declarations for recovery text already used by the current GUI. Those declarations were added without changing English wording or runtime flow. The next run exposed dropped settings-error context and an incomplete new VRAM fixture; both were corrected before the complete passing run. Four existing formatting differences were corrected for the required formatting stage. Failed gate evidence remains in its original ignored directories; passing evidence does not erase those attempts.

Source counting used the temporary syntax audit under `local/evidence/core-slice-count/`: **31,131 -> 30,928 counted lines**, excluding **19,464 -> 19,645 Rust test/helper lines**. Gross physical source is **50,595 -> 50,573 lines**. The difference between physical deletion and counted reduction includes old direct-execution helpers now compiled solely for existing isolated tests; they are no longer production APIs. No product logic was moved to scripts, data or generated output. The [roadmap's measured gap](roadmap.md#size-budget-and-measured-gap) replaces speculative per-area allocations; complete 10,000-line feasibility remains open.

## GUI simplification checkpoint

On 18 October 2026, the complete non-hardware gate passed for the subsequent GUI batch at the same base revision plus captured working-tree inputs. Evidence: `local/evidence/m1-gate-20261018T052505288Z/report.json`. All ten stages passed, including 466 product Rust tests, 12 separate catalogue tests, copy validation, strict Clippy, binary builds and API documentation. An earlier attempt stopped on a missing test-only `Plan` import after removal of its production import; the fixture was corrected before this passing run.

The GUI was traced by user task: select/inspect, edit intent, choose an action, acknowledge preparation, follow progress and complete/read back. Existing fields, callbacks, frame and copyable log remain shared. The two dialog families now use one footer for action choices, guards, Close/Cancel, focus reveal and scrolling. Redundant Apply/Verify plan stores were replaced by pure validation of the callback's captured plan. Real enrollment/removal/reset continuation state remains. Apply/Verify no longer build hidden running-body summaries; core plan facts and technical details remain. Obsolete cache/summary tests were removed or replaced by acknowledgement drift/failure cases and shared action/disabled-control checks.

The same syntax-based audit measures **30,928 -> 30,617 counted program/tool lines**, with **19,645 -> 19,425 excluded Rust test/helper lines** and **50,573 -> 50,042 gross source lines**. GUI code including Slint is **8,638 -> 8,327 counted lines**. This batch removes 311 counted and 531 physical code lines. These are the earlier wider repository counts. Under the clarified `src/` scope, this checkpoint is 28,627 counted lines; subsequent cleanup is recorded below. No program logic was relocated into excluded files.

The existing isolated layout tests were rerun with `HYPER_GPU_LAYOUT_RENDER_DIR` and one test thread to retain software-rendered dialogs under `local/evidence/gui-simplification-renders/`. Both dialog-family and operation-dialog tests passed. Light/dark, 700x520 and 1240x860 renders were visually inspected, including More Actions, save, reset result, running progress and the completed log. This is isolated component evidence; no installed GUI/native interaction, protected installation, credentials, guest, VM or GPU acceptance was run. Remaining controller consolidation and full-budget feasibility stay open under OPT-009/OPT-010.

## Captured GUI requests checkpoint

The request-capture follow-up passed the complete non-hardware gate on 18 October 2026: `local/evidence/m1-gate-20261018T053123962Z/report.json`. All ten stages passed, including 466 product Rust tests, 12 separate catalogue tests, strict Clippy, four binary builds and API documentation. The gate's 217 source/build/test input hashes were checked unchanged after completion; the final documentation update passed the documentation checker separately.

Enrollment, profile deletion and partial removal now pass their concrete checked requests directly into the existing operation callbacks. Five further global GUI fields were removed: enrollment review, setup artifact snapshot, deletion review, partial-removal review and an enrollment-recovery revision that had no production consumer. Their invalidation code and the obsolete generic cancellation wrapper were deleted. Request validation, cancellation/release suppression, protected receipts and independent worker revision/package/authority checks remain. Enrollment binding checks the complete captured draft; deletion still binds selection/pairing and readable saved-profile revisions. Meaningful drift, cancellation, retained-profile and failure tests pass; tests of removed caches were removed or adapted.

The scope is now **`src/` only**, excluding Rust tests/test-only helpers. Current measured totals are **28,541 counted lines**, **17,129 excluded Rust test/helper lines** and **45,670 physical lines**. This follow-up removes another 86 counted lines. The complete GUI batch removes 397 counted and 609 physical lines; GUI code including Slint is 8,241 counted lines. Across the core and GUI slices, the corrected baseline is **29,141 -> 28,541 counted lines** and **46,301 -> 45,670 physical lines**. Repository tooling is outside the goal; no product logic was moved outside `src/`.

The shared Slint components are unchanged from the visually inspected software renders above. No installed GUI/native interaction, privileged installation, credentials, guest access or VM/GPU effects were run. **18,541 counted lines remain above the goal**. Next: complete the source-only native/guest/worker replacement design and consolidate repeated orchestration without weakening authority or uncertain-outcome recovery.

## Completion report

State the behavior/files changed, why, exact validation performed or deferred, relevant uncertainty, and next task. Keep the report short, but disclose failures and untested product changes. Update milestone status only from its gate evidence. Publishing, pushing, deployment, and live qualification require their own applicable user instruction.
