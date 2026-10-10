# Configuration ownership

## Implemented baseline

The CLI reads `--config FILE` through `Configuration::parse`. Normal GUI startup
reads real inventory and protected committed per-VM intent separately from enrollment.
`Configuration::read_vm_file` now provides a bounded read-only per-VM boundary;
`View::from_vm_file` / `saved_desired_text` let GUI-002 load/display the same intent
without observation or draft inference. Reader/serializer and state separation
passed [9 October testing](evidence/GUI-002-testing.md); protected saving remains open.
Plain `--mock-gui` still uses the earlier session-only simulated
configuration and accepts no config argument. `--mock-gui --snapshot FILE` reads
an existing inventory JSON capture through the real-data presentation, with all
observations historical and no effects or persistent writes. It supports shared
recorded-input draft/plan rehearsal, without credential/backend effects. It
does not import desired configuration or authorize enrollment. The optional
`--config GUID.toml` loads candidate intent separately for recorded-plan rehearsal.
This implemented route passed scoped automated/headless runtime testing; full
desktop and protected-effect qualification remain open. Future real-data reads must not write
files, vault entries, journals, enrollment or audit records. Preserve mandatory
runner audit; do not use audited discovery as a strict no-write mock source.
[product.example.toml](../config/product.example.toml)
retains compatible runtime schema 2: selected VM GUID, exact GPU interface and enabled state.
Schema 3 additionally accepts optional Compute/Encode/Decode triples alongside VRAM.
All triples use raw provider-defined units, not GiB/percentages/enforced ceilings. Normalize
GUIDs; reject duplicate VMs and unknown fields. Several VMs may select the same GPU,
subject to qualification. Parse once and pass typed values; no product config is embedded.

Discover names, disks, drivers, payloads, hashes/counts/catalogs and capabilities.
Administrator installation generates protected VM/GPU enrollment. Caller config cannot
broaden it; changing pairs requires administrator re-enrollment. Credentials belong
only in ephemeral memory or explicit per-user/per-VM Windows Credential Manager entries.
Installation validates schema, target bounds and unique VM identities even for native
callers, then requires each selected VM to be discovered as Generation 2 and each GPU
interface to match current discovery before changing installed artifacts or task state.
VM display names and discovered driver versions do not bind enrollment.

Compute/encode/decode schema, capability, plan, write/readback and existing GUI
editor bindings are implemented in source, pending tests/build/UI/live validation.
Current artifacts/enrollment/
journals use protected `HyperGpuSupportProduct` known-folder directories, not the
approved future configuration location. Protected worker publication/import and
durable save-only recovery exist in `configuration_store{,/protected}.rs` and
`worker.rs`; their expanded acceptance remains pending. The older client-side
`gui_model::save_configuration` helper retains conflict logic but is not the
protected live publisher. Save failure never grants authority to replay GPU effects.

## OPEN-01: per-VM format

**Selected for implementation, 9 October 2026 (DEC-032); not a completed persistence gate.** Keep
schema-2 input compatibility and the existing `Configuration` / `Target` / `Allocation` models;
DEC-033 adds schema 3 for four-category intent.
Each `%ProgramData%\HyperGpuSupportProduct\config\vms\<vm-guid>.toml` contains exactly
one target. The array envelope is retained deliberately so the CLI parser,
validation and serialization remain identical; a new flat JSON/TOML model buys
no behavior and would require migration. No separate mock configuration format.

```toml
schema = 2

[[targets]]
vm_id = "a5801e91-1083-4e79-a803-000000000003"
gpu_interface = '\\?\PCI#VEN_10DE&DEV_2D05#SAMPLE_GPU_B#{064092b3-625e-43bf-9eb5-dc845897dd59}\GPUPARAV'
enabled = true

[targets.vram]
minimum = 268435456
optimal = 536870912
maximum = 1073741824
```

This is the exact [production-format sample](../config/samples/vms/a5801e91-1083-4e79-a803-000000000003.toml),
with synthetic identities and illustrative raw integers, not measured limits.

| Field | Required / meaning |
|---|---|
| `schema` | Required integer, `2` or `3`; unknown versions fail; schema 2 rejects Compute/Encode/Decode intent |
| `targets` | Required array; exactly one entry for a per-VM file |
| `targets.vm_id` | Required canonical GUID syntax; normalize content to lowercase, bind to the lowercase filename |
| `targets.gpu_interface` | Required exact partitionable physical GPU interface; reuse current syntax validation, then fresh discovery/enrollment checks before effects |
| `targets.enabled` | Required boolean expressing desired support, never inferred from attachment |
| `targets.vram` | Optional table; omission requests **no explicit allocation write**, with provider defaults for new/recreated adapters and retained values for adapters kept in place |
| `targets.compute`, `targets.encode`, `targets.decode` | Optional schema-3 tables; each omitted category requests no explicit category write |
| `minimum`, `optimal`, `maximum` | All required if a resource table exists; unsigned `u64`, minimum ≤ optimal ≤ maximum; zero is an explicit value, not absence |

No VM display name, disks, CPU/RAM, guest OS, power, Secure Boot, driver version,
payload digest, capabilities, verification timestamp, credential reference,
enrollment authority, recovery flags or GUI theme/slider preference goes in this
intent document. Discover those facts or store them in their existing separate
protected records. Credentials remain keyed to VM GUID in the vault, never files.

Compute/encode/decode triples remain **rejected in schema 2**. Schema 3 keeps this
envelope and reuses `Allocation` for optional `[targets.compute]`, `[targets.encode]`
and `[targets.decode]` tables. Parsing and `vm_documents` preserve the declared
version; reading never rewrites or imports a file. `into_schema3` explicitly upgrades
typed intent in memory. Protected publication after separately reviewed Apply writes
schema 3, stated in Review; explicit imports retain their supplied schema and exact
source/revision contract. Mixed protected files produce a schema-3 in-memory view,
without changing source files or granting authority.

File decoding preserves structurally valid unsupported intent. Enabled plans reject
requested categories without complete ordered provider ranges. Disabled intent may
retain those requests but performs no resource writes. Native writes use fresh
`Msvm_PartitionableGpu` limits and `Msvm_GpuPartitionSettingData` read/write properties
for each category; final readback, save-only and reconciliation compare every request.
See Microsoft's [capability properties](https://learn.microsoft.com/en-us/windows/win32/hyperv_v2/msvm-partitionablegpu)
and [partition settings](https://learn.microsoft.com/en-us/windows/win32/hyperv_v2/msvm-gpupartitionsettingdata).
Missing/malformed category capability is unavailable, independently of other
categories; old snapshots' direct VRAM ranges remain readable, with missing new
capabilities unknown. Omitted observed values mean unobserved/unset, never zero.
An omitted request performs no explicit category write; new/recreated attachments
use provider defaults, and attachments retained in place keep existing values.
Provider semantics and enforcement remain unqualified. All new source remains
untested. Subsequent GPU-010.3 source maps native VRAM byte requests to decimal GB
(DEC-034) without changing this schema: the slider and raw fields produce the same
integer triple. Historical units remain unknown; actual GUI/provider behavior and
physical enforcement remain unverified.

## Read and validation contract

1. Resolve the production directory through the Windows known-folder API; its
   trusted installer/worker path and ACL policy remain CFG-001/SEC-001 work.
   Fixture callers explicitly pass `config/samples/vms/<guid>.toml`; no fallback
   from a failed production read to samples.
2. `Configuration::read_vm_file` reads at most 64 KiB of UTF-8 TOML and delegates
   to `parse_vm_file`, then `Configuration::parse`. Return typed intent plus exact
   original source text for conflict checks; do not reread/parse during ordinary
   editing. Reject malformed/unknown fields, unknown schema, empty/multiple
   per-file targets, invalid IDs/interfaces and incomplete/unordered triples.
3. Require the normalized target GUID to equal the canonical lowercase filename.
   Never locate by display name. Uppercase GUID content can normalize; noncanonical
   filenames require explicit import/rename, not a silent production rewrite.
4. Saved data can load even if a VM/GPU is missing, denied or currently outside
   provider limits. Report these as runtime eligibility states. Structural parsing
   is independent of provider limits, Generation 2, guest health and authorization.
5. Before planning/applying, recheck current identities, enrollment, provider
   capability/units/ranges, stable power, conflicts and recovery holds through
   existing shared logic. A parsed file or imported enrollment snapshot is never
   authority for effects. Do not clamp, guess units or erase unsupported intent.
6. Future directory loading must isolate per-file missing/denied/invalid errors,
   reject duplicate normalized identities, preserve other valid saved entries and
   retain each source revision. Missing file is unconfigured; unreadable/invalid
   file is unknown/error, never silently a default. The implemented single-file
   reader does not yet enumerate directories or establish ACL/reparse trust.

## Saved, observed and unsaved state

| State | Source and lifetime | Meaning |
|---|---|---|
| Saved desired | Parsed per-VM `Target` plus exact source text/revision | Last committed intent; loaded read-only, unchanged during editing |
| Observed | Fresh independent `VmState` / GPU capabilities, with provenance/freshness | Actual attachment/settings/power; may disagree with saved intent |
| GUI draft | One in-memory copy of desired `Target`; raw invalid editor text remains separate | Unsaved proposed intent; changing/discarding it writes nothing |
| Verified-but-unsaved operation | Bound protected result/pending-save record (planned) | Effects passed readback but saving failed; retry saving only |
| Enrollment / journal | Existing protected authorization and operation records | Neither saved intent nor fresh guest health; cannot be cleared by editing |

For a VM without a saved file, provider observations or enrollment may supply a
**labelled suggested draft**, never a fictional saved configuration. Refresh
updates observations separately; it must not replace saved intent with enrollment
or silently overwrite an existing draft. External config/provider changes mark the
draft stale and block Apply until explicitly resolved. GUID lookup connects states.

`View` already holds configuration, inventory, draft, `unsaved` and readback gates.
The new `View::from_vm_file` supplies saved configuration through the production
reader; `saved_desired_text` reads only that configuration. The current live
controller still overwrites its configuration with enrollment in `State::refresh`;
that must be removed when saved files are bound. The earlier mock `Saved` type
also stores GPU indices, a GB preference and string fields; it must be replaced
with the shared `Target` for intent (raw editor strings/session presentation may
remain). Neither is accepted as the new configuration architecture.

## Existing storage audit and migration gaps

| Existing code | Reuse / gap |
|---|---|
| `model::Configuration::parse/validate`, `Target::validate`, `Allocation::validate` | Reuse strict schema/GUID/duplicate/order/range rules; current execution supports VRAM only |
| CLI `--config` | Already parses each sample unchanged; existing multi-target files remain valid explicit CLI input |
| `gui_model::save_configuration` | Reuse expected-source conflict checks and successful-but-unsaved separation; not the protected per-VM writer |
| `runner::atomic_json`, `security::verify`, `guest::replace_file` | Existing flush/replace/ownership/reparse mechanisms; adapt safely for TOML and ordinary-reader access, not a second storage stack |
| Existing enrollment/journals | Remain in `HyperGpuSupportProduct`; do not copy them into intent or infer intent from them |
| `Saved` / live enrollment-backed drafts | Transitional presentation only; migrate to loaded `Target` and explicit provenance after this review |

Schema-2 bundles need an **explicit** split/import into one-target documents using
the same serializer. No automatic startup migration, enrollment expansion or GPU
replay. If a destination already exists, conflict requires operator resolution;
retain the source. No content migration is necessary for a one-target schema-2
file, but its canonical filename and trusted storage publication need validation.

### Explicit import contract

`Configuration::vm_documents()` implements the shared **read-only preparation**
step. It validates the whole schema-2 input, normalizes a cloned target GUID and
serializes one schema-2 document per canonical filename. It retains GPU interface,
enabled state and the optional raw triple exactly; it does not change the source
or create any files. Comments/formatting are not transferred by serialization;
retain the original source for provenance and conflict resolution.

The eventual protected import operation must:

1. Be explicitly selected; never run during startup, Refresh, fixture loading or
   ordinary editing. Use the fixed known-folder destination, deriving every leaf
   from a validated GUID rather than accepting arbitrary destination paths.
2. Present the complete destination set and check existing files before publishing.
   Any existing destination, including identical intent, requires explicit conflict
   resolution; no silent merge/overwrite. Missing, denied and unreadable destinations
   are distinct. An unchanged-content SHA/text revision is a conflict token, not
   authorization or a substitute for trusted file ownership.
3. Under protected host-wide coordination, recheck directory/file ACLs, reparse
   protection, destination revisions and approved scope immediately before commit.
   Preserve existing enrollment/journals/audits; importing intent never enrolls a
   new pair, clears recovery, prepares a guest or changes Hyper-V.
4. Flush and atomically replace each approved destination using the protected
   storage machinery. Several files are not one atomic filesystem transaction:
   retain attributable per-file progress/recovery and report partial publication.
   On interruption/failure, stop further publication and require reconciliation;
   no blind reimport or automatic rollback of already committed files.
5. Keep the legacy source unchanged. Publication/recovery/expected revisions must
   be independently rechecked before a retry; configuration import never replays
   successful GPU operations. Future schema upgrades must be explicit versioned
   transformations and preserve omissions, not fill unknown resources with zero.

No import CLI operation or privileged publisher is implemented by OPEN-01. Those
mechanisms and their independent boundary review remain CFG-001/SEC-001/CORE-028.
The format/identity/import contract is resolved; code qualification is deferred
under the milestone policy.

Remaining architecture: installer-created config directory/read ACLs; protected
worker-only TOML commits; race-safe revision comparison under host-wide locking;
fresh readback binding; durable save-only records and crash recovery; directory
aggregation/error isolation; shared full-resource schema/capabilities; deliberate
legacy import. A source-text check plus client `rename` is not a complete protected
transaction. These are CFG-001/SEC-001/CORE-028 gates, not waived by format choice.

The [three samples](../config/samples/README.md) cover enabled/no-write VRAM,
explicit raw VRAM, disabled intent and shared-GPU representation. GUI loading and
display entry points use the production parser; the format prerequisite for
controller integration is resolved. Added fixture/parser/import coverage remains **unrun**; no builds,
tests, GUI launch, host/VM queries, config-store publication or migration performed.

## Protected persistence contract (still planned)

The backend persistence direction from historical DEC-029 remains scoped here;
[CFG-001](BACKLOG.md#cfg-001) owns implementation. The contract above resolves the
format choice, not protected write or expanded-resource acceptance.

- One file per stable Hyper-V VM GUID under
  `%ProgramData%\HyperGpuSupportProduct\config\vms\`. Names are display data, never keys.
- Desired intent includes physical GPU, enabled state and Min/Optimal/Max for VRAM,
  compute, encode and decode. Discover inventory; unsupported/unknown capabilities
  explicitly block unsafe requests. Raw values imply neither GiB nor enforcement.
- Saved desired state, observed state and one in-memory GUI draft are separate.
  Editing never writes configuration or changes Hyper-V.
- Only the restricted elevated worker atomically commits machine-wide configuration
  after independently verified successful operations. CLI and GUI use the same path.
- Detect external file and relevant Hyper-V/provider changes before Apply and save;
  block stale intent and require refresh, without silent merge or overwrite.
- Verified GPU success followed by save failure retains a distinct pending-save
  record. A fixed save-only request revalidates identity, successful operation,
  current readback and expected file revision; it never repeats GPU modification.
  Exact immutable binding and crash behavior remain a CFG-001 security question.
- Protected enrollment, operation/recovery journals and admission/audit records
  remain separate from desired configuration. A config edit cannot clear a recovery
  hold or authorize a new pair. Reuse existing ACL, atomic-write and reparse safeguards;
  qualify changed location/ownership and migration before deployment.
- Current GUI appearance/window/split state is session-only; persistent preferences absent from the approved prototype are retired v1.0 GUI scope. Session-only human
  diagnostics do not imply deleting protected security/recovery records.

Host GPU partition count is observed/read-only. No credentials belong in files.

## Contributor configuration

`config/project.toml`, `runner-policy-v1.json` and `artifact-pins.toml` belong to the
standalone laboratory. Preserve its target, golden-disk and installed safeguards.
Maintained scripts use the shared laboratory reader. Build lab artifacts separately;
the production package neither imports this package nor consumes its pins.
Protocol constants, safety bounds and graphics oracles stay in code. Operation
manifests/receipts are generated data, not additional operator settings authorities.
