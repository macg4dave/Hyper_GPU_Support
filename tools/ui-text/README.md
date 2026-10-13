# UI text validator

Independent contributor crate for LANG-007. It has no product, Slint or Windows-backend dependency. Its isolated lockfile uses the root lockfile's resolved Serde/TOML versions and dependency closure. No root build wiring or runtime loader is added here.

Run from the repository root:

```powershell
cargo test --locked --manifest-path tools/ui-text/Cargo.toml
cargo run --locked --manifest-path tools/ui-text/Cargo.toml -- tests/fixtures/ui-text/validator/baseline
cargo run --locked --manifest-path tools/ui-text/Cargo.toml -- tests/fixtures/ui-text/validator/baseline --generate local/evidence/ui-text-generated
python -B scripts/testing/test_check_ui_copy.py
python -B scripts/testing/check-ui-copy.py --json
```

Add `--offline` to Cargo commands when the pinned dependencies are cached. Resource input is read at execution time: invalid catalogues do not prevent compiling the validator. A failed check returns nonzero and never constructs a partially validated catalogue. CLI generation validates the whole set before creating output. Outputs must be outside the resource tree and are rewritten only when bytes differ. Existing generated files are not a fallback after a failed check; build callers must propagate the error.

[contract.rs](src/contract.rs) is the independent developer-owned active contract, without resource wording. Its nine keys form the closed LANG-004 representative subset: fixed Verify title, conditional Apply power consent, the three required power categories, guest sign-in message, two captions and explanation wrapper. Fixture TOML under [the baseline directory](../../tests/fixtures/ui-text/validator/baseline) supplies source-derived verbatim wording. Other inventory keys remain pending and are rejected as unknown until their prepared extraction batch explicitly activates them.

Both CLI and future `build.rs` use `validate_directory(path, contract)` and `Catalogue::generate()`. Validation covers strict UTF-8/schema, exact metadata, owning files/keys, template grammar, types/occurrences, generated name conflicts and native UTF-16 budgets including termination. Only validated catalogues can generate sources. Generation uses parsed segments and contract argument order, preserves decoded text, escapes Rust/Slint separately, and inserts arguments as data. It returns deterministic UTF-8/LF source bytes; the build adapter will own Cargo rerun tracking and output publication in LANG-004.

The copy scan enforces absence of known baseline wording in the contract/generator and lists advisory candidates in reviewed pending product paths. [Its reviewed policy](../../tests/fixtures/ui-text/validator/copy-scan.json) records why each path is pending. Rust test modules and test fixtures are excluded. Lexical scanning is deliberately limited: it is not a Rust/Slint parser, secret detector, complete English detector or inventory reconciliation. Narrow/activate consumer guards as extraction proceeds; do not treat advisory counts as migration coverage.

Focused tests use synthetic edits, local temporary files, CLI subprocesses and a temporary Windows directory junction (or Unix symlink) to exercise rejection without reading its target. No native prompt, GUI launch, guest, VM, GPU, driver, protected runner or actual vault operation is used. Generated-source compilation, two-window globals, TOML-only rebuild propagation and product integration remain LANG-004. Rendered/editor acceptance remains later work.
