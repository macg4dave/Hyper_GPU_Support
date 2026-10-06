# Project configuration

[`config/project.toml`](../config/project.toml) is the sole hand-edited, non-secret
source for mutable machine, VM/image, driver/tool and test values. Rust validates
it into typed `ProjectConfiguration` once at the boundary. Maintained scripts use
`scripts/common/project-config.ps1`; script parameters may choose a configuration
file instead of establishing competing defaults.

## Ownership

| Value | Authority |
|---|---|
| Configured VM/GPU, disposable image chain and guest identity | TOML intent/enrollment; discover and verify actual Windows state before effects |
| Windows directory, inbox executables and system modules | Discover through reliable OS facilities |
| Active driver service/INF/package and associated files | Native discovery; verify reviewed driver/signature/hash pins |
| Data, child, runner, staging and result roots | Configure roots; derive subordinate paths and enforce containment |
| Resource requests, recipe settings and operation deadlines | Typed TOML operator/test intent |
| Tool/sample versions, URLs, revisions and archive hashes | TOML pins; derive extracted executable paths |
| Protocol/API constants, fixed safety ceilings and probe correctness oracles | Implementation code |
| Passwords, credentials and keys | Runtime credential facilities or prompts; never repository configuration |

A mutable value belongs in configuration when changing the machine, target,
image, driver, tool or run would otherwise require edits to Rust or several scripts.
Unit-test identities are isolated fixtures, not hardware selection.

## Implemented schema and remaining integration

The current TOML schema owns slot/VM/GPU/image pins, package integrity, external
data/output roots, runner installation/enrollment settings, guest identity/staging
deadlines, inventory/discovery deadlines and probe tool/run inputs. It rejects
unknown keys/schema versions and unsafe or ambiguous values. Development binaries
currently embed the validated document; runtime operator file selection and concise
schema/help are [CORE-021](BACKLOG.md#core-021), rather than an already supported CLI
flag.

Current `[driver_manifest]` pins the older package manifest: source package,
driver/INF/catalog/signature identity, package file/byte extent, tree digest and
encoded package-manifest digest. Native discovery additionally produces the
complete environment manifest, including external Windows destinations. The current
package digest/count must not be described as the full environment identity.
The complete writer derives its separate receipt digest from fresh native discovery
and the full encoded environment. [CORE-022](BACKLOG.md#core-022) implements bounded
guest writing, receipts and apply verification, qualified by live fresh apply and
matching reapply on a clean child. The package pins continue to guard the selected service package.

The configured package count/bytes/digests are observed integrity pins for the
currently qualified development driver, not universal payload requirements or
operator intent about how many files a GPU needs. Discovery has no fixed total
destination count. Current package-pin validation deliberately rejects driver
drift; automatic regeneration is not implemented. CORE-015 must regenerate these
integrity artifacts from the newly discovered signed host environment, compare
with the prior receipt, restage and requalify. An updated driver must not be made
to match old file counts, paths or hashes, nor may drift checks simply be disabled.

Current `[resources]` fields accept `provider-default` or exact
`minimum,maximum,optimal` triples in opaque provider units. The checked-in
development configuration requests the validated explicit triples. The fixed
`configure-slot` operation requires explicit values, checks freshly observed provider
ranges, and applies them with the typed `[vm_profile]` memory, processor, MMIO,
cache, virtualization and checkpoint/stop settings. These settings are pinned in
the installed policy; a configuration change requires a reviewed rebuild and runner
update. Fresh independent readback determines success and matching reapply. The
[architecture recipe](ARCHITECTURE.md#hyper-v-settings-and-gpu-partition-resources)
and [measured baseline](evidence/GPU-PV-BASELINE.md) define the target behavior;
mutable environment values are changed in TOML, not copied into future code/prompts.

`driver_manifest.discovery_timeout_seconds` bounds full native association
enumeration separately from short product inventory and guest transfers. COM
connection and individual object resolution remain synchronous and are not
cancelled by that enumeration deadline.

`[validation]` owns the guest readiness window, sampling interval, process and
independent worker deadlines, expected NVIDIA query version, retained CUDA input
directory and operator-owned x64 CRT source root. Rust validates these bounds once.
The nine flat validation filenames are a fixed software allowlist; fresh host
lengths/hashes authorize transfer, and the guest verifies every input before and
after workloads. Shaders are embedded in the D3D binaries. The worker starts no
caller-selected program or command, and its hard watchdog remains active if the
host/session disconnects. Public `validate` requires an elevated development token
until the final public operation integration; credentials stay in local prompts.

Runner inspection and reset limits are inactivity watchdogs: native process read
byte activity restarts the idle clock, allowing parent verification to finish
under slow storage. This is activity evidence, not an integrity or success claim;
the hash and native readback still decide success. Each adapter also has a finite
budget derived from `runner.task_execution_timeout_seconds`, reserving 30 seconds
for result publication and the configured transition limit when one follows.
Start, shutdown and GPU changes retain their fixed transition limits. The client
waits for the outer task budget plus transport allowance. Failures report adapter
phase, elapsed time, last read activity and bytes; uncertain effects retain the
reconciliation marker.

## Integrity, build drift and servicing

Discover and check target, sources and destination state immediately before
mutation. Keep observed inventory, configured intent and last successful validation
distinct. Future maintenance regenerates the full manifest from the selected signed
host driver and verifies the guest after restaging; a stale receipt does not prove
current readiness.

The configured host build is a qualification baseline. Measured host/guest build
differences are reported as warnings, not automatic failures. Driver/package
identity, signatures and stable servicing remain mandatory. Active servicing or
pending rename/replacement fails before staging.

`allowed_pending_delete_sources` contains exact reviewed raw Windows source
records for delete-only cleanup, not patterns or a general allowlist. Only matching
source/empty-destination pairs may be reported as cleanup warnings. Unknown
deletions, malformed data or any pending destination fail; the project never clears
Windows servicing state to pass a check.

## Generated policy and artifact pins

`config/runner-policy-v1.json` and `config/artifact-pins.toml` are generated
artifacts, not extra settings. The installed administrator-owned policy pins the
privileged subset and is checked byte-for-byte. Executable hashes live outside the
executable to avoid self-reference.

After a release build intended for installation:

```powershell
cargo build --release --locked --bins
.\scripts\setup\update-project-pins.ps1
.\scripts\setup\update-project-pins.ps1 -Check
```

`-PolicyOnly` regenerates/checks policy while iterating. Rebuild and refresh all
pins before installation because the binaries embed configuration. Normal checks
reject generated-policy drift. Installing/updating the bounded runner is authorized
development; physical-host lifecycle retains its explicit permission boundary.

Guest credentials are acquired at runtime and omitted from command lines, reports
and errors. Local configuration overlays remain ignored; the current schema has
no credential fields. Configuration-file convenience never expands the installed
privileged runner's enrolled target or allowed operation surface.
