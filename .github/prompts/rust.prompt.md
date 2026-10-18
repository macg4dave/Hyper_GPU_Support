# Rust

Follow [AGENTS.md](../../AGENTS.md). Apply when writing or reviewing Rust.

## Engineering Principles

**Primary objective: Build the simplest maintainable implementation that meets the application's actual requirements.**

1. **Reuse before creating.** Search for existing functionality before adding new functions, modules, helpers or state.
2. **Prefer simplification.** Modify or remove existing code when appropriate rather than adding another layer.
3. **One source of truth.** Do not duplicate application state, validation rules or business logic.
4. **Avoid speculative engineering.** Do not implement hypothetical requirements, compatibility layers or recovery mechanisms without an identified need.
5. **Keep workflows straightforward.** Operations should follow a clear sequence without unnecessary indirection.
6. **Remove obsolete code.** When replacing functionality, remove unused implementations and redundant paths.
7. **Justify abstractions.** New abstractions must solve a real maintenance or reuse problem, not merely reorganise code.
8. **Keep the GUI separate from operational logic.** The GUI should present state and invoke operations, not duplicate their implementation.
9. **Fix causes, not symptoms.** Do not add workarounds that conceal underlying architectural problems.
10. **Maintain proportional complexity.** A small feature or bug fix should not normally require a major expansion of the codebase.

For significant changes, review the existing implementation and propose the simplest approach before coding.

After changes, report what was added, removed, reused and simplified.

Do not perform live integration tests unless explicitly requested.

## Rust guidance

- Use small, cohesive modules with one responsibility. Split large operation/controller files along domain boundaries; keep a readable orchestrator. Avoid monolithic modules, arbitrary file-size rules, and abstraction for its own sake.
- Prefer plain names, explicit inputs/results, short functions, and readable control flow. Reuse existing types/interfaces; keep public surface minimal.
- Document public contracts. Comment intent, invariants, units, non-obvious decisions, and failure/recovery behavior; do not narrate obvious syntax. Keep comments accurate.
- Prefer borrowing and concrete types. Clone or share ownership for a clear reason. Use checked conversions/arithmetic at external boundaries.
- Model operations and failures with typed enums/results. Propagate actionable context; no panic/unwrap for expected runtime failures. Never swallow errors or report incomplete work as success.
- Keep unsafe Windows/FFI code narrow, with safety comments covering validity, ownership, lifetime, and cleanup. Respect existing lints; no broad suppression.
- Keep effects behind existing backend interfaces. Separate planning, effects, readback, and persistence; make each stage diagnosable without secrets.
- Add thorough inline `#[cfg(test)]` operation tests: ordinary paths, stage order, boundary values, rejected input, partial failure, stale state, interruption, and recovery. Use isolated backend doubles and assert results, state, and permitted/forbidden effects.
- Keep tests deterministic, focused, and independently runnable with descriptive names and useful assertions. Self-tests must not mutate real VMs/GPU or run at product startup.
- Reuse useful coverage; avoid tests that merely copy implementation. Test execution follows the root policy. Use pinned toolchain/dependencies and existing formatting/lint conventions.
