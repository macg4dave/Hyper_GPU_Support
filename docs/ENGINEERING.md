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

Validation is recorded as **Not run**, **Pending milestone gate**, **Passed** with evidence/scope, or **Failed** with the failing stage. Use **Unknown** for capability behavior that cannot be established from existing evidence. Source assessment separately uses Implemented/Incomplete/Unknown; see [baseline](ARCHITECTURE.md).

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
| [UI text inventory](UI_TEXT_INVENTORY.md) | Presentation consumer/classification audit and extraction coverage |
| [UI text baseline](UI_TEXT_BASELINE.md) | Final extraction mapping, historical output fixtures and semantic regression requirements |
| [Architecture](ARCHITECTURE.md) | Components, state separation, data flow |
| [Configuration](CONFIGURATION.md) | Production intent schemas, storage, compatibility |
| [Agreed workflow](../README.md#agreed-workflow) | Existing user workflow |
| [Security](SECURITY.md) | Authority, trust, recovery semantics |
| [Validation](#validation-and-live-acceptance) | Check effects, gate evidence and live acceptance boundaries |
| `.github/prompts/*.prompt.md` | Focused task/coding guidance routed by AGENTS.md; no duplicated product requirements |

Change the authoritative page, then update links/summaries that need it. Source behavior outranks stale descriptions; current user decisions outrank historical plans. Keep machine-specific values in explicit local configuration/evidence and general policy in the handbook. Old task IDs remain context only; new IDs are stable once assigned.

The root agent file routes Codex to only the applicable specialist/task guidance. Supporting prompt files are read through that explicit routing; their filenames are not an automatic Codex loading mechanism. Keep operational guidance concise and product requirements in their owning pages.

## Tooling and change hygiene

Rust is product code. Narrow PowerShell development scripts may orchestrate Cargo, documentation checks, and explicitly authorized diagnostics. They must not become alternate product logic. Keep the default quality gate non-elevated and non-hardware; do not restore old scripts, CI, agents, or laboratory code wholesale.

Preserve user edits/deletions; do not reset/clean the working tree or stage unrelated changes. No dependency upgrades, format-all rewrites, or generated artifact updates belong in a documentation reset. Current toolchain versions come from manifests, not memory. Reassess removed CI and helpers individually with a concrete purpose and effect review.

## Validation and live acceptance

Run `powershell -NoProfile -File scripts/testing/check.ps1` from the repository root at integration/migration checkpoints. Inspect the script and test effects first. The gate requires a non-elevated shell and records revision, working tree, input hashes, commands and stage results in `local/evidence/`. It runs catalogue failure tests, resource validation, copy scanning, documentation, formatting, non-hardware tests, lint, binary builds and API documentation. Stop on a failing stage; remaining stages are Not run. The explicit resource rebuild proof is `python -B scripts/testing/test-ui-text-rebuild.py`; it temporarily changes prepared resources and restores them in `finally`.

These checks do not authorize a live application launch, native credential prompt, guest access, installed runner/setup, protected writes, VM power transitions or GPU/driver effects. Prepare exact targets, effects, evidence destination and stop conditions before requesting explicit authorization. Use local credential prompts and opt-in storage. Retain uncertainty and recovery evidence; do not blindly replay, clear holds or force power-off.

For UI text acceptance, prepare representative labels, dialogs/consent, progress, accessibility and long content at the supported minimum size, recorded scaling and themes. Record wrapping/clipping, reachable actions, keyboard focus and scrolling, with revision and actual coverage. Rendering does not establish guest or VM behavior. Keep L3/L4 open while their required visual/interaction checks are unrun. Existing source and historical acceptance are separate from new evidence.

## Optimization measurements

OPT-001 uses the existing protected diagnostics opt-in. It adds no configuration switch, telemetry service or caller-selected log path. [Work counters](../src/work_counts.rs) are a fixed array of saturating unsigned integers, local to the calling thread. Incrementing a counter performs no I/O, privilege check, allocation or fallible operation. Disabled diagnostics retain only that bounded in-memory accounting. Counters cannot authorize admission, skip authentication, certify completion or retire recovery.

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

### Original isolated expectations

[Baseline fixtures](../src/workflow/baseline_tests.rs) freeze the pre-optimization workflow requests. The following are source-derived expectations, **not executed measurement results**; their first execution is pending the milestone gate. The fixture uses one generation-2 VM, one supported GPU, four requested resource categories, deterministic payload identity and no concurrent manager. Setup starts with no journal/adapter and default settings; later cases begin with completed setup, matching settings and an Off VM unless stated. Fixture construction is excluded from subsequent counters.

| Journey including review/admission | VM/GPU/journal reads | Journal writes | Workflow payload calls | Starts/shutdowns |
| --- | --- | --- | --- | --- |
| First setup from Off | 47 / 43 / 49 | 7 | 2 | 3 / 3 |
| First setup from Running | 41 / 38 / 44 | 7 | 2 | 2 / 3 |
| VRAM-only Apply, eligible preliminary GUI plan | 21 / 19 / 19 | 3 | 1 | 0 / 0 |
| Same Apply, full explicit plan/current CLI path | 21 / 19 / 19 | 3 | 2 | 0 / 0 |
| Unchanged enabled Apply | 10 / 8 / 8 | 1 | 1 | 0 / 0 |
| Disable | 16 / 13 / 13 | 2 | 0 | 0 / 0 |
| Reenable completed setup | 26 / 23 / 23 | 3 | 1 | 0 / 0 |
| Verify from Off / Running | 8 / 5 / 12 | 4 | 2 | 1 / 1 or 0 / 0 |
| Reinstall/repair from Off, attached | 48 / 43 / 49 | 7 | 2 | 3 / 3 |
| Save-only read/validation contract | 1 / 1 / 1 | 0 | 1 | 0 / 0 |

The VRAM-only original path requests all four categories; reinstall detaches/reattaches and requests all four. Setup writes compatibility settings and attaches once. Disable restores owned settings and detaches without payload/guest work. Reenable writes settings, attaches and requests all four without preparation/graphics. These assertions protect permitted/forbidden effects as well as counts. Save-only fixtures stop before protected publication and do not simulate a successful save.

The worker's independent final payload authentication is outside the workflow-double fixture: current source adds one full host pass for enabled Apply and Verify, producing three for setup/reinstall/Verify and full-plan ordinary Apply, or two for eligible preliminary ordinary Apply. Disable remains payload-free. These are path counts, not executed native results. Save-only authenticates once when its receipt carries a payload digest and never replays Apply.

Startup, polling and explicit Refresh currently share full protected Discover. On the successful native discovery path, for V VMs, A VMs with adapters and G partitionable GPUs, source inspection gives V+2 provider connections and 2+4V+A+G queries per Discover, excluding artifact verification/admission and unrelated completion work. Every selected-VM inspection creates its own session; KVP is queried per successfully inspected VM and signed-driver metadata per GPU. Partial-read failures change these counts. Installed artifact verification hashes each enrolled artifact per invocation; record the actual policy membership and number of invocations. Native inventory/hash/transfer totals need the instrumented authorized run, not invented Fake provider counts.

### Comparison and acceptance procedure

1. Retain an unoptimized source/build identity before changing each affected path. At the coherent checkpoint, run the complete non-hardware gate; `cargo test --lib workflow::baseline_tests -- --nocapture` can reproduce the isolated expectations when test execution is requested. Preserve failures rather than automatically replacing fixture expectations. An optimized expectation change must explain the removed calls and unchanged safety checks.
2. Prepare exact candidate identities, VM/GPU targets, initial power/attachment/settings, management and recovery receipts, operator/enrollment/configuration revisions, driver/manifest membership and helper/payload file counts/bytes. Record host/guest versions, inventory size, cache/warm-up conditions, concurrent workload and diagnostics settings. Never clear recovery or reset a VM merely to make measurements comparable.
3. Under separate live authorization, compare open/idle polling/Refresh, setup from Off and Running (attached/unattached where eligible), ordinary VRAM/toggle changes, disable, Verify, repair, enrollment continuation, recovery, save-only and profile deletion. Capture disjoint root counters and stage durations, external whole-journey elapsed time, acknowledged bytes, actual transitions, terminal product result, independent saved/current readback, final power and retained recovery. Keep raw non-secret output in ignored `local/evidence/` only when collected.
4. Match diagnostics settings before/after. Compare several equivalently prepared runs and report range/variability, including a separately authorized diagnostics-disabled timing pair where practical. Do not subtract an assumed constant flush cost or invent percentage targets. Disclose the excluded terminal flush and unexported frontend/PowerShell work; use external elapsed time for whole-operation cost.
5. Cover rejected current OS, failed required/optional reads, same-version changed payload/helper, stale review/revision/topology, foreground/poll races, read cancellation/deadline expiry, progress disconnect, partial mutation/preparation, failed final readback/publication and interrupted cleanup in the owning source packages. Isolated failure fixtures run without real VMs; unsafe live fault scenarios stay unrun unless specifically prepared and authorized. No cancellation or retry of effects is introduced by measurement.

OPT-001 source includes aggregate attribution, nested/failed/incomplete diagnostic records, disabled-diagnostics behavior, hash/partial-transfer accounting and frozen workflow fixtures. Automated execution is pending the milestone gate. Machine timings, native count validation, protected installation and installed/VM/GPU acceptance remain pending under OPT-008; no speedup or acceptance is established by this source package.

## Completion report

State the behavior/files changed, why, exact validation performed or deferred, relevant uncertainty, and next task. Keep the report short, but disclose failures and untested product changes. Update milestone status only from its gate evidence. Publishing, pushing, deployment, and live qualification require their own applicable user instruction.
