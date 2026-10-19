# UI text generator

Build-time library used by the root `build.rs`. [load](src/load.rs) discovers the languages under `resources/i18n/`, validates each against the `en-GB` reference (see the [catalogue format](../../resources/i18n/README.md)) and [generate](src/generate.rs) emits three files into `OUT_DIR`:

- `ui_text.rs`, included by the application's `src/ui_text.rs`: one accessor per key, such as `ui_text::dashboard::vm_list::count(n)`. Messages without arguments return `&'static str`; messages with arguments return `UiMessage`. Each accessor reads the active language, set at runtime with `ui_text::set_current`.
- `ui-text.slint`: the `UiText` global, with a property per message (initialised with reference wording) and pure functions for messages with Slint-compatible arguments.
- `ui_text_apply.rs`, included in `src/gui/mod.rs`: `apply_ui_text`, which copies the active language into the `UiText` global after a language change.

[template.rs](src/template.rs) parses `{name}` placeholders once; inserted values are never parsed. [plural.rs](src/plural.rs) defines the CLDR integer plural rules used for validation and for both generated selectors.

Tests use isolated temporary locale roots plus a check of the repository catalogue:

```powershell
cargo test --locked -p ui-text
```
