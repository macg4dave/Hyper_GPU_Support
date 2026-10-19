# UI text and languages

All user-visible GUI wording comes from this folder. Each language is a directory named by its BCP 47 tag (for example `en-GB`, `fr-FR`), with one TOML file per feature (`common.toml`, `dashboard.toml`, …). Every language is compiled into the application; the Settings page switches between them at runtime (default: follow the Windows display language, falling back to British English).

`en-GB` is the reference catalogue. It defines every key, the arguments each message accepts and their types. Other languages only translate text.

## File format

```toml
# Plain message: section path + key, for example dashboard.actions.more_label
[actions]
more_label = "More actions…"

# Message with arguments (declared in en-GB only); {name} inserts an argument, {{ and }} are literal braces
[credentials.guest_sign_in.message]
args = ["vm_name: text"]
text = "Sign in to {vm_name}"

# Plural message: `plural` names the count argument; forms follow the language's CLDR rule
[vm_list.count]
args = ["count: usize"]
plural = "count"
one = "{count} VM shown"
other = "{count} VMs shown"
```

Argument types: `text` (plain data), `message` (another catalogue message), `formatted` (a code-formatted value), `usize`, `u64`, `i32`. Key segments use lowercase words joined by `_`; `text`, `args`, `plural` and the plural category names are reserved. `common.language.name` is the language's native name shown in the picker.

## Adding a language

1. Create a directory for the tag, for example `resources/i18n/fr-FR/`, with any of the reference feature files.
2. Translate the text, keeping each key and using exactly the reference placeholders. Do not copy `args` or `plural` lines; they are declared once in `en-GB`. Plural messages use the forms of the language (French: `one`, `other`; Russian: `one`, `few`, `many`, `other`).
3. Set `[language] name` in `common.toml` (for example `name = "Français (France)"`).
4. Build. The build fails on unknown keys or files, placeholder mismatches, invalid templates or plural forms the language does not use. Untranslated messages fall back to British English with a build warning.

Plural rules are defined in [plural.rs](../../tools/ui-text/src/plural.rs). A language without a rule fails the build until one is added there. Wording is never read from disk at runtime and installed binaries do not use this folder. The generator is described in [tools/ui-text](../../tools/ui-text/README.md).

Backend and worker messages (errors from protected operations, guest checks and the administrator command line) are not yet in the catalogue and remain English.
