# Compiled UI wording

`en-GB/` contains the 408-key active catalogue in ten feature TOML files, as permitted by [the independent contract](../../tools/ui-text/src/contract.rs). Additional inventory keys/files are activated by later extraction work.

The current decoded wording is verbatim. British English editing is a later phase after migration acceptance; developers own keys, metadata, parameters, API bounds and consumers. See the [text guide](../../docs/UI_TEXT_GUIDE.md) for the strict schema and actual checks.

The root build discovers locale directories, validates every complete catalogue and compiles one resolved locale's Rust/Slint accessors into `OUT_DIR`. British English is the required default/fallback and the sole available language. The optional contributor build variable `HYPER_GPU_UI_LOCALE` requests an exact locale; unavailable requests resolve to `en-GB`. Installed binaries do not read this directory at startup or refresh. Generated files are never edited or stored here. Historical comparison text remains in test fixtures, not a runtime fallback.

Validate the locale root with `cargo run --locked --manifest-path tools/ui-text/standalone/Cargo.toml -- resources/i18n --locale en-GB`. Future authorized translations use the same feature files and stable keys under their own locale directory, with matching locale headers; no application call sites or registration list change. Do not add incomplete or placeholder catalogues. Only regular locale directories and this regular `README.md` are permitted at the root. See the [future-language workflow and formatting/plural boundaries](../../docs/UI_TEXT_GUIDE.md#internationalisation-architecture).
