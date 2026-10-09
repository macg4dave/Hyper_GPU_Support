# Hyper GPU Support v1.0 roadmap

**Updated:** 9 October 2026. **Primary objective:** deliver the completed Slint GUI
as the main Windows application, using the existing Rust backend wherever appropriate.
The prototype defines the interface: no redesign or restoration of removed features.
[GUI_GUIDE](GUI_GUIDE.md) defines scope; [BACKLOG](BACKLOG.md) owns status.

## Delivery priorities

1. Promote existing sources to the main executable: no arguments open Slint;
   explicit CLI commands run headlessly. This dispatch is implemented.
2. Preserve the current design and working interactions; fix integration defects.
3. Bind existing controls to shared Rust discovery, plans, execution, credentials
   and persistence where supported. Label all remaining mocks honestly.
4. Preserve CLI capabilities and backend security, validation and recovery.
5. Finish backend gaps required by existing controls without adding interface.
6. Package and validate the Windows application and necessary payloads/notices.

The protected SYSTEM runner remains the current backend boundary. A restricted
same-executable per-operation worker remains planned. Presentation promotion does
not implement that consolidation. Guest/probe payloads remain bounded. Console
packaging must preserve reliable CLI stdout/stderr and exit codes.

## Milestones and retained evidence

| Milestone | Result / remaining gate |
|---|---|
| M1 — product boundary | Accepted 8 October: existing-VM enrollment, protected runner, lab separation. [Evidence](evidence/M1.md). |
| M2 — NVIDIA core | Retain attach/render/reapply/disable results; affected fresh preparation remains on ARCH-001/GPU-012. [Evidence](evidence/M2.md). |
| M3 — main Slint application | Approved prototype promoted, GUI/CLI dispatch implemented. Backend bindings, supported allocations, secure persistence/execution and UX qualification remain. |
| R1 — packaged v1.0 | CORE-017/DOC-003/GPU-014: candidate, install/update/removal, licensing, GUI/CLI regressions and separately authorized hardware qualification. |
| M4 — later qualification | Additional vendors and simultaneous sharing after supported-hardware qualification; no fairness or scheduler claim. |

GUI-001 is completed history. APP-001 owns remaining console/worker consolidation;
GUI-002 binds existing controls; CORE-012 handles discovery/errors; GPU-010 validates
existing allocations; CFG-001/CORE-021 cover persistence; SEC-001/CORE-028 reuse
security/recovery; GUI-003 qualifies the approved UI. See [delivery map](GUI_ROADMAP.md).

## Release constraints

Existing user-selected Generation 2 VMs, NVIDIA first. Dynamically discover signed
current driver payloads; no fixed laboratory identities, disks, versions or counts
in product intent. Preserve disks, CPU/RAM, Secure Boot, credentials, enrollment,
authenticated pipes, locks, journals and independent readback. Unknown capability
is not support; opaque units are not GiB or enforcement. The slider's real mapping
is unresolved.

GUI effects use fresh plans, reviewed protection, actual stages and explicit
downtime consent. Saving is separate from GPU replay; uncertain outcomes require
manual reconciliation. Existing CLI semantics remain until separately implemented
and validated changes. Ship no proprietary drivers, disks, secrets or lab dependencies.
Packaging does not authorize publication.

Earlier GUI requirements absent from the approved prototype are retired, including
Activity, search/filters, extra actions/pages, persistent preferences and session
activation. Retain historical task IDs/decisions/evidence in
[BACKLOG_HISTORY](BACKLOG_HISTORY.md) and DEC-029. BLK-005's historical hang cause
remains unknown and its investigation remains closed.

