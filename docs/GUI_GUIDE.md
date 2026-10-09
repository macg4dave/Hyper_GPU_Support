# Hyper GPU Support — Slint GUI Guide

**Status:** Approved UX/architecture policies; technical implementation questions open
**Updated:** 2026-10-09  
**Target:** Windows desktop · Rust backend · Slint frontend · Hyper-V GPU partitioning (GPU-P)  
**Companions:** [docs-audit brief](GUI_PROMPT.md) · [delivery map](GUI_ROADMAP.md) · [Slint rules](SLINT_RULES.md) · [implementation prompt](../.github/prompts/SLINT_CODEX_PROMPT.md)

> **Planning document, not an implementation order.** This records decisions made so far and sets boundaries for later Codex work. Items marked **OPEN** must be investigated and resolved deliberately; Codex must not infer approval to implement them.

## 1. Project intent

Provide a straightforward, maintainable Windows application for inspecting existing Hyper-V VMs and configuring their GPU-P support. The Slint frontend should let the user select a physical GPU, edit supported partition allocations, stage a desired configuration, review its real effects, authorise protected operations, and inspect the result.

Development is AI-assisted, primarily with Codex in VS Code. Make modules easy for an AI and a human to inspect, edit, test, and debug individually. Slint live preview and mock-backed UI development are first-class requirements.

### Ground rules

- **The current Win32 GUI is a disposable experiment.** There is no requirement to port, preserve, refactor, or achieve pixel/feature parity with `windows_gui.rs`. Rebuild the presentation layer cleanly in Slint.
- **The application is early in development.** Internal APIs, configuration schema, and module boundaries may be improved where justified. Do not create compatibility scaffolding solely to support the experimental GUI.
- **Reuse sound backend code, not architectural mistakes.** Inspect the actual repository before deciding what to keep or replace. The existing Rust backend/CLI should remain useful and share one set of domain rules with the GUI.
- **No parallel Hyper-V implementation inside the GUI.** Presentation code must not issue its own unreviewed VM or GPU commands.
- **Planning remains active.** This guide is authoritative for recorded user decisions, but unanswered architecture questions are not settled by its publication.
- Avoid gratuitous frameworks, abstraction layers, tests-of-tests, and diagnostic infrastructure. Prioritise a small, understandable system.

## 2. Recorded UX decisions

| ID | Decision | Agreed design |
|---|---|---|
| D01 | Toolkit | Slint UI with Rust application/backend, developed in VS Code with Codex |
| D02 | Main layout | VM list left; contextual VM details panel right |
| D03 | Details panel | Hybrid: immediate status/actions plus expandable advanced details |
| D04 | Enable/disable | Staged GPU support switch; no immediate Hyper-V change |
| D05 | Switching VMs with a draft | Prompt before switching; never silently apply/discard |
| D06 | GPU management | Fully editable GPU selection and allocation; extend backend as necessary |
| D07 | Resource editor | Four expandable categories, each with Min / Optimal / Max |
| D08 | Review & Apply | Adaptive: simple review for small changes, guided steps for complex ones |
| D09 | VM power | Managed graceful shutdown only with explicit approval; safely restore original running state |
| D10 | Privileges | GUI normally unelevated; narrow on-demand elevation for protected actions |
| D11 | Progress | Real stage-by-stage progress; expandable technical detail; retain results in current session |
| D12 | Diagnostic retention | Session-only user-facing diagnostics; no automatic persistent history |
| D13 | Window resizing | Always keep split layout; use horizontal scrolling when too narrow |
| D14 | Visuals | Fluent widgets with modest, centralised custom styling; Windows/Light/Dark theme |
| D15 | VM listing | Selectable VM cards with icons and meaningful statuses, not a table |
| D16 | Visibility | Show every discovered VM, with search, state indicators and filters |
| D17 | First configuration | Directly in the right-hand panel, including enrollment; no separate setup wizard |
| D18 | Initial resource values | Suggest editable values from reported GPU capabilities; no invented recommendations |

IDs in this document are stable design-reference IDs; renumber only with explicit agreement.

### Confirmed architecture decisions (planning conversation A18–A32)

| ID | Topic | Agreed policy |
|---|---|---|
| A18 | Configuration granularity | One file per VM, keyed by stable Hyper-V VM GUID; discover unconfigured VMs normally |
| A19 | Host GPU properties | Host-wide GPU settings, including partition count, are read-only in the GUI; per-VM allocation remains editable |
| A20 | GPU operation failure | Stop and require manual recovery; no automatic GPU rollback, corrective modification or retry |
| A21 | External changes | Block stale edits and require refresh; no automatic merge or overwrite; preserve draft for inspection |
| A22 | Config location | Machine-wide `%ProgramData%\HyperGpuSupport\config\vms\`; separate protected records and per-user UI preferences |
| A23 | Save timing | Keep drafts in memory; after approved Apply and readback, save VM config; failed save uses save-only retry |
| A24 | Startup | No arguments opens Slint GUI immediately and discovers host/VMs/GPUs in background; no startup VM changes |
| A25 | Packaging | One `hyper-gpu-support.exe` for GUI (no arguments), explicit CLI commands and internal worker mode |
| A26 | Elevation lifetime | One restricted elevated instance of the same executable per approved operation; it exits afterward |
| A27 | Config file writes | Elevated worker exclusively owns VM configuration writes, including save-only recovery |
| A28 | IPC | Private, local Windows Named Pipes; typed, bounded, access-controlled protocol; no arbitrary command API |
| A29 | GUI closing | Defer normal closure while a worker operation is active; minimisation allowed; handle crashes/timeout separately |
| A30 | Concurrency | One modifying GPU operation across the host at a time, shared by GUI, CLI and workers |
| A31 | GUI instances | One GUI window per Windows session; a second launch activates the existing window; CLI and worker unaffected |
| A32 | Interrupted startup | Open normal dashboard with persistent recovery warning; block new GPU modifications until reconciliation |

These are **agreed behaviours**, not confirmation that current Rust code already implements them. Exact API choices and applicability of vendor/provider features still require repository and Windows-host verification.

The [9 October repository audit](ARCHITECTURE.md#repository-audit--9-october-2026)
records the implemented runner, Named Pipe, exclusive lock, journals, native VRAM
and presentation state to reuse. Missing Slint/dispatch, expanded allocation,
worker-owned saving and revised recovery admission remain planned. No new runtime
verification was performed. This guide owns requirements/OPEN questions;
ROADMAP owns product gates and BACKLOG owns task status/dependencies/acceptance.

## 3. Information architecture

### Navigation

A conventional desktop sidebar with:

1. **Virtual Machines** — principal work area.
2. **System Information** — host GPUs, capabilities, available partitioning information and prerequisites.
3. **Settings** — appearance, UI preferences and guest credential management; not a second GPU editor.
4. **About** — version, build information, relevant notices, help/documentation.

No host-wide changes should be implied merely by navigating to System Information or Settings.

### Main workspace

- Navigation sidebar remains available.
- Left side: VM search, status filters, refresh and vertically scrolling VM cards.
- Right side: selected VM summary, observed status, staged GPU-P controls, actions and expandable advanced details.
- Adjustable splitter between the two work panels; layout uses proportional/content constraints.
- Permanent two-pane workspace, **including at narrow window widths**. Allow horizontal scrolling of the workspace instead of collapsing into a single-pane view.
- Independently scrollable VM cards and details content. Scroll, focus and selection should not reset on incidental status updates.
- If no VM is selected, show a clear selection prompt instead of fictional values or disabled-looking controls without explanation.

## 4. VM cards

Cards display at least:

- VM name and useful identification; power state.
- GPU support/enrollment state, with distinct labels for **configured**, **setup required**, **unsupported**, **unknown**, and relevant failures.
- Attached GPU indication where observed; do **not** equate attachment with successful guest graphics verification.
- Selected-card highlighting and a visible keyboard focus indicator.
- Warnings whose text remains meaningful without colour.

Interaction:

- The entire card is one selection target. No enable switch, Apply button or duplicate allocation controls on cards.
- Search by VM name; filters include All, Configured, Setup required, Unsupported and Unknown. Use actual capability/eligibility evidence to classify VMs; discovery failure means Unknown, not Unsupported.
- Show all discovered VMs, including ineligible and unenrolled ones. The details panel explains why a VM cannot currently be configured.
- Preserve selected VM through refresh when its stable ID remains present. Handle rename, removal, duplicate visible names and stale discovery without a crash.
- Use stable VM identifiers, not row indices, names or order, in event handling and pending drafts.
- Refresh cards with lightweight discovery/status. Never authenticate/hash the full
  driver payload or run guest graphics verification merely to refresh the dashboard;
  those belong to deliberate planning/Apply/Verify actions. Report partial access
  failure per VM as unknown/denied, preserving valid rows and draft intent.
- Consider a virtualised Slint `ListView` for a large collection rather than instantiating an unbounded number of cards.

## 5. Right-hand VM details panel

### Always-visible section

- Name, power state, generation/eligibility (where known).
- **Observed attachment**, **desired GPU support**, **pending draft** and **graphics verification** must be clearly separate.
- Staged GPU support on/off switch; changing it never directly mutates Hyper-V.
- Selected GPU, with unavailable/disconnected/unenrolled states clearly explained.
- Actions: **Review & Apply**, **Discard draft**, **Verify Graphics**, **Reapply / Update**, enabled only where meaningful.
- Explanation of blockers and actionable next steps for unsupported/unenrolled/unknown states.

### Expandable advanced section

- GPU identity, driver/provider versions, device/interface identifiers where available.
- Raw provider capability ranges, allocation settings, attached interfaces, observed power and last recorded verification information when supported.
- Technical diagnostic information, clearly labelled with provenance and freshness.
- Copy-friendly identifiers, but avoid exposing secrets.

### Configuring GPU-P

- An unconfigured VM is edited **in the same right-hand panel**. No separate onboarding wizard.
- Physical GPU dropdown lists detected eligible GPUs; explain why an option is unavailable rather than hiding the entire VM.
- Four independently expandable resource groups: **VRAM**, **Compute**, **Encode**, **Decode**.
- Each group contains **Minimum / Optimal / Maximum** values, with provider units, valid bounds and inline validation.
- Show **current observed** and **requested** allocation where both exist, without presenting the two as equivalent.
- For initial configuration, populate only values legitimately derived from provider capabilities. Label them *provider-reported suggestions*, **not** user-workload recommendations or percentages of GPU power.
- Never invent missing values. If the backend cannot validate a category or GPU, show Unavailable/Unknown and block only the affected unsafe operation with a reason.
- Treat host-wide GPU partition count as separate from per-VM allocation: display only, do not edit it (A19).

## 6. Drafts and navigation

- One VM draft at a time in the initial product.
- A draft may represent: new enrollment, physical GPU reassignment, changes to any of the 12 allocation fields, enable/disable, or reapply/update intent.
- Draft values exist separately from committed desired configuration and observed Hyper-V state.
- Selecting a different VM while a draft exists opens a confirmation with **Stay on VM**, **Discard & Switch**, and **Review Changes**.
- Reviewing a draft stays on its VM. It does not imply that the selection changes automatically after a successful Apply.
- Closing the app or otherwise losing a draft requires clear handling. Do not suggest Discard can undo a completed Hyper-V operation.
- Never silently overwrite an existing draft because a background discovery result arrived.
- Ordinary navigation to other pages must not accidentally erase or apply a draft; precise UX on page switching is **OPEN-07**.

## 7. Review & Apply

An **adaptive** flow, not a fixed wizard:

1. Validate requested values, GPU capability, VM eligibility, enrollment/permissions and current host state.
2. Read current observed state and build a fresh, **specific** plan: proposed changes, possible interruption, guest-driver effects, elevated actions and restoration policy.
3. For simple operations, present a concise confirmation. For complex actions, disclose additional required approvals in guided steps.
4. Acquire required administrator authorisation **before** initiating any VM shutdown.
5. Ask explicitly before a required graceful VM shutdown. Do not silently power off the guest.
6. Execute the approved plan through the shared protected backend; emit real stage events.
7. Read back observed state; reconcile desired configuration; elevated worker saves the matching per-VM configuration only after verified success.
8. Restore the VM's original running state only when safe and authorised, and report actual outcome. A failed/uncertain GPU change stops further GPU modifications and requires manual recovery.

If the VM was originally off, do not leave it running just because setup or verification required a temporary start. A forced power-off requires **separate** approval; never use it as a silent fallback. Never reboot, log out of, or shut down the Windows **host** during routine GUI operations.

A plan becomes stale when the VM, selected GPU, provider capabilities, permissions, config file or relevant host allocation state changes. Revalidate before execution; a materially different plan requires a new confirmation.

## 8. Permission boundaries

- Slint GUI starts without administrator privileges and should continue normal read-only work without elevation.
- Use a **restricted elevated instance of the same executable** for each approved operation requiring administrator rights. Reuse or reshape the existing runner/enrollment contracts where appropriate; no separate permanent service.
- Use a private Windows Named Pipe between the unelevated frontend and the short-lived elevated worker. Authenticate/authorise the specific operation, VM and GPU identity, requested parameters and plan revision at the privileged boundary. The worker must independently validate inputs; message bounds, peer checks, IPC ACLs and timeouts are mandatory.
- UI confirmation, Windows elevation and guest credentials are **different permissions** and must remain visibly distinct.
- Declining elevation retains the draft and does not shut down the guest.
- Do not expose a generic privileged shell-command facility from the GUI. The worker accepts only bounded, typed, fixed operations and reports structured progress/results.
- All machine-wide per-VM configuration writes belong to the elevated worker. Saving without reapplying a completed GPU change is its own narrow operation.
- Guest credentials use an appropriate protected Windows facility; never store secrets in ordinary configuration, progress events, screenshots or exported diagnostics.

## 9. Operation progress and recovery

### Progress interface

- Stage list with explicit states: Pending / Running / Complete / Failed / Skipped / Unknown.
- Show actual stage events from the backend; **no fake percentage** or animation standing in for real backend progress.
- Expandable, copyable technical details; preserve completed/failed operation results until the application session ends.
- A closed progress window does not claim to cancel native work. Only offer cancellation where a specific stage is proven safely cancellable.
- Failures should identify what definitely completed, what is unknown, and the safe next action.

### Operational correctness

- No blind retry of a potentially completed Hyper-V operation.
- A successful Hyper-V change followed by failed config save is **partially successful**, not an overall rollback. Offer retrying **save only** after readback.
- Treat timeout or worker disconnection as *uncertain*, not proof of failure.
- Use minimal **durable** recovery/journal state where necessary to reconcile interrupted operations after a crash. This is **not** a persistent user-facing diagnostic history and must not become a large logging subsystem.
- **Manual GPU failure recovery only:** stop further GPU modification, read back where safe, retain minimal recovery state and report completed/failed/unknown stages. Do not automatically roll back, retry or repair a GPU configuration.
- Restore guest power after a failure only if separately authorised and verified safe; otherwise leave it stopped and explain why.
- On startup show the normal dashboard plus persistent recovery warning, but block new modifying GPU operations host-wide until reconciliation.
- If a GUI close is requested during a known active worker operation, defer close; do not treat a lost pipe or worker timeout as completed work.

### Diagnostics policy

- Session-only operational history/technical detail. No automatic 30-day logging, no retention configuration or background diagnostic archive.
- Preserve existing protected admission/audit and recovery records needed for
  security/reconciliation; they are not user-facing diagnostic history. Do not
  delete them during consolidation merely to satisfy session-only presentation.
- User can explicitly copy/export a report; remove passwords, tokens, credential material and other secrets.
- Ordinary UI preferences may persist (theme, window placement/size and splitter ratio). Keep these separate from operational recovery state and diagnostic data.

## 10. Shared state and concurrency

Use three distinct models:

1. **Desired configuration** — the application's requested settings, shared by GUI and CLI.
2. **Observed state** — discovered directly from Hyper-V/provider and possibly from guest verification.
3. **Draft state** — one uncommitted edit in the GUI.

The GUI and CLI must share validation, planning, execution and save semantics; don't build a competing GUI-only command path.

- Discover and display out-of-band changes from PowerShell, CLI or Hyper-V Manager.
- Detect file or observed Hyper-V changes immediately before Apply/save; on conflict block the stale draft and require refresh, without merge or overwrite.
- Permit only **one modifying GPU operation host-wide**, coordinated across GUI, CLI and elevated worker with an OS-backed cross-process mechanism. Read-only inspection may continue where safe. A free lock is not proof an interrupted operation was reconciled.
- Do not allow a stale view to overwrite the actual VM or host state automatically.
- Readback is independent of the intended outcome; report mismatches explicitly.
- External configuration changes must not silently replace a pending GUI draft.
- Per-VM files live in `%ProgramData%\HyperGpuSupport\config\vms\`, keyed by VM GUID; exact format/schema and migration remain open. Host GPU inventory is observed dynamically, not duplicated as desired configuration.
- Only the restricted elevated worker writes these files, atomically and after successful readback. A failed save offers save-only retry, never replay of Hyper-V mutation.
- The single executable launches GUI with no arguments and headless CLI for explicit commands; the internal worker mode is not a general user CLI.
- Enforce one GUI per Windows session: second launch raises/restores that session's existing window without clearing its draft.

## 11. Slint layout, appearance and accessibility

### Layout

- Use Slint's `HorizontalLayout`, `VerticalLayout`, `GridLayout` and/or other currently supported layout primitives; use content-driven preferred sizes, minimums, stretching and sensible constraints.
- **Avoid scattered hard-coded x/y positions and fixed pixel widths.** Small fixed touch targets, icons, padding or deliberate minimums are acceptable if justified and centrally defined.
- Two panels remain side-by-side; horizontal scrolling reveals clipped workspace at narrow widths. Include independently scrolling pane content and a draggable splitter **only after validating the best Slint implementation**.
- Ensure the sidebar stays accessible, mouse and keyboard can reach all horizontally scrolled content, and no action buttons become unreachable.
- Verify DPI, Windows text scaling, long VM names, different locale lengths, narrow and large windows, and multi-monitor movement.
- Use a virtualised list where practical for VM cards; don't rebuild all cards for unrelated status changes.

### Look and feel

- Built-in **Fluent** controls with restrained custom colour/spacing/icon choices; no hand-built control library for standard buttons and fields.
- Theme preference: Follow Windows / Light / Dark, with centralised theme tokens and no duplicated per-page styling.
- Accessible text contrast, visible keyboard focus, labelled controls, semantic statuses not communicated by colour alone.
- Test keyboard use and actual assistive-technology behaviour; do **not** assert Windows screen-reader parity without verification.

### Rendering and testability

- Aim for a robust renderer choice on a machine where GPU drivers may be malfunctioning; evaluate Slint's supported software renderer/fallback during setup (**OPEN-06**).
- All UI components must be inspectable with test/mock data without Hyper-V, elevation or Windows guest credentials.
- Preview `.slint` components in VS Code and verify screenshots/interactions when supported by the installed Slint tooling.

## 12. Proposed modular organisation (subject to repo audit)

These are **responsibilities**, not a demand to create this exact directory tree or a file for every function:

| Area | Responsibility |
|---|---|
| Slint shell / navigation | Window, sidebar, split workspace, page routing |
| Slint VM cards | Search/filter, selection, status presentation |
| Slint VM details | Summary, staged controls, blockers and advanced sections |
| Slint GPU editor | GPU choice, four resource groups, validation feedback |
| Slint review/progress/dialogs | Adaptive confirmations, stage list, error/recovery messaging |
| Slint shared design | Fluent variants, reusable controls, spacing/icons/theme |
| Rust GUI state | Selected VM, one draft, page state, busy state, theme and UI preferences |
| Rust presenter/controller | Typed UI intents/results, model mapping and Slint event-loop updates |
| Rust application/domain | GPU/VM discovery, desired/observed models, allocation validation, planner |
| Rust protected execution | Enrollment, scoped elevation, hypervisor operations and readback |
| Rust recovery | Minimal interrupted-operation reconciliation, safe config writes |

Prefer many **coherent** modules over one giant `gui.rs`, but avoid making dozens of single-function files. A large file triggers review, not an arbitrary line-count failure. Do not put Hyper-V logic inside `.slint` or duplicate core rules in GUI adapters.

## 13. Codex and VS Code workflow

- Install/use the official Slint VS Code extension for syntax diagnostics and live preview.
- Use the official Slint AI/Codex guidance if available; verify the command and compatibility against current upstream docs at setup time.
- Codex should inspect the actual Cargo workspace, binaries, config types, runner, workflow, existing tests and project roadmaps **before** proposing concrete modules.
- Use static/mock fixtures first: multiple VMs, no VMs, many VMs, missing GPU, long text, unknown state, partial failure, denied elevation, stale plan.
- Keep UI-thread work minimal; deliver worker results via Slint's event-loop-safe mechanism, not a timer that repeatedly rebuilds all widgets.
- Test one vertical slice at a time: UI action → typed request → fake backend → state transition → visual update.
- Mock-backed UI work must not invoke Hyper-V effects. Separately selected live
  testing follows AGENTS designated-disposable authorisation and identity checks;
  immediate permission is always required for physical-host lifecycle. The current
  documentation task permits no live access or implementation.
- Do not introduce optional tooling, database, service, logging stack or complex code generation without evidence it solves a current requirement.

## 14. Implementation sequence summary (planned)

The detailed, gated plan lives in [`GUI_ROADMAP.md`](GUI_ROADMAP.md). This section is a summary; roadmap steps cannot authorise implementation on their own.

1. **Reuse the source audit.** PLAN-001 records the repository/docs result; investigate only remaining technical questions needed by the selected card.
2. **UI prototype with mock data.** Slint shell, resizable/scrolled split, VM cards, hybrid details, centralised Fluent styling; prove VS Code preview.
3. **Shared data/contracts.** Per-VM ProgramData config, desired/observed/draft state, typed operations, schema decision and no-argument GUI/explicit-CLI dispatch.
4. **GPU selection/allocation backend.** Discovery, provider validation, all four resource groups and consistent CLI access.
5. **Protected plan/execution.** Per-operation elevated mode, local Named Pipe, host-wide modification lock, re-enrollment, VM power policy, event progress, readback and minimal recovery.
6. **Slint integration.** Replace mocks progressively; maintain a usable frontend at each integration milestone.
7. **Verification.** Failure injection, preview staleness, narrow layout, keyboard/scaling, mock tests and explicitly approved live tests.
8. **Documentation and cleanup.** Remove the disposable Win32 experiment when appropriate; do not expend effort preserving its structure or behaviour.

The delivery map links each slice to BACKLOG acceptance/dependencies; this summary
does not create another gate/status register. No work is authorised by this sequence alone.

## 15. Acceptance criteria

A release candidate should demonstrate:

- [ ] A clean Slint GUI builds with a modular frontend; no dependence on `windows_gui.rs`.
- [ ] GUI previews with mock data without accessing Hyper-V.
- [ ] All discovered VMs appear as keyboard-accessible cards with correct status distinctions.
- [ ] Split panels, drag divider and horizontal/vertical scrolling are usable at multiple widths and DPI settings.
- [ ] Right-hand panel manages existing and first-time VM/GPU configurations directly.
- [ ] Each allocation category exposes editable Min/Optimal/Max in verified units with validated bounds.
- [ ] One draft exists at a time; switching selected VM asks before losing it.
- [ ] Review plan is fresh, specific and requires all necessary separate approvals.
- [ ] Administrator-only work occurs behind a validated, narrow privileged boundary.
- [ ] Graceful guest shutdown requires approval; safe restoration follows the approved plan; host is untouched.
- [ ] Progress is based on actual stage results and partial/uncertain states are correctly represented.
- [ ] Interruption/recovery cannot re-run a previously completed destructive stage blindly.
- [ ] CLI and GUI share behaviour and do not overwrite conflicting external changes.
- [ ] Session-only diagnostics; no accidental persistent log database or secret exposure.
- [ ] One executable dispatches GUI/CLI/worker modes correctly, without opening Slint in CLI mode.
- [ ] Per-VM configs are written only by the elevated worker after readback; save-only retry never reapplies GPU changes.
- [ ] One host-wide modifying operation is enforced across processes; recovery warnings survive crashes and block modifications.
- [ ] Private Named Pipe protocol rejects untrusted peers, malformed or oversized messages, stale plans and unsupported operations.
- [ ] Second launch within one Windows session activates the existing GUI without discarding its draft; closing is deferred during active operations.
- [ ] Unit, integration, mock UI and agreed visual/accessibility checks pass.

## 16. Remaining open implementation questions — resolve from repo/host evidence

**Resolved policy choices:** A18–A32 above replace older draft questions about storage location, host GPU editability, concurrency granularity, elevated worker lifetime, IPC transport, failure rollback and startup recovery. Do **not** reopen those as if the user had not decided.

| ID | Open technical detail | Required investigation |
|---|---|---|
| OPEN-01 | Per-VM schema and migration | CFG-001: extend existing schema-2 TOML/parser where sound; settle version/import without accidental loss |
| OPEN-03 | Host-wide lock adaptation | SEC-001: reuse existing exclusive file-handle lock; verify all-mode/session scope, crash/handover and durable recovery admission; do not invent lock expiry as reconciliation |
| OPEN-04 | Windows 11 GPU-P feature reality | Verify GPU selection, all twelve provider fields, guest prep, vendor/driver constraints on this host; classify unsupported actions truthfully |
| OPEN-05 | Elevated worker security | SEC-001: adapt current SYSTEM runner and ACL/SID/framing/nonces/audit to same-exe elevation; bind a particular same-user worker, approved plan and session; bounded server/progress waits |
| OPEN-06 | Renderer resilience | Select Slint renderer/software fallback and test on broken/absent GPU drivers |
| OPEN-07 | Draft and non-VM navigation | Exact prompt/persistence when switching app pages and attempting to close with an unsaved draft |
| OPEN-08 | Manual recovery reconciliation details | Define how to inspect/reconcile incomplete stages, clear recovery holds safely and handle partially successful save/restoration without GPU auto-fixes |
| OPEN-09 | Windows hybrid executable/packaging | GUI/CLI console subsystem and stdout/exit codes, install/UAC, per-session activation, upgrades/uninstall and ProgramData ACL provisioning |
| OPEN-10 | GPU allocation semantics | Provider units, unset/default behaviour, bounds, cross-resource constraints, host capacity and overcommit |
| OPEN-11 | Accessibility | Practical keyboard navigation, screen readers, text scale and minimum viable support in Slint |
| OPEN-12 | Config save-only authorisation | Exact immutable, validated payload to save after an earlier successful readback without replaying privileged mutation |

These are **technical questions**, not permission for Codex to choose different user-facing policies. Require evidence, alternatives, impact and explicit approval before committing high-risk design changes.

## 17. Authoritative references for tool setup

Consult current upstream documentation when implementation begins; links are guidance, not a pinned dependency version:

- [Slint VS Code integration](https://docs.slint.dev/latest/docs/slint/guide/tooling/vscode/)
- [Slint coding agents / Codex guidance](https://docs.slint.dev/latest/docs/slint/guide/tooling/ai-coding-assistants/)
- [Slint positioning and layout constraints](https://docs.slint.dev/latest/docs/slint/guide/language/coding/positioning-and-layouts/)
- [Slint `ListView`](https://docs.slint.dev/latest/docs/slint/reference/std-widgets/views/listview/)
- [Slint `ScrollView`](https://docs.slint.dev/latest/docs/slint/reference/std-widgets/views/scrollview/)
- [Slint widget styles and theme variants](https://docs.slint.dev/latest/docs/slint/reference/std-widgets/style/)
- [Slint event-loop interface](https://docs.slint.dev/latest/docs/rust/slint/fn.invoke_from_event_loop)

---

**Next step:** Use BACKLOG and the [implementation prompt](../.github/prompts/SLINT_CODEX_PROMPT.md)
for an explicitly requested card. Reuse the recorded source audit; resolve the
card's OPEN items with evidence. This documentation task begins no implementation.
