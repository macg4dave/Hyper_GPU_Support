# UI text development roadmap

Status: L1 deliverables recorded; LANG-001 inventory, LANG-002 design and LANG-003 source-derived baseline complete; LANG-013 source separation and LANG-007 isolated validation/generation implemented with focused checks. L2 remains open. Source assessment: 13 October 2026. This page owns scope, sequence and milestone gates; [backlog](backlog.md) owns tasks. The previous APP backlog is complete by user confirmation and is not a prerequisite for this work.

## Outcome and viability

Make application-owned English GUI text editable in UTF-8 TOML resources without Rust or Slint source edits for subsequent wording changes. A rebuild is acceptable. Extract existing wording verbatim, prove equivalence, then perform a separate British English editorial pass. Codex owns code/integration and semantic/visual review; the language editor owns wording only.

**Viable as a staged presentation refactor.** Rust already uses TOML, Slint 1.18.1 compiles through [build.rs](../build.rs), and components already consume string properties and Rust models. No runtime catalogue loader, translation framework, new GUI or alternate operation backend is needed. This is source/design evidence, not a compiled prototype or runtime proof. Estimate effort after the inventory: static labels are straightforward; tracing dynamic/error/progress consumers is the larger part.

### Source findings

| Source | Development consequence |
|---|---|
| [Slint UI](../src/gui/ui/app.slint) embeds labels, hints, defaults, consent and accessibility text | Include hidden/disabled states and composed labels, not just `text:` literals |
| [Operation wording](../src/gui/wording.rs) now selects guest explanations by local operation identity | Introduce a stable operation identity before extracting titles |
| [Progress UI](../src/gui/live/progress.rs) now derives counters and row identity from immutable worker input before rendering; [reporting](../src/reporting.rs) defines the transfer format | Preserve wire formats. Derive identity/counters from immutable input before formatting editable rows; never parse edited display text |
| [Credentials](../src/credentials.rs) detects cancellation by an exact error string | Preserve the sentinel and map cancellation to separate editable presentation; preserve local prompts and opt-in storage |
| Reporting uses structured failure envelopes and some text-based classification | Catalogue copy belongs after classification/redaction. Preserve envelope tokens, allowed numeric facts, limits and secret filtering |
| [Dialog host](../src/gui/mod.rs) separates operator summaries and diagnostics | Reuse that boundary; do not expose arbitrary adapter output as catalogue parameters |
| [Documentation checker](../scripts/testing/check-docs.py) expects removed APP cards | Validate the new LANG plan; historical APP references are completed context, not active dependencies |

## Scope and invariants

Include app-owned GUI titles, labels, tooltips/hints, accessibility names, confirmations/consent, unavailable-action reasons, status/progress, results/error summaries and native credential-explanation copy. Trace shared backend/reporting prose actually displayed on these surfaces. Identify and check shared CLI consumers affected by extraction.

Exclude protocol/action/dialog tokens, resource keys, storage schemas, filenames, credential identifiers, diagnostic envelopes, raw Windows/driver messages and third-party/native widget wording. Inventory app-owned technical-detail prose and either extract it or justify each exception; ordinary UI copy cannot hide under a blanket diagnostics exception.

- Preserve the approved Slint design, Rust core and protected operation architecture: admission, target identity, consent, final power, failure semantics, recovery holds and save-only behavior.
- Editable copy never selects an operation, classifies a result or proves completion. Insert dynamic names/paths/numbers once as data, with no recursive placeholder expansion.
- Preserve permitted diagnostic facts and current sanitisation. Never put passwords or arbitrary unsafe adapter output into resources or parameters.
- Extraction preserves exact wording, punctuation, whitespace and numeric/unit formatting. British English improvements start only after the baseline gate.
- GUI launches, credentials, guest access, VM/GPU effects, protected writes, installation and driver changes still require prepared authorization under [testing](TESTING.md). Documentation/source work does not authorize these effects.

## Architecture direction

Use one authoritative catalogue in `resources/i18n/en-GB/`, split by coherent feature, for example `common.toml`, `dashboard.toml`, `dialogs.toml`, `operations.toml`, `credentials.toml`. Inventory determines final grouping. Keys describe meaning, not current English or source locations.

Each entry contains text plus context: screen/state, audience/intent, consequences/severity, named placeholders, and applicable space constraints. Suggested starting shape:

```toml
[messages."operations.verify_title"]
text = "Verify GPU setup"
context = "Action/title for a reviewed driver/setup and rendering check."
placeholders = []
```

The [LANG-002 guide](UI_TEXT_GUIDE.md) fixes the full schema, parameter types, literal-brace escaping, multiline rules and protected semantics; the abbreviated example above is illustrative rather than the complete schema. Maintain an independent code-owned required-key/parameter contract: editing text and metadata together must not defeat validation. Length guidance is advisory unless a real API limit requires rejection; character counts do not prove layout fit.

Extend the existing build path to validate TOML and generate typed Rust accessors and a Slint string global into `OUT_DIR`. Static components import the global; parameterised wording uses the same catalogue through Rust presenters and existing properties. [Slint globals](https://docs.slint.dev/docs/slint/guide/language/coding/globals/) support cross-component access and host bindings. Generated imports, syntax escaping and any per-window initialization must be proved on pinned 1.18.1 in LANG-004.

Generated code is not hand-edited or a second English source. Generate deterministically and track both resource contents and directory membership for rebuilds. A TOML-only edit must update Rust and Slint after rebuilding; adding/deleting files must trigger validation. Runtime files are not loaded by privileged workers. `build.rs` needs explicit build-time TOML validation through the shared tooling library; the existing runtime dependency alone is insufficient. Reuse pinned versions without upgrades.

Provide a standalone validator sharing the build rules. Reject malformed/duplicate/unknown/missing keys, invalid schemas, empty required text, placeholder mismatch and generated identifier collisions. No blank/wrong-message fallback. A hard-coded-copy scan supplements the inventory and review; it cannot prove coverage alone.

## Milestones

| Milestone | Cards | Exit gate |
|---|---|---|
| L1: Design and baseline | LANG-001, LANG-002, LANG-003 | Complete inventory/exceptions, consumer map, schema/formatting decision and verbatim comparison baseline |
| L2: Safe integration | LANG-013, LANG-007, LANG-004 | Copy/control separation, validator failure tests and representative integration pass. TOML-only edits rebuild into Rust and Slint; no live effects needed for this proof |
| L3: Verbatim migration | LANG-005, LANG-006, LANG-008, LANG-009 | Inventory reconciled; baseline output matches; editor guide delivered; non-hardware gate passes; separately authorized representative rendered/interaction checks evidenced |
| L4: Editorial delivery | LANG-010, LANG-011, LANG-012 | Resource-only diff reviewed for accurate consequences; validation/build pass; affected rendered surfaces checked; recurring handoff documented |

Run focused validator/formatting failure checks when developing those contracts. Run the existing non-hardware gate at integration/migration checkpoints, not after every extraction batch. Record Passed, Failed and Not run separately. Rendering/build checks do not prove VM behavior; do not repeat driver setup solely for screenshots. See [testing](TESTING.md) for effects and evidence.

L1 inventory/design/baseline deliverables are recorded through static and targeted fixture review: [inventory](UI_TEXT_INVENTORY.md), [occurrence register](UI_TEXT_OCCURRENCES.md), [catalogue contract](UI_TEXT_GUIDE.md) and [verbatim baseline/batches](UI_TEXT_BASELINE.md). Expected strings are source-derived, not executed product/visual evidence; no compiled/runtime milestone is claimed. Repository documentation checking still reports links to deleted handbook pages. LANG-013 control separation and LANG-007 isolated validator/generator checks have passed. The product catalogue/build integration is not implemented yet; LANG-004 is ready for representative L2 integration; L2–L4 gates remain pending. The old APP plan is accepted complete: no legacy card, removed evidence page or old live scenario blocks this work. Acceptance does not create new test evidence. Existing product/security behavior remains the baseline to preserve.

## Delivery order and editorial handoff

LANG-001, then LANG-002/LANG-003, then LANG-013/LANG-007 before LANG-004. Extract under LANG-005/LANG-006, finish LANG-008 and close LANG-009. Only then perform LANG-010/LANG-011 and finish LANG-012. IDs remain stable from the proposal; numeric order is not dependency order.

Editors receive the catalogue, `docs/UI_TEXT_GUIDE.md` and available non-secret context/screenshots. They edit `text` fields only, flag ambiguity/length risk, and preserve keys, placeholders, metadata/contracts, severity, consent and functional meaning. Codex validates resources and semantics, rebuilds and checks affected layouts. Automated validation cannot prove truthful wording.

Out of scope: additional languages, runtime language switching/hot reload, general GUI redesign, dependency upgrades, backend operation rewrites, wire/storage format changes and reopening completed APP tasks.
