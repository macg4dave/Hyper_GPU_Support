# Reapply slowdown and reported host lockup — 10 October 2026

Reapply and potentially destabilising testing are stopped at the user's request.
This investigation inspected source, existing operation results and a bounded
read-only System event query. It did not rerun qualification, boot a guest,
launch graphics, build, or restart the physical host. The earlier investigation
closed by user decision remains closed; this report concerns the new incident.

## Confirmed evidence

- The user reports the whole host became unresponsive without a BSOD during
  Reapply. Kernel-Power 41 at **02:20:24 UTC** records an unclean restart with
  `BugcheckCode=0`; EventLog 6008 also records unexpected shutdown. These events
  establish an unclean restart, not its cause or the exact freeze time.
- Existing `reapply.json`, operation `50c9c68089666ba5aff806cd97e54c59`, reports
  `prepared=false`, `verified=true`, `saved=true`, and final guest power Off.
  The full driver-transfer branch was skipped. That functional result does not
  qualify host stability.
- Existing timing captures place the preceding Plan at approximately 98 seconds
  and the Reapply worker at approximately 333 seconds. Reapply result was written
  at 02:17:40 UTC; the later final status at 02:18:36 UTC reports Off, detached,
  and no pending journal recovery. These are pre-restart observations, not a
  new post-restart VM inspection.
- The System log returned no critical/error/warning entries in the bounded
  02:00–02:20 UTC window. In particular, this query found no recorded display,
  storage-reset, WHEA or resource-exhaustion warning in that interval. Missing
  events do not exclude those failures during a hard hang.
- There are no timestamped per-copy/per-discovery/per-trust stage timings or
  contemporaneous Reapply commit/available-memory/disk-latency samples. GUI
  builds and mock rendering checks also ran during the qualification sequence;
  their contribution to host load was not measured. Fresh preparation's earlier
  increasing process read-byte counters are not Reapply telemetry.

Local evidence: `local/evidence/closure-sprint/`, including `reapply.json`,
`final-status.json`, and `reapply-restart-events.xml` captured read-only afterward.
Host: Windows x64 build 26300.9457; RTX 5060 driver 32.0.16.1692. Disposable guest
build 26200.9457 x64; selected VM `2627e735-5b33-4104-b739-622727dd3a40`.

## Source findings

| Finding | Code and consequence |
|---|---|
| Four complete payload passes per enabled Reapply | `runner.rs` Plan invokes `workflow::plan`; `worker.rs` repeats that for admission, `apply_approved` repeats `decision`, and the worker checks `backend.payload` again after effects. Every `NativeBackend::payload` performs native discovery, manifest hashing and signature/catalog validation; its retained manifest is not reused by the next call. This is definite excess work, although its measured share of the 333 seconds is unknown. |
| Payload freshness is expensive even without copying | `payload.rs::discover` recursively expands associated DriverStore packages and hashes every deduplicated file. `trust.rs::validate` checks signatures, and unsigned embedded-file results lead to catalog hashing and catalog verification attempts. This adds file access beyond the manifest SHA-256 pass. The 300-second discovery budget does not cover subsequent hashing/trust work. |
| Verification still replaces guest tools | `guest_transport.ps1` deletes existing protected bootstrap leaves and copies `hyper-gpu-guest.exe` and `d3d11-probe.exe` on every mode, including verify. Installed artifacts total **2,001,408 bytes**. Those small transfers remain possible stall points, but the skipped preparation branch rules out this run copying the full 271-file driver payload. |
| Fresh preparation retransfers all staging files | Only prepare mode sequentially deletes and recopies every manifest file into the bundle. Guest Rust can skip matching final destinations, but that happens after transfer. Subsequent source, destination/partial/final hashing and `sync_all` add guest disk work. This explains why fresh preparation is much more than one local file copy; it is not the Reapply branch. |
| Copy stalls have coarse supervision | `guest.rs` gives the whole bridge 3,600 seconds, not a per-file/no-progress deadline. Copies have no byte-progress reports. `process.rs` drains bounded output into buffers and returns it at completion; it does not deliver transfer progress. Existing stage messages cannot identify where the time was spent. |
| Native provider calls have a timeout gap | `windows_driver.rs::dependent_file_name` uses synchronous `IWbemServices::GetObject` without a per-call timeout. The outer discovery deadline is checked between references, so it cannot interrupt one blocked call. The worker's 3,900-second watchdog is much coarser. |
| Process limits are not host-wide containment | `process.rs` constrains spawned child jobs to 1 GiB, four processes and 25% CPU. Payload work gets calling-thread background priority. These do not constrain external WMI providers, guest/VM activity or kernel GPU/storage driver activity. Reapply also boots the attached guest and runs the checked D3D11 probe. |

The manifest has a file-count bound but no aggregate-byte budget. Hashing uses
a 64 KiB streaming buffer, and guest publication uses streaming `io::copy`;
the reviewed paths do not establish a full-payload Rust heap allocation or an
unbounded copy-process spawning loop. This does not establish PowerShell's or
external providers' actual memory use during the incident.

## Likely explanations and limits

The strongest code explanation for unnecessary Reapply delay is repeated complete
payload discovery/authentication, combined with expensive per-file native lookups.
A remoting/provider stall, storage or antivirus contention, and the child CPU cap
could add delay. No stage timings establish which dominated this run.

The cause of the host lockup remains **unconfirmed**. GPU-PV boot/render/driver
activity remains a plausible separate failure path; severe storage or resource
pressure is also possible. Neither the small bootstrap copy nor repeated hashing
alone proves a host-level deadlock. The successful terminal result and later
unclean restart make the exact onset important; buffered progress cannot time it.

## Specific recommended fixes

1. Remove the redundant worker pre-effect payload pass by carrying its freshly
   validated admission decision into execution under the existing exclusive
   operation lock. Preserve stale-plan checks, trusted file identity/content
   checks and independent post-effect payload validation. Do not substitute a
   driver-version-only or persistent unchecked cache for payload trust.
2. Reuse bootstrap artifacts only after protected ownership/ACL/reparse and
   locked-handle hash validation. When they differ, transfer into a protected
   temporary leaf, validate, then atomically replace. Do not delete a valid
   executable before its replacement is available.
3. Add bounded stage durations and file-count/byte progress to the fixed transfer
   bridge, with a separate per-file/no-progress budget supervised by Rust.
   Interrupted transfers must retain partial/uncertain recovery state and never
   trigger an automatic Apply retry. Preserve DEC-028's narrow bridge boundary.
4. Bound associated-file resolution itself, rather than relying only on checks
   between synchronous calls. Record discovery versus hashing versus trust time
   separately before selecting a narrower native lookup optimisation. Use an
   explicit byte/time safety budget for payload processing, failing visibly
   without silently omitting required signed files.

These are recommendations, not implemented or tested fixes. No increase in CPU,
memory or timeout limits is justified by the current evidence. M2, GPU-012 and
ARCH-001 remain open. Any later qualification must address the new test hold;
this report does not authorise another reproduction.
