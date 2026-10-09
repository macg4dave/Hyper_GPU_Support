# Codex — Integrate the approved Slint application

Follow [AGENTS](../../AGENTS.md), [ENGINEERING](../../docs/ENGINEERING.md),
[GUI_GUIDE](../../docs/GUI_GUIDE.md), [SLINT_RULES](../../docs/SLINT_RULES.md)
and the selected [BACKLOG](../../docs/BACKLOG.md) card. User scope is authoritative.

The completed prototype, now in `src/gui/ui/` and `src/gui/`, defines v1.0. Preserve
its layout, controls, navigation, styling and behavior. Do not redesign, recreate
removed Activity/search/filter features, or add pages/controls without approval.
No Win32 parity requirement; that presentation has been removed.

No arguments open live Slint with actual data and supported real bindings;
unconnected actions remain unavailable. `--mock-gui` explicitly selects no-write
rehearsal, currently fixtures, then real read-only data and shared plans as integration
progresses. Simulate effects only in this mode; never persist config/credentials/
enrollment/journals/audits or launch guest probes. Do not suppress required runner
audit; qualify a no-write reader/snapshot separately. Bind existing controls
incrementally to the shared Rust core, reusing sound `gui_model`, discovery,
workflow/plans, configuration, credentials, runner, Named Pipe, lock and journals.
Keep all CLI/backend capabilities, even when absent from the GUI. Do not create
a parallel backend or security system.

Refresh uses lightweight discovery/status; Review & Apply needs a fresh plan and
independent privileged revalidation. Preserve draft, observed and committed state
separation. Validate provider units/bounds/readback; GPU Memory currently has only
mock meaning and does not promise physical GB allocation. Real progress replaces
the explicit simulation timer only when backed by actual events.

Keep the protected runner until independently reviewed worker consolidation.
Preserve enrollment, artifact trust, ACL/reparse, bounded messages, replay/audit,
lock and recovery contracts. Save-only retry never replays a GPU operation.
Real GUI effects require appropriate elevation, guest credentials and downtime
consent. Uncertain outcomes require reconciliation, not automatic rollback/retry.

Use mock data for GUI validation. Follow the active request for live-test permission;
this promotion request permits no Hyper-V/GPU/driver/VM-power effects without
explicit permission. Host lifecycle always needs immediate permission. Run actual
build/check/render/interaction tests and report passes, mocks and remaining gaps.

## Testing Policy — Codex

Stop running tests, builds, compilation checks, or launching the UI after every small change.

Follow these rules:

1. **During development:** Make changes without automatically running tests, `cargo check`, `cargo test`, `cargo build`, or launching the GUI.
2. **At milestone completion:** Run relevant tests and build checks once, after all work for that milestone is complete.
3. **On explicit request:** Run tests whenever I specifically instruct you to.
4. **Small changes:** Do not test individual edits, UI adjustments, layout changes, refactoring, or documentation updates.
5. **Failures:** If a milestone test fails, fix the relevant issue and rerun only the necessary checks. Avoid repeatedly running the entire test suite.
6. **Exceptions:** If you believe immediate testing is essential, explain why and request permission first.

Prioritise implementing the planned work over repeatedly validating intermediate states.

**Important:** Do not interpret completing an individual task or subtask as completing a milestone. A milestone is complete only when all its planned tasks are finished.

At the end of each task, briefly report what changed and whether it remains untested. Do not automatically start validation.

This policy overrides existing instructions to test continuously unless I explicitly tell you otherwise.
