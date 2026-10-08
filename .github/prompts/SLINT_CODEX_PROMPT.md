# Codex — Slint GUI

Implement the requested Slint task in `BACKLOG.md` (or the next ready task). Follow `AGENTS.md`, `ENGINEERING.md`, and `GUI_GUIDE.md`. Preserve other edits. If implementation is not authorised, plan only.

Use small, cohesive `.slint` components and Rust modules. The old Win32 GUI is disposable. Keep one shared Rust core for GUI/CLI; no Hyper-V logic or privileged commands in the UI. Use Slint layouts and constraints, not absolute positioning or arbitrary pixel sizes. Prefer built-in Fluent controls and centralised styling.

Follow the agreed design: VM cards, adjustable split view with horizontal scrolling, hybrid details panel, and editable GPU selection plus Min/Optimal/Max for VRAM, compute, encode and decode. Show all VMs. Separate observed state, saved configuration and one pending VM draft; prompt before switching with pending edits. Use only verified provider values and units.

Review and validate changes before applying. Elevate only protected operations; obtain approval before graceful VM shutdown. Never force power-off or change host lifecycle without permission. Recheck state, handle partial/uncertain results and never blindly retry Apply.

Keep work off the UI thread; report real progress. Use session-only diagnostics except essential recovery state. Test with mock data and Slint preview; verify sizing, DPI, keyboard use and error states. Avoid unrelated lab work.

Make one reviewable change at a time. Run relevant checks and report changes, test results and remaining issues.
