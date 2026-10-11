# Slint

Follow [AGENTS.md](../../AGENTS.md). Apply to `.slint` changes and Rust UI wiring; also read [Rust](rust.prompt.md) for Rust edits.

- Check the pinned Slint version; use matching official docs or the available Slint skill/docs tools for uncertain APIs.
- Keep the existing design and reusable components. Split components by responsibility; avoid one growing UI/controller file.
- Keep business logic, validation, and effects in Rust. Slint renders state and forwards interactions through typed properties/callbacks; reuse existing bindings.
- Use explicit property directions, reactive bindings, unit-correct values, and layouts. Avoid competing geometry bindings, binding loops, and hardcoded positioning where layout suffices.
- Run blocking work off the UI thread. Return updates through the Slint event loop with weak handles; ignore obsolete completions.
- Preserve models, stable target identity, selection, drafts, focus, and scroll position across refreshes. Do not identify operations by a mutable row index.
- Keep keyboard access, readable errors, long-content scrolling, themes, and busy/modal behavior usable.
- Add meaningful Rust controller/state tests. Builds and real visual/interaction checks follow root validation/authorization policy; no retired mock previews.
- Report unrendered UI changes as awaiting visual validation, not visually verified.
