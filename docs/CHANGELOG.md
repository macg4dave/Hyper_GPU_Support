# Changelog

## 2026-10-06 - Runner activity supervision

- CORE-022 live fresh apply and verified no-op reapply passed on a clean child: complete 271-file environment through PowerShell Direct. Settings and workload integration remain separate tasks.

- Inspection/reset now use Rust read-activity supervision instead of a single elapsed-time cutoff, with a finite task budget and reserved transition/publication time. Client waits cover that outer budget.
- Failure cleanup reaps the adapter and drains bounded output; watchdog diagnostics include phase and read activity. Independent hash/readback and reconciliation requirements remain authoritative.
- Interactive staging asks for guest credentials before the lengthy native driver scan, so the local test window displays its prompt immediately.

## 2026-10-05 - Complete Rust driver environment writer

- `hyper-gpu-stage` now discovers and stages the complete native manifest instead of the older package/CUDA-alias subset. Fresh staging requires an unattached guest; reapply verifies the full manifest receipt and every guest length/hash.
- Reused protected target, signature, servicing, credential and session boundaries; partial or interrupted writes retain the recovery lock. Added focused source/receipt and real local-copy tests, plus mapping coverage for the 271-file baseline.
- Independent review found and closed a volume-root preflight defect. Live apply/reapply was subsequently qualified on 2026-10-06; settings and workload integration remain separate tasks.

## 2026-10-05 â€” Engineering delivery plan

- Rebuilt M1â€“M3 around Rust reproduction, usable operation/maintenance and packaged delivery. Added CORE-022 full writer and CORE-023 settings; moved automated probes into M1. Retained GPU-006 as clean-child integration acceptance.
- Removed GPU-004, merged duplicate audit/CI/recovery/release cards, and moved resource/driver-transition research outside the v1 gate. Closed BLK-003 with the discarded comparison path and corrected CORE-002's stale result.
- Made the measured [full recipe and inventory](evidence/GPU-PV-BASELINE.md) self-contained; removed active reference-project links/lookup instructions and condensed obsolete documents/decision history. Simplified all agent prompts.
- Preserved target/parent/runner and physical-host lifecycle protections. This change edits documentation and source comments only; no new implementation or hardware run.

## 2026-10-05 â€” Complete working GPU-PV baseline

- GPU-009/GPU-005 established sustained Code 0, nvidia-smi, checked D3D11/D3D12 and CUDA on the complete provisioning/settings recipe.
- Native Rust discovery matched the full measured manifest. Full writer/settings integration and repeatable product automation remain delivery work.

Earlier completed changes are retained in the [historical changelog](evidence/CHANGELOG-HISTORY.md); read it only for a specific historical result.
