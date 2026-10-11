# Testing and qualification

[Roadmap](ROADMAP.md) owns milestone gates; [backlog](BACKLOG.md) owns tasks. This page owns validation timing, scenarios, and evidence.

## Timing and authorization

Run builds and automated tests at agreed milestone completion or on explicit request. Individual tasks are not milestones. Test source may be added earlier and labeled unrun.

Documentation-only work may check links, anchors, task consistency, and diffs without compiling or launching the product. Live GUI launch, guest access, VM/GPU changes, privileged setup/enrollment, and protected writes require explicit authorization for a concrete sequence. The documentation Q&A authorizes none of those effects.

Retired mock/snapshot GUI tooling is not used or maintained. Meaningful isolated backend unit tests remain useful; inspect their effects before selecting them.

## Planned automated gate

APP-008 prepares the gate against the current source and runs it at M1 completion. The prior gate script is absent. Do not claim an absent command passed or restore old tooling wholesale.

Repeat the [APP-001 through APP-008 source review](evidence/m1-source-review.md) after APP-006/APP-007 implementation and before this gate. Current-source fixes and added regressions do not constitute a passing gate.

| Order | Check | Intended command or behavior |
|---|---|---|
| 1 | Documentation | Local links/anchors, task references/dependencies, instruction consistency, diff whitespace |
| 2 | Formatting | `cargo fmt --all -- --check` |
| 3 | Non-hardware tests | `cargo test --locked --all-targets`, after inspection and explicit exclusion of any effectful suites |
| 4 | Lint | `cargo clippy --locked --all-targets -- -D warnings` |
| 5 | Product binaries | `cargo build --locked --bins` |
| 6 | API documentation | `cargo doc --locked --no-deps` |

The gate is non-elevated, fails on the first failed stage, reports what ran, and retains local evidence. It does not install software, launch the GUI, prompt for credentials, modify protected stores, access VMs, or execute hardware workloads.

## Non-hardware acceptance scenarios

- VRAM minimum is 1 GB; upper bound follows matched card capacity; units/provider mapping remain truthful; incompatible settings are not silently rewritten.
- Edits have no effects; review binds the target, proposed settings, and lifecycle; stale review is rejected.
- Initial setup includes verification and confirmed final-off intent; later toggles/VRAM changes do not repeat preparation or graphics verification and always finish off. Cover ordinary recovery without guest access, refused/unconfirmed shutdown, and explicit setup/update review for invalid preparation.
- Credential explanation names the VM and precedes the local prompt; cancelling either panel prevents worker submission and retains edits/recovery; remembering requires opt-in; explanation/vault failures remain explicit.
- Settings survive reload, drafts remain separate, and conflicts block stale execution.
- Additive enrollment and selected-VM operations preserve other targets.
- Publication failure permits bound save-only retry; uncertain effects retain recovery and never trigger blind replay.
- Reconciliation handles a start error after a power change, safely restores only unchanged approved facts, retains refused-shutdown failures, and blocks restoration after changed facts. Pending journals without a save record still block new controller reviews.
- Existing trust, IPC, locking, transport/output budgets, and interruption boundaries remain enforced.

Add coverage for real missing behavior and failure boundaries. Reuse meaningful existing tests. Dedicated retired GUI simulation tests were removed through APP-001. The replacement startup-dispatch regression and retained live controller/state tests await APP-008 execution.

## Live qualification scenarios

APP-009 prepares the exact environment, targets, effects, workload interval, evidence locations, final power states, and stop conditions before requesting authorization.

| Scenario | Acceptance |
|---|---|
| Initial setup | Review explains lifecycle and time; credential explanation identifies guest sign-in; hardware verification reports pass/fail; completed setup ends with VM off |
| Ordinary changes | Later toggle and VRAM updates use reviewed effects without repeated full preparation or initial graphics verification |
| Lifecycle | Actual power transitions and final state match the review; failures report independently observed state |
| Saved state | Settings survive reopen; drafts and observations remain distinct |
| Concurrent use | Two guests identify the selected hardware and run checked rendering during an overlapping interval |
| Independent operation | Selected-VM changes preserve the other guest's attachment, settings, and rendering |
| Real GUI usability | Names, scrolling, keyboard, review, progress, cancellation, and failure presentation work in the authorized live workflow |
| Failure/recovery | Safely applicable approved scenarios retain evidence and holds; save-only does not replay GPU effects |
| Host observation | Record responsiveness and any failures during the defined interval |

Select exact host/guest builds, GPU/driver, two existing Generation 2 Windows 11 guests, and representative workloads before executing. Qualification checks can observe concurrent rendering without adding repeated rendering verification to ordinary product operations.

Do not force power-off, induce hangs, restart the host, or automatically retry. Stop on uncertain outcomes and retain evidence. Live fault injection requires specific authorization. APP-010 adds separately authorized package installation and upgrade trials.

## Evidence and claims

Record revision/working-tree context, commands, exact environment, authorized sequence, timestamped stages, outcomes, and exclusions in ignored local evidence storage. Flush operation-boundary records so crashes retain the last known stage. Exclude credentials.

Report Passed, Failed, Not run, and Unknown honestly. A scoped pass applies to its recorded environment and interval; it does not prove universal stability or VRAM isolation. Readback or sequential verification alone does not prove concurrent sharing.
