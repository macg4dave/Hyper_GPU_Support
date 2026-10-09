# v1.0 GUI delivery map

**Updated:** 9 October 2026. Deliver the completed Slint GUI as the main Windows
application using the existing Rust backend. [GUI_GUIDE](GUI_GUIDE.md) freezes
the interface; [BACKLOG](BACKLOG.md) owns status and acceptance.

| Existing task | Scope |
|---|---|
| GUI-001 | Completed prototype history; sources promoted to `src/gui/ui/` and `src/gui/`. No further design task. |
| APP-001 | No-argument Slint/headless CLI dispatch implemented. Console packaging and restricted same-executable worker remain pending; retain protected runner meanwhile. |
| GUI-002 / CORE-012 | Live cards/Refresh/System discovery implemented; configuration/status/action binding remains. `--mock-gui` uses fixtures now, then progressively actual read-only data/plans with execution rehearsed without persistent writes. |
| GPU-010 | Validate selector/advanced allocation units, bounds and readback; resolve real meaning of GPU Memory slider without enforcement promises. |
| CFG-001 / CORE-021 | Reuse parser/conflict handling and successful-but-unsaved separation for safe persistence and existing forget/save-only behavior. No new configuration UI. |
| SEC-001 / CORE-028 | Adapt runner/pipe/lock/journals for approved execution, real progress, downtime consent, uncertainty and manual reconciliation. Independent changed-boundary review. |
| GUI-003 | Qualify current screens/themes/navigation/dialogs, keyboard, splitter/scrolling, DPI, accessibility and software rendering. No added features. |
| CORE-017 / DOC-003 / GPU-014 | Package Windows candidate, verify actual commands/notices and separately authorized changed-path hardware qualification. |

Prototype development and removed Activity/search/filter requirements are retired.
No Win32 parity, additional wizard, persistent preferences or activate-existing
feature is added to v1.0. Preserve all working CLI/backend capabilities even if
absent from the GUI. Mock controls stay labelled; they are not hardware evidence.
Normal mode uses actual data and supported real bindings, never simulated success;
unconnected actions are unavailable. Mock real-data reads must preserve mandatory
audit by using a qualified authorized no-write reader/snapshot, not disabling audit.
The optional `--mock-gui --snapshot FILE` historical inventory reader is implemented
but unqualified; it blocks editing, live plans, credentials and execution. Plain
mock mode retains fixtures; recorded-plan/stage rehearsal remains open.
For this task live Hyper-V, GPU, driver and VM-power operations require explicit
permission. Physical-host lifecycle always requires immediate permission.
