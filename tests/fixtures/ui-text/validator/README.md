# Validator fixtures

LANG-007 executable inputs, separate from the immutable LANG-003 JSON comparison evidence above this directory.

`baseline/` is a strict flat nine-key catalogue for the representative LANG-004 subset. Its decoded text is copied verbatim from the historical mapping; metadata matches the independent Rust contract. These are contributor test inputs, not shipped resource files or runtime fallback wording. LANG-004 now activates the matching nine keys under `resources/i18n/en-GB`; these fixtures remain the frozen comparison baseline.

`invalid/` contains parser/duplicate-field and adversarial template specimens. The validator tests also mutate copies of the valid documents in memory to isolate missing/unknown fields, duplicate keys/tables, wrong metadata/types, occurrence constraints, collisions, encoding, native bounds and deterministic generation. Expected English UI wording is not duplicated in Rust tests.

`copy-scan.json` is the reviewed path policy for the focused advisory scan; the active nine-key literals are enforced in tooling and migrated product paths, while other lexical candidates remain advisory. Rust test modules are excluded. This is separate from the complete inventory/reconciliation gate.
