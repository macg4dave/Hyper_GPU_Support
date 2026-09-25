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
and percent signs in values are escaped. Adapter launch, exit and protocol
failures return process exit code 1 without echoing raw command output.

## Boundaries and introduced paths

- `src/inventory.rs`: typed facts/statuses, deterministic report rendering,
  validated UTF-8-hex adapter protocol, replaceable `InventorySource` trait and
  hardware-independent behavior tests.
- `src/windows_inventory.rs`: Windows-only adapter invoking one fixed,
  parameter-free PowerShell query set. It accepts no user script or target and
  contains no mutation cmdlet. Values are hex encoded across the process boundary
  before Rust validates and renders them.
- `src/cli.rs`, `src/main.rs`: `inventory` parsing, dispatch and exit behavior.
- `tests/cli.rs`: executable inventory smoke test that accepts missing/denied
  facilities and therefore remains valid on hosted Windows without Hyper-V/GPU.
- `README.md`: command, schema, status and source-boundary documentation.

The adapter uses the installed Windows PowerShell/CIM and Hyper-V facilities
because Hyper-V's GPU-P cmdlets are the installed client interface measured by
HV-001. Process access is confined behind the Rust trait and structured protocol;
future direct Windows bindings can replace it without changing report logic.
No dependency, feature, unsafe code, GUI, daemon, configuration schema or mutating
operation was introduced.

## Target observation

Run unelevated on Windows 11 Pro 25H2 build `26200.9457`, x64, project revision
`60fe3d1af909a0f0c744fba7996b90345e6ca9e2`:

```text
inventory.schema=1
gpu.driverinf=known:oem59.inf
gpu.driverversion=known:32.0.16.1692
gpu.model=known:NVIDIA GeForce RTX 5060
gpu.pciid=known:VEN_10DE&DEV_2D05&SUBSYS_8A151043&REV_A1
gpup.interface=denied
host.architecture=known:X64
host.build=known:26200.9457
host.edition=known:Professional
host.version=known:25H2
hyperv.moduleversion=known:2.0.0.0
vm.count=denied
vm.selection=denied
```

This is inventory evidence only. It does not prove a guest, GPU assignment or
workload capability. HV-001's separately approved administrator query found zero
registered VMs; the normal CLI run intentionally did not reuse that authorization.

## Verification

Verification commands and final outcomes are recorded on the
[CORE-001 task card](../BACKLOG.md#core-001). No host/guest mutation or elevated
project binary execution occurred.
