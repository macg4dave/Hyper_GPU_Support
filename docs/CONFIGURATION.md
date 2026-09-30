# Project configuration

[`config/project.toml`](../config/project.toml) is the authoritative, non-secret
source for values expected to change between machines, test slots, images, driver
or tool versions, and experiment runs. Rust deserializes it once into
`ProjectConfiguration`, validates identities, hashes, paths and deadlines, then
passes typed values to application code. Maintained PowerShell scripts use the
small shared reader in `scripts/common/project-config.ps1`; command-line parameters
may override the configuration-file location, not silently duplicate its values.

## Audit result

The centralized settings cover:

- disposable slot, VM, GPU, parent/child image and driver-manifest identities;
- external data/test-output roots;
- runner installation directories, account/task identifiers and deadlines;
- CUDA/CMake/DXC/Windows SDK versions, upstream revisions, download URLs and
  expected archive hashes; and
- inventory and host-probe timeouts, repetitions and output locations.

`config/runner-policy-v1.json` and `config/artifact-pins.toml` are generated
artifacts, not additional editing surfaces. The policy contains the subset installed
outside the repository and remains byte-for-byte checked by the compiled runner;
the pin manifest avoids the impossible self-reference that would result from
embedding an executable's expected hash into that executable. Regenerate both after
a reviewed release build:

```powershell
cargo build --release --locked --bins
./scripts/setup/update-project-pins.ps1
./scripts/setup/update-project-pins.ps1 -Check
```

Use `-PolicyOnly` to regenerate/check policy drift while iterating, but rebuild and
refresh all pins before installation because Rust embeds the validated project
configuration. The normal check lane rejects policy identity/hash drift.

Implementation constants stay in code: schema/protocol spellings, fixed Windows
API and well-known SID values, frame/output safety limits, shader dimensions,
probe algorithms/expected deterministic output, and enum definitions. A value
belongs in project configuration when changing the machine, slot, image, driver,
tool input or test run would otherwise require editing Rust or several scripts.

Do not add passwords, credentials, API keys, private keys or other secrets to
`config/project.toml` or generated policy. Use ignored `local/`, environment
variables, Windows credential facilities or a runtime prompt appropriate to the
consumer. Local TOML overlays and secret-named configuration files are ignored;
the current schema deliberately has no credential field and rejects unknown keys.
