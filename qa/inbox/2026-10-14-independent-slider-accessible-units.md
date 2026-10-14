# GPU memory slider exposes 60 under a GB label while the GUI shows 6 GB

**Date:** 14 October 2026  
**Severity:** P3, minor accessibility misinformation; visible text provides the correct value. Enabled-control impact remains untested.  
**Feature:** Dashboard GPU memory request, Windows accessibility value.  
**Status:** Confirmed observable GUI/accessibility mismatch; development handoff, no fix implemented.  
**Installed app:** `C:\Program Files\HyperGpuSupportProduct\hyper-gpu-support.exe`, About 0.1.0. Hash in the session evidence folder. Installed source revision not independently established.

## Starting state

All four VMs display Stopped. `win_26300.9457 VM1` has NVIDIA GeForce RTX 5060, a 6 GB request and a retained GPU-verification recovery hold. GPU editing controls are disabled. Initially VM2 is selected. No operation is running.

## Exact reproduction

1. Launch the installed application normally.
2. On Dashboard, select `win_26300.9457 VM1`.
3. Observe the GPU Memory request summary and numeric edit field: 6 GB / 6.
4. Inspect the same screen using Windows accessibility tooling. The slider is named `Requested GPU memory in GB` but its exposed numeric value is 60.
5. Select VM3, gracefully close and reopen the application, then select VM1 again.
6. Observe the same 6 GB / 60 discrepancy.

No value was entered, consent granted, effect invoked or hold cleared.

## Expected

A slider explicitly labelled in GB exposes a value consistent with the visible 6 GB request, or supplies an accessible value description that unambiguously communicates 6 GB.

## Actual

The Windows accessibility observation reports:

```text
text 6 GB
edit (disabled, settable, string) GPU memory request in decimal GB Value: 6
slider (disabled, settable, float) Requested GPU memory in GB Value: 60
text 1 GB
text 8 GB
```

The accessible slider value conflicts with the visible request and exceeds the displayed 8 GB endpoint. This report establishes the exposed-value mismatch, not a wrong saved allocation or a specific screen-reader announcement. Screen-reader speech, underlying cause, accessible range metadata and enabled-control editing were not tested.

## Evidence and reproducibility

Reproduced on both VM1 selections, including after app restart (2/2 observations):

- `local/evidence/qa/2026-10-14-independent/13-vm1-selected.json` and `13-vm1-selected-0.png`.
- `local/evidence/qa/2026-10-14-independent/16-vm1-slider-repeat.json` and `16-vm1-slider-repeat-0.png`.

The JSON observations retain timestamps, Windows accessibility text and window identity. Screenshots independently show the 6 GB summary. No VM state changes were requested; the session ended with all four VMs displaying Stopped, the recovery hold retained, original VM2 selection restored and app closed.

## Independent fix retest

After development installs a fix, repeat the two read-only observations above and compare the accessible units/value against the visible request. In a separately authorized session with editing available, check keyboard increments and screen-reader output without applying a VM change unless that effect is explicitly authorized.
