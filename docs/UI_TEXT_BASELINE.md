# Verbatim UI text baseline and extraction batches

LANG-003, 13 October 2026. This fixes the inventory-to-key mapping and records expected pre-migration text under the [catalogue contract](UI_TEXT_GUIDE.md). The [inventory](UI_TEXT_INVENTORY.md) describes consumers/branches; the [occurrence register](UI_TEXT_OCCURRENCES.md) preserves the original source literals. The [roadmap](roadmap.md) and [backlog](backlog.md) own sequence and status.

These are source-derived comparison fixtures, not observed GUI output or executed Rust tests. Original literals were decoded and evaluated with explicit prepared values, selected branches, concatenation order and existing integer formatting rules. No product build, rendering, native prompt or VM/guest operation ran. Migration must compare actual Rust/Slint presenters with these independent expected strings at the later checkpoints; matching the planning evaluator alone is not implementation proof.

## Recorded artifacts and traceability

| Artifact | Content / use |
|---|---|
| [mapping.json](../tests/fixtures/ui-text/mapping.json) | All 819 distinct copy/mixed candidates, original literal/template and source locations, final semantic display keys, named typed parameters, source format specifications, optional-branch variants and retention reasons. Also retains the other 529 literal occurrences and separates five display labels from immutable resource/stage identities |
| [outputs.json](../tests/fixtures/ui-text/outputs.json) | 841 expected outputs: one binding example for every final key, selected complete review/result/history compositions, numeric boundaries, adversarial data and CLI/reporting boundaries; exact UTF-8 output plus SHA-256 |
| [control-contract.json](../tests/fixtures/ui-text/control-contract.json) | Twelve CD1–CD12 semantic regression requirements, immutable inputs, deliberate test-only copy edits and expected decisions/model behaviour for LANG-013; these are requirements, not passing tests |
| [fixture README](../tests/fixtures/ui-text/README.md) | Reading/comparison rules and validation limits |

Snapshot: source revision `bb9c478e5bcff6dd3f6c75f66fec9bafafc5efc4`; the prior LANG-002 documentation edits were already in the working tree. Product source is unchanged. The mapping includes SHA-256 source fingerprints, exact source spellings and every grouped occurrence line. Candidate numbers only join fixtures to frozen audit records; they are not catalogue keys or runtime identities. Final semantic keys replace the temporary `.variant_NNN` inventory proposals. The resources themselves are not implemented and these files are never runtime fallback wording.

Every original occurrence is either a mapped display producer or explicitly retained with a reason. A mixed producer keeps its original classifier/protocol input; only its separately selected display is extracted. Worker persistence/publication/settings/power aliases map to the current friendly GUI label rather than making a second catalogue entry for a wire string never displayed on that surface. Unknown/unmapped phases retain the existing original label as the display baseline while the original wire token stays immutable.

There are 696 distinct final display keys (691 from candidate mappings and five separated resource/stage labels). Identical fixed labels share a key only for the same presentation meaning, including shared branding, power/support labels and action titles. `Off` and `Stopped`, upper/lower-case support labels, short/default/full absence descriptions and different review/result purposes remain distinct. Key names describe purpose/state and do not contain source line numbers, English sentences or variant ordinals.

## Old-to-new comparison contract

For each mapped producer, retain `original_literal`, its decoded `original_template`, source lines and the original Rust field/format token sequence. The proposed `template` replaces those tokens with named parameters without changing literal segments. Rust debug/hex/number/unit formatting belongs in code-owned prepared values, not editor-supplied formatting expressions. `source_bindings` records each original token/spec and the corresponding named type; repeated references use the same prepared parameter once as data.

The evaluated `expected` strings are authoritative comparison values for the recorded inputs. Compare decoded UTF-8 bytes, including leading/trailing spaces, final newlines, blank lines, arrows, middle dots, ellipses, slash direction, case and punctuation. JSON escapes encode values; do not compare JSON indentation/escape spellings with rendered text. `expected_sha256` guards the output bytes; source hashes guard the input snapshot. Do not trim, Unicode-normalize, recursively substitute or round values to make a comparison pass.

`source_template` binding examples exercise formatting only; they do not establish that a branch is admitted or reachable. `assembled_review_summary`, `assembled_history`, reporting/CLI boundary and named journey cases include explicit selectors. Prepared `message` values in binding examples are synthetic safe catalogue submessages; prepared `formatted` values stand for code-owned formatting. Future tests may prepare these in test-only helpers inside the presentation module; do not expose an unchecked production constructor or resource-controlled error forwarding to accommodate a fixture.

For `fixed_worker_alias`, evaluate the immutable input through the existing alias selection before comparing the catalogue display. For the four Slint concatenations, replace the complete expression: VM count, fixed-capacity `Use ` button, allocation accessibility label and numeric default suffix. Keep the original fragment as the old-source reference, and use the recorded complete named template as the new display contract.

Optional notes/save failures use separate `with_note`/`without_note` and `with_failure`/`without_failure` keys. The base entry identifies a branch family, not an extra required key. An absent source string remains a code-owned empty sentinel; no blank `UiMessage` accessor is introduced. The fixtures preserve the current extra blank line when an Apply effect summary already ends in LF and the outer unsaved result adds another LF.

Source-template outputs before an error dialog are not automatically the final displayed error summary. The existing dialog host reclassifies error bodies; shared CLI `public_result` likewise reclassifies string fields. The fixtures name that boundary. Preserve immutable input/classification first, catalogue only the selected safe presentation, and compare both the relevant producer and final reporting boundary. Raw parser/adapter errors are never new parameters. Current status/inactive-profile wrappers containing an `error` variable are marked mixed with a binding warning: capture known app-owned/safe values, trace/redact unknown inputs, and do not use the baseline to approve arbitrary forwarding.

## Extraction sequence

Activate only the exact keys/resources/contracts for a prepared batch; unmigrated rows stay pending. Validate active keys strictly, then reconcile each migrated row to its final consumer and baseline. This does not authorize automatic execution of another card.

| Batch | Extraction order / owning feature | Acceptance comparison |
|---|---|---|
| B1a | `common`, `dashboard`: branding/navigation, defaults, empty/updating states, cards/accessibility | Fixed decoded text and property/default branches; CD5/6 separation before power/status extraction |
| B1b | `system`, `settings`, `resources`: System/About/Settings, memory/advanced editor labels, placeholders and disclosure | Whole concatenations, numeric values unchanged, disabled/hidden/setup panels retained; CD8 first |
| B1c | `dialogs`: More actions, consent labels, cancel/close/retry/follow-up controls | Immutable callbacks/kinds/focus/state conditions remain; all conditional labels match |
| B2a | `operations`: first permission/setup, ordinary Apply, verification-only Apply, update, repair and removal | Correct branch order, before/after values, step numbers, downtime/time guidance and conditional final power |
| B2b | `operations`: Verify/profile deletion/partial removal reviews and results; saved versus unsaved effects | Driver files/receipt versus rendering, profile records versus host removal, two-step authorization, receipt/readback uncertainty |
| B2c | `operations`: recovery/end-attempt/save-only, selection/draft/action availability and status retries | Holds/admission and pending edits unchanged; no replay, no blank failure reason; retained original result not accumulated retry text |
| B3 | `progress`: aliases/statuses, transfer display, current-stage/step list/event log | Immutable original stages/counters before editable formatting; fraction/coalescing and failed/done boundaries preserved; CLI wire progress unchanged |
| B4 | `diagnostics`, `system`, separated `common.resource` labels: reporting, host comparison and app-owned details | Classification/redaction/code namespaces/limits first; safe facts and uncertainty retained; shared CLI boundaries compared |
| B5 | `credentials`: native captions/explanations/cancellation and credential settings results | Original name filter, local explanation/prompt order, opt-in storage, cancellation sentinel and native UTF-16 budget; no live prompt in automated checks |

LANG-004's representative active set is the fixed `operations.verify.title`, named `operations.apply.power_consent`, and native `credentials.guest_sign_in.message` plus necessary fixed power/caption/wrapper dependencies. LANG-007 can validate that closed subset before bulk extraction. No missing/unknown-key fallback is added for pending batches. Bulk B1 belongs to LANG-005; B2–B5 to LANG-006; editor handoff to LANG-008; complete reconciliation/automated comparison and separately authorized layouts to LANG-009.

## Representative journey and formatting cases

The artifacts include every final key's binding example. These selected case families additionally capture composed output and branch consequences. All targets/names/IDs/pins are synthetic.

| Case family in `outputs.json` | Selected source branches / exact distinctions |
|---|---|
| `review.setup.initial` | Initial setup with driver preparation, compatibility/attachment/allocation, rendering and graceful final-off; numbered order and time/consent prose |
| `review.apply.ordinary` | Running VM/retained attachment, allocation change, shutdown, no credentials/preparation/graphics check; final `Stopped` |
| `review.apply.verification_only` | Current files/receipt/rendering check without reinstalling; no preparation-time sentence; reviewed final-off |
| `review.setup.update`, `review.repair` | Invalid/changed preparation versus explicit same-history reinstall; detach/start/prepare/stop/attach/check sequence, exact before/after values and preserved settings notice |
| `review.removal`, `partial_removal.*` | Disable exact managed attachment and restore compatibility versus bound partial-removal recovery; retain files/history; no guest connection; preserve existing `MB` wording in partial-removal review and `MiB` in other MMIO labels |
| `review.apply.no_effects`, `review.recovery.pending`, `review.power.unknown_helper_branch` | Empty steps still save reviewed intent; pending recovery warning; synthetic unknown-power helper output without claiming that admission permits this plan |
| `verify.review.*`, `verify.consent.*`, `apply.consent.*`, `recovery.consent.*` | Off/Running/Unknown lifecycle and consent; Verify reflects observed power, Apply reflects the reviewed final power |
| `guest_progress.*`, `credentials.cancel.*` | Apply/Verify/recovery/credential-management/generic purposes; cancelled guest/recovery actions retain draft/hold and do not claim completion; exact original sentinel recorded separately |
| `profile_deletion.review/completed` | Delete app settings/pairing/history only; inactive/off prerequisite; VM/disks/checkpoints/guest files/other profiles retained |
| `setup.result.saved`, `apply.result.saved`, `*.result.unsaved.*` | Rendering checked versus no guest check; exact `3.000000001 GB`; confirmed effects versus configuration publication; absent and present save failure with exact whitespace |
| `verify.result.*`, `completion.*`, `recovery.result.*`, `recovery.review.*`, `save_only.*` | Current check pass/unknown, missing/changed completion receipt, historical success distinct from finalization, recovery ended/confirmed/held and publication-only retry |
| `failure.*`, `reporting.condition.*` | Missing/changed files, wrong receipt, denied/timed-out/unavailable facts, absent attachment, pending/partial recovery, failed restoration-proof persistence and UAC cancellation; safe selected guidance without raw adapter data |
| `cli.progress.*`, `cli.public_result.*`, reporting cases with `Command` | Typed status plus fixed stage/newline; stable JSON keys/operation ID; raw condition versus already-presented failure at the `public_result` boundary. Existing repeated-classification output is preserved rather than silently corrected during extraction |
| `transfer.*` | Empty/no-percentage, zero/partial/full acknowledged copy, invalid counters, singular bytes, decimal unit rounding/promotion, 99% at overflow/rounded boundaries; 100% is not verified setup |
| `memory.*`, `draft.pending.*` | Exact decimal GB/numeric input, binary GiB display and separately rounded endpoint; below-minimum request retained; `6000000001` bytes → `6.000000001`; capacity `8278507520` bytes → exact `8.27850752`, endpoint `8 GB` |
| `data.*` | Quotes, braces, backslashes/forward slashes, LF/CR/tab, Unicode/combining mark and long names. Ordinary name text remains data; native names remove controls and keep at most 256 scalars, including the 512-unit worst-case UTF-16 name |
| `history.latest.*`, `host_comparison.*` | New adverse/unknown check plus last successful Unix UTC history, not current health; unknown saved/host data and expected attachment missing remain distinct |

The baseline retains existing wording even when a shorter or clearer message would be desirable. Do not update these historical outputs during LANG-010 to make them become the new editorial expectations; deliberately approved copy gets separate revised expectations while the extraction baseline remains traceable.

## Semantic regressions versus verbatim checks

Verbatim migration compares exact recorded inputs/outputs and reconciles all keys/exceptions. Semantic tests assert operation identity, admission, typed state/effects, fraction, internal row identity, failure boundaries, cancellation, hold retention and allowed facts. Their assertions must survive edited titles/prefixes/status/absence wording. They may refer to immutable protocol fixtures but must not depend on the English catalogue text used for dispatch/classification.

The control contract covers all twelve inventory dependencies:

| ID | LANG-013 isolated proof / later acceptance |
|---|---|
| CD1 | Stable operation identity chooses purpose and silent status-refresh behaviour; edited titles do not change worker dispatch or busy/modal state |
| CD2 | Immutable counter parser supplies fraction; edited prefix/punctuation is never parsed; empty/invalid/full-copy cases remain indeterminate or copying-only as appropriate |
| CD3 | Stage/status identity drives in-place rows and history boundaries; edited row strings do not change coalescing/model reuse; phase changes clear fraction |
| CD4 | Original native sentinel alone denotes local cancellation; edited cancel copy is not a sentinel; no new guest work/draft loss/hold release |
| CD5 | Typed power/historical/updating state supplies colour category independently of label |
| CD6 | Typed eligibility/setup presentation state supplies panels; edited label does not grant admission or invent a currently unproduced setup status |
| CD7 | Collected typed facts supply details visibility; edited headings/absence phrases are not evidence predicates |
| CD8 | Bytes supply numeric input directly, preserving exact requests/capacity independently of unit text |
| CD9 | Original envelopes/needles/allowlist/compound markers and raw safe limits remain inputs; edited guidance/static stage copy cannot change classification or leak untrusted data |
| CD10 | Explicit allowed/blocked/valid/invalid state or enforced required nonempty reasons; blank edited text cannot enable an action or grant effects |
| CD11 | Immutable `Preparation record` discriminator preserves journal-specific recovery/unknown handling; displayed stage may change |
| CD12 | Additional facts remain reachable and duplicate suppression is reviewed; text similarity never controls authority/holds. Isolated visibility checks first; actual detail interaction remains LANG-009 |

The controller/state/worker/reporting tests identified in the inventory remain semantic evidence only when actually executed. Add edited-copy regressions through those existing interfaces in LANG-013, then compare these golden strings before extraction and at LANG-009. No restored mock/snapshot GUI tooling, live credential prompt, installation or VM workload is needed for text/control contracts.

## Validation and remaining limits

Targeted baseline review passed (`python -B -X utf8 local/evidence/lang-003/review.py`): source hashes/literal locations, all 1,404 occurrence dispositions, 696 unique/schema-valid final keys, Rust/Slint symbol collisions, complete named parameter bindings, optional-branch coverage, 841 output hashes, exact old-to-new template equivalence for the prepared examples, all required journeys and CD1–CD12 coverage. `git diff --check` passed. Local planning scripts create/check the historical fixtures; they are not product presenters or a passing test suite. `python scripts/testing/check-docs.py` failed on the same 22 links to deleted handbook pages; unrelated deletions were preserved.

Actual Rust evaluation, generated code compilation, TOML-only incremental rebuild, real catalogue validation, full non-hardware tests and rendered/interaction acceptance are not run by this card. They remain the specific later gates. L1 inventory/design/baseline deliverables are recorded; no compiled/runtime or visual milestone is claimed. Next source card: LANG-013, then LANG-007 before LANG-004.
