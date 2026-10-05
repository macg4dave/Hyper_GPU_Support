# Validated normal-Hyper-V GPU-PV baseline

Measured on 2026-10-05. This is the accepted experimental baseline for delivery,
not a fresh hardware run or proof that the complete Rust writer already exists.
Our architecture and source define the implementation; no reference repository
is needed to reproduce the recipe.

## Environment and provisioning

| Fact | Measured value |
|---|---|
| Host | Windows 11 Pro x64, build 26300.9457 |
| Guest | Windows 11 Pro x64, build 26200.9457, normal Generation 2 Hyper-V VM |
| GPU/driver | NVIDIA RTX 5060 8 GB, 616.92 / 32.0.16.1692 |
| Discovery | 236 signed-driver associations; package expansion yields 271 destinations: 217 package files and 54 external files |
| Placement | DriverStore package trees become the guest HostDriverStore mirror; other discovered Windows-relative paths retain their logical location |
| Transfer | Ordinary byte copies, independent full length/hash verification, before attached boot |
| VM profile | 3 GiB low / 32 GiB high MMIO; guest-controlled cache types; static 8 GiB RAM; four vCPU; virtualization extensions; checkpoints disabled |
| Security | Secure Boot On/MicrosoftWindows and vTPM retained; configuration version 12.0 observed |
| GPU resources | min=max=optimal: VRAM/Decode/Compute 500,000,000; Encode 9,223,372,036,854,775,808; opaque provider units |
| Guest binding | Inbox VirtualRender/vrd.inf 10.0.26100.1150; no conventional NVIDIA INF installation or registry import |

The complete [measured source/destination/length/hash inventory](GPU-PV-BASELINE-INVENTORY.tsv)
is a driver-specific fixture. Native Rust COM/WMI discovery independently matched all
271 records. File count, paths and resource values are not universal constants.
The current checked-in configuration still describes the older package manifest
and provider-default resources; CORE-022/023 integrate the complete recipe.

## Actual execution results

| Check | Observed result |
|---|---|
| Readiness | 116 consecutive Code 0 samples over approximately 120 seconds; no remaining problem devices |
| nvidia-smi | Exit 0, RTX 5060 / driver 616.92 |
| D3D11 | Exit 0, hardware GPU-PV, FL 12_1, checked 256Ã—256 frame |
| D3D12 | Exit 0, same hardware render adapter, FL 12_2, checked 256Ã—256 frame |
| CUDA | Exit 0 / Result = PASS, sm_120 vector addition: allocation, transfer, kernel and CPU comparison |
| CUDA identity diagnostic | Exit 2; physical host/CUDA LUID 00000000:00014b54 differs from guest graphics LUID 00000000:00005425 |

Both graphics outputs have SHA-256
`00f88da6c22b46ab45bfc6659e601ebcefe324d52a3302e614d4a7fb3de4`.
Graphics selection uses D3DKMT physical PCI identity when guest DXGI omits fields;
software/indirect rendering remains rejected. CUDA computation success is distinct
from cross-namespace identity correlation; no graphics/compute interop claim is made.

Probe artifacts/FATBIN and three verified app-local x64 Microsoft CRT DLLs were
staged separately. They are verification prerequisites, outside the 271-file GPU
inventory. Release packaging must handle their actual runtime/license requirements.

## Retained measurement artifacts

The original ignored local run filenames are retained as provenance, not commands
future agents should execute:
- local/evidence/easy-gpu-pv-Verify.json and its .files.json: 271 matching file hashes.
- local/evidence/easy-gpu-pv-Resources.json: fresh settings/resource readback.
- local/evidence/easy-gpu-pv-guest-baseline.json: inbox binding and sustained PnP samples.
- local/evidence/easy-gpu-pv-guest-probes.json: checked workloads (17:54:28 UTC).
- local/evidence/easy-gpu-pv-native-environment.json and easy-gpu-pv-native-comparison.json:
  native discovery at 18:16:04 UTC and complete parity at 18:17:12 UTC.

Earlier provisioning changed files and settings together. Their individual necessity
is unisolated; minimisation is post-v1. The full native manifest writer, Rust settings
integration and automated clean-child reproduction remain implementation tasks.
Historical: the initial recipe was derived during reference implementation research
and subsequently validated independently on the target system.
