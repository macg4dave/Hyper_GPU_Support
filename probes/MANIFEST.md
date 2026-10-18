# Standalone GPU probe manifest

This is historical metadata for retained shader/CUDA inputs and former standalone
probe artifacts. The values and historical outcomes have not been reverified by
the 2026-10-11 documentation reset. This file is not current qualification evidence,
a release package inventory, or a set of executable setup instructions.

The current product has [a D3D11 probe source](../src/bin/d3d11-probe.rs).
Laboratory D3D12/CUDA identity binaries, setup/build scripts, and old evidence were
deleted. Reassess them individually if a new task needs them; do not restore/run
them merely because they are named below.

## Direct3D probes

The former laboratory used `d3d11-probe`, `d3d12-probe`, and `cuda-identity`.
Only the product D3D11 probe source remains a current Cargo binary target here.
Historical executable hashes below do not identify a build of current source.

The recorded host-control build used Rust 1.94.0, Cargo 1.94.0, `windows` 0.62.2
and MSVC 19.51.36257. Its release executable hashes were:

| Artifact | SHA-256 |
|---|---|
| `d3d11-probe.exe` | `862d3c57096e9284a84d4584707cde2372131d06efba3f1733a319166b211c2d` |
| `d3d12-probe.exe` | `178282c8531e3611386c1629adea4d358e66073c959182e5c31893f7ca1a2c79` |
| `cuda-identity.exe` | `22f188017fe7b030543181417b0a6e0d76045e62da77c865fb2f5af0b6993117` |

The former shader build script was `scripts/testing/build-probe-shaders.ps1`
(now deleted). Its recorded D3D11 tool was Windows SDK
10.0.26100.0 `fxc`; D3D12 uses official DXC v1.9.2607 archive
`dxc_2026_07_29.zip`, SHA-256
`a1dfb116ba3eeae6a1582291b53a8e7bf65ad760676bd3194685c8f7367cd241`.

| Artifact | SHA-256 |
|---|---|
| `shaders/offscreen.hlsl` | `3cd81147b68ea5f887f6927557ac68e90925e275fc667f17f7e41b1b6fc842b8` |
| `shaders/compiled/d3d11-vs.dxbc` | `124b6b3a4ef68c9c1c1203f46073d4afe1cffce739413a3d2aad9de50ee7e051` |
| `shaders/compiled/d3d11-ps.dxbc` | `03bc7f25907b91ca18772da569082c1d96a79c445ec613899406e9e0015bb191` |
| `shaders/compiled/d3d12-vs.dxil` | `d0bceaeca17f417966de76628bf7bf580dcef06be911511f6d4228afb670da21` |
| `shaders/compiled/d3d12-ps.dxil` | `c8d8531bc05c2d68685afd1452934f0824760169f683a213fc8f193e9e35a54d` |

## CUDA probe

The deleted `scripts/setup/prepare-cuda-probe.ps1` recorded these archive inputs.
Their versions/hashes are historical metadata, not current acquisition advice:

| Input | Version | SHA-256 |
|---|---:|---|
| CUDA NVCC | 13.4.59 | `06a4fe6ec543030c5e5a7b85493cda90612015d9fff2a54ed0227162c937ea46` |
| CUDA CRT | 13.4.59 | `f969a0e3b086a48f940563cd1b965cb5197fd9540c79cc952b1abfa479f9ede7` |
| CUDA Runtime/headers | 13.4.49 | `e6663f3d3e8949eedc2d5ab92c7c5b9fa3f2a222086d91c42bfc5a38bf2b0225` |
| CUDA cuobjdump | 13.4.49 | `9d1aeb5a25ea4be1abb9ae34985ce9734332d686c314ce597b45d79967286a42` |
| libNVVM | 13.4.59 | `a7a07bcc21cc05bce83eedebcbcd9b9f5418c58316b0fa28a57cb2c548b29841` |
| CCCL | 13.3.4.2.1 | `ea3ebdd98d4d98819cc26b66bc5a0931004ed35ff18eea8b93e0bd2c8bfa5d7c` |
| CMake Windows x64 ZIP | 4.4.3 | `4d52ebab7193a698651639ed80d8d04fd903358843572cf44c7fd234cb7c26ab` |

The deleted `scripts/testing/build-cuda-probe.ps1` recorded NVIDIA CUDA Samples commit
`5443602d89ed99aede2e4b7bf329daddeadb320e`, tree
`532bbdd145b2a9dc49638b18eb2ab696b73ae57c`, required a clean tree, and built
the unchanged `cpp/0_Introduction/vectorAddDrv` target with CMake, MSVC and
`CMAKE_CUDA_ARCHITECTURES=120`. It retained the sample source and BSD-3-Clause
license under `cuda/upstream/`.

| Retained artifact | SHA-256 |
|---|---|
| `cuda/upstream/vectorAddDrv.cpp` | `58aee6e905e885d4decf9e3111a4e8811cab7fff30405c1909605855e9a787a5` |
| `cuda/upstream/vectorAdd_kernel.cu` | `72df5af82eebd612fc02b6e4554c6b006dabfa07c5714ff872d6f76bc8d161e9` |
| `cuda/compiled/vectorAddDrv.exe` | `180216fef76d36c89c8ac66d0d82bd31c9942015c112c39950fb8b4cb9f23b3c` |
| `cuda/compiled/vectorAdd_kernel64.fatbin` | `2ed942fddfd59e0ff8de285eb8bbc789d22d3986b01a63ce568e2cf4ea1d6953` |

## Current use and limitations

The retained CUDA sample/FATBIN is historical input, not a current product
verification requirement. The former identity helper, public laboratory `validate`
command, CRT transfer workflow, and host-control script are absent. CUDA computation,
cross-API identity matching, and old guest passes do not establish the new release's
two-guest graphics requirement.

V1-014 selects justified qualification inputs against current product source.
V1-016 assesses package contents and notices. Rehash selected transferred artifacts
during an authorized run; keep upstream licensing/provenance intact. Live hardware
execution requires its own reviewed sequence and authorization.
