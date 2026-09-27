# CORE-004 configuration and CLI contracts

## Implemented contract

The project now has a dependency-free version-one `key=value` configuration in
`src/config.rs`. It identifies exactly one canonical Hyper-V VM GUID, one explicit
GPU-P interface and one immutable runtime-manifest identity/SHA-256. Unknown and
duplicate fields fail closed, so credentials, scripts, paths and default-GPU
selection cannot be added accidentally.

Resource requests are independently represented for VRAM, encode, decode and
compute as either `provider-default` or an exact
`minimum,maximum,optimal` triple. The values remain opaque provider-defined units;
the contract does not convert them to percentages or physical VRAM. Ranges require
`minimum <= optimal <= maximum` and a nonzero maximum. Omitted resource fields
default to `provider-default` and canonical rendering makes those defaults explicit.

The same module defines:

- the stable `inventory`, `plan`, `apply`, `status`, `validate`, `remove`,
  `recover`, `start`, `shutdown` and `restart` operation inventory;
- immutable configuration/environment plan fingerprints;
- explicit succeeded/failed/blocked/untested report outcomes; and
- stable error categories with exit codes 2, 3, 4, 5, 6 and 70.

The CLI help declares every operation. Only `inventory` is implemented at this
stage; every other declared command returns exit code 70 and a clear error instead
of reporting success. `examples/gpu-pv-v1.conf` demonstrates the format with the
measured VM/GPU identities and provider defaults. Its all-zero manifest hash is an
intentional placeholder, not an apply-ready manifest; CORE-009 must replace it with
the immutable generated manifest hash.

## Validation

The hardware-independent suite covers canonical round trips, default resources,
the host's observed provider ranges including `u64::MAX` encode values, invalid
ranges, ambiguous/default GPU selection, noncanonical VM IDs, unknown credential
fields, duplicates, unsupported schemas, operation names, plan fingerprints,
report outcome consistency and executable exit behavior.

No host, Hyper-V, VM, GPU, driver, runner installation or guest state was changed.
The maintained Rust check passed formatting, strict locked Clippy, 28 library,
2 runner-binary, 2 main-binary, 5 integration and 1 doc test, locked build and
rustdoc with warnings denied. The documentation check passed 32 Markdown files
and `git diff --check`.

The independent architecture review initially found that the GPU identity check
accepted wildcard-shaped selectors and that CORE-005's freshness helper did not
bind the requested operation. The final patch validates the complete supported
PCI interface structure, exact GPU-P class and safe instance characters, and adds
wildcard, whitespace, path-injection and malformed-selector regressions. It also
binds runner authorization to the trusted operation before nonce consumption.
The re-review closed both findings and approved CORE-004 completion. Runtime model
metadata was unavailable; the configured reviewer role alone is not recorded as
proof of a specific model.
