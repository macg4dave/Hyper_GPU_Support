---
agent: agent
description: Refactor code while preserving behavior and component boundaries
---

Perform the requested refactor.

- Establish current observable behavior from code, tests, and documentation.
- Keep the patch narrow and separate formatting-only churn.
- Preserve public APIs, errors, data formats, host/guest behavior, and component
  boundaries.
- Avoid new dependencies, global state, unsafe lifetime/ownership assumptions,
  and unrelated modernization.
- Keep core logic separate from privileged Windows/Hyper-V operations and
  hardware side effects without adding cross-platform abstractions.
- Retain all existing tests and add coverage for behavior newly exposed by the
  refactor.
- Update architecture documentation only if ownership or boundaries change.
- Run focused build/tests and report exact results.
