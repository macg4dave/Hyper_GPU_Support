# Rust — code review

Use [rust.prompt.md](rust.prompt.md) for implementation rules. Review how the application **should work**, not merely how it was written.

- Trace real workflows: GUI → backend → privileged actions → readback → persistence. Check ordering, failure paths, stale state, and user-facing behaviour.
- Identify duplicate logic or state, redundant operations, unnecessary refreshes, excessive abstractions, and dead code.
- Check Rust correctness: ownership, concurrency, error propagation, input boundaries, FFI safety, and meaningful test coverage.
- Separate defects from style preferences. Rank findings by impact and cite the relevant files and lines.
- Fix confirmed issues using the shared implementation rules; do not redesign unrelated code.
- Report findings, fixes, removed complexity, verification, and outstanding risks concisely.
