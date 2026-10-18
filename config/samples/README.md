# Production-format configuration samples

The files in `vms/` use synthetic identities and the same schema-2 TOML parsed by the core, CLI, and GUI model. They demonstrate intent format, not live enrollment or measured GPU capabilities. Never install these synthetic pairs against real VMs.

| VM GUID suffix | Saved intent |
|---|---|
| `000000000001` | Enabled; no explicit VRAM request |
| `000000000003` | Enabled; explicit ordered VRAM triple |
| `000000000006` | Disabled; same synthetic GPU identity as the first file |

Names, observed power, hardware capabilities, and journals are separate from desired intent. Matching display names do not identify a VM. Two files selecting one GPU demonstrate representation, not qualified concurrent sharing.

`Configuration::parse_vm_file` validates one target and filename/GUID agreement. `Configuration::vm_documents` prepares canonical documents without writing. Schema 3 additionally supports optional Compute/Encode/Decode triples; these examples remain schema 2 for compatibility.

These are parser examples, not a GUI rehearsal workflow. No product tests ran during the documentation rewrite.
