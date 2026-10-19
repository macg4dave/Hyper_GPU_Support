# Slint

Follow [AGENTS.md](../../AGENTS.md). Apply to `.slint` changes, Rust UI wiring and live GUI testing; also read [Rust](rust.prompt.md) for Rust edits.

- Check the pinned Slint version; use the matching official Slint docs for uncertain APIs.
- Keep the existing design and reusable components. Split components by responsibility; avoid one growing UI/controller file.
- Put all user-visible GUI wording in the [catalogue](../../resources/i18n/README.md) (`en-GB` first); use `UiText` in Slint and `ui_text` accessors in Rust, never hard-coded text.
- Keep business logic, validation, and effects in Rust. Slint renders state and forwards interactions through typed properties/callbacks; reuse existing bindings.
- Use explicit property directions, reactive bindings, unit-correct values, and layouts. Avoid competing geometry bindings, binding loops, and hardcoded positioning where layout suffices.
- Run blocking work off the UI thread. Return updates through the Slint event loop with weak handles; ignore obsolete completions.
- Preserve models, stable target identity, selection, drafts, focus, and scroll position across refreshes. Do not identify operations by a mutable row index.
- Keep keyboard access, readable errors, long-content scrolling, themes, and busy/modal behavior usable.
- Add meaningful Rust controller/state tests. Builds and real visual/interaction checks follow root validation/authorization policy; no retired mock previews.
- Report unrendered UI changes as awaiting visual validation, not visually verified.

## Control the running GUI

Prefer Slint's embedded MCP interface for this application's GUI journeys. Use it to inspect and operate the real installed application through its normal controls. Use native Computer Use only for necessary Windows surfaces outside Slint; native sign-in prompts require the user's local assistance when unavailable to the tools. Follow existing session authorisation; this guidance grants no new live effects.

1. Read the official Slint runtime/MCP docs for the pinned version. For an authorised exploratory build, enable inspection only on the command line:

   ```powershell
   $env:SLINT_EMIT_DEBUG_INFO = '1'
   cargo build --locked --bins --features slint/mcp
   ```

   Keep `mcp` out of `Cargo.toml`. Locate the actual Cargo output directory, including any `CARGO_TARGET_DIR` override. Install the four artifacts with `cargo run -p app-install -- --skip-build` ([Install](../../README.md#install)) and verify installed hashes against enrollment pins. For Windows GUI-subsystem CLI commands, use `Start-Process -Wait -PassThru` with redirected output to obtain the actual exit code; direct PowerShell invocation may return before the process finishes.

2. Launch the installed executable without arguments, using a free local port via process-scoped `SLINT_MCP_PORT` (for example `9315`). Keep the ordinary GUI backend. Record the inspection feature/build identity separately from the ordinary-feature automated gate and release candidate. Remove temporary environment overrides after launch/build.

3. Connect to `http://127.0.0.1:9315/mcp` with JSON-RPC over HTTP, using `Content-Type: application/json` and `Accept: application/json, text/event-stream`. Discover `tools/list` and use its returned schemas. Keep one persistent client and its current handles for the session.

4. Call `list_windows`, then `get_window_properties` for the returned window. Obtain `rootElementHandle` from those properties; window and element handles are distinct. Read `get_element_tree`, or query descendants/IDs, to locate controls by role, label, ID and enabled state. Require one matching enabled control, particularly when modal and background buttons share a label. Refresh the tree after each action and rediscover handles after reopening.

5. Use `click_element` or accessibility actions for buttons/switches, `set_element_value` for text inputs, and key/scroll tools for their normal interactions. Entering a memory value requires Enter or **Set request**, then Apply starts preparation and execution through the normal progress screen. Do not inject controller state or bypass consent, planning, enrollment or recovery.

6. Capture `take_screenshot` before/after significant transitions, inspect the image, and save screenshots plus non-secret JSON observations under `local/evidence/qa/`. Record timings and repeated refresh samples. A Complete stage is not the final outcome; wait for the terminal result and required state readback. Gracefully close the app at the end to stop its inspection listener.

This method exercised the real installed GUI on 13 October 2026. It does not cover native dialogs, every keyboard/accessibility path, or release acceptance automatically. Summarise actual coverage and limitations in chat. Create QA reports or update case/index records only when explicitly requested; follow the AGENTS.md QA-report rule.
