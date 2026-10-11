# Rust

Follow [AGENTS.md](../../AGENTS.md). Apply when writing or reviewing Rust.

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
