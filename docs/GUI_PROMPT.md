# GUI documentation maintenance brief

The approved completed prototype is now the main Slint application, in root
`src/gui/ui/` and `src/gui/`. [GUI_GUIDE](GUI_GUIDE.md) defines its v1.0 scope;
[ROADMAP](ROADMAP.md) owns priorities and [BACKLOG](BACKLOG.md) task status.
Use this brief for requested documentation maintenance, not a repeated design audit.

Preserve the existing interface; retire requirements absent from the prototype.
Do not restore Activity, search/filters, extra pages/actions or a setup wizard.
Keep historical task IDs, evidence and decisions as history, without treating
superseded GUI planning as current requirements.

Normal GUI startup uses live data/supported bindings; unavailable actions never
simulate success. `--mock-gui` selects rehearsal: fixtures now, real read-only data
and shared plans progressively, with no effects or persistent writes. Preserve
required runner audit and qualify a separate no-write data source. CLI/backend
functionality remains working. Same-executable worker, persistence and real stages remain
on existing cards. Preserve security and all CLI functionality regardless of GUI scope.

Update current claims and links against source and actual tests. Do not invent
hardware passes or real bindings. A documentation-only request authorizes docs
and relevant checks, not implementation/live effects. The
[implementation prompt](../.github/prompts/SLINT_CODEX_PROMPT.md) guides requested code work.

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
