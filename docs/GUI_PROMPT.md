# Documentation reconciliation brief

**Status:** Repository/documentation audit recorded on 9 October 2026 under
[PLAN-001](BACKLOG.md#plan-001). No implementation or live testing was performed.
Use this brief for a requested documentation refresh; do not restart the audit
merely because the project has selected Slint.

Read [AGENTS](../AGENTS.md), [README](../README.md), [ROADMAP](ROADMAP.md),
[BACKLOG](BACKLOG.md), [GUI_GUIDE](GUI_GUIDE.md), [GUI_ROADMAP](GUI_ROADMAP.md),
[ARCHITECTURE](ARCHITECTURE.md), [CONFIGURATION](CONFIGURATION.md),
[DECISIONS](DECISIONS.md) and relevant Rust source/test code. Follow the active
user request: documentation editing may be authorized even when code is not.

## Audit and directly reconcile

- Preserve implemented native backend, parser, shared workflow/plans, protected
  enrollment/runner, Named Pipe authentication, operation lock and recovery journals.
  Reuse sound `gui_model` state/save-conflict logic; Win32 presentation is disposable.
- Distinguish current implementation, source-level tests, historical passes and
  planned behavior. Use the [source matrix](ARCHITECTURE.md#repository-audit--9-october-2026)
  as the starting point; do not invent new test results or capability guarantees.
- Apply approved GUI_GUIDE decisions: one GUI/CLI/restricted-worker executable,
  VM cards and persistent split, one staged draft, physical GPU selection and all
  twelve resource fields, fresh Review & Apply, worker-only post-readback per-VM
  ProgramData saving, save-only recovery and stale-change blocking.
- Retain one host-wide modifying operation, one GUI per Windows session, normal
  close deferral and dashboard recovery warning/blocked modifications after interruption.
  No automatic GPU rollback or blind retry.
- Separate lightweight status/discovery from driver validation/planning. Expose
  provider unsupported/unknown states and raw units truthfully. Assign unverified
  provider/Slint/security details to owning cards rather than quietly reducing scope.
- Correct current links/paths and task responsibilities/dependencies. Preserve task
  IDs, completed milestones and historical evidence. Do not create duplicate guides.

ROADMAP owns milestones; BACKLOG owns status/dependencies/card acceptance; GUI_GUIDE
owns user requirements and OPEN questions; ARCHITECTURE owns source audit/current
behavior; CONFIGURATION owns storage contract; SLINT_RULES owns toolkit practice.
[The implementation prompt](../.github/prompts/SLINT_CODEX_PROMPT.md) is for separately
requested code work and links to these authorities instead of duplicating them.

## Documentation-only restrictions and completion

Edit documentation directly when requested. Do not change Rust, Slint, Cargo,
scripts or configuration; do not build, install, elevate, access credentials,
query/modify Hyper-V or change guest/host power. Do not migrate existing data.

Check Markdown targets/anchors, task IDs/dependencies, consistency and diff hygiene.
Report changed files, key corrections and remaining technical questions/user scope
decisions. Do not begin implementation. Future implementation/live tests follow
the active user scope and AGENTS; this brief creates no new approval boundary.
