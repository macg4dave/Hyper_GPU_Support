# UI text catalogue and generation contract

LANG-002 design, 13 October 2026. This guide fixes the resource/formatting/build contract for the [roadmap](roadmap.md), grounded in the [inventory](UI_TEXT_INVENTORY.md) and [source register](UI_TEXT_OCCURRENCES.md). The [LANG-003 baseline](UI_TEXT_BASELINE.md) now records final keys, source-derived expectations and extraction batches. LANG-007 now implements the isolated validator/generator and LANG-013 separates control from copy. LANG-004 now integrates the nine-key representative product catalogue and generated build bindings. Bulk extraction remains LANG-005/006. LANG-008 completes the editor handoff with actual commands and catalogue entries.

## Ownership and files

The authoritative wording will live in `resources/i18n/en-GB/`. Use these flat feature files, with no nested directories or runtime loading:

| File | Keys / purpose |
|---|---|
| `common.toml` | `common.*`: reusable labels, power/status display variants, app branding, generic controls |
| `dashboard.toml` | `dashboard.*`: navigation, VM cards/details, selection/empty/updating states, accessibility |
| `system.toml` | `system.*`: GPU/environment facts and capability explanations |
| `settings.toml` | `settings.*`: appearance, credentials settings, About/help |
| `resources.toml` | `resources.*`: memory/allocation editor labels, range notes and validation presentation |
| `dialogs.toml` | `dialogs.*`: dialog controls, More actions, pending-edit switch/close choices |
| `operations.toml` | `operations.*`: enrollment/setup, Apply, Verify, repair, removal, profile deletion, recovery and save-only bodies/results/consent |
| `progress.toml` | `progress.*`: GUI stage/status labels, transfer presentation and progress-detail headings |
| `diagnostics.toml` | `diagnostics.*`: safe reporting guidance, host comparison, app-owned technical-detail explanations |
| `credentials.toml` | `credentials.*`: native captions, sign-in explanation/wrapper and cancellation presentation |

The independent code contract records allowed/required files and exact required keys. LANG-003 assigns final keys to inventory rows; later extraction batches may activate contract entries incrementally, but every active contract must have an exact matching resource set. Inventory entries not yet migrated remain explicit pending rows. No empty files/optional fallback keys are used to disguise coverage.

Key segments match `[a-z][a-z0-9]*(?:_[a-z0-9]+)*`; keys have two or more dot-separated segments and their first segment equals the feature. Use meaning and branch, for example `operations.verify.review.off`, `operations.verify.review.running` and `operations.recovery.blocked.result`. Do not use English sentences, line numbers, temporary inventory variants or file/function locations as final keys. Share a key only when meaning, parameters and presentation constraints agree; similar words in distinct consent/results contexts may need different keys.

Developers own keys, schema, metadata, parameter bindings, independent contracts and call sites. Editors change `text` only after the verbatim migration gate. Until then, extract current English exactly, including existing spelling; the `en-GB` directory does not authorize an early editorial pass. Generated files are never edited, installed as resource files or treated as another English source.

## Strict TOML schema

Each file contains exactly `schema`, `locale`, `feature` and `messages`. `schema = 1`, `locale = "en-GB"`, and `feature` equals the file stem. TOML message keys are quoted dotted strings, not implicit nested message tables. A message entry contains these fields:

| Field | Required type / meaning |
|---|---|
| `text` | String; exact decoded presentation template, nonempty after Unicode whitespace checking |
| `screen` | Nonempty string identifying actual consuming surface(s) |
| `state` | Nonempty string describing the selected branch/state |
| `audience` | Enum: `operator` or `administrator`; the latter denotes technical detail audience, not execution authority |
| `intent` | Nonempty string explaining what the reader needs to understand or do |
| `consequences` | Nonempty string recording the actual effects, exclusions or uncertainty to preserve; use `No effects; presentation only.` for neutral labels |
| `severity` | Enum: `neutral`, `information`, `warning` or `error`; editorial metadata never selects dialog kind or result classification |
| `placeholders` | Array of inline tables with exactly `name` and `type`; empty array for fixed text |
| `space_hint` | Nonempty advisory string describing wrapping/width/API surface; not a numeric rejection threshold |

Unknown root/entry/placeholder fields are errors. Metadata values and placeholder declarations must equal the code-owned contract; placeholder table order is immaterial and names are unique. Missing metadata is not silently defaulted. There are no resource-controlled `allow_empty`, parameter-length overrides, formatting flags, executable expressions, includes or fallback locales.

All resource files are strict UTF-8 without a BOM. Reject malformed UTF-8, duplicate TOML tables/fields, duplicate keys across files, unknown files/keys, missing required files/keys, wrong feature assignment and unsupported schema/locale. Discover lowercase `.toml` regular files at the feature directory's top level; reject unexpected non-TOML files, subdirectories and links/reparse points. Keep the editor guide/screenshots elsewhere.

Do not trim or normalize decoded `text`: retain spaces, punctuation, Unicode and line breaks byte-for-byte. Validate emptiness without modifying valid text. Reject NUL and control characters other than tab, LF and CR in templates. Empty string sentinels for allowed actions, empty selection, unset values, cleared dialogs and absent optional messages stay code-owned. Optional presentation is represented by an explicit branch/`Option`, not a required key returning blank text.

Prefer basic TOML strings with explicit `\n`, `\r` or `\t` when whitespace matters. TOML multiline strings may trim the first newline and line-continuation whitespace; the decoded value, not source-file indentation, is the baseline. Do not assume CRLF resource-file bytes imply CRLF output. Use explicit escapes for output CRLF when the baseline requires it. No blanket whitespace reformatting accompanies extraction.

## Named placeholders and formatting

Templates use `{name}` with placeholder names matching `[a-z][a-z0-9]*(?:_[a-z0-9]+)*`. Literal braces are `{{` and `}}`. A single unmatched brace, an undeclared placeholder, an unused declaration, `{}`, `{name:format}`, dotted/indexed fields or expressions are errors. No template references another resource key. Reordering is allowed; each required parameter must occur at least once. Repetition is allowed unless the independent per-key contract sets a stricter occurrence limit (for example an exactly-once required consent fact). The maximum/minimum occurrence constraints are code-owned, not resource metadata.

The shared validator parses decoded templates into literal and parameter segments once. Generation emits those segments as append operations; each parameter is supplied once as an argument and may be appended repeatedly without reevaluating it. Never implement formatting as chained `replace` calls or run a formatted result through a template parser. Inserted `VM {final_power}`, quotes, backslashes, newlines or code-like strings remain data. Literal `{{name}}` produces `{name}` and is not expanded again.

| Parameter type | Generated Rust argument / output contract |
|---|---|
| `text` | Borrowed `&str` from a documented application field such as a VM/GPU name, ID or path; no generic error/adapter-output argument |
| `u64` | `u64`, unsigned base-10 integer with no grouping or locale conversion |
| `usize` | `usize`, unsigned base-10 count/index; do not narrow to Slint `int` for formatting |
| `i32` | `i32`, signed base-10 integer, for already permitted numeric facts |
| `message` | Borrowed opaque `UiMessage` produced by catalogue accessors; composed purpose/lifecycle/selected safe guidance, not arbitrary adapter prose |
| `formatted` | Borrowed opaque `DisplayValue` created by code-owned numeric/unit/typed-fact presenters; retains exact existing byte triples, decimal GB, GiB, percentage, hexadecimal or debug representation |

All accessors return `UiMessage`: fixed text borrows a static decoded literal; parameterised output owns the composed string. `UiMessage` exposes display borrowing/owned conversion but no public unchecked string constructor. `DisplayValue` likewise has only code-owned constructors for allowed formatting categories; an error string cannot be wrapped as a safe value at a call site. Generated signatures use names/types from the independent contract, with deterministic contract argument order, not edited template order.

The `text` type does not itself certify safe input. Each contract parameter records its exact source and preparation rule; developers must preserve existing sanitisation/redaction at that boundary. Safe native codes come from typed numeric facts. Do not add passwords, vault contents, raw Windows/driver/process/parser errors or arbitrary adapter output as any parameter type. No validator can prove truthful consequences or detect secrets introduced by an unsafe caller.

Booleans, power/availability/resource display choices and singular/plural wording are selected by typed Rust state and catalogue submessages. Resource/WMI identities and enum wire/debug names stay immutable. Code retains numeric rounding, unit conversion and requested-byte semantics. Move only the minimum shared formatting needed for library callers into a reusable presenter; do not duplicate the memory or transfer arithmetic. Fixed text and dynamic templates use the same catalogue; Slint never parses resources or expands placeholders at runtime. The four existing Slint compositions use generated pure concatenation functions from the validated segments. Their arguments remain existing field/default/range strings or the existing nonnegative Slint model length; no Rust `usize` or `u64` is narrowed for formatting. Rust keeps its typed accessors for these keys. Other numeric values remain Rust-prepared; the validator rejects `u64` Slint function arguments.

## Independent code contract

LANG-007 implements a Rust contract in [the validation/generation tool](../tools/ui-text/src/contract.rs), separate from all resource files. For each key it records:

- Required owning feature/file, semantic key and generated Rust/Slint names; explicit fixed-versus-parameterised consumer kind.
- Exact non-text metadata from the schema above, including protected consequence/severity context.
- Ordered parameter names/types, allowed occurrence range, source/preparation notes, and any proven maximum UTF-16 units for a bounded native argument.
- Fixed-text/nonempty requirements and any genuine code-owned API budget. Advisory layout guidance does not become a hard limit.

Validation compares resources with that independent contract. An editor changing `text` and `placeholders`, `space_hint` or severity together cannot redefine accepted parameters or bypass native bounds; changing the contract requires developer source review. Metadata does not prove that new wording retains its meaning: consent, uncertainty, final power and next-step accuracy still require semantic review.

Derive Rust nested modules from dot segments; use snake_case leaf functions. Flatten segments for Slint properties with `-` separators. Validate both complete generated symbol spaces, including module/function/reserved-name conflicts and Slint's hyphen/underscore name equivalence. For example `operations.a.b_c` and `operations.a_b.c` must be rejected if they flatten to one property name. Do not rename colliding identifiers automatically or return the wrong message. LANG-003 chooses final unique keys before bulk extraction.

The contract contains no fallback English. The resource text is the only shipped wording source; historical comparison fixtures can retain old output explicitly as tests/evidence, not alternate runtime text.

## Grounded examples

These are design examples, not resource files already consumed by the application. Wording comes from current producers. Final inventory-to-key assignment belongs to LANG-003.

Fixed `operations.toml` entry from the Verify title in [wording.rs](../src/gui/wording.rs):

```toml
schema = 1
locale = "en-GB"
feature = "operations"

[messages."operations.verify.title"]
text = "Verify GPU setup"
screen = "Verify review and operation dialog"
state = "Verify selected; title only"
audience = "operator"
intent = "Name the reviewed current-setup check."
consequences = "Title only; never selects an operation or grants approval."
severity = "information"
placeholders = []
space_hint = "Dialog heading or action button; inspect long wording at supported width/scaling."
```

Parameterised consent from `wording::apply_consent`, in the same file:

```toml
[messages."operations.apply.power_consent"]
text = "Approve reviewed starts/shutdowns; final state after successful completion: {final_power}."
screen = "Apply review consent"
state = "Reviewed power changes require consent"
audience = "operator"
intent = "Approve the reviewed lifecycle and disclose its successful final power."
consequences = "Use the plan's final power; success is conditional, and the checkbox is separate authority."
severity = "warning"
placeholders = [{ name = "final_power", type = "message" }]
space_hint = "Wrapped consent text; do not omit the conditional final-state fact."
```

The code contract requires `final_power` exactly once; the presenter selects the current surface's `Off`, `Running` or `Unknown` catalogue submessage from `plan.preview.restore_power`. It does not use edited text to choose power. Other surfaces currently say `Stopped`; that distinct baseline stays distinct.

Multiline native `credentials.toml` entry from `credentials::guest_prompt_message`:

```toml
schema = 1
locale = "en-GB"
feature = "credentials"

[messages."credentials.guest_sign_in.message"]
text = "Sign in to {vm_name}\n\nEnter the Windows administrator username and password you use inside this virtual machine. The app uses them for the guest steps you approve in review.\n\nRemember is optional. Leave it unchecked to use your details just this once; selecting it saves them securely on this computer for this VM."
screen = "Native Windows guest credential prompt; explanation dialog body"
state = "No saved opt-in credential; explanation precedes input"
audience = "operator"
intent = "Identify the guest account and explain optional local storage."
consequences = "Approved guest steps only; Remember is opt-in for the current user and exact VM."
severity = "information"
placeholders = [{ name = "vm_name", type = "text" }]
space_hint = "Native message budget is code-owned; explanation and prompt must remain readable."
```

The parameter source is the existing name filter: remove control characters and take at most 256 Unicode scalar values. That bound can occupy up to 512 UTF-16 code units. Preserve it; do not truncate the composed consent/instruction text. The separate fixed native caption is `Hyper GPU Support - Sign in`; the MessageBox caption is `Sign in to your virtual machine`. Its wrapper adds exactly `\n\nChoose OK to enter your details. Cancel returns to the app.` after the generated message, using a `message` parameter. The native cancellation sentinel and vault key are never catalogue entries.

The Windows `CREDUI_INFOW` API specifies caption/message limits. The pinned `windows` 0.62.2 bindings expose `CREDUI_MAX_CAPTION_LENGTH = 128` and `CREDUI_MAX_MESSAGE_LENGTH = 1024`. Code-owned budgets conservatively reserve one UTF-16 unit for termination: validate native caption payload plus terminator against 128, and worst-case formatted prompt payload plus terminator against 1024. Count literal segments plus every occurrence of each bounded argument; repeated `vm_name` consumes its bound each time. Unknown/unbounded native parameters are rejected until given a proven producer bound. These are API budgets, not claims about visual fit. [Microsoft CREDUI_INFOW documentation](https://learn.microsoft.com/en-us/windows/win32/api/wincred/ns-wincred-credui_infow).

The explanation MessageBox does not inherit the Credential UI message limit merely because it reuses the message. Its layout remains an advisory/visual check. The current credential test's `<400` assertion is an existing wording expectation, not an API bound; LANG-003 records it separately, and deliberate editorial changes must not be rejected solely by that obsolete expectation.

Placeholder example: a template `Recorded name: {vm_name}; literal {{final_power}}` with `vm_name = "VM {final_power}"` yields `Recorded name: VM {final_power}; literal {final_power}`. Neither occurrence is recursively expanded. This illustrates grammar only and is not a new application message.

## Build and consumer integration

The [tools/ui-text crate](../tools/ui-text/README.md) now provides the reusable validation/generation library and standalone validator binary. The root package declares an explicit path build dependency on that library. Its TOML/Serde dependencies reuse the root lockfile's resolved versions (currently TOML `0.9.12+spec-1.1.0`, Serde `1.0.229`); adding build/standalone entries must not upgrade existing dependencies. Root `toml = "0.9.8"` is a compatible requirement, not the resolved version. The standalone tool has an isolated workspace/lockfile and no dependency on the product, Slint, Windows backend or resources at compile time. Keep its dependency versions aligned with the root lockfile.

This makes TOML parsing explicitly available to the build without relying on the product's runtime dependency. Build and standalone both call the same library functions for file discovery, schema/contract validation, template parsing and deterministic generation. The standalone accepts a catalogue directory and validates it at execution time, so malformed resources do not stop compilation of the validator itself. The tool is contributor tooling, not a new installed product binary or privileged runtime loader.

[build.rs](../build.rs) implements the representative integration as follows:

1. Discover the resource directory and contract inputs; emit Cargo rebuild directives for the directory, each discovered file and code contract/tool sources. Directory tracking catches additions/deletions. Required directory/file disappearance is a clear build error.
2. Validate the whole active catalogue and contracts before creating usable generated outputs. Report sorted, actionable file/key/field errors; a failed catalogue stops the build, with no stale generated output accepted as fallback.
3. Generate UTF-8 Rust `ui_text.rs` and Slint `ui-text.slint` into `OUT_DIR`. Sort files, keys and contract arguments; use LF output and no timestamps, absolute paths or machine identifiers. Rewrite output only when bytes differ. No output is checked into the catalogue directory.
4. Add `OUT_DIR` to `slint_build::CompilerConfiguration::with_include_paths`, retaining the current `fluent` style and `src/gui/ui/app.slint` entrypoint. The pinned 1.18.1 compiler implements this include-path API. Components import `UiText` from the generated `ui-text.slint` file rather than spelling an absolute generated path into tracked Slint.
5. Include generated Rust through a public library presentation module (proposed `src/ui_text.rs`). The application GUI, library reporting/planning and Windows credentials reuse those accessors. Generated Rust contains literals and append operations only, with no TOML/filesystem parsing during refresh or product startup.

All generated Rust functions use fixed validated keys and typed arguments, never runtime key lookups/maps. Shared CLI presentation uses the same accessors where it already shares reporting/planning/credentials. Immutable worker progress, diagnostic envelopes, stage classification inputs, CLI flags, schema and storage fields remain the existing formats. Where a producer has mixed wire/display use, LANG-013 preserves the immutable original and provides a separate GUI presenter; do not edit worker strings to perform catalogue migration.

Generate the Slint global with fixed-text properties bound to decoded, escaped literals at declaration. Static components use `UiText.operations-verify-title`, for example. Parameterised templates are formatted in Rust and assigned through existing properties/models; no Slint callbacks, runtime key lookup or `replace-all` template engine are needed.

Globals are per component/window instance, not shared across independent windows. Literal defaults therefore provide the catalogue immediately for every instance and require no host-side initialization loop or initial blank-value fallback. Re-export the global from the entrypoint if Rust global access is required by integration/tests; there is no need to expose mutators for ordinary presentation. LANG-004 must compile and test the chosen read-only property direction, imports/re-export, hidden/default bindings and two independently created instances without launching the live GUI. LANG-004 compiles this read-only global on 1.18.1 and checks two headless instances without host initialization. [Slint globals documentation](https://docs.slint.dev/docs/slint/guide/language/coding/globals/).

## Generated escaping and failure policy

Decode TOML and tokenize placeholders before code generation; never emit resource text as source syntax. Rust literal encoding escapes quotes, backslashes, newlines, tabs/CR and control characters using Rust syntax. Do not choose raw-string delimiters from resource text or accept format-string expressions supplied by the editor.

For Slint fixed text, first collapse catalogue `{{`/`}}` into literal braces. Emit quote/backslash as escaped literal characters; use `\n` for LF and `\u{...}` for tab/CR or other permitted scalars needing an escape. Literal `{` is plain text in Slint; only unescaped `\{...}` invokes interpolation. Escaping every resource backslash before emission prevents resource text such as `\{root.action(...)}` becoming an expression. Slint does not accept every Rust escape, so do not reuse a Rust/JSON encoder as its serializer. Preserve other Unicode scalars exactly. [Slint string syntax](https://docs.slint.dev/docs/slint/reference/property-types/strings/).

Validation/generation fails for malformed input, missing/duplicate/unknown entries, metadata/type/placeholder mismatch, symbol collisions, forbidden emptiness/control characters and native budget overflow. No blank text, wrong-message fallback, partial catalogue, recursive expansion or last-known generated file is used to recover. Failures include key and filename, not arbitrary runtime values. A semantic wording mistake can still pass structural validation and must be caught in review.

LANG-007 tests invalid fixtures and adversarial quotes/braces/slashes/newlines/Unicode, deterministic order, duplicate cross-file keys, changed metadata/contract mismatch and native worst-case length. LANG-004 proves compilation-safe generation and TOML-only rebuild propagation to Rust and Slint, directory addition/deletion invalidation and restored verbatim wording. These checks do not require guest/VM/GPU effects or a live GUI launch.

## Editor workflow and validation status

Editors receive the catalogue and context after LANG-009. Edit only `text`; preserve placeholders, keys, metadata, severity, consent, conditional success/unknown wording and the distinction between copying, preparation, verification and saved permission. Report ambiguous meaning or clipping risk rather than revising operation semantics. British English editorial improvements belong to LANG-010; key/source changes return to the developer.

The standalone command now targets the independent tool manifest and resource directory, using the exact code-owned active set. [Executable commands and validation scope](../tools/ui-text/README.md) are documented; LANG-004 adds its shared checks to [the existing non-hardware gate](../scripts/testing/check.ps1), including resource/contract hashes in evidence inputs. LANG-008 supplies actual editor validation/rebuild commands and screenshots/context available at handoff.

Design/source review passed using current producers, Cargo manifest/lockfile, pinned Slint build API/docs and Windows bindings/docs. `python -B -X utf8 local/evidence/lang-002/review.py` passed guide links/whitespace, three TOML example blocks, required fields/parameter declarations, verbatim source comparison and native worst-case UTF-16 arithmetic (818 units including termination for the prompt example). `git diff --check` passed. `python scripts/testing/check-docs.py` failed on the same 22 links to deleted handbook pages; no unrelated files were restored or changed to resolve them.

No product compilation, generated prototype, Rust test execution, GUI/native prompt, guest access, protected write, VM/GPU effect or installation ran. These example checks do not validate a formatter/generator implementation, which does not exist yet. The subsequent [LANG-003 baseline](UI_TEXT_BASELINE.md) records final mapping and source-derived expectations; current next work is in the [backlog](backlog.md#next-implementation-action).


## LANG-007 implemented checks

The isolated tool accepts the nine-key baseline under `tests/fixtures/ui-text/validator/baseline` with strict UTF-8 TOML and independently fixed metadata/types/occurrences/native budgets. It has no fallback English in its contract, no resource input at tool compilation time, and no dependency on product or Slint. Generated output is deterministic and uses separate Rust/Slint serializers. LANG-004 now proves generated compilation and representative consumer integration, as recorded below.

```powershell
cargo test --locked --manifest-path tools/ui-text/standalone/Cargo.toml
cargo run --locked --manifest-path tools/ui-text/standalone/Cargo.toml -- resources/i18n/en-GB
cargo run --locked --manifest-path tools/ui-text/standalone/Cargo.toml -- resources/i18n/en-GB --generate local/evidence/ui-text-generated
python -B scripts/testing/test_check_ui_copy.py
python -B scripts/testing/check-ui-copy.py --json
```

The nine Rust tests exercise schema/encoding/file/key failures, metadata/type changes, grammar/occurrences, generated name collisions, deterministic/adversarial source serialization, native UTF-16 boundaries and filesystem/CLI failure publication. A temporary Windows junction verifies reparse rejection; it does not access guests or protected data. Two Python tests exercise the copy scan, including Rust test exclusion. Known active baseline literals in tooling are hard errors; product findings stay advisory under reviewed pending-extraction exceptions. The scan does not replace inventory reconciliation or semantic wording review.

Focused checks passed on 13 October 2026; repository documentation checking still fails on 22 existing deleted-handbook links. This paragraph records the LANG-007 boundary before integration; the subsequent LANG-004 results are recorded below.


## LANG-004 representative integration

The active product resources are now `resources/i18n/en-GB/{common,operations,credentials}.toml`, with the nine-key independent contract. The build calls the shared validator before emitting `OUT_DIR/ui_text.rs` and `OUT_DIR/ui-text.slint`; failed validation stops compilation instead of accepting stale generated files. Required file/directory and tooling-source changes trigger rebuilds. No runtime filesystem/TOML access is added to product presentation.

The root build dependency uses `tools/ui-text/Cargo.toml`. The isolated CLI uses `tools/ui-text/standalone/Cargo.toml` and its own aligned lockfile, compiling the same source/test files without product or Slint dependencies. Root default-member builds retain the existing product binaries. No existing dependency version was upgraded.

The Verify title is bound in Rust and Slint, including hidden dialog action paths. Apply consent takes a typed final-power catalogue message. Native prompt captions, the guest explanation and wrapper use typed accessors after the existing 256-scalar/control-character name filter. Cancellation, account prompts, vault opt-in and reviewed effects are unchanged. Shared power labels and initial unknown display defaults use the catalogue; remaining source copy is pending bulk extraction. English expectations stay in TOML fixtures, not duplicate Rust assertions.

```powershell
cargo test --locked --test ui_text
cargo test --locked --bin hyper-gpu-support gui::catalogue_tests
python -B scripts/testing/test-ui-text-rebuild.py
powershell -NoProfile -File scripts/testing/check.ps1
```

The rebuild script is an explicit contributor proof: run it alone. It temporarily edits only prepared resource text, tests generated Rust output against current resource bytes and Slint defaults in two headless instances, proves unknown-file addition and required-file deletion invalidate builds, and rejects malformed templates. It restores the original resource bytes in `finally`, then rebuilds/verifies the verbatim baseline. Adversarial quotes, backslashes, literal braces, Unicode and newlines remain data. No live application, renderer, native prompt or backend is launched.

The non-hardware gate now runs the isolated validator tests, active resource validation and copy scan before its existing stages. Its evidence hashes include resources, tool contracts/manifests/lockfiles and tests. Rendered fit, focus/interaction and live native prompts remain separately authorized later acceptance; a headless getter check does not prove appearance.


LANG-004 final automated results, 13 October 2026: `cargo fmt --all -- --check`, `cargo test --locked --all-targets`, strict `cargo clippy --locked --all-targets -- -D warnings`, product binary builds and API documentation generation all passed against the completed source. The final generator revision also passed the restored baseline and resource-only rebuild proof. The standalone validator's nine tests and two copy-scan tests passed through the gate. The complete `check.ps1` run still failed at Documentation with the existing 22 links to deleted handbook pages; its later stages remained Not run in that gate report and were executed separately. This does not close L2 or establish rendered/native/live behavior.

## Editor handoff

LANG-008: use the active entries in all ten TOML feature files under `resources/i18n/en-GB/`. Historical paragraphs above describe earlier checkpoints. The validator now requires the expanded product catalogue, so the old nine-key fixture alone is not valid standalone CLI input. Generated fixed accessors also expose static constants for existing borrowed-string presenters; both forms derive from the same validated text. Independent fixed/dynamic contracts are developer-owned files beside `contract.rs`, included at tool compilation time; they contain no fallback wording.

Until LANG-009 passes, preserve the original English. An editor receives the catalogue and this guide, then changes only `text` fields in those feature files. Preserve every other field, key, placeholder name/type, effect, uncertainty, consent fact, final-power condition and next action. Screens/state/intent/consequences describe the consuming branch. `space_hint` flags a layout concern; it is not proof that a sentence fits. For each changed entry, record its key, reason for editing, and any unresolved meaning or length question. Put review notes outside the resource directory.

Use British English consistently during the later editorial pass. Use **VM** for the existing virtual machine, **GPU support** for the reviewed attachment/settings workflow, **VRAM request** for requested provider values, **Verify GPU setup** for the current reviewed guest check, and **saved settings** for committed intent. A previous check is historical; copied files are not verified setup; saved intent is not an observation or authorization. Do not describe a request as reserved or isolated VRAM. Preserve named actions where the sentence directs the operator to a specific button.

Walkthrough: `operations.verify.title` is fixed heading/button copy and cannot select Verify. `credentials.guest_sign_in.message` inserts the filtered VM name once as data and explains separate, optional local remembering; keep `{vm_name}` intact and respect its native bound. `operations.apply.power_consent` is a warning with exactly one `{final_power}` selected from the reviewed plan; retain the condition of successful completion. Reordering placeholders is allowed within the independent occurrence contract, but neither removing a consequence nor changing severity is editorial work. Escaped `{{literal}}` remains literal text, and inserted braces never trigger another substitution.

Example editorial request: “After the verbatim gate, improve clarity and British English in the selected `text` fields. Preserve placeholders and all operation/uncertainty/consent facts. Return a resource-only diff plus key-specific ambiguity and length notes. Leave uncertain entries unchanged for semantic review.” This does not authorize another agent or live effects.

Validate and review from the repository root:

```powershell
git diff -- resources/i18n/en-GB
cargo run --locked --manifest-path tools/ui-text/standalone/Cargo.toml -- resources/i18n/en-GB
python -B scripts/testing/check-ui-copy.py
powershell -NoProfile -File scripts/testing/check.ps1
```

Codex reviews the diff against actual operation branches, validates, rebuilds, and arranges separately authorized affected-surface checks. During migration, executable golden tests intentionally reject changes from the historical baseline. After explicit editorial approval, update only expectations for the deliberately changed keys while retaining semantic/control regressions and the historical extraction artifacts. A passing validator or build does not approve changed meaning or prove wrapping, clipping, focus, scrolling or native appearance.

Future features follow the same handoff: the developer adds a semantic key, independent contract, safe typed binding, context and inventory disposition; the editor changes text; Codex reviews meaning and automated checks; affected surfaces receive visual/interaction review at the milestone. Existing features do not acquire authority from edited text. Keep new in-scope wording in the catalogue and record specific exceptions without blocking unrelated development on an editorial pass.

Current extraction checkpoint, 13 October 2026: the full non-hardware gate and restored resource-only rebuild proof passed against the expanded 408-key catalogue. The 381 fixed bindings and 18 data-only dynamic keys execute against frozen historical fixtures; two headless windows exercise the four Slint compositions and edited inserted data. The guide walkthrough and all documentation links pass. Full dynamic/reporting extraction, full clean-build migration and rendered/native acceptance remain pending; the editorial phase has not started. `dashboard.memory.capacity_button` retains its historical key name: its current input is the labelled range-start endpoint used by the existing button, not a new maximum-capacity action.
