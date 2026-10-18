# UI text catalogue and generation contract

The active catalogue contains 495 keys in ten feature files under `resources/i18n/en-GB/`. Generated Rust and Slint bindings supply compiled presentation text. Full dynamic/reporting extraction and rendered/native acceptance remain incomplete. The [active contract](../tools/ui-text/src/contract.rs) defines compiled keys; the extraction register has not been reconciled with all later additions; the [backlog](backlog.md#next-action) records current project work.

This page owns the schema, generation rules and editor handoff. Production resources and the independent code contract define the current catalogue.

## Internationalisation architecture

Only `en-GB` is available. [The locale loader](../tools/ui-text/src/locales.rs) discovers `resources/i18n/<locale>/<feature>.toml`, validates every discovered catalogue against the same code-owned contract, and requires a valid `en-GB` default. Each locale must contain the complete active file/key set with matching locale headers. Invalid catalogues fail the build; an unavailable exact requested tag resolves to `en-GB`. There is no per-message fallback or host-language selection.

[build.rs](../build.rs) compiles one resolved catalogue into Rust accessors/constants and the Slint `UiText` global. It tracks `HYPER_GPU_UI_LOCALE`, directory membership, required files and generator sources for rebuilds. Generated locale metadata exposes the selection. Installed presentation performs no resource parsing or filesystem reads; another build selection requires a rebuild.

Rust/native and Slint consumers share this generator, including named typed parameters and native text budgets. A second gettext catalogue would duplicate that contract. Additional languages, a selector and runtime switching require separate work.

Future translations must retain stable keys, metadata, types and occurrence constraints, translating only `text` in a complete locale directory. Validate the entire locale root and review the resulting display. Current numeric/unit formatting and singular/plural branches remain code-owned. There is no plural-form schema or automatic plural-rule engine; adding one requires a shared Rust/native/Slint contract using the original numeric count.

## Ownership and files

Active catalogue wording lives in `resources/i18n/en-GB/`. Use these flat feature files, with no nested directories or runtime loading:

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

The independent code contract records allowed/required files and exact required keys. Activate new entries incrementally, keeping an exact match between the active contract and resource set. Unextracted source text remains outside the active catalogue. No empty files/optional fallback keys are used to disguise coverage.

Key segments match `[a-z][a-z0-9]*(?:_[a-z0-9]+)*`; keys have two or more dot-separated segments and their first segment equals the feature. Use meaning and branch, for example `operations.verify.review.off`, `operations.verify.review.running` and `operations.recovery.blocked.result`. Do not use English sentences, line numbers, temporary inventory variants or file/function locations as final keys. Share a key only when meaning, parameters and presentation constraints agree; similar words in distinct consent/results contexts may need different keys.

Developers own keys, schema, metadata, parameter bindings, independent contracts and call sites. Editors change `text` only after the verbatim migration gate. Until then, extract current English exactly, including existing spelling; the `en-GB` directory does not authorize an early editorial pass. Generated files are never edited, installed as resource files or treated as another English source.

## Strict TOML schema

Each file contains exactly `schema`, `locale`, `feature` and `messages`. `schema = 1`, `locale` equals the enclosing locale directory (`"en-GB"` today), and `feature` equals the file stem. TOML message keys are quoted dotted strings, not implicit nested message tables. A message entry contains these fields:

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

Unknown root/entry/placeholder fields are errors. Metadata values and placeholder declarations must equal the code-owned contract; placeholder table order is immaterial and names are unique. Missing metadata is not silently defaulted. There are no resource-controlled `allow_empty`, parameter-length overrides, formatting flags, executable expressions, includes or fallback policy; the loader owns the British English fallback.

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

Generated message accessors return `UiMessage` (fixed entries also expose borrowed constants): fixed text borrows a static decoded literal; parameterised output owns the composed string. `UiMessage` exposes display borrowing/owned conversion but no public unchecked string constructor. `DisplayValue` likewise has only code-owned constructors for allowed formatting categories; an error string cannot be wrapped as a safe value at a call site. Generated signatures use names/types from the independent contract, with deterministic contract argument order, not edited template order.

The `text` type does not itself certify safe input. Each contract parameter records its exact source and preparation rule; developers must preserve existing sanitisation/redaction at that boundary. Safe native codes come from typed numeric facts. Do not add passwords, vault contents, raw Windows/driver/process/parser errors or arbitrary adapter output as any parameter type. No validator can prove truthful consequences or detect secrets introduced by an unsafe caller.

Booleans, power/availability/resource display choices and singular/plural wording are selected by typed Rust state and catalogue submessages. Resource/WMI identities and enum wire/debug names stay immutable. Code retains numeric rounding, unit conversion and requested-byte semantics. Move only the minimum shared formatting needed for library callers into a reusable presenter; do not duplicate the memory or transfer arithmetic. Fixed text and dynamic templates use the same catalogue; Slint never parses resources or expands placeholders at runtime. The four existing Slint compositions use generated pure concatenation functions from the validated segments. Their arguments remain existing field/default/range strings or the existing nonnegative Slint model length; no Rust `usize` or `u64` is narrowed for formatting. Rust keeps its typed accessors for these keys. Other numeric values remain Rust-prepared; the validator rejects `u64` Slint function arguments.

## Independent code contract

The validator uses an independent Rust contract in [the validation/generation tool](../tools/ui-text/src/contract.rs), separate from all resource files. For each key it records:

- Required owning feature/file, semantic key and generated Rust/Slint names; explicit fixed-versus-parameterised consumer kind.
- Exact non-text metadata from the schema above, including protected consequence/severity context.
- Ordered parameter names/types, allowed occurrence range, source/preparation notes, and any proven maximum UTF-16 units for a bounded native argument.
- Fixed-text/nonempty requirements and any genuine code-owned API budget. Advisory layout guidance does not become a hard limit.

Validation compares resources with that independent contract. An editor changing `text` and `placeholders`, `space_hint` or severity together cannot redefine accepted parameters or bypass native bounds; changing the contract requires developer source review. Metadata does not prove that new wording retains its meaning: consent, uncertainty, final power and next-step accuracy still require semantic review.

Derive Rust nested modules from dot segments; use snake_case leaf functions. Flatten segments for Slint properties with `-` separators. Validate both complete generated symbol spaces, including module/function/reserved-name conflicts and Slint's hyphen/underscore name equivalence. For example `operations.a.b_c` and `operations.a_b.c` must be rejected if they flatten to one property name. Do not rename colliding identifiers automatically or return the wrong message. Choose unique semantic keys before extraction.

The contract contains no fallback English. Active resources supply compiled catalogue wording; comparison fixtures are test inputs only.

## Native text limits

Native VM-name parameters use the existing filter: remove control characters and take at most 256 Unicode scalar values. That bound can occupy up to 512 UTF-16 code units. Preserve it; do not truncate the composed consent/instruction text. The separate fixed native caption is `Hyper GPU Support - Sign in`; the MessageBox caption is `Sign in to your virtual machine`. Its wrapper adds exactly `\n\nChoose OK to enter your details. Cancel returns to the app.` after the generated message, using a `message` parameter. The native cancellation sentinel and vault key are never catalogue entries.

The Windows `CREDUI_INFOW` API specifies caption/message limits. The pinned `windows` 0.62.2 bindings expose `CREDUI_MAX_CAPTION_LENGTH = 128` and `CREDUI_MAX_MESSAGE_LENGTH = 1024`. Code-owned budgets conservatively reserve one UTF-16 unit for termination: validate native caption payload plus terminator against 128, and worst-case formatted prompt payload plus terminator against 1024. Count literal segments plus every occurrence of each bounded argument; repeated `vm_name` consumes its bound each time. Unknown/unbounded native parameters are rejected until given a proven producer bound. These are API budgets, not claims about visual fit. [Microsoft CREDUI_INFOW documentation](https://learn.microsoft.com/en-us/windows/win32/api/wincred/ns-wincred-credui_infow).

## Build and consumer integration

The [tools/ui-text crate](../tools/ui-text/README.md) provides the reusable validation/generation library and standalone validator binary. The root package declares an explicit path build dependency on that library. Build and standalone manifests use aligned lockfiles; inspect those files for resolved dependency versions. The standalone tool has no product, Slint or Windows-backend dependency and reads resources only when executed.

This makes TOML parsing explicitly available to the build without relying on the product's runtime dependency. Build and standalone both call the same library functions for file discovery, schema/contract validation, template parsing and deterministic generation. The standalone accepts a catalogue directory and validates it at execution time, so malformed resources do not stop compilation of the validator itself. The tool is contributor tooling, not a new installed product binary or privileged runtime loader.

[build.rs](../build.rs) integrates the catalogue as follows:

1. Discover the resource directory and contract inputs; emit Cargo rebuild directives for the directory, each discovered file and code contract/tool sources. Directory tracking catches additions/deletions. Required directory/file disappearance is a clear build error.
2. Validate the whole active catalogue and contracts before creating usable generated outputs. Report sorted, actionable file/key/field errors; a failed catalogue stops the build, with no stale generated output accepted as fallback.
3. Generate UTF-8 Rust `ui_text.rs` and Slint `ui-text.slint` into `OUT_DIR`. Sort files, keys and contract arguments; use LF output and no timestamps, absolute paths or machine identifiers. Rewrite output only when bytes differ. No output is checked into the catalogue directory.
4. Add `OUT_DIR` to `slint_build::CompilerConfiguration::with_include_paths`, retaining the current `fluent` style and `src/gui/ui/app.slint` entrypoint. The pinned 1.18.1 compiler implements this include-path API. Components import `UiText` from the generated `ui-text.slint` file rather than spelling an absolute generated path into tracked Slint.
5. Include generated Rust through a public library presentation module ([`src/ui_text.rs`](../src/ui_text.rs)). The application GUI, library reporting/planning and Windows credentials reuse those accessors. Generated Rust contains literals and append operations only, with no TOML/filesystem parsing during refresh or product startup.

All generated Rust functions use fixed validated keys and typed arguments, never runtime key lookups/maps. Shared CLI presentation uses the same accessors where it already shares reporting/planning/credentials. Immutable worker progress, diagnostic envelopes, stage classification inputs, CLI flags, schema and storage fields remain the existing formats. Where a producer has mixed wire/display use, preserve its immutable control input and select display text separately; do not edit worker strings to perform catalogue migration.

Generate the Slint global with fixed-text properties bound to decoded, escaped literals at declaration. Static components use `UiText.operations-verify-title`, for example. Dynamic Rust messages are assigned through existing properties/models; the four simple Slint compositions use generated pure functions; no Slint callbacks, runtime key lookup or `replace-all` template engine are needed.

Globals are per component/window instance, not shared across independent windows. Literal defaults therefore provide the catalogue immediately for every instance and require no host-side initialization loop or initial blank-value fallback. Re-export the global from the entrypoint if Rust global access is required by integration/tests; there is no need to expose mutators for ordinary presentation. Headless tests check imports, read-only defaults and two independent instances without host initialization. [Slint globals documentation](https://docs.slint.dev/docs/slint/guide/language/coding/globals/).

## Generated escaping and failure policy

Decode TOML and tokenize placeholders before code generation; never emit resource text as source syntax. Rust literal encoding escapes quotes, backslashes, newlines, tabs/CR and control characters using Rust syntax. Do not choose raw-string delimiters from resource text or accept format-string expressions supplied by the editor.

For Slint fixed text, first collapse catalogue `{{`/`}}` into literal braces. Emit quote/backslash as escaped literal characters; use `\n` for LF and `\u{...}` for tab/CR or other permitted scalars needing an escape. Literal `{` is plain text in Slint; only unescaped `\{...}` invokes interpolation. Escaping every resource backslash before emission prevents resource text such as `\{root.action(...)}` becoming an expression. Slint does not accept every Rust escape, so do not reuse a Rust/JSON encoder as its serializer. Preserve other Unicode scalars exactly. [Slint string syntax](https://docs.slint.dev/docs/slint/reference/property-types/strings/).

Validation/generation fails for malformed input, missing/duplicate/unknown entries, metadata/type/placeholder mismatch, symbol collisions, forbidden emptiness/control characters and native budget overflow. No blank text, wrong-message fallback, partial catalogue, recursive expansion or last-known generated file is used to recover. Failures include key and filename, not arbitrary runtime values. A semantic wording mistake can still pass structural validation and must be caught in review.

Validator tests cover invalid fixtures and adversarial quotes/braces/slashes/newlines/Unicode, deterministic order, duplicate cross-file keys, changed metadata/contract mismatch and native worst-case length. Integration checks cover compilation-safe generation and TOML-only rebuild propagation to Rust and Slint, directory addition/deletion invalidation and restored verbatim wording. These checks do not require guest/VM/GPU effects or a live GUI launch.

## Editor handoff

Edit active entries in the ten TOML feature files under `resources/i18n/en-GB/`. Generated fixed accessors also expose static constants for existing borrowed-string presenters; both forms derive from the same validated text. Independent fixed/dynamic contracts are developer-owned files beside `contract.rs`, included at tool compilation time; they contain no fallback wording.

Preserve verbatim English until migration acceptance and an explicitly requested editorial pass. An editor receives the catalogue and this guide, then changes only `text` fields in those feature files. Preserve every other field, key, placeholder name/type, effect, uncertainty, consent fact, final-power condition and next action. Screens/state/intent/consequences describe the consuming branch. `space_hint` flags a layout concern; it is not proof that a sentence fits. For each changed entry, record its key, reason for editing, and any unresolved meaning or length question. Put review notes outside the resource directory.

Use British English consistently during the later editorial pass. Use **VM** for the existing virtual machine, **GPU support** for the reviewed attachment/settings workflow, **VRAM request** for requested provider values, **Verify GPU setup** for the current reviewed guest check, and **saved settings** for committed intent. A previous check is historical; copied files are not verified setup; saved intent is not an observation or authorization. Do not describe a request as reserved or isolated VRAM. Preserve named actions where the sentence directs the operator to a specific button.

`operations.verify.title` supplies copy and cannot select Verify. Native sign-in text inserts the filtered VM name as data and explains optional local remembering. Keep native bounds and mandatory consent facts intact. Reordering placeholders is allowed within the independent occurrence contract, but neither removing a consequence nor changing severity is editorial work. Escaped `{{literal}}` remains literal text, and inserted braces never trigger another substitution.

Review `git diff -- resources/i18n/en-GB`, then use the [validation commands](#validation-and-evidence).

Codex reviews the diff against actual operation branches, validates, rebuilds, and arranges separately authorized affected-surface checks. During migration, executable golden tests intentionally reject changes from the historical baseline. After explicit editorial approval, update only expectations for the deliberately changed keys while retaining semantic/control regressions and independent comparison fixtures. A passing validator or build does not approve changed meaning or prove wrapping, clipping, focus, scrolling or native appearance.

Future features follow the same handoff: the developer adds a semantic key, independent contract, safe typed binding, context and extraction coverage; the editor changes text; Codex reviews meaning and automated checks; affected surfaces receive visual/interaction review at the milestone. Existing features do not acquire authority from edited text. Keep new in-scope wording in the catalogue and record specific exceptions without blocking unrelated development on an editorial pass.

## Validation and evidence

Run checks from the repository root when validating a completed change:

```powershell
cargo run --locked --manifest-path tools/ui-text/standalone/Cargo.toml -- resources/i18n --locale en-GB
python -B scripts/testing/check-ui-copy.py
powershell -NoProfile -File scripts/testing/check.ps1
```

The gate includes validator/failure tests, resource validation, copy scanning and product checks. The separate `python -B scripts/testing/test-ui-text-rebuild.py` proof temporarily changes prepared resources, verifies rebuild propagation and restores original bytes in `finally`; run it alone. See [tool commands and test effects](../tools/ui-text/README.md).

Use [recorded gate/install evidence](ENGINEERING.md#optimization-validation-checkpoint) only for its captured inputs. Headless bindings do not establish wrapping, focus, scrolling or native appearance. Full migration and rendered/native acceptance remain pending.

The extraction register still lists 408 active keys while the current resources contain 495. Reconcile coverage against resources, the active contract and actual consumers before claiming migration completion; adding keys alone does not establish that all source text was extracted.

`dashboard.memory.capacity_button` retains its key name: its input is the labelled range-start endpoint, not a maximum-capacity action.
