# Configuration ownership

## Implemented baseline

The CLI reads `--config FILE` through `Configuration::parse`. Normal GUI startup
reads real inventory; saved-file loading is not connected to its controller yet.
`Configuration::read_vm_file` now provides a bounded read-only per-VM boundary;
`View::from_vm_file` / `saved_desired_text` let GUI-002 load/display the same intent
without observation or draft inference. Reader/serializer and state separation
passed [9 October testing](evidence/GUI-002-testing.md); protected saving remains open.
Plain `--mock-gui` still uses the earlier session-only simulated
configuration and accepts no config argument. `--mock-gui --snapshot FILE` reads
an existing inventory JSON capture through the real-data presentation, with all
observations historical and no editing, plan or credential/backend requests. It
does not import desired configuration or authorize enrollment. The optional
`--config GUID.toml` loads candidate intent separately for recorded-plan rehearsal.
This implemented route passed scoped automated/headless runtime testing; full
desktop and protected-effect qualification remain open. Future real-data reads must not write
files, vault entries, journals, enrollment or audit records. Preserve mandatory
runner audit; do not use audited discovery as a strict no-write mock source.
[product.example.toml](../config/product.example.toml)
documents runtime schema 2: selected VM GUID, exact GPU interface and enabled state.
Optional VRAM triples use provider-defined units, not GiB/enforced ceilings. Normalize
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

Compute/encode/decode fields are not implemented. Current artifacts/enrollment/
journals use protected `HyperGpuSupportProduct` known-folder directories, not the
approved future configuration location. `src/gui_model.rs::save_configuration`
checks expected contents, flushes a temporary file, rechecks conflicts and renames;
its unsaved state prevents repeating successful GPU operations for save failure.
This is client-side whole-file saving, not worker-only ProgramData persistence or
durable save-only recovery. Reuse the separation/conflict logic under CFG-001.

## OPEN-01: per-VM format

**Selected for implementation, 9 October 2026 (DEC-032); not a completed persistence gate.** Keep
schema-2 TOML and the existing `Configuration` / `Target` / `Allocation` models.
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
| `schema` | Required integer, exactly `2`; unknown versions fail |
| `targets` | Required array; exactly one entry for a per-VM file |
| `targets.vm_id` | Required canonical GUID syntax; normalize content to lowercase, bind to the lowercase filename |
| `targets.gpu_interface` | Required exact partitionable physical GPU interface; reuse current syntax validation, then fresh discovery/enrollment checks before effects |
| `targets.enabled` | Required boolean expressing desired support, never inferred from attachment |
| `targets.vram` | Optional table; omission requests **no allocation write**, preserving existing/provider values rather than resetting them |
| `minimum`, `optimal`, `maximum` | All required if VRAM table exists; unsigned `u64`, minimum ≤ optimal ≤ maximum; zero is an explicit value, not absence |

No VM display name, disks, CPU/RAM, guest OS, power, Secure Boot, driver version,
payload digest, capabilities, verification timestamp, credential reference,
enrollment authority, recovery flags or GUI theme/slider preference goes in this
intent document. Discover those facts or store them in their existing separate
protected records. Credentials remain keyed to VM GUID in the vault, never files.

Compute/encode/decode triples are **missing from schema 2**, not silently dropped
or represented as zero. Their presence currently fails strict parsing. The initial
release still requires GPU-010 to extend the shared intent/capability/readback
model. A schema-3 extension and an explicit schema-2→3 import will be needed if
those fields are introduced; no sample should pretend current execution supports
them. The GPU Memory slider has no validated persistent/provider mapping yet.

The recommended future extension keeps this envelope and reuses `Allocation`
for optional `[targets.compute]`, `[targets.encode]` and `[targets.decode]`
tables, each with the same required three integers. It must use a new declared
schema version and explicit import; do not add accepted-but-ignored fields to
schema 2. File decoding may preserve unsupported intent, but plans/effects must
reject any requested category without qualified input semantics and readback.
This future shape is a proposal, not implemented parser support or a capability
claim. Current samples intentionally contain only fields production accepts today.

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
