# Copilot repository instructions

Follow [AGENTS](../AGENTS.md) and affected [ENGINEERING](../docs/ENGINEERING.md)
sections. [ROADMAP](../docs/ROADMAP.md) owns delivery order;
[BACKLOG](../docs/BACKLOG.md#next-implementation-action) owns task selection.

Implement the next ready functional GUI slice using existing Rust backend code.
Preserve the approved [Slint interface](../docs/GUI_GUIDE.md), CLI, exact protected
enrollment, privilege/IPC/trust boundaries, recovery and host-lifecycle rules.
Inspect source first; do not restart completed research, diagnostics or lab work.
Record concrete blockers and move to another safe implementation slice.

Author focused tests for meaningful new logic. Follow AGENTS milestone testing
policy: no automatic tests/builds/GUI launches after edits or individual tasks.
Live Hyper-V/GPU/driver/VM-power tests require explicit permission; host restart,
shutdown or session termination always requires immediate explicit permission.
Report actual changes and deferred validation; keep documentation updates narrow.
