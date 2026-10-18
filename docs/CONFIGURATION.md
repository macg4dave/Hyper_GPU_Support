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

The workflow computes an explicit category write set: a retained adapter receives only specified triples that differ from fresh observations, while a created/recreated adapter receives every specified triple. Matching categories still participate in capability validation, drift guards and independent final readback. Desired TOML and saved revisions are unchanged by skipping a redundant write. Automated and installation evidence is recorded in the [checkpoint](ENGINEERING.md#optimization-validation-checkpoint); live allocation acceptance remains pending.

The memory slider displays 1 decimal GB through the independently detected physical card capacity, using 0.1 GB steps and an exact final endpoint. Reported partition metadata does not impose a VRAM admission ceiling in the slider, shared planner or native writer. Byte values are sent unchanged; no capacity ratio is used. Raw provider metadata does not establish a physical GB ceiling. Allocation behavior remains unqualified; saved requests are retained without a guessed ratio or rewritten intent.

Use the toggle or More actions > Remove GPU support to disable support. Reading or displaying saved settings never clamps or rewrites them to the new slider minimum; older requests remain exact. Advanced Allocation retains the VRAM triple for compatibility/correction. Slider edits change the optimal byte request and expand an existing minimum/maximum only as needed to retain ordered bounds; other resource categories and other VMs remain unchanged. Schema-3 advanced categories remain compatibility context, not new GUI requirements. Conversion, requested values, observed values, and enforced limits are different concepts. No reservation, fair-sharing, or isolation guarantee follows from the display.

## Input and committed files

- CLI `--config FILE` accepts a production configuration bundle, including a single target. Selection options do not grant authority.
- Committed intent uses lowercase `<vm-guid>.toml` files, each containing exactly one target with the matching canonical GUID.
- The per-VM reader is bounded to 64 KiB of UTF-8 TOML; the CLI bundle and protected enrollment readers use the 1 MiB frame limit. The guest preparation manifest reader is bounded to 32 MiB. These limits apply while reading, before parsing the complete input. A valid candidate file is still untrusted input, not a protected-store admission result.
- `Configuration::vm_documents` produces canonical documents without writing them. Schema 2 remains readable; current worker effect publication serializes schema 3.
- The protected store is under the product's fixed Windows known-folder data root, in `config/vms`. Paths are derived by the product; callers do not choose privileged publication destinations.
- Expected revisions detect external changes. Unreadable, malformed, untrusted, or conflicting committed state must be reported as unknown/blocked, not silently replaced by invented values.

The [example](../config/product.example.toml) and [samples](../config/samples/README.md) use synthetic identities. They must not be installed against real targets. Do not add passwords, payload integrity pins, machine names, display names, or observed power/health to desired intent.

## Publication and import

Profile deletion uses a `RemovalReview` containing the existing exact pair/operator/policy review plus complete GUID-keyed saved intent and managed-journal revision maps, including selected absence. The existing enrollment-pending record gains an optional `removal` member with that immutable review and durable configuration/history deletion flags. Legacy additive enrollment holds omit the member and retain readback-only reconciliation; profile-removal recovery can finish only remaining bound deletions and revoke the exact pairing last. A completed deletion must stay absent; changed/recreated or unrelated records block recovery. Per-VM TOML and managed JSON are removed through fixed protected locations, while shared artifacts and operational audits remain. A final empty target list retains installed policy/operator/artifacts and supports later separately reviewed enrollment. Older installed readers reject the new operation/removal hold; source edits do not upgrade installation.

The workflow journal includes default-false `settings_released` to the schema-1 workflow journal. Successful removal or management reconciliation of disabled intent sets it only after host/final-off readback, preserving preparation and graphics history. Released or absent journals provide no authority to restore compatibility fields. Otherwise each field is restored only while its current value equals the last recorded product write. A later enable captures the current baseline and reacquires ownership through confirmed managed settings work. Legacy journals retain their existing ownership interpretation; older readers reject the new field. Disabled intent retains resource requests for compatibility but performs no allocation writes.

The journal also includes optional `preparation_started` (absent for legacy journals) and default-false `check_restored` workflow evidence. Only new admission can establish preparation-not-started; only a terminal guest observation followed by independent original-host restoration can establish a restored failed check. The recovery receipt's optional `retirement` binds the fixed negative-completion reason, unchanged host observation and exact before/after journals before changing the pending journal. A lost reply or interruption permits resuming only those bound states. These additive schema-1 fields do not authorize publication, replay or automatic repair; older readers fail closed on new fields.

The operation plan's optional `reinstall` flag defaults to false for older plans. It requests a single reviewed complete preparation sequence and is bound into fresh worker plan comparison and the recovery receipt's plan revision. It is not stored in desired TOML intent, enrollment authority or an automatic repair policy. A disabled target cannot request reinstall. Reinstall and Verify use their fresh prepared-worker exchange; the unused `ReinstallPlan`, `VerifyPlan` and `PreliminaryPlan` read RPCs and stand-alone reinstall/preliminary preview APIs are removed. Current plan JSON no longer has `authentication_deferred`; unknown plan fields, including that retired marker, are rejected. These are ephemeral plan/read-protocol changes, not migrations of saved intent, journals, recovery receipts or historical audits. Use matching candidate binaries; updated source does not update installed binaries.

The protected per-VM workflow journal retains schema 1 with an additive optional `latest_check` field. Older journals without it remain readable as historical receipts, without a fresh check claim. New records distinguish incomplete/interrupted, passed, failed, unavailable and proven payload mismatch; unknown enum values are rejected. `last_verified` remains the historical successful lifecycle-complete timestamp. A newer adverse outcome prevents historical success from skipping verification, and proven payload mismatch removes the prepared digest so unchanged host payloads still require preparation. Older binaries that deny unknown journal fields reject new records; do not downgrade against them. These fields are operation evidence, not desired TOML intent.

Normal effect publication binds the target, approved plan, destination revision, driver/payload identity, independently verified state, and canonical source. A verified-but-unsaved operation permits saving that exact bound intent; it does not permit repeating GPU work. See [security/recovery](SECURITY.md).

Protected operation receipts also record `expected_power` independently of `initial.power`. This is operation/recovery evidence, not desired TOML intent. Initial setup binds Off; other operations bind their reviewed final state. Older receipts without this field retain their original-power contract. Save-only must observe the recorded final power as well as the verified identity/allocation state.

Explicit CLI `import` is distinct from effects. It imports an approved bundle into absent per-VM destinations, does not enroll pairs, and does not apply GPU state. A multi-file import has per-file receipts rather than an all-or-nothing transaction. `import-reconcile --operation ID` can resume only the recorded destination set under its conflict checks. The GUI currently reports pending import for CLI resolution.

## Per-user GUI preference

On graceful close, the GUI saves the selected VM GUID to `%LOCALAPPDATA%/HyperGpuSupport/selected-vm.txt` and restores it before initial discovery. The bounded reader accepts only a 36-character GUID and normalises it to lowercase; missing, malformed or unreadable preferences use normal discovery selection. Selection follows the GUID across list reordering and VM renaming. If that VM is absent, discovery selects the first available VM, or no VM when the list is empty.

This preference stores no drafts, GPU settings, credentials or operation authority. Saved intent and recovery records continue to come from their existing protected readers. Restoring selection does not resume operations or clear recovery. A failed preference write is returned as an application shutdown error; abrupt termination does not save a new selection.

### Remembered guest OS icons

GUI discovery, including setup discovery and automatic status refresh, saves each available guest-reported OS name in `%LOCALAPPDATA%/HyperGpuSupport/guest-os/<vm-guid>.txt`. These per-user display metadata files are separate from protected GPU intent. When live OS metadata is unavailable, including while a VM is off or after reopening the app, the icon uses its last saved OS name. A fresh OS name replaces it, including an unfamiliar OS that requires the generic icon. Names and renames never determine OS identity; a new VM GUID does not inherit another VM's icon. A VM never observed reporting its OS retains the generic icon. No guest is started or entered to populate this cache.

Reads are bounded to 1024 bytes of UTF-8 OS text and filenames require canonical GUIDs. Missing, malformed or unreadable cache entries provide no fallback. Cache write failures appear as discovery warnings while fresh inventory remains usable. Cached OS information affects presentation only and grants no operation, enrollment or recovery authority. The CLI and protected runner continue to report live inventory without this GUI fallback.

Installed-GUI icon acceptance remains pending.

## Development configuration

`config/project.toml`, `config/artifact-pins.toml`, and `config/runner-policy-v1.json` are retained development context, not runtime intent. Use them only when applicable to an explicitly prepared development environment.

Keep mutable host/guest/GPU paths and identities in explicit local environment configuration/evidence rather than general handbook or prompts. Use ignored `local/` or `config/*.local.toml` where suitable. Do not silently wire old development pins into the production schema or turn a single recorded environment into a universal support promise.

## Reinstall and native recovery receipts

Routine administrator reinstall preserves existing enrollment targets/operator even when desired configuration fields differ. Its schema-2 `installation-pending.json` binds before/after artifact policy and pending-pair receipts. Publication can resume only the recorded package and authority states. Before an authority write, both pending-pair revisions are rebased to the new pins; after a completed authority write, its original reviewed token remains historical and only the resulting token is updated. Removal snapshots and progress remain unchanged. Final policy/artifact/pair readback and an archived reinstall receipt precede removal of the installation marker. The separate GUI first-install receipt is never replaced through this route.

New schema-2 `provider-operation-<vm-guid>.json` records include the exact VM/GPU and host-effect stage, full before state, requested effect, workflow/configuration revisions, pre-existing job identities and optional typed native failure. Schema-1 records remain readable but missing scope is never invented. Administrator native review saves `provider-review.json` with exact pending-marker revision, independent terminal job evidence and fresh host readback; its SHA-256 token binds the separate confirmation. A captured terminal receipt can outlive its provider job, but unavailable evidence or changed bindings retain recovery. Original marker and completion evidence are archived before retirement; workflow holds remain separate. See [security recovery](SECURITY.md#operation-recovery) for the supported administrator commands and evidence requirements.

### More Actions and profile deletion

Delete profile removes the selected app settings, pairing, managed history and bound interrupted-operation records. It is available independently of VM power and GPU attachment. It dispatches no VM, GPU, power or guest changes. The foreground action prepares a fresh exact pairing/revision review and proceeds directly when current and available; there is no separate confirmation dialog. Protected deletion, audit, resumable progress and readback remain mandatory. See [recovery semantics](SECURITY.md#operation-recovery).

## VM-scoped recovery storage

New workflow receipts retain the schema-1 `SaveRecord` format and use protected `operation-recovery-<vm-guid>.json` files. The canonical filename must match the record VM; duplicate VM/operation identities, invalid versions and unreadable receipts fail closed. Existing `operation-recovery.json` VM receipts are read without changing them. Before an admitted VM worker operation, under the common lock and durable admission audit, the worker copies any legacy receipt to its canonical VM file, verifies identical readback, and atomically replaces the legacy slot with a schema-2 `{ "schema": 2, "per_vm": true }` marker. No VM work starts before this succeeds. An interruption can leave both identical copies; that exact legacy/canonical pair counts as one hold and migration resumes without rewriting the copy. Conflicting copies preserve both and block admission.

The marker makes older singleton readers refuse admission while VM receipts exist; it carries no VM outcome and is never accepted as a `SaveRecord`. The final VM receipt and marker retire only after its durable terminal outcome. Reconciliation and save-only select their receipt by exact operation identity after migration; they do not clear another VM's hold or repeat its effects.

Provider receipts likewise use canonical `provider-operation-<vm-guid>.json` paths. Legacy `provider-operation.json` is retained and uses its validated scope; missing legacy scope blocks all new mutations. Native review selects the marker by the independently observed job VM, and confirmation still binds the exact pending-marker revision. VM receipts remain independently recoverable after restart. Shared enrollment/import stores and the common operation lock are unchanged. Installed multi-VM acceptance remains pending; automated evidence is in the [checkpoint](ENGINEERING.md#optimization-validation-checkpoint).

## Completed storage policy

Protected `audit/` read records follow the [retention contract](SECURITY.md#completed-record-retention): seven days, 512 records, 4 MiB, with a minimum 32 records and ten-minute grace period. Safety exclusions and administrator `<record>.retain` markers take precedence over those bounds. The protected `audit-retention.json` checkpoint limits attempts to once per minute, with cancellation/deadline checks. Cleanup is best effort after durable terminal auditing, under the common lock; failed or uncertain evidence remains available.

Protected `staging/` descriptors bind a unique guest bundle to its operation, VM, manifest, mode and file count. Pending descriptors never authorize deletion. Completed acknowledgements are queued after terminal audit and exact recovery retirement, then consumed at most two per invocation by a later authorized guest session for that VM. Matching guest staging alone can be removed; installed destinations and preparation receipts are preserved. The host queue remains on interruption or failed cleanup. This policy introduces no user-configurable path, guest credential storage or automatic guest lifecycle work. Legacy bundles remain unacknowledged and are retained.
