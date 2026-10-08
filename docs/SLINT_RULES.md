# Codex Rules — Slint GUI Development

**Project:** Hyper GPU Support (Rust / Windows / Hyper-V GPU-P)  
**Status:** Development rules; **not** authorisation to implement  
**Companion documents:** `GUI_GUIDE.md` (approved UX and open decisions), `GUI_PROMPT.md` (current Codex task)

## Instruction to Codex

When reviewing, planning, implementing, or debugging the Slint frontend, follow these rules. Read `GUI_GUIDE.md` and the current task instructions first. **Do not write code or change project files until the user explicitly authorises implementation.** This document governs *how* Slint work should be done once authorised; it does not approve unresolved architecture decisions or host/VM operations.

Use the installed official Slint skill for Codex when available. Consult the current Slint documentation for the version actually used by the repository. Do not invent properties, APIs, controls, or extension capabilities. If a requirement is not directly supported by Slint, identify that gap and propose the smallest viable solution before building it.

## 1. Keep the architecture small and modular

- Put UI markup in dedicated `.slint` files, independent from Rust business logic and non-UI assets.
- Keep a small application window/composition layer and separate cohesive components for navigation, VM cards, VM details, allocation editor, review dialogs, and progress display. These are *responsibilities*, not a mandate for one file per widget.
- Put GUI presentation state, commands, event handling, and background-operation coordination in appropriately scoped Rust modules.
- Keep Hyper-V operations, validation, privileged enrollment, configuration persistence, and recovery in the shared Rust backend; the CLI and GUI must reuse those domain rules.
- Expose a narrow, typed Rust–Slint boundary: display data and user-intent callbacks, not unrestricted backend objects or arbitrary commands.
- Do not create another monolithic GUI source file, but also avoid unnecessary micro-modules and generic frameworks. Split when a file contains unrelated responsibilities or becomes hard to navigate and test.
- The previous `windows_gui.rs` was experimental. Do not port its raw Win32 layout/event architecture or add compatibility scaffolding for it.

## 2. Use Slint layout management, not manual coordinates

- Prefer `HorizontalLayout`, `VerticalLayout`, `GridLayout`, and supported standard layout widgets. Use `FlexboxLayout` only when supported by the pinned Slint version and genuinely helpful.
- Avoid absolute `x`/`y` positioning for ordinary application forms, cards, lists, and panels. Reserve explicit coordinates for exceptional visual elements where justified.
- Use content-driven sizing, minimum/maximum/preferred dimensions, stretch factors, alignment, layout spacing, and padding to express intent.
- **Do not ban every pixel value.** Small logical-pixel values for spacing, borders, icon dimensions, and reasonable minimum widths are acceptable when centrally controlled. Avoid arbitrary fixed screen coordinates, rigid control widths/heights, and physical-pixel (`phx`) assumptions.
- Use central design tokens/style metrics for recurring spacing, typography, and component dimensions. No repeated magic numbers across files.
- Keep the navigation sidebar and both workspace panels visible at all window widths. The chosen design does **not** collapse into a single panel.
- Support a user-adjustable left/right split. If Slint lacks a suitable built-in splitter, design one focused, reusable component rather than scattering drag calculations throughout the UI.
- When the window is narrower than the panels' minimum useful width, permit **horizontal workspace scrolling**. Each panel must support independent vertical scrolling. Test nested scrolling and pointer/keyboard interactions explicitly.
- Respect Windows DPI scaling, large text, long translated labels, unusual VM names, and narrow remote desktop windows. No clipping, overlap, or inaccessible Apply/Cancel controls.

## 3. Use the right widgets and models

- Prefer standard Slint widgets (`std-widgets.slint`) over recreating buttons, switches, text inputs, dialogs, and scrollbars for cosmetic reasons.
- The VM list uses **selectable cards**, not a table; build one reusable VM-card component.
- For a potentially long VM-card collection, use a data model and `ListView` (which instantiates visible items) rather than placing every card inside a `ScrollView`. Use `ScrollView` for ordinary free-form detail content.
- Identify VMs by stable Hyper-V IDs, **not list indices or display names**. Filtering and sorting must never redirect an operation to a different VM.
- When updating models, preserve selection, scroll position, input focus, expanded sections, and in-progress edits where valid. Do not rebuild every row/card on an unrelated status change.
- Use reactive bindings for presentation. Avoid imperative chains that repeatedly reset unrelated widgets.
- Placeholders, empty states, loading states, unavailable data, and errors must be explicit; never represent a failed discovery as "unsupported".

## 4. Keep state and events unambiguous

- Maintain three separate concepts: **observed Hyper-V state**, **saved desired configuration**, and **one unapplied VM draft**. Display them distinctly.
- A UI control changes the draft only. No switch, slider, card click, or dropdown selection should directly modify Hyper-V.
- Use typed actions/callbacks to communicate intent from Slint to Rust. Validate and plan in Rust; do not put business-policy decisions or Hyper-V command construction in `.slint` expressions.
- Selecting a different VM with a dirty draft must invoke the approved confirmation behaviour; never silently discard or apply the draft.
- Enable/disable controls based on actual eligibility and operation state. Provide a plain-language reason when an action is unavailable.
- Avoid circular property bindings and unclear two-way ownership. Prefer one authoritative owner for editable values and explicit update events.
- Distinguish loading, staged, authorisation required, running, completed, failed, uncertain, and recovery-required states. Never infer completion merely because a window closed or a timeout elapsed.

## 5. Keep the UI thread responsive

- Create and interact with Slint components on the UI/event-loop thread. Do not mutate UI handles from worker threads.
- Run slow discovery, Hyper-V operations, file I/O, credential interactions, and validation requiring external processes without blocking rendering or input.
- Deliver worker results back to the UI using Slint's documented event-loop facilities (for example `invoke_from_event_loop` / weak-handle methods as appropriate to the selected version).
- Use weak component references in callbacks/workers where required to avoid ownership cycles and updates to destroyed windows.
- Prefer typed progress/result messages over generic JSON blobs routed through arbitrary UI timers. Do not fabricate stage percentages or operation progress.
- Prevent accidental double-submit, re-entrant Apply, and conflicting concurrent GUI/CLI operations.
- A close request during active work is **not** cancellation. Report whether work remains active or is in an uncertain state.

## 6. Styling, theme, and accessibility

- Use the **Fluent** standard widget style with restrained, centralised customisation. Prefer Slint `Palette` and `StyleMetrics` over hard-coded colour copies.
- Support Follow Windows / Light / Dark user preferences, but **verify the implementation**: Slint's built-in widget style is chosen at compilation, while the default Fluent variant follows the system's light/dark setting. Do not claim arbitrary runtime style switching works without an explicitly tested mechanism.
- Keep standard platform interaction conventions, clear labels, visible focus, readable contrast, and text alternatives to icon-only controls. Never communicate state through colour alone.
- Custom interactive elements must have appropriate accessibility roles, labels, actions, and keyboard focus/activation. Prefer built-in accessible widgets where possible.
- Test keyboard traversal and activation, screen-reader exposure where supported, high-DPI rendering, large-text conditions, and Windows light/dark appearances.
- Maintain consistent visual states: normal, hovered, focused, disabled, pending, busy, success, warning, and error.
- Keep the markup friendly to the VS Code Slint extension and visual preview. Do not unnecessarily hide visual layout in Rust or runtime-generated structures.

## 7. GPU-P-specific guardrails

- GPU selection and all twelve resource fields (Min / Optimal / Max for VRAM, compute, encode, decode) are editable by design, but values require backend validation before Apply.
- Use reported GPU provider capabilities only as labelled suggestions. Do not invent ranges, percentages, GiB values, or performance guarantees from opaque provider units.
- First-time setup happens in the **existing right-hand details panel**, not in a separate setup wizard.
- Reassignment/enrollment must go through a restricted, explicitly authorised elevated path. The normal GUI runs unelevated. Never execute arbitrary elevated shell commands assembled from UI values.
- Review & Apply must operate on a fresh, validated operation plan and check it again at the privileged boundary. Obtain administrator authorisation before approved VM shutdown.
- Request explicit permission for graceful guest shutdown and safe restoration; never force power-off or modify host power state without separately scoped permission.
- After changes, read back the real Hyper-V state. Preserve and report partial success, unknown outcomes, and recovery-required conditions; never blindly retry a possibly completed action.
- User-facing progress/diagnostics last for the current session only. Minimal durable recovery records needed for interrupted operations are separate from diagnostic history.
- Never expose or log guest credentials, passwords, sensitive tokens, or privileged request secrets.

## 8. Make Codex's visual iteration reliable

- Use mock inventory, GPU capabilities, and operation results for layout/interaction work. The interface must be previewable without a working Hyper-V host, administrator elevation, or real VMs.
- Keep a small, deterministic set of mock scenarios: no VMs; many VMs; long names; unconfigured/unsupported/unknown; changing GPU; dirty draft; operation in progress; failed/uncertain/recovery states.
- Inspect the `.slint` files with the VS Code Slint extension or `slint-viewer` during authorised implementation. Do not mark a visual change complete solely because it compiles.
- If the official Slint Codex skill or embedded MCP inspection is installed and configured, use it to inspect the running UI and test interactions. Do not presume those capabilities are available or silently install anything.
- Compare screenshots/preview results at multiple widths and scaling factors. Describe what was actually inspected; do not claim tests passed without running them.
- Keep changes small and attributable. After each significant UI adjustment, validate the affected view before moving to unrelated pages.

## 9. Testing and debugging discipline

- Keep layout/presentation tests separate from live Hyper-V integration tests. Never trigger real VM shutdown, enrollment, driver updates, or privileged operations merely to test a widget.
- Test input validation, mapping from VM IDs to cards, draft preservation, confirmation handling, disabled controls, stale preview detection, and partial-operation result presentation.
- Verify the GUI with formatting/linting and the project's relevant build/test commands **only when that phase is authorised**. Report commands executed and exact results.
- Consider software rendering/fallback for graphics-driver problem cases. Choose supported Slint backends/renderer options after checking the pinned version; test the fallback rather than assuming it works.
- During debugging, isolate the failing layer (Slint binding/layout, Rust presentation state, shared backend, elevated helper) before changing unrelated modules.
- Fix root causes rather than adding polling, arbitrary delays, repeated refreshes, suppressions, or extra logging layers that disguise a state bug.

## 10. Required Codex workflow

1. **Read:** `GUI_GUIDE.md`, `GUI_PROMPT.md`, this file, repository instructions, and the relevant existing modules.
2. **Verify:** Confirm the actual Slint version, available widgets/APIs, and any associated tooling in official documentation.
3. **Scope:** State the components/modules affected, assumptions, unresolved design issues, and whether any privileged or live VM action would be required.
4. **Plan:** Propose a small, independently testable change before implementation. Never silently settle `OPEN-*` questions from the guide.
5. **Implement only when authorised:** Keep `.slint` responsible for presentation and Rust responsible for state/operations. Use mocks for first-pass UI work.
6. **Check:** Format/build/test as authorised; preview visually and exercise keyboard, resize, scrolling, and failure states.
7. **Report:** Summarise files changed, what was verified, what is still unverified, and any deviations from this file or `GUI_GUIDE.md`.

**Stop and ask for a decision** if a proposed change contradicts an approved UX decision, depends on undocumented Slint behaviour, expands privilege scope, or modifies live VM/host state without authorisation.

## Review checklist (use before marking any Slint task complete)

- [ ] UI markup is modular; no new all-in-one `.slint` or Rust GUI file.
- [ ] Layout uses constraints/containers rather than fragile absolute coordinates.
- [ ] Permanent two-panel UI, draggable split, and horizontal overflow behave as agreed.
- [ ] VM cards are model-backed; selection uses stable VM IDs.
- [ ] Scroll, focus, selection, and pending changes survive unrelated redraws.
- [ ] GUI shows observed vs desired vs staged state accurately.
- [ ] No blocking external work on the UI thread; no unsafe cross-thread UI access.
- [ ] Fluent styling is consistent; accessibility/keyboard support is checked.
- [ ] No Hyper-V business logic or privileged shortcuts have leaked into Slint components.
- [ ] Mocks, visual checks, and relevant tests were run and truthfully reported.
- [ ] No unapproved VM/host changes, destructive commands, or file migrations occurred.

## Official references (check current version)

- Slint best practices: https://docs.slint.dev/latest/docs/slint/guide/development/best-practices/
- Positioning and layouts: https://docs.slint.dev/latest/docs/slint/guide/language/coding/positioning-and-layouts/
- Layout constraints: https://docs.slint.dev/latest/docs/slint/reference/layouts/overview/
- Slint standard widgets and styles: https://docs.slint.dev/latest/docs/slint/reference/std-widgets/style/
- VM card collection (`ListView`): https://docs.slint.dev/latest/docs/slint/reference/std-widgets/views/listview/
- Scrollable free-form panels (`ScrollView`): https://docs.slint.dev/latest/docs/slint/reference/std-widgets/views/scrollview/
- Rust event loop: https://docs.slint.dev/latest/docs/rust/slint/fn.invoke_from_event_loop
- Weak component handles: https://docs.slint.dev/latest/docs/rust/slint/struct.Weak
- Accessibility properties: https://docs.slint.dev/latest/docs/slint/reference/common/
- VS Code and live preview: https://docs.slint.dev/latest/docs/slint/guide/tooling/vscode/
- Official Slint skill for Codex: https://docs.slint.dev/latest/docs/slint/guide/tooling/ai-coding-assistants/
