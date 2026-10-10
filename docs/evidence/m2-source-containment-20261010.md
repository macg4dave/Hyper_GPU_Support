# M2 source containment — 10 October 2026

**Closed with M2 on 10 October.** The user confirmed completed live tests and
working behavior, accepted the baseline as known good and directed no further
M2 live qualification. The repository quality gate subsequently passed, including
the regression tests described here. See [M2 acceptance](M2.md).

The original implementation slice below was source-only under the then-active
test hold. Its original deferred-validation statements describe that point in
time, not an outstanding task. The lockup cause remains unconfirmed.

## Implemented scope

- Retained the existing working-tree admission reuse fix: the restricted worker
  authenticates the approved plan once before effects and consumes its admission
  under the same exclusive operation lock. Fresh VM/GPU/journal checks, guest
  hashes and independent final payload authentication remain required.
- Changed associated-file lookup from synchronous `GetObject` to semisynchronous
  completion. Each requested wait is at most one second and no greater than the
  remaining discovery budget. Preserve the positive timeout HRESULT and native
  operation failures separately. Completed object retrieval requests zero wait.
- Admit the complete deduplicated payload against a 16 GiB aggregate safety
  ceiling before hashing. Expansion and streaming hashing share a separate
  300-second cooperative deadline. Reject non-files, overlarge inventories,
  arithmetic overflow and length changes while hashing; never omit files to fit.
  These ceilings are fixed safety limits, not a driver inventory contract.
- Copy bootstrap executables into unique protected staged leaves, validate their
  hashes while holding write/delete-denying handles, then publish by file
  replacement or move. Replacement supplies a unique backup; remove it only
  after protected ACL application and successful locked-handle published hash
  validation. Failed copies or staged hashes retain the old executable. Publication
  failure remains uncertain and can leave staging/backup artifacts for explicit
  recovery; no automatic rollback or retry occurs. Existing final locked-handle
  validation before worker launch remains.

The bootstrap change stays within DEC-028's approved integrity/transfer bridge.
Rust still owns payload membership, destination mapping, preparation and receipts.
There is no persistent unchecked payload or executable cache.

### Bounded transfer follow-up

Completed CORE-028.2 source on 10 October. The fixed bridge transports bootstrap
and complete payload files in at most 1 MiB chunks. Each remote stream write/flush
returns its position; the host bridge emits numeric begin/bytes/end/done records
without credentials or paths. Rust validates file order and exact expected sizes,
rejecting duplicate, skipped, oversized, incomplete and malformed acknowledgements.
Zero-length files require explicit begin/end records. Acknowledgements establish
transport progress only; existing locked bootstrap hashes, complete guest payload
hashes and receipt identity checks independently authenticate success.

Rust monitors 300-second per-file, 60-second no-byte-progress and 240-second
pre-transfer/inter-file budgets. Existing worker progress IPC may spend 15 seconds
in a send and 5 seconds draining cancellation; local kill-job termination may lag
these budgets by that bounded 20 seconds, plus normal scheduling/OS cleanup. The
3,600-second whole-bridge budget has the same progress-delivery caveat. A blocking
filesystem/remoting call produces no acknowledgement and cannot reset the budgets.
After all files and manifest/ACL transport complete, independent guest verification
remains under the existing outer/guest-worker deadlines.

Reader-thread progress is coalesced into one pending snapshot; caller-thread
delivery uses existing authenticated worker IPC. The existing GUI progress list
updates one transfer row and retains later terminal stages within its 256-row bound.
Totals include bootstrap artifacts as well as driver files. Partial copies, uncertain
publication and terminated local sessions retain existing pending/recovery state.
Killing the local bridge does not establish cancellation of guest work. No retries,
rollback, new caches or payload omissions were added.

Independent static review found the bounded progress-IPC timing caveat above and no
other blocking correctness/security/recovery issue. The supervisor rechecks budgets
after progress delivery; timing limits are explicit. Regression test source covers
ordering, byte totals, zero-length files, stalls, immutable file budgets, incomplete
success and monitored process output. Tests, builds and GUI/live checks remain unrun.

Independent static architecture/security review found no blocking issue in
admission reuse, WMI completion, payload limits or the final backup amendment.
Review identified a native replacement failure caveat; the final change supplies
a backup and retains publication uncertainty. Review does not substitute for
compilation, tests or interrupted guest qualification.

## Technical limits

Requested WMI waits are bounded; COM dispatch, connection and provider cancellation
are not established as hard deadlines. The payload deadline is checked around
filesystem operations and reads; it cannot interrupt a blocked kernel read or
native trust call. Trust validation remains outside the new hashing deadline.
Existing process supervision remains the outer bound. Neither limit contains
Hyper-V, WMI-provider or kernel GPU activity across the host.

Bootstrap publication is per executable, not an atomic pair update. A failure
between publications prevents launch and retains uncertain/pending operation
state. Bootstrap files are still copied each time; verified reuse has not been
introduced. Full driver transfer still uses the existing bundle path, now with
monitored per-file/stall budgets in addition to the whole-bridge deadline.

M2 closure is recorded in [M2 acceptance](M2.md). The former live preparation,
maintenance and interruption qualification checklist is retired by the user's
confirmation and closure instruction. No further M2 live tests are required.

Existing functional results in [M2](M2.md) and the new incident's
[investigation](reapply-investigation-20261010.md) remain evidence. No measured
speedup, new hardware pass or host-stability resolution is claimed.
