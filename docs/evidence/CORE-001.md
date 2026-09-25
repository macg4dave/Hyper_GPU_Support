# CORE-001 Rust inventory

## Result

The Rust CLI/library now exposes `hyper-gpu-support inventory`, a read-only
Windows inventory command. It reports host edition/version/build/architecture,
the Hyper-V module, the selected RTX 5060 PCI/driver identity, GPU-P interface
availability and registered-VM selection state. The observed non-elevated target
report correctly classified the GPU-P interface and VM enumeration as `denied`
while returning the host and driver facts established by HV-001.

The report begins with `inventory.schema=1` and emits sorted
`key=status[:value]` records. Status is one of `known`, `missing`, `denied` or
`unavailable`; a successful command can contain any of them. Record delimiters
and percent signs in values are escaped. Adapter launch, timeout, output-limit,
exit and protocol failures return process exit code 1 with bounded, sanitized
diagnostics.

## Boundaries and introduced paths

- `src/inventory.rs`: typed facts/statuses, deterministic report rendering,
  validated UTF-8-hex adapter protocol, replaceable `InventorySource` trait and
  hardware-independent behavior tests.
- `src/windows_inventory.rs`: Windows-only adapter invoking one fixed,
  parameter-free PowerShell query set. It accepts no user script or target and
  contains no mutation cmdlet. Rust enforces a 15-second deadline, kills/reaps on
  timeout and bounds stdout/stderr to 64 KiB each. Values are hex encoded across
  the process boundary; Rust validates them, correlates the exact RTX PCI identity
  to its GPU-P interface and performs unambiguous VM selection.
- `src/cli.rs`, `src/main.rs`: `inventory` parsing, dispatch and exit behavior.
- `tests/cli.rs`: executable inventory smoke test that accepts missing/denied
  facilities and therefore remains valid on hosted Windows without Hyper-V/GPU.
- `README.md`: command, schema, status and source-boundary documentation.

The Rust-native registry/WMI/COM alternatives and their cancellation limitation
were evaluated in [DEC-013](../DECISIONS.md#dec-013). The bounded child is retained
only as query transport for installed Windows/CIM and Hyper-V facilities;
selection, policy, error semantics and reporting remain Rust. Process access is
confined behind the Rust trait and structured protocol so direct Windows bindings
can replace it without changing report logic. No dependency, feature, unsafe code,
GUI, daemon, configuration schema or mutating operation was introduced.

## Target observation

Run unelevated on Windows 11 Pro 25H2 build `26200.9457`, x64, project revision
`60fe3d1af909a0f0c744fba7996b90345e6ca9e2`:

```text
inventory.schema=1
gpu.driverinf=known:oem59.inf
gpu.driverversion=known:32.0.16.1692
gpu.model=known:NVIDIA GeForce RTX 5060
gpu.pciid=known:VEN_10DE&DEV_2D05&SUBSYS_8A151043&REV_A1
gpup.interface=denied:access.denied
host.architecture=known:X64
host.build=known:26200.9457
host.edition=known:Professional
host.version=known:25H2
hyperv.moduleversion=known:2.0.0.0
vm.count=denied:access.denied
vm.selection=denied:access.denied
```

This is inventory evidence only. It does not prove a guest, GPU assignment or
workload capability. HV-001's separately approved administrator query found zero
registered VMs; the normal CLI run intentionally did not reuse that authorization.

## Verification

On Windows 11 Pro 25H2 build `26200.9457` x64, the final working tree on base
revision `1f21a5d3881415befe5cc307e74f9193fec4fcea` passed:

- `cargo fmt --all -- --check`;
- strict locked Clippy across workspace/all targets/all features;
- locked tests: 16 library, 2 binary, 4 executable integration and 1 doc test;
- locked workspace build and rustdoc with warnings denied;
- real unelevated `inventory` launch, returning exact host/RTX facts and structured
  `denied:access.denied` GPU-P/VM results;
- Git whitespace and targeted Markdown/task/link consistency checks.

The independent architecture reviewer initially identified unbounded process
execution, an undocumented query-transport choice, uncorrelated GPU-P identity,
missing-facility/error-classification gaps and missing deterministic branches.
After fixes, a second pass found a noncanonical-index panic; canonical validation
and the requested single-VM/no-match fixtures resolved it. The final recheck found
no blocking CORE-001 issue and independently passed all eight adapter tests plus
`git diff --check`. Reviewer runtime model metadata was unavailable.

Positive administrator VM/GPU-P behavior and guest workloads remain unverified;
there is no registered VM. DEC-013 requires the query transport to be revisited
after HV-003. No host/guest mutation or elevated project binary execution occurred.
