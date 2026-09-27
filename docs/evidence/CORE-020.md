# CORE-020 standalone baseline probe implementation

## Outcome

The standalone M1 probe kit is implemented and its host controls pass. No VM,
driver, Windows feature, signing configuration, installed toolkit or host
lifecycle state was changed. CUDA and CMake were assembled only under ignored
repository staging from hash-pinned official archives.

The retained build/source/hash inventory and reproducible commands are in
[`probes/MANIFEST.md`](../../probes/MANIFEST.md). The implementation comprises:

- Rust D3D11 and D3D12 offscreen probes with explicit physical RTX 5060
  selection, no software fallback, exact feature/shader reporting, full
  256x256 readback and the canonical image hash;
- a D3DKMT adapter-type check that excludes the installed Phaze indirect-display
  adapter even though it exposes the same NVIDIA description and PCI identifiers;
- a Rust CUDA Driver API identity companion that restricts DLL loading to
  System32 and requires device 0 name, compute capability 12.0, UUID and LUID to
  match the selected physical DXGI adapter; and
- the unchanged pinned NVIDIA `vectorAddDrv` source, BSD notice, `sm_120`
  FATBIN and executable.

## Hardware identity and host environment

The final control ran 2026-09-27 at `2026-09-27T08:07:45.8440237Z` in Windows session
2 on Windows 11 Pro x64 10.0.26200 build 26200. The physical adapter was:

- NVIDIA GeForce RTX 5060, driver 616.92 / `nvcuda.dll` 32.0.16.1692;
- PCI `00000000:01:00.0`, 8,151 MiB reported by `nvidia-smi`;
- DXGI/CUDA LUID `00000000:000149be`; and
- CUDA UUID `4520f8ad-0ebe-dac8-bb9f-d8813ef6955d`, compute capability 12.0,
  driver API version value 13040 and device-node mask 1.

Read-only DXGI inventory also found a Phaze virtual display adapter presenting
the RTX 5060 description and PCI IDs at LUID `00000000:000207c5`. D3DKMT marked
it `IndirectDisplayDevice`; the selector rejects it before ambiguity resolution.
Microsoft Basic Render Driver was separately reported as software. This is why
name/vendor/device matching alone is not accepted.

## Build evidence

Commands actually run successfully:

```powershell
.\scripts\setup\prepare-cuda-probe.ps1
.\scripts\testing\build-cuda-probe.ps1
.\scripts\testing\run-host-probe-controls.ps1 -SelfTest
.\scripts\testing\run-host-probe-controls.ps1
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

The CUDA build used official repository-local CUDA Toolkit 13.4 Update 1
components, nvcc 13.4.59, cuobjdump 13.4.49, CMake 4.4.3, MSVC 19.51.36257 and
Windows SDK 10.0.26100.0. CUDA Samples commit/tree and all package, source,
shader, executable and FATBIN hashes are recorded in the manifest. `cuobjdump
--dump-elf` reported all three required `sm_120` indicators.

The normal Rust quality run passed strict Clippy, 46 unit/integration tests
(41 library/binary unit tests plus 5 CLI integration tests) and one doctest,
including hardware-free tests for
success parsing, malformed/unknown output, wrong/software/indirect identity,
zero LUID, wrong image hash and stable exit classes. The CUDA companion also
tests canonical UUID and DXGI-order LUID formatting.

## Host control matrix

`scripts/testing/run-host-probe-controls.ps1` enforced 15-second D3D/identity,
30-second CUDA, 1 MiB per-process output and 90-second suite limits. A harmless
self-test first proved timeout/output-overflow termination and retained both
failure records with cleanup confirmed under Windows PowerShell 5.1. The final
run completed in 2,718 ms and retained its detailed ignored local log at
`local/evidence/CORE-020-host-controls.json`. One warm-up and all three measured
iterations passed:

| Phase | D3D11 | D3D12 | CUDA identity + workload |
|---|---|---|---|
| warm-up | FL 12_1, 169 ms, exact hash | FL 12_2 / max SM 6_8, 203 ms, exact hash | CC 12.0 / matching LUID; `Result = PASS` |
| measured 1 | FL 12_1, 163 ms, exact hash | FL 12_2 / max SM 6_8, 198 ms, exact hash | CC 12.0 / matching LUID; `Result = PASS` |
| measured 2 | FL 12_1, 157 ms, exact hash | FL 12_2 / max SM 6_8, 196 ms, exact hash | CC 12.0 / matching LUID; `Result = PASS` |
| measured 3 | FL 12_1, 158 ms, exact hash | FL 12_2 / max SM 6_8, 198 ms, exact hash | CC 12.0 / matching LUID; `Result = PASS` |

Every D3D run used LUID `00000000:000149be` and returned SHA-256
`00f88da6c22b46ab45bfc5fbc6659e601ebcefe324d52a3302e614d4a7fb3de4`
for 256x256 tightly packed opaque-magenta RGBA bytes. Every CUDA repetition
loaded `vectorAdd_kernel64.fatbin`, executed the allocation/transfers/kernel/CPU
comparison path and exited zero with `Result = PASS`.

## Validation boundary

This proves the kit on the Windows host only. It does not prove GPU-PV, guest
driver staging, reference/native guest execution, transport or presentation.
Those remain owned by GPU-004/005/006, GPU-009/010/011 and CORE-003. The host
artifacts and exact rules are now available for those targets without requiring
CUDA toolkit installation in either guest.

The required independent architecture review initially found process-boundary
and shader-model compatibility defects. After the timeout/output/failure harness,
live failure self-tests and descending D3D12 shader-model negotiation were fixed,
the re-review reported no blocking findings and accepted CORE-020 for completion.
It noted only bounded poll overshoot/cleanup time and the intentionally unvalidated
guest work above. Reviewer runtime model metadata was unavailable.
