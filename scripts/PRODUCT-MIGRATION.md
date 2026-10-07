# Product and laboratory boundary audit

Rebased 2026-10-07 under [DEC-028](../docs/DECISIONS.md#dec-028).
The old application has moved intact to [tools/lab](../tools/lab/README.md).
Historical port results remain in Git and the backlog; they do not qualify new
runtime enrollment or workflows. The root Cargo workspace builds only the product.

| Source | Responsibility / classification |
|---|---|
| `src/model.rs`, `main.rs` | Runtime target intent and thin CLI; no golden disks, package pins or SDK paths |
| `src/workflow.rs` | Shared enable/disable/verification, attributable settings and durable interruption recovery |
| `src/windows_hyperv.rs`, `windows_wmi.rs`, `windows_com.rs` | Native existing-VM discovery, power, GPU assignment and settings/readback |
| `src/windows_driver.rs`, `payload.rs`, `trust.rs` | Dynamic signed-driver discovery, safe mapping, manifests and native signature/catalog-member verification |
| `src/guest.rs`, `bin/hyper-gpu-guest.rs` | Rust transfer verification, guest preparation, device health and checked graphics launch |
| `src/guest_transport.ps1` | Fixed inbox PowerShell Direct session/transfer/launch and protected bootstrap only; DEC-028's explicit bounded exception |
| `src/runner.rs`, `windows_pipe.rs`, `security.rs`, `process.rs` | Native install/enrollment, authenticated bounded requests, serialization, protected state and process supervision |
| `src/credentials.rs`, `windows_gui.rs` | Per-user optional Windows Credential Manager, native controls calling the same core |
| `src/bin/d3d11-probe.rs`, `windows_probe.rs`, `probe.rs` | One checked hardware graphics workload and runtime identity |
| `tools/lab/src`, `tools/lab/tests` | Preserved original application, historical experiments, optional extended diagnostics and regression fixtures |
| `tools/test-harness/reset_*.rs` | Lab-only reset fragments; never imported by root product |
| `scripts/setup/install-runner-v1.ps1`, `restore-runner-v1.ps1`, `update-project-pins.ps1` | Existing laboratory runner maintenance; product install/enrollment is Rust |
| `scripts/common/project-config.ps1` | Contributor schema-1 reader only; product deserializes schema 2 in Rust |
| `scripts/testing/check.ps1`, `check-docs.ps1` | Product quality and documentation gates, independent of lab package/VM pins |
| `scripts/testing/check-project-config.ps1` | Optional laboratory configuration drift check |
| CUDA preparation/build, host controls, clean-child qualification and `scripts/diagnostics/` | Contributor setup, qualification or optional diagnosis |

No product reset protocol, golden-parent guard, CUDA alias writer, mandatory CUDA
probe or package-only staging remains in the root graph. The preserved lab is not
a product backend. No general virtualization framework or active HCS replacement
has been introduced.

Build the lab using its own manifest and `--target-dir local/lab-target`; never
re-pin/install its runner from root product artifacts. Its schema-1 artifact pins
must be regenerated against the separate lab build before updating that runner.
The currently installed laboratory runner is not replaced by product installation.

The changed privilege boundary requires independent review and affected hardware
qualification. Compilation and historical GPU results do not complete ARCH-001.
