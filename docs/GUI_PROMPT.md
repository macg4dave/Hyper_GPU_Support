# GUI documentation maintenance brief

The approved completed prototype is now the main Slint application, in root
`ui/` and `src/gui/`. [GUI_GUIDE](GUI_GUIDE.md) defines its v1.0 scope;
[ROADMAP](ROADMAP.md) owns priorities and [BACKLOG](BACKLOG.md) task status.
Use this brief for requested documentation maintenance, not a repeated design audit.

Preserve the existing interface; retire requirements absent from the prototype.
Do not restore Activity, search/filters, extra pages/actions or a setup wizard.
Keep historical task IDs, evidence and decisions as history, without treating
superseded GUI planning as current requirements.

Distinguish presentation promotion from backend integration: operational GUI data
and callbacks remain mocked; CLI/backend functionality and protected runner remain
working. Same-executable worker, secure persistence and real stages/recovery remain
on existing cards. Preserve security and all CLI functionality regardless of GUI scope.

Update current claims and links against source and actual tests. Do not invent
hardware passes or real bindings. A documentation-only request authorizes docs
and relevant checks, not implementation/live effects. The
[implementation prompt](../.github/prompts/SLINT_CODEX_PROMPT.md) guides requested code work.
