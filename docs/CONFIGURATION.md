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
- pinned guest identity, staging root and PowerShell Direct deadlines;
- CUDA/CMake/DXC/Windows SDK versions, the Visual Studio developer-shell path,
  upstream revisions, download URLs and expected archive hashes; and
- inventory and host-probe timeouts, repetitions and output locations.

The 2026-10-04 Rust/script audit applies this classification:

| Value class | Ownership | Audit disposition |
|---|---|---|
| Windows directory, inbox executables and system module roots | Discover | PowerShell, Task Scheduler and the Hyper-V module root now derive from the OS-reported Windows directory; maintained scripts use Windows/PowerShell environment discovery. |
| Selected VM/GPU and guest identity | Configure intent, discover and verify state | The slot VM/GPU pins and guest mismatch checks remain centralized. CORE-021 will simplify selector authority where the privileged enrollment no longer requires an exact safety pin. |
| VHDX, external data, runner, staging and evidence roots | Configure one root, derive children | Current roots are centralized and scripts construct subordinate paths. Known-folder derivation remains CORE-021 follow-up where it does not weaken installed-runner policy. |
| Driver package/version and integrity hashes | Configure/manifest, verify discovered state | Exact reviewed staging inputs remain pins; CORE-021 may discover the active package location from the selected GPU while retaining version, signature and digest verification. |
| CUDA/CMake/DXC/upstream inputs | Configure pins, derive extracted tools | Versions, URLs, revisions and hashes are centralized; extracted executable paths derive from configured staging roots. Remaining `PATH`-resolved development tools are bounded follow-up. |
| Deadlines and retries | Configure only environment-sensitive bounds | Hardware/process deadlines remain centralized; short polling intervals and fixed safety ceilings remain implementation constants. |
| Temporary paths and deterministic probe output | Derive/implementation constant | Rust uses temporary-directory APIs and repository output roots; deterministic image hashes are test oracles, not environment state. |
| Unit-test VM/GUID/path literals | Test fixture | Allowed when isolated from hardware/integration selection; process-launch tests now discover Windows PowerShell. |

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

The inbox Windows PowerShell executable and its protected module directory are
derived from the Windows installation directory reported by the operating system;
they are not configuration because they are reliable host inventory. The `[guest]`
table pins the disposable guest's computer name and MachineGuid as defense-in-depth
checks after selecting the authoritative `slot.vm_id`, plus the protected staging
root and separate session, single-file transfer and whole-manifest staging deadlines.
Guest credentials are acquired at execution
time and never belong in this file. CORE-008 accepts one flat filename relative to
`guest.staging_root`; an absolute/nested path, parent traversal, reparse traversal,
broad write ACL, existing destination or interrupted partial file fails closed.

The `[driver_manifest]` table pins the GPU-correlated host package path, INF and
catalog names, INF/active-driver versions, host-build qualification baseline,
complete file/byte extent, catalog hash, required Authenticode files/signer,
canonical package-tree digest and encoded manifest identity. The measured host and
guest builds are recorded at apply time; a host/guest difference or a difference
from `host_build` is a visible qualification warning rather than an automatic
failure. Driver/package identity, signatures and stable servicing state remain hard
gates. Update the manifest pins together after
reviewed driver servicing; CORE-009 rejects partial, changed or incorrectly signed
trees before guest mutation.

`discovery_timeout_seconds` separately bounds native WMI enumeration of the full
signed-driver association closure (1 to 300 seconds). This is a read-only discovery
deadline, distinct from the short product inventory and guest-copy deadlines.
COM connection/query setup and individual provider object resolution are synchronous
and cannot be cancelled by this enumeration deadline.

The apply preflight distinguishes active/pending replacement servicing from exact
reviewed delete-only cleanup. `allowed_pending_delete_sources` pins the raw Windows
source records observed and reviewed for the current machine; it is not a path pattern
or general allowlist. CBS, Windows Update, Installer/setup state, malformed pending
rename data, any pending destination, and any unlisted deletion remain hard failures.
Matched source/empty-destination pairs are returned in the receipt and surfaced as a
qualification warning; the application never clears or edits Windows servicing state.
