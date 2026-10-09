# Configuration ownership

## Implemented baseline

The CLI reads `--config FILE`. Normal GUI startup reads real inventory; configuration
loading/editing is not connected yet. Plain `--mock-gui` uses session-only simulated
configuration and accepts no config argument. `--mock-gui --snapshot FILE` reads
an existing inventory JSON capture through the real-data presentation, with all
observations historical and no editing, plan or credential/backend requests. It
does not import desired configuration or authorize enrollment. Snapshot-route
qualification remains outstanding. Future real-data reads must not write
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

## Approved per-VM contract (planned)

The backend persistence direction from historical DEC-029 remains scoped here;
[CFG-001](BACKLOG.md#cfg-001)
owns implementation. Exact format, schema version and import/migration remain
a CFG-001 implementation question. Prefer extending the existing typed TOML parser unless evidence justifies
another format. GUI promotion performs no migration or real configuration writes.

- One file per stable Hyper-V VM GUID under
  `%ProgramData%\HyperGpuSupport\config\vms\`. Names are display data, never keys.
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
