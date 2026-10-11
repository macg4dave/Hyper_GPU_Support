# Production-format configuration samples

`vms/` contains synthetic development identities in the **same schema-2 TOML**
parsed by the CLI, core and GUI model. These are not live enrollments or measured
GPU capabilities. Never install these synthetic pairs against a real VM.

| VM GUID suffix | Saved intent |
|---|---|
| `000000000001` | Enabled; no explicit VRAM allocation request |
| `000000000003` | Enabled; an explicit ordered raw VRAM triple |
| `000000000006` | Disabled; same synthetic GPU identity as the first file |

These GUIDs align with existing UI fixture identities. Names, power, generation,
GPU capabilities, observed attachment and journals belong in separate inventory
fixtures; matching display names never links a configuration. Two files selecting
one GPU demonstrate intent representation, not qualified concurrent sharing.

`Configuration::read_vm_file` uses `Configuration::parse_vm_file`, which delegates
to the existing `Configuration::parse` and validates one target plus filename/GUID
agreement. `View::from_vm_file` loads those exact values as saved intent;
`View::saved_desired_text` displays them independently of observation/draft.
The CLI can already parse each sample with `--config`; no alternate fixture parser
or configuration schema is introduced. These are shared loading/display entry
points. The live controller reads committed per-VM intent separately from observed
inventory; snapshot rehearsal can read a candidate through
`--mock-gui --snapshot FILE --config GUID.toml`.
`Configuration::vm_documents` prepares canonical files from schema-2 bundles
without writing files or modifying the original intent.

See the [configuration contract](../../docs/CONFIGURATION.md) and
[source baseline](../../docs/BASELINE.md). Schema 3 also supports optional
Compute/Encode/Decode triples; these retained samples deliberately remain schema 2.
The former external integration suite was deleted. Surviving inline parser/model
tests and controller paths were inspected, but no tests or GUI runs were performed
during the documentation reset.
