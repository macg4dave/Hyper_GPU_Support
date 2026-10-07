# Disposable VM development helpers

These Rust fragments preserve the fixed runner's disposable-child reset, disk
creation and protected audit/reconciliation workflow. They compile only with the
explicit, non-default `dev-harness` feature. Default builds deny `reset-slot`,
do not parse `native-reset` and contain no disk reset effects. The reserved
protocol enum remains for compatibility; it grants no permission.

```powershell
cargo build --manifest-path tools/lab/Cargo.toml --target-dir local/lab-target --locked --release --features dev-harness
```

Use the existing reviewed pin/install workflow before running
[`run-clean-child-qualification.ps1`](../../scripts/testing/run-clean-child-qualification.ps1).
Installation validates artifact hashes and enrolled VM/GPU/disk pins. Never
package a `dev-harness` artifact or use an all-features build as a release candidate.
Inclusion in the existing runner preserves its fixed-token, lock, identity and
preimage boundaries without introducing a new privileged API.

The golden master -> disposable child -> apply product -> verify -> discard/rebuild
workflow belongs to contributors. Users bring an existing VM. Remaining golden
configuration/enrollment coupling is tracked under CORE-021/027; product recovery
under CORE-010 must preserve user disks.

The shared native ancestor guard now requests directory read access to enforce
rename exclusion. Unit tests verify the handle lifetime; access under the enrolled
runner token still needs qualification before deploying that change.
