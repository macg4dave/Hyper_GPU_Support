# Compiled UI wording

`en-GB/` contains the nine-key LANG-004 representative catalogue. Only the three feature TOML files currently permitted by [the independent contract](../../tools/ui-text/src/contract.rs) belong inside that flat directory. Additional inventory keys/files are activated by later extraction cards.

The current decoded wording is verbatim. British English editing is a later phase after migration acceptance; developers own keys, metadata, parameters, API bounds and consumers. See the [text guide](../../docs/UI_TEXT_GUIDE.md) for the strict schema and actual checks.

The root build validates these inputs and compiles Rust/Slint accessors into `OUT_DIR`. Installed binaries do not read this directory at startup or refresh. Generated files are never edited or stored here. Historical comparison text remains in test fixtures, not a runtime fallback.
