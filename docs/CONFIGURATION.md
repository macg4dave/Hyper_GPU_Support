# Configuration

This page owns runtime intent, parsing, storage, and compatibility. The production types and parser are in [model](../src/model.rs); the protected storage contract is in [store](../src/configuration_store.rs) and its [writer](../src/configuration_store/protected.rs).

## Runtime intent

`Configuration` contains `schema` and `targets`. Each `Target` contains a stable `vm_id`, exact partitionable `gpu_interface`, `enabled`, and optional resource request triples. The same types/parser serve the CLI, GUI, imports, and sample intent. Preserve this shared production schema.

| Schema | Supported intent |
|---|---|
| 2 | VM/GPU identity, enabled flag, optional VRAM triple |
| 3 | Schema-2 fields plus optional Compute, Encode, and Decode triples |

Each triple has `minimum`, `optimal`, and `maximum` unsigned integers, with `minimum <= optimal <= maximum`. Fresh provider capability is required before a requested category is applied. VRAM requests use bytes and are checked for ordering, without imposing reported partition metadata as a byte ceiling; Hyper-V acceptance and independent readback remain required. Compute/Encode/Decode retain provider range validation. Unknown/unavailable categories must not acquire invented limits. Unknown TOML fields and unsupported schemas are rejected. Extended category requests require schema 3; parsing an older file does not rewrite it.

An omitted triple requests **no explicit category write**. A newly attached/recreated adapter gets provider defaults; an adapter retained in place keeps its existing values where no write is requested. Omission does not mean zero allocation or a guaranteed reset. Blank all three advanced VRAM fields to omit the VRAM request; a partial triple is not a valid request. Compute/Encode/Decode are not edited by this GUI; existing schema-3 requests are preserved for compatibility and shown in review.

The memory slider displays 1 decimal GB through the independently detected physical card capacity, using 0.1 GB steps and an exact final endpoint. [APP-011](BACKLOG.md#app-011--correct-physical-vram-and-provider-mapping) removes the metadata-derived VRAM admission ceiling from the slider, shared planner/GUI admission and native writer. Byte values are sent unchanged; no capacity ratio is used. Raw provider metadata does not establish a physical GB ceiling. Allocation behavior remains unqualified; saved requests are retained without a guessed ratio or rewritten intent.

Use the toggle or More actions > Remove GPU support to disable support. Reading or displaying saved settings never clamps or rewrites them to the new slider minimum; older requests remain exact. Advanced Allocation retains the VRAM triple for compatibility/correction. Slider edits change the optimal byte request and expand an existing minimum/maximum only as needed to retain ordered bounds; other resource categories and other VMs remain unchanged. Schema-3 advanced categories remain compatibility context, not new GUI requirements. Conversion, requested values, observed values, and enforced limits are different concepts. No reservation, fair-sharing, or isolation guarantee follows from the display.

## Input and committed files

- CLI `--config FILE` accepts a production configuration bundle, including a single target. Selection options do not grant authority.
- Committed intent uses lowercase `<vm-guid>.toml` files, each containing exactly one target with the matching canonical GUID.
- The per-VM reader is bounded to 64 KiB of UTF-8 TOML; the CLI bundle reader uses the 1 MiB frame limit. A valid candidate file is still untrusted input, not a protected-store admission result.
- `Configuration::vm_documents` produces canonical documents without writing them. Schema 2 remains readable; current worker effect publication serializes schema 3.
- The protected store is under the product's fixed Windows known-folder data root, in `config/vms`. Paths are derived by the product; callers do not choose privileged publication destinations.
- Expected revisions detect external changes. Unreadable, malformed, untrusted, or conflicting committed state must be reported as unknown/blocked, not silently replaced by invented values.

The [example](../config/product.example.toml) and [samples](../config/samples/README.md) use synthetic identities. They must not be installed against real targets. Do not add passwords, payload integrity pins, machine names, display names, or observed power/health to desired intent.

## Publication and import

APP-022 uses a `RemovalReview` containing the existing exact pair/operator/policy review plus complete GUID-keyed saved intent and managed-journal revision maps, including selected absence. The existing enrollment-pending record gains an optional `removal` member with that immutable review and durable configuration/history deletion flags. Legacy additive enrollment holds omit the member and retain readback-only reconciliation; profile-removal recovery can finish only remaining bound deletions and revoke the exact pairing last. A completed deletion must stay absent; changed/recreated or unrelated records block recovery. Per-VM TOML and managed JSON are removed through fixed protected locations, while shared artifacts and operational audits remain. A final empty target list retains installed policy/operator/artifacts and supports later separately reviewed enrollment. Older installed readers reject the new operation/removal hold; source edits do not upgrade installation.

APP-021 adds default-false `settings_released` to the schema-1 workflow journal. Successful removal or management reconciliation of disabled intent sets it only after host/final-off readback, preserving preparation and graphics history. Released or absent journals provide no authority to restore compatibility fields. Otherwise each field is restored only while its current value equals the last recorded product write. A later enable captures the current baseline and reacquires ownership through confirmed managed settings work. Legacy journals retain their existing ownership interpretation; older readers reject the new field. Disabled intent retains resource requests for compatibility but performs no allocation writes.

Review fixes add optional `preparation_started` (absent for legacy journals) and default-false `check_restored` workflow evidence. Only new admission can establish preparation-not-started; only a terminal guest observation followed by independent original-host restoration can establish a restored failed check. The recovery receipt's optional `retirement` binds the fixed negative-completion reason, unchanged host observation and exact before/after journals before changing the pending journal. A lost reply or interruption permits resuming only those bound states. These additive schema-1 fields do not authorize publication, replay or automatic repair; older readers fail closed on new fields.

The operation plan's optional `reinstall` flag defaults to false for older plans. It requests a single reviewed complete preparation sequence and is bound into fresh worker plan comparison and the recovery receipt's plan revision. It is not stored in desired TOML intent, enrollment authority or an automatic repair policy. A disabled target cannot request reinstall. The protected runner's `ReinstallPlan` operation is read-only; effects still use the existing reviewed Apply worker. Older installed runners cannot preview this new operation; updated source does not update installed binaries.

The protected per-VM workflow journal retains schema 1 with an additive optional `latest_check` field. Older journals without it remain readable as historical receipts, without a fresh check claim. New records distinguish incomplete/interrupted, passed, failed, unavailable and proven payload mismatch; unknown enum values are rejected. `last_verified` remains the historical successful lifecycle-complete timestamp. A newer adverse outcome prevents historical success from skipping verification, and proven payload mismatch removes the prepared digest so unchanged host payloads still require preparation. Older binaries that deny unknown journal fields reject new records; do not downgrade against them. These fields are operation evidence, not desired TOML intent.

Normal effect publication binds the target, approved plan, destination revision, driver/payload identity, independently verified state, and canonical source. A verified-but-unsaved operation permits saving that exact bound intent; it does not permit repeating GPU work. See [security/recovery](SECURITY.md).

Protected operation receipts also record `expected_power` independently of `initial.power`. This is operation/recovery evidence, not desired TOML intent. Initial setup binds Off; other operations bind their reviewed final state. Older receipts without this field retain their original-power contract. Save-only must observe the recorded final power as well as the verified identity/allocation state.

Explicit CLI `import` is distinct from effects. It imports an approved bundle into absent per-VM destinations, does not enroll pairs, and does not apply GPU state. A multi-file import has per-file receipts rather than an all-or-nothing transaction. `import-reconcile --operation ID` can resume only the recorded destination set under its conflict checks. The GUI currently reports pending import for CLI resolution.

## Development configuration

`config/project.toml`, `config/artifact-pins.toml`, and `config/runner-policy-v1.json` are retained development context, not runtime intent. Their former scripts were deleted. Assess individual files only when required for the current task; APP-009 selects the actual qualification environment.

Keep mutable host/guest/GPU paths and identities in explicit local environment configuration/evidence rather than general handbook or prompts. Use ignored `local/` or `config/*.local.toml` where suitable. Do not silently wire old development pins into the production schema or turn a single recorded environment into a universal support promise.
