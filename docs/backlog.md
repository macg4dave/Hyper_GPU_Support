# UI text development backlog

Tasks implement the [roadmap](roadmap.md). [Engineering](ENGINEERING.md) defines separate implementation/validation status. LANG-001 inventory, LANG-002 design and LANG-003 baseline and LANG-013 control separation are complete; LANG-007 validation tooling is complete; product catalogue/integration remains pending. Open cards do not authorize automatic execution.

## Completed legacy work

The previous APP backlog is complete by user confirmation on 13 October 2026. APP references in the handbook are historical context, not active cards or dependencies. Removed evidence pages are not restored or treated as blockers. This acceptance does not invent test results or authorize repeating live operations.

## Next implementation action

**LANG-004: prove representative build and consumer integration.** Use the shared [validator crate](../tools/ui-text/README.md) and the exact nine-key active contract/fixture subset. Wire the root build to validation/generation, prove generated Rust/Slint compilation and TOML-only directory/file rebuilds, then migrate the prepared representative consumers. Complete one requested card at a time.

## Task register

| Task | Implementation | Validation | Dependencies | Milestone |
|---|---|---|---|---|
| LANG-001 | Done | Static review passed; repository docs check failed (deleted handbook links) | None | L1 |
| LANG-002 | Done | Passed: design/targeted checks; Failed: repository docs links | LANG-001 | L1 |
| LANG-003 | Done | Passed: targeted baseline review; Failed: repository docs links | LANG-001, LANG-002 | L1 |
| LANG-004 | Ready | Not run | LANG-003, LANG-007, LANG-013 | L2 |
| LANG-005 | Blocked | Not run | LANG-004 | L3 |
| LANG-006 | Blocked | Not run | LANG-005 | L3 |
| LANG-007 | Done | Passed: focused validator/scan checks; L2 integration pending | LANG-002, LANG-003 | L2 |
| LANG-008 | Blocked | Not run | LANG-004, LANG-007 | L3 |
| LANG-009 | Blocked | Not run | LANG-005, LANG-006, LANG-008 | L3 |
| LANG-010 | Blocked | Not run | LANG-009 | L4 |
| LANG-011 | Blocked | Not run | LANG-010 | L4 |
| LANG-012 | Blocked | Not run | LANG-011 | L4 |
| LANG-013 | Done | Passed: focused semantic checks; pending L2 gate | LANG-003 | L2 |

Blocked refers only to outstanding new LANG dependencies. No legacy APP gate applies. Future validator commands must be documented when implemented; the existing non-hardware gate is `powershell -NoProfile -File scripts/testing/check.ps1`.

## Work cards

### LANG-001 - Inventory and classify UI text

Dependencies: None. Scope: audit only.

Start with Slint components and Rust `gui/wording.rs`, `actions.rs`, `live_state.rs`, `live/presentation.rs`, `live/progress.rs`, `result_notice.rs`, `gui_model.rs`, `reporting.rs`, `credentials.rs`; trace additional producers actually displayed. Create `docs/UI_TEXT_INVENTORY.md` with source location, original template, consumer/screen/state, parameters, branch conditions, proposed key and batch. Include accessibility, disabled/empty/recovery states, native explanations and app-owned details.

Classify copy, immutable control/protocol strings, external data and justified exceptions. Trace title dispatch, progress parsing/coalescing, cancellation and error classification through callers/tests. Identify shared CLI consumers and text-specific assertions; record uncertain paths.

Acceptance: every discovered producer has a disposition; exceptions have reasons; ambiguous copy/control uses have a resolution path. Validation: static consumer/documentation review; no build/live run.

Delivered 13 October 2026: [inventory](UI_TEXT_INVENTORY.md) and [source occurrence register](UI_TEXT_OCCURRENCES.md), including consumers/branches, provisional key families, parameters/batches, exception reasons, shared CLI consumers, text assertions and twelve copy/control dependencies. Static producer/caller review passed. `python scripts/testing/check-docs.py` failed with 22 links to deleted handbook files; unrelated deletions were preserved. Targeted inventory/register source, links and whitespace checks passed. Product source and wording are unchanged; no build, Rust tests or live operations ran. L1 still requires LANG-002/003.

### LANG-002 - Define catalogue and generation contracts

Dependencies: LANG-001.

Draft `docs/UI_TEXT_GUIDE.md`: feature files, semantic keys, schema, UTF-8/multiline rules, named parameter types and literal-brace escaping. Define editor context, protected semantics, empty-string rules and advisory versus real API length limits. Specify an independent code-owned required-key/parameter contract.

Design build-time Rust accessors and a Slint global using existing compilation, including generated imports, per-window initialization if needed, library/application consumers and explicit build dependencies. Specify deterministic escaping, failure policy and resource/directory rebuild tracking. No runtime loader or fallback.

Acceptance: fixed, parameterised, multiline and native-prompt examples have unambiguous contracts grounded in current consumers. Validation: design/source review; prototype proof belongs to LANG-004.

Delivered 13 October 2026: [catalogue/generation contract](UI_TEXT_GUIDE.md), including feature/schema/key rules, typed parameters and one-pass formatting, independent required-key/metadata/API contracts, native bounds, per-window Slint literal defaults, Rust/library/shared CLI integration, deterministic escaping/rebuild tracking and a product-independent validator design. Static source/design review, targeted guide links/whitespace, TOML examples/parameter declarations, verbatim source comparison and native worst-case length arithmetic passed (`python -B -X utf8 local/evidence/lang-002/review.py`); `git diff --check` passed. `python scripts/testing/check-docs.py` failed on the same 22 links to deleted handbook pages. No catalogue/tool/product implementation, build, Rust tests or live operations ran. Compile/incremental proof remains LANG-004; L1 still requires LANG-003.

### LANG-003 - Record baseline and extraction batches

Dependencies: LANG-001, LANG-002.

Map producers to keys and preserve original punctuation, whitespace, branches and number/unit formatting, including existing spelling until editorial review. Define feature batches and expected outputs for setup, ordinary Apply, Verify, repair, removal, deletion, cancellation, uncertainty and recovery. Include synthetic quotes/braces/slashes/newlines/Unicode/long names. Separate stable semantic tests from verbatim migration checks.

Acceptance: traceable old-to-new comparison and LANG-013 coverage of control-data risks. Validation: baseline review; no invented visual/live results.

Delivered 13 October 2026: [baseline/batches](UI_TEXT_BASELINE.md), [final source-to-key mapping](../tests/fixtures/ui-text/mapping.json), [expected outputs](../tests/fixtures/ui-text/outputs.json) and [semantic control requirements](../tests/fixtures/ui-text/control-contract.json). All 1,404 inventoried occurrences have a disposition; 696 final keys have typed bindings and comparison examples; 841 source-derived expectations cover required journeys/data/format boundaries and all twelve control risks are assigned to LANG-013. `python -B -X utf8 local/evidence/lang-003/review.py` and `git diff --check` passed; `python scripts/testing/check-docs.py` failed on the same 22 links to deleted handbook pages. These are reviewed source-derived outputs, not executed product Rust/Slint tests. No product changes, catalogue implementation, builds, rendering, credentials or VM/guest/protected effects ran. Actual presenter comparison and generation/runtime proof remain later gates.

### LANG-004 - Integrate the build-time catalogue

Dependencies: LANG-003, LANG-007, LANG-013.

Extend existing `build.rs`; generate into `OUT_DIR` and share the validated catalogue across Rust and Slint through existing presenters/models. Integrate a fixed label, named-parameter operation message and native credential explanation with original wording. Support all required consumers/windows; avoid repeated parsing on UI refresh and recursive template replacement.

Document the actual standalone validator command and add shared checks to the existing non-hardware gate without restoring removed CI/tooling. Prove a TOML-only edit updates both consumers after rebuilding; restore original text afterward. Verify file addition/deletion invalidation and safe generated syntax.

Acceptance: representative integration and resource-only rebuild proof pass; invalid resources fail clearly. Validation: targeted generation/rebuild checks and L2 automated checkpoint; rendered acceptance remains separately authorized/evidenced.

### LANG-005 - Extract fixed screen and dialog copy

Dependencies: LANG-004.

Migrate static wording by batch, including accessibility, defaults, hidden/disabled states and menu reasons. Preserve callbacks, action/dialog identifiers, property directions, focus, selection and layout. Reconcile inventory/exceptions; external OS/widget text remains documented.

Acceptance: all inventoried fixed app-owned UI wording uses the catalogue and matches the baseline. Validation: focused review/comparison during extraction; build/visual coverage at LANG-009.

### LANG-006 - Extract dynamic, progress and failure presentation

Dependencies: LANG-005.

Migrate Rust templates and affected shared reporting consumers, retaining types, event order and data formatting. Format progress after immutable parsing; retain percentages, coalescing and failure/completion boundaries. Copied files are not verified setup.

Catalogue safe summaries and app-owned details after classification/redaction. Preserve sentinels, envelopes, allowed raw codes and secret filtering. Cover unavailable observations, partial/secondary failures, cancellation and recovery/save-only wording without changing admission/holds. Treat inserted values as data, never templates.

Acceptance: inventory reconciles; baseline output matches; edited titles/status cannot affect selection, progress or cancellation. Validation: meaningful presentation/failure regressions and focused checks where needed; full execution at LANG-009.

### LANG-007 - Add validation before bulk extraction

Dependencies: LANG-002, LANG-003.

Implement one core shared by standalone validation and build generation. Reject malformed TOML, duplicate cross-file keys, strict-schema violations, missing/unknown keys, generated-name collisions and empty required text. Check exact parameter names/types, permitted repetition/reordering and escaping against the independent contract.

Add isolated failure fixtures, deterministic generation checks and adversarial Unicode/braces/quotes/newlines/code-like text. Invalid resources must stop generation/build. Add a focused hard-coded-copy scan with reviewed exceptions; use lexical scans as advisory where classification is unreliable. Reconcile inventory separately.

Acceptance: documented local checks reject each invalid fixture and accept baseline resources; build/standalone checks share rules. Validation: execute targeted validator/failure tests here; generated compile proof at LANG-004. No GUI/hardware effects.

Delivered 13 October 2026: isolated `tools/ui-text` library/CLI, pinned aligned lockfile, independent nine-key active contract, strict baseline/failure fixtures, deterministic segment-based Rust/Slint generation and focused copy scan with reviewed pending-source exceptions. Rust tests use synthetic edits and semantic validation assertions rather than duplicated UI English. The historical mapping/outputs are unchanged; no shipped catalogue or root build integration was added.

Focused validation: nine Rust validator tests and two Python scan tests passed; standalone baseline validation/generation and copy scan passed. Formatting/diff checks passed. Repository docs checking still reports the existing 22 deleted-handbook links. Generated-source compile proof, root non-hardware/L2 gate, product rebuild propagation and rendered/live behavior remain pending LANG-004/later gates. See [tool commands and boundaries](../tools/ui-text/README.md).

### LANG-008 - Complete the editor guide

Dependencies: LANG-004, LANG-007.

Finish `docs/UI_TEXT_GUIDE.md`: exact editable fields/files, context, glossary, British English, placeholders, consequences/severity and space guidance. Editor changes `text` only; keys, metadata/contracts, meaning and source stay immutable. Explain ambiguous-copy escalation, safe details and exceptions. Include an editorial request and actual validation/rebuild commands.

Acceptance: an editor can produce a reviewable resource-only diff using guide/catalogue alone. Validation: walkthrough of fixed/dynamic/warning entries and static docs checks.

### LANG-009 - Verify the verbatim migration

Dependencies: LANG-005, LANG-006, LANG-008.

Reconcile inventory/exceptions; run catalogue checks and the existing non-hardware gate after reviewing test effects. Compare representative baseline branches and affected shared CLI output; verify incremental resource rebuild and a clean build. Record revision, inputs, exact checks/results and exclusions in local evidence.

Prepare separately authorized representative GUI checks for labels/dialogs, consent, progress, accessibility and long content using current UI. No retired mock/snapshot tooling or repeated setup solely for screenshots. Record actual coverage and Not run cases.

Acceptance: automated checks pass, wording is preserved, exceptions justified, required visual/interaction checks evidenced. Implementation and awaiting visual acceptance may be recorded separately; L3 stays open until its checks pass. No VM-behavior claim from catalogue/render tests.

### LANG-010 - Perform the editorial-only pass

Dependencies: LANG-009. Owner: language editor.

Improve clarity, British English and consistent terminology through `text` fields only. Preserve placeholders, meaning, consequences, uncertainty, severity and consent. Flag ambiguity and length risk. Deliver resource-only diff and notes; no source/keys/metadata/generated-file edits.

Acceptance: ownership boundary holds and semantic ambiguities are identified. Validation: editor self-review; technical/semantic acceptance belongs to LANG-011. This planned role does not automatically invoke or authorize another agent.

### LANG-011 - Validate revised copy and layouts

Dependencies: LANG-010. Owner: Codex.

Validate resources, compare changed meaning with actual operations and build. Update only tests tied to deliberately approved wording. Run control-data/semantic regressions proving labels do not affect dispatch, percentages, coalescing or cancellation.

Inspect affected separately authorized GUI/native surfaces for wrapping, clipping, reachable actions, focus, scrolling, supported minimum window, themes and recorded scaling. Resolve necessary layout defects within the existing design; do not silently rewrite approved copy. Return ambiguous meaning to the editor.

Acceptance: checks/build pass, copy accurately describes behavior and affected layout/interaction checks pass with exact coverage. No guest/VM effects implied.

### LANG-012 - Establish recurring copy review

Dependencies: LANG-011.

Update guide/engineering workflow: developer adds key/contract/context, editor changes text, Codex reviews semantics/validation/build, affected surfaces receive milestone visual review. New in-scope copy uses the catalogue; maintain inventory/exceptions without blocking unrelated development on editorial review. Record final milestone evidence and maintenance action.

Acceptance: future features can use the handoff without bulk extraction; owning pages agree. Validation: static docs checks and one example future-feature walkthrough.

### LANG-013 - Separate control data from editable copy

Dependencies: LANG-003.

Required before LANG-004.

Replace displayed-title dispatch in `wording::guest_progress` and callers with stable operation identity while preserving output/worker dispatch. Derive progress identity/counters/coalescing from immutable input before formatting; rendered rows use internal identity, never catalogue text. Preserve current wire formats; wider IPC redesign is outside scope.

Preserve cancellation sentinels and classification inputs, mapping to copy only after decisions. Audit availability/result helpers for other display-string comparisons. Add regressions with deliberately edited titles/prefixes/status: operation selection, percentage, in-place updates, failure/completion boundaries, cancellation and recovery decisions remain the same.

Acceptance: audited paths have no control dependency on editable copy; original protocol and verbatim output remain intact. Validation: meaningful isolated regressions/focused contract checks, included in L2 checkpoint; no guest/protected/VM effects.


Delivered 13 October 2026: local operation identities, immutable worker progress metadata and row slots, typed power/status/action/draft flags, numeric VRAM editor formatting, and collected diagnostic facts. Worker strings, cancellation sentinel, envelopes, classification inputs and preparation-record discriminator are unchanged. Existing live wording is preserved; the unused title branch for inactive-pair removal had no caller and was removed. The actual Delete app profile operation retains its existing generic read explanation. Copy-only wording assertions were replaced with semantic checks rather than duplicated catalogue English. See the [control audit](UI_TEXT_BASELINE.md#lang-013-source-separation).

Focused validation: 100 GUI helper/state tests, 11 reporting tests, 7 credential tests and 16 inventory-state tests passed; Rust/Slint compiled through the test target. Formatting and diff checks passed. Repository docs checking still fails on the existing 22 links to deleted handbook pages. Full L2 gate, live/rendered GUI and guest/VM/protected behavior remain untested by this card.
