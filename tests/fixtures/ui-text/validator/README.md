# Validator fixtures

LANG-007 executable inputs, separate from the immutable LANG-003 JSON comparison evidence above this directory.

`baseline/` is a strict flat nine-key catalogue for the representative LANG-004 subset. Its decoded text is copied verbatim from the historical mapping; metadata matches the independent Rust contract. These are contributor test inputs, not shipped resource files or runtime fallback wording. Product catalogue activation belongs to LANG-004.

`invalid/` contains parser/duplicate-field and adversarial template specimens. The validator tests also mutate copies of the valid documents in memory to isolate missing/unknown fields, duplicate keys/tables, wrong metadata/types, occurrence constraints, collisions, encoding, native bounds and deterministic generation. Expected English UI wording is not duplicated in Rust tests.

`copy-scan.json` is the reviewed path policy for the focused advisory scan; its pending product exceptions expire as the corresponding extraction cards migrate consumers. Hard errors currently guard against baseline fallback copy in the independent contract and generator. This is separate from the complete inventory/reconciliation gate.
