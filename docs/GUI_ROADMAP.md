# Hyper GPU Support — Slint GUI Roadmap

**Status:** Planning-only v0.1  
**Updated:** 2026-10-09  
**Authoritative choices:** [`GUI_GUIDE.md`](GUI_GUIDE.md), especially architecture decisions A18–A32  
**Codex planning brief:** [`GUI_PROMPT.md`](GUI_PROMPT.md)

> This roadmap authorises **no code, build, installation, elevation, VM power change or GPU operation**. Obtain explicit approval for an implementation milestone. `windows_gui.rs` is a disposable experiment; do not port it for compatibility.

## Principles and gates

- One Windows executable: default Slint GUI, explicit CLI subcommands, internal elevated worker mode; common Rust core and validation.
- Per-VM configs in ProgramData keyed by VM GUID; draft in memory; protected worker writes only after Apply/readback. Save-only retry never repeats GPU operations.
- GUI normally unelevated. One short-lived elevated worker per authorised operation; fixed typed operations via private local Windows Named Pipe.
- Exactly one host-wide modifying GPU operation across GUI/CLI/workers; recovery hold remains distinct from lock state.
- On failure, stop modifying GPU state; manual reconciliation only. Read-only discovery remains available; persistent recovery warning on startup.
- Host GPU settings read-only. Physical GPU selection and per-VM VRAM/compute/encode/decode Min/Optimal/Max editable after capability validation.
- No application code from this plan until a separate implementation task is authorised. The old Win32 GUI need not be preserved.

## Milestone 0 — Read-only repository and host capability audit

**Scope:** Inspect Cargo structure, CLI routing, core API, existing config/runner/permission/recovery contracts, tests and relevant roadmaps. Verify Microsoft GPU-P provider semantics and current Slint APIs; read only unless an explicit test is approved.

**Deliver:** Reality-check matrix (existing / missing / unsupported / needs experimental verification); proposed compact module ownership; list of OPEN items with evidence and options; adjusted milestone estimates without pretending uncertain APIs are proven.

**Gate:** Review findings and resolve security/configuration interface decisions before any core change.

## Milestone 1 — Slint visual shell with mock backend

**Scope:** Modular shell/navigation, VM cards, split layout (never collapses), resizable divider, horizontal scroll for narrow windows, hybrid right-hand panel, Fluent theme tokens and static/mock states.

**Gate:** VS Code Slint preview; no Hyper-V needed; keyboard selection, focus, scroll, long names, multiple DPI/text scale and empty/error states visually checked.

## Milestone 2 — Executable dispatch and shared domain contracts

**Scope:** One executable routes no args to GUI, explicit args to CLI, restricted/internal worker to privileged path; common domain layer for VM identity, desired/observed/draft and typed operation states. No duplicate Hyper-V business rules in Slint.

**Gate:** CLI remains headless with reliable output/exit status; GUI launch has no unwanted console; mock tests show both entry points reuse the same validation contracts.

## Milestone 3 — Per-VM configuration persistence

**Scope:** Settle schema/versioning based on repo evidence, ProgramData paths and ACLs, stable VM GUID file names, atomic write and conflict detection, per-user visual preferences separately. Reads not treated as privilege grants.

**Gate:** No stale overwrite; safe rename/missing file handling; config edits stay in memory; only worker can commit; file-save failure is distinguishable from successful Hyper-V operation.

## Milestone 4 — GPU capabilities and allocation model

**Scope:** Detect eligible GPUs, current GPU identity and per-VM state; classify all twelve provider fields and input semantics; suggest values from real provider capabilities, not invented percentages. Host partition count displayed only.

**Gate:** Every field has verified units, bounds, or an honest unsupported/unknown state; mock-backed card/editor validation and CLI parity.

## Milestone 5 — Secure privileged execution boundary

**Scope:** Reuse/reshape approved fixed-operation runner, one per-operation elevated instance, private local Named Pipe, access control, peer authentication, message framing/bounds/timeouts, strict identity/plan validation. Coordinate one host-wide modifying operation across processes.

**Gate:** Denied UAC makes no VM change; malformed/untrusted/stale messages cannot run operations; simulated disconnection produces *uncertain*; locks do not erase durable unresolved recovery state.

## Milestone 6 — Plan, apply, progress and manual recovery

**Scope:** Adaptive Review & Apply, explicit guest-shutdown permission, stage progress, safe restoration of previously running state, independent readback, worker-owned post-success per-VM save, narrow save-only retry, minimal recovery journal. On GPU failure stop and require manual reconciliation.

**Gate:** Fault-injected partial success, crashes, timeouts and save failures do not trigger automatic GPU retry/rollback; stage status is accurate; recovery warning blocks new modifications after restart.

## Milestone 7 — Connect Slint controls to real backend

**Scope:** Replace mocks incrementally for discovered VMs, selection/filtering, four expandable allocation groups, pending-draft prompt, progress/recovery notices and credentials; preserve single pending VM draft.

**Gate:** UI and CLI produce the same plan for identical inputs; stale external file/Hyper-V changes block Apply; VM selection and unrelated refreshes preserve pending intent and scroll context.

## Milestone 8 — Process lifetime, integration and release readiness

**Scope:** Single GUI per Windows session with activate-existing behaviour, close deferral during active worker operation, crash/lost-pipe recovery, session-only diagnostics, GUI settings, installer path/ACLs and renderer fallback. Evaluate accessibility and actual Slint feature support.

**Gate:** Ordinary close cannot abandon a known active operation; another GUI launch does not clear draft; CLI and worker modes unaffected; mock/negative tests pass. Real VM tests only with specific user permission; no host reboot/shutdown without approval.

## Work that remains explicitly open

See `GUI_GUIDE.md` section 16 for the current `OPEN-*` table. Codex should challenge assumptions with evidence, not re-open settled user choices. The next actionable step is **Milestone 0, read-only audit**, after the user chooses to initiate it.
