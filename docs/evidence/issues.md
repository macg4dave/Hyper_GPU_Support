# Open issue tracker

Updated: 11 October 2026. This replaces BUGS-001.md and BUGS-002.md. Scope is their
remaining defects and acceptance gaps; other product work stays in [BACKLOG](../BACKLOG.md).
Legacy references below preserve the original report/bug IDs without linking to
deleted files. Owners are existing backlog workstreams; no individual is assigned.

Keep these IDs stable. Update status, evidence and next action on the owning row/card
as work progresses. Close a defect only after its acceptance criteria pass; code
completion alone moves it to pending acceptance. Validation gaps do not assert that
the reported defect still reproduces. P1 is the next implementation priority; P2 is
remaining scoped acceptance. ROADMAP still controls milestone and release gates.

## Active issues

| ID | Priority | Issue | Type / status | Owner | Legacy references |
|---|---|---|---|---|---|
| ISS-001 | P1 | Physical VRAM versus provider request range | Source implemented; above-range mapping unresolved; acceptance pending | [GPU-010.3](../BACKLOG.md#gpu-0103--gpu-memory-slider) | BUGS-002/BUG-02 |
| ISS-002 | P1 | Useful, credential-safe component errors | Source implemented / acceptance pending | GUI-002, CORE-028 | BUGS-002/BUG-10 |
| ISS-003 | P2 | Explicit allocation draft, Review, write and readback | Acceptance pending | GPU-010.2–3, GUI-002.3 | BUGS-001/BUG-06; BUGS-002/BUG-02–03 |
| ISS-004 | P2 | Installed enrollment dialog acceptance | Acceptance pending | SEC-001.1–2, GUI-002.1, GUI-003 | BUGS-002/BUG-04–05; BUGS-001/BUG-05 |
| ISS-005 | P2 | Native credential dialog visual acceptance | Acceptance pending | GUI-002, GUI-003 | BUGS-002/BUG-07 |
| ISS-006 | P2 | Idle refresh after external VM power changes | Acceptance pending | GUI-002.2 | BUGS-001/BUG-09 |
| ISS-007 | P2 | Apply failures and interrupted recovery boundaries | Acceptance pending | CORE-028, GUI-002, GUI-003 | BUGS-002/BUG-09–10 |
| ISS-008 | P1 / report order 1 | Apply progress overlaps the information header | Source implemented / acceptance pending | GUI-002, GUI-003 | Apply & Main Window/BUG-01 |
| ISS-009 | P1 / report order 2 | Duplicate Transfer acknowledged progress message | Source implemented / acceptance pending | GUI-002 | Apply & Main Window/BUG-02 |
| ISS-010 | P1 / report order 3 | Progress does not follow new messages automatically | Source implemented / acceptance pending | GUI-002, GUI-003 | Apply & Main Window/BUG-03 |
| ISS-011 | P1 / report order 4 | Technical Details becomes inaccessible as progress grows | Source implemented / acceptance pending | GUI-002, GUI-003 | Apply & Main Window/BUG-04 |

### Source work — 11 October 2026

- **ISS-008–011:** the information header, bounded scrollable progress, independently
  scrollable Technical Details and fixed footer now have separate layout areas.
  Progress follows content/viewport changes while at the bottom; user scrolling
  pauses following until returning to the bottom. The duplication came from
  displaying each received event in both the header and list. Updates now append
  once to the list; transfer text coalescing was removed, preserving distinct events.
- **ISS-001:** existing native DXGI/PCI identity matching supplies physical capacity.
  The live slider's maximum now uses that selected GPU's detected capacity, including
  an exact non-grid endpoint. Unknown/ambiguous capacity disables it explicitly.
  Supported selections retain exact bytes through the shared allocation draft;
  above-provider-range selections retain the draft and explain the supported range.
  No supported physical/provider ratio or mapping above the native range is
  established; such selections remain unsupported. Documentation/instruction review
  found decimal-unit constants, provider evidence and illustrative fixture values,
  rather than a product-wide fixed 1 GB limit. Updated current guidance accordingly.
- **ISS-002/007:** source-created enum/numeric failure envelopes carry component,
  effect stage, cause, HRESULT/provider/guest exit codes through existing authenticated
  worker replies. Guest stdout forwards only typed failure facts on unsuccessful
  worker completion; Rust interprets bridge exit/HRESULT fields. Arbitrary appended
  text and unknown fields cannot enter typed diagnostics. Recovery/admission and
  terminal-audit contracts are preserved; unclassified failures remain explicit.
- **ISS-006:** idle read-only polling retries even after failed reads disable
  execution readiness. The installed-only polling path never initiates setup/UAC.
- **ISS-003–005/007:** inspected existing allocation review/write/readback/publication,
  enrollment/consent/modal eligibility, named native credential prompt and durable
  recovery paths. No additional source defect was established for their remaining
  acceptance criteria. Those criteria still require the recorded authorized checks.
- Focused physical endpoint/GPU selection and safe failure regressions were authored
  or updated. No tests, builds, compilation checks, renders, GUI launches, installed
  updates or live operations were run. No issue is closed by these source changes.

### ISS-001 — Physical VRAM versus provider request range

- **Remaining problem:** Hyper-V reports a maximum native VRAM request of
  1,000,000,000 bytes on the tested RTX 5060. The slider uses this discovered range,
  not a hard-coded cap. Physical memory is separate inventory; a physical-card-sized
  request mapping is unqualified.
- **Dependency:** a documented or qualified provider/driver interpretation under
  [DEC-034](../DECISIONS.md#dec-034).
  Multiplying by a card/provider ratio is not accepted evidence.
- **Next action:** establish the supported interpretation before changing conversion
  or limits; record an unsupported result explicitly if no mapping can be established.
- **Close when:** physical capacity and supported request range are presented
  consistently, and any conversion has exact draft/configuration/provider readback
  evidence. Do not claim memory reservation or enforcement from a slider value.
- **New report:** Apply & Main Window/BUG-05 below expands this issue with dynamic
  selected-GPU capacity, slider/advanced-field consistency and documentation review.
  The physical-capacity requirement remains open; the provider interpretation above
  must be resolved without inventing a conversion or weakening legitimate limits.

### ISS-002 — Useful, credential-safe component errors

- **Remaining problem:** known safe application messages, stages, classifications
  and native codes are exposed; arbitrary adapter text is withheld. Full component
  failure diagnostics need a structured safe contract.
- **Where:** `src/reporting.rs`, failing native/guest adapters, authenticated worker
  replies and `src/gui/live.rs` Technical Details.
- **Next action:** return typed safe operation/stage/code/cause context from adapters
  and render it in Technical Details, with concise recovery guidance in the main view.
- **Close when:** representative failures retain their actual safe cause and code;
  credentials/tokens and untrusted appended text are excluded. Existing exact-message
  allowlisting alone does not close this issue.

### ISS-003 — Explicit allocation acceptance

- **Already established:** conversion/draft tests pass; live 0.6 GB slider editing
  updated its label and draft, then Discard cleared it. Suggested advanced defaults
  were visible. The successful Apply omitted all explicit resource triples.
- **Next action:** review an explicit supported request through slider and advanced
  fields, confirm the exact triple in Review, then exercise the approved native write
  and independent readback on an authorized target. Check saved intent and reload.
- **Close when:** UI, Review, saved intent and observed provider values agree for
  supported VRAM/Compute/Encode/Decode writes. Omitted categories create no write;
  unsupported values remain explained. This does not establish enforcement.

### ISS-004 — Installed enrollment dialog acceptance

- **Already established:** enrollment wording/layout is implemented; shared Review
  software renders passed, including narrow/200% fixed-footer checks. The live VM
  was already enrolled, so its run did not exercise a fresh enrollment dialog.
- **Next action:** use an authorized existing unenrolled pair to inspect actual
  installed enrollment text, technical expansion, resizing, cancellation and completion.
  Reproduce the reported switch-availability sequence across dialog/page transitions.
- **Close when:** identifiers stay in Technical Details, permission and subsequent
  GPU approval are clear, text wraps, controls remain usable, and unrelated dialog
  interactions leave the GPU switch eligible. Preserve the existing enrollment boundary.

### ISS-005 — Native credential dialog visual acceptance

- **Already established:** native prompt text names the fresh VM and explains guest
  administrator credentials versus the Windows host. Source/unit coverage and the
  installed named Apply progress render passed. Successful Apply reused stored credentials.
- **Next action:** exercise the native prompt for an authorized VM needing credentials;
  inspect its name/account/purpose text and wrapping, then cancel and complete locally.
- **Close when:** the actual native dialog is readable, names the correct VM, and
  cancellation/storage behavior remains correct. Never put credentials in chat or evidence.

### ISS-006 — External VM power refresh acceptance

- **Already established:** two live idle polling intervals left navigation enabled
  and created no draft. Explicit post-Apply Refresh showed the attached, stopped VM.
- **Next action:** observe an authorized external guest Running/Stopped transition
  while the GUI is idle, without restarting it or using manual Refresh.
- **Close when:** the next successful automatic poll reflects each state, keeps
  navigation responsive and preserves drafts; failed reads clearly mark historical state.

### ISS-007 — Apply failure and recovery acceptance

- **Already established:** unmatched reconciliation failures retained recovery and
  showed a safe cause. Supported baseline retirement and a fresh Apply succeeded.
  Independent static recovery review found no blocking safety issue.
- **Next action:** exercise representative controlled Apply/component failures and
  interrupted retirement around archive, journal retirement and terminal audit. Retain
  durable stage/audit evidence and the subsequent independent reconciliation result.
- **Close when:** failed stage/cause/next action are useful, uncertain effects retain
  their hold, restart cannot replay GPU work, and recovery releases only after proven
  state and terminal audit. Preserve original-versus-verified reconciliation wording.
  Capture guest build/raw probe output for any broader workload qualification claim.

## GUI Bug Report – Apply Approved Changes & Main Window

Recorded: 11 October 2026. These are newly reported defects and requested fixes,
not independently reproduced findings or completed work. BUG-01–05 are local to
this report; existing ISS IDs and earlier BUGS report IDs remain stable.

### Apply Approved Changes

#### BUG-01 / ISS-008: Progress text overlaps the information header

During Apply Approved Changes, progress messages clip into the information text
at the top of the window.

- Keep the information header in its own clearly defined layout area.
- Contain progress messages so they cannot overlap the header or other UI elements.
- Adjust correctly as messages arrive and prevent clipping or overlap on resize.
- **Acceptance:** the header, progress and other controls remain readable and
  separated throughout a long-running operation and window resizing.

#### BUG-02 / ISS-009: Duplicate "Transfer acknowledged" message

The report identifies two displays of **Transfer acknowledged**, also overlapping
the information text at the top.

- Investigate the event source and display path to establish why it appears twice.
- Display each progress event once; inspect other messages for the same defect.
- Keep the message within the progress area. Preserve distinct legitimate events
  even when their text matches.
- **Acceptance:** each event appears once, including transfer acknowledgements,
  without overlapping the header or other UI content.

#### BUG-03 / ISS-010: Progress output does not automatically scroll

New progress messages do not automatically become visible.

- Scroll to the newest message as updates arrive while the user is at the bottom.
- Pause automatic scrolling when the user scrolls up to inspect earlier messages.
- Resume automatic scrolling when the user returns to the bottom.
- **Acceptance:** the latest operation information stays visible without manual
  scrolling, and inspecting earlier output preserves the user's scroll position.

#### BUG-04 / ISS-011: Technical Details becomes inaccessible as progress grows

New messages push **Technical Details** out of the usable window.

- Bound progress output in a scrollable area instead of expanding indefinitely.
- Keep the overall layout stable and important controls and sections accessible.
- Allow Technical Details to be expanded, collapsed and inspected during operations.
- **Acceptance:** growing progress output never pushes Technical Details or important
  controls outside the usable window, including while resizing.

### Main Window

#### BUG-05 / ISS-001: GPU VRAM slider still limited to 1 GB

The report identifies an incorrect 1 GB maximum. The requested slider maximum is
derived dynamically from the selected GPU's detected physical VRAM capacity.

- Detect the actual VRAM capacity of the selected GPU.
- Set and refresh the slider range from that GPU's available physical capacity
  whenever the selected GPU changes.
- Keep the displayed maximum, selected allocation and Advanced Allocation fields
  consistent.
- Do not use a fixed 1 GB maximum or another arbitrary hardcoded cap.
- Keep physical capacity distinct from the provider's supported request values.
  Resolve the existing ISS-001 interpretation dependency before presenting a
  physical-capacity value as a supported provider allocation.
- **Acceptance:** selected-GPU capacity determines the slider maximum; switching
  GPUs updates the range and allocation correctly; slider and advanced values
  agree through Review, saved intent and provider readback. Legitimate provider
  safety restrictions remain enforced and explained.

**Documentation and AI instruction review:** investigate whether documentation,
configuration, examples or AI instructions introduce or reintroduce the 1 GB cap.
Review relevant `docs/ROADMAP.md`, `docs/BACKLOG.md`, `AGENTS.md`, Codex prompts and
AI development instructions, GUI architecture and implementation guides, default
configuration/examples, and any other GPU memory allocation limit definitions.
Correct instructions that incorrectly make 1 GB a hard maximum. Distinguish
examples, defaults, minimum allocations and actual hardware limits; preserve
legitimate minimums and safety restrictions. Documentation must consistently
specify the selected GPU's detected physical VRAM capacity as the slider maximum,
while explaining supported provider mapping and any unresolved limitations.

### Implementation order and requirements

1. Fix progress layout, clipping and overlapping text (BUG-01).
2. Remove duplicate progress events (BUG-02).
3. Implement automatic scrolling with manual-scroll pause/resume (BUG-03).
4. Keep Technical Details accessible throughout operations (BUG-04).
5. Fix VRAM detection and the dynamic slider maximum (BUG-05 / ISS-001).
6. Correct documentation or AI prompts that could reintroduce the 1 GB limit.

This is the priority order within this report; ROADMAP milestone gates remain
authoritative. Implement functional fixes by investigating root causes. Avoid
cosmetic workarounds, unnecessary tests and unrelated refactoring. Preserve
operation approval and recovery behavior. Change GPU safety limits only when
demonstrably incorrect. Update the owning backlog cards as implementation advances,
and move implemented issues to pending acceptance until their criteria pass;
code or documentation changes alone do not close them. Avoid extensive GUI testing
unless explicitly requested and follow the repository validation authorization policy.

**Objective:** keep Apply Approved Changes readable and usable throughout long
operations, and base GPU memory allocation on the selected GPU's actual capabilities
instead of a fixed 1 GB restriction.

## Accepted scope and evidence

The 11 October `test-001` run succeeded: 273 dynamically discovered files,
3,409,186,372 transferred bytes, exact enrolled RTX 5060 attachment, guest PnP
health and checked D3D11 hardware rendering, graceful return to Stopped, and enabled
schema-3 configuration publication. Terminal audit session:
`3e67973d6cfb5776099f0fa6d95ea6cf`. Journal is prepared/nonpending with a successful
verification timestamp; recovery is absent. Independent readback preserved six
CPUs, 4 GiB startup RAM, disk identity and Secure Boot On.

Host: Windows 26H2 build 26300.9457; x64/MSVC debug product and guest binaries;
RTX 5060 driver 32.0.16.1692. Verification requires Code 0, the selected NVIDIA
paravirtualized adapter and validated 256×256 D3D11 output (`vs_5_0,ps_5_0`). Guest
OS build and raw probe JSON were not captured; this is scoped Apply acceptance.

Workspace tests: 173 passed/four opt-in tests ignored; strict Clippy passed;
opt-in software layout passed; installed/source/policy hashes matched all four
binaries, preserving both enrolled pairs. Live Review/Apply/details wrapping,
default suggestions, switch reversal, slider draft/discard, navigation, idle polling
and clean close passed. Sample/software dirty-close and expanded-footer checks passed.
These accepted scopes cover the reported Review summary/technical separation,
footer, ordinary status wording and false automatic/clean-close drafts. They do not
close the active acceptance gaps above or an aggregate milestone.

Run artifacts are local, ignored files: `local/evidence/test001-dialog-fix/`
(terminal audit/journal, saved TOML, before/after identity, stage samples, close result),
`local/evidence/bugs-20261011/` (installed screenshots/polling),
`local/evidence/gui-003-layout/` (software renders), and
`local/gui-update/994940d366fc46be8249ecfd703652bb/` (installed update).
The earlier recovery-blocked attempts were superseded by successful supported
reconciliation; the old interruption is archived as
`audit/ended-be0fbf90e7dc0a1cd79c453fec143c65.json`. No new open incident is asserted
for that resolved record.
