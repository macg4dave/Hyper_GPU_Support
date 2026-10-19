# Rust

Follow [AGENTS.md](../../AGENTS.md). Apply when writing or reviewing Rust.

## Engineering Principles

Understand the affected workflow before changing code. Reuse or simplify existing implementations first.
Preserve required functionality, permissions, error handling, and recovery.
Keep one source of truth. Remove obsolete paths rather than adding another layer.
Separate GUI presentation from backend operations. Use cohesive modules and a readable orchestrator.
Comment contracts and non-obvious decisions, not obvious syntax.
Update focused, deterministic tests for changed behaviour. Tests must not affect real VMs, GPUs, or host settings.
Do not run live integration tests without explicit permission.
Briefly report what changed, what was removed, and what was verified.
For significant changes, review the existing implementation and propose the simplest approach before coding.

## Rust guidance

- Use small, cohesive modules with one responsibility. Split large operation/controller files along domain boundaries; keep a readable orchestrator. Avoid monolithic modules, arbitrary file-size rules, and abstraction for its own sake.
- Prefer plain names, explicit inputs/results, short functions, and readable control flow. Reuse existing types/interfaces; keep public surface minimal.
- Document public contracts. Comment intent, invariants, units, non-obvious decisions, and failure/recovery behavior; do not narrate obvious syntax. Keep comments accurate.
- Prefer borrowing and concrete types. Clone or share ownership for a clear reason. Use checked conversions/arithmetic at external boundaries.
- Model operations and failures with typed enums/results. Propagate actionable context; no panic/unwrap for expected runtime failures. Never swallow errors or report incomplete work as success.
- Keep unsafe Windows/FFI code narrow, with safety comments covering validity, ownership, lifetime, and cleanup. Respect existing lints; no broad suppression.
- Keep effects behind existing backend interfaces. Separate planning, effects, readback, and persistence; make each stage diagnosable without secrets.
- Reuse useful coverage; avoid tests that merely copy implementation. Test execution follows the root policy.

## Rust craft

- Make invalid states unrepresentable: newtypes for IDs/units/validated input, enums over boolean flags and stringly-typed state, `#[must_use]` on results callers must not drop.
- Errors: typed error enums per domain with `Display` that says what failed and where; convert with `From`/`map_err` adding context; use `?` rather than manual matching. `unwrap`/`expect` only for proven invariants, with the invariant stated in the `expect` message.
- Ownership: take `&str`/`&[T]`/`&Path` and `impl AsRef<_>` at inputs, return owned values; avoid `clone()` to satisfy the borrow checker—restructure instead. Prefer iterators and slice patterns over index arithmetic; use `checked_*`/`try_from` at untrusted boundaries.
- Concurrency: keep locks short and never hold one across blocking I/O or callbacks; prefer message passing to shared mutable state; make `Send`/`Sync` requirements explicit.
- Resources: wrap Windows handles/COM objects in RAII types whose `Drop` releases them; unsafe blocks stay minimal and each one carries a `// SAFETY:` comment.
- Tests: name the behaviour under test, cover failure and boundary paths, not only the happy path; tests are deterministic and use temp directories.
- Lints are signals, not obstacles: fix the code, never suppress to make a check pass (see [Honest engineering](../../AGENTS.md#honest-engineering-no-cheating)). Prefer the clippy-suggested idiom unless it hurts clarity, and say so if you deviate.
