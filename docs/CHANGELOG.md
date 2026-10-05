# Changelog

## 2026-10-05 â€” Engineering delivery plan

- Rebuilt M1â€“M3 around Rust reproduction, usable operation/maintenance and packaged delivery. Added CORE-022 full writer and CORE-023 settings; moved automated probes into M1. Retained GPU-006 as clean-child integration acceptance.
- Removed GPU-004, merged duplicate audit/CI/recovery/release cards, and moved resource/driver-transition research outside the v1 gate. Closed BLK-003 with the discarded comparison path and corrected CORE-002's stale result.
- Made the measured [full recipe and inventory](evidence/GPU-PV-BASELINE.md) self-contained; removed active reference-project links/lookup instructions and condensed obsolete documents/decision history. Simplified all agent prompts.
- Preserved target/parent/runner and physical-host lifecycle protections. This change edits documentation and source comments only; no new implementation or hardware run.

## 2026-10-05 â€” Complete working GPU-PV baseline

- GPU-009/GPU-005 established sustained Code 0, nvidia-smi, checked D3D11/D3D12 and CUDA on the complete provisioning/settings recipe.
- Native Rust discovery matched the full measured manifest. Full writer/settings integration and repeatable product automation remain delivery work.

Earlier completed changes are retained in the [historical changelog](evidence/CHANGELOG-HISTORY.md); read it only for a specific historical result.
