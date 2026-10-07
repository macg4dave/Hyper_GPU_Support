# Configuration ownership

## Product configuration

CLI/GUI read `--config FILE`. [product.example.toml](../config/product.example.toml)
documents runtime schema 2: selected VM GUID, exact GPU interface and enabled state.
Optional VRAM triples use provider-defined units, not GiB/enforced ceilings. Normalize
GUIDs; reject duplicate VMs and unknown fields. Several VMs may select the same GPU,
subject to qualification. Parse once and pass typed values; no product config is embedded.

Discover names, disks, drivers, payloads, hashes/counts/catalogs and capabilities.
Administrator installation generates protected VM/GPU enrollment. Caller config cannot
broaden it; changing pairs requires administrator re-enrollment. Credentials belong
only in ephemeral memory or explicit per-user/per-VM Windows Credential Manager entries.

## Contributor configuration

`config/project.toml`, `runner-policy-v1.json` and `artifact-pins.toml` belong to the
standalone laboratory. Preserve its target, golden-disk and installed safeguards.
Maintained scripts use the shared laboratory reader. Build lab artifacts separately;
the production package neither imports this package nor consumes its pins.
Protocol constants, safety bounds and graphics oracles stay in code. Operation
manifests/receipts are generated data, not additional operator settings authorities.
