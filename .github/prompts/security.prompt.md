---
agent: agent
description: Review and harden host, guest, API, driver, and parsing boundaries
---

Review or harden the requested area.

Follow [AGENTS.md](../../AGENTS.md) and the engineering standards for
[errors and lifetimes](../../docs/ENGINEERING.md#errors-and-operation-lifetimes),
[dependencies](../../docs/ENGINEERING.md#toolchain-dependencies-and-features),
[testing](../../docs/ENGINEERING.md#testing), and
[required checks](../../docs/ENGINEERING.md#required-checks-and-ci).

Check for:

- untrusted API, guest, file, path, process, and device inputs;
- overflow, truncation, alignment, range, lifetime, and encoding errors;
- command injection, unsafe path handling, excessive privileges, and secret leaks;
- insecure loopback/API ownership assumptions and unexpected network exposure;
- weakened VM isolation, signing, Secure Boot, driver, or host boundaries;
- dependency, downloaded artifact, vendored code, and binary provenance risks;
- denial-of-service inputs, crashes, unchecked assumptions, and unsafe code;
- cancellation, timeout, partial-failure recovery, and resource cleanup behavior.

Keep fixes minimal and add a regression test for each fixed issue where feasible;
document alternative validation when a test is impractical. Preserve compatibility
where possible and update affected security/contract docs. Do not perform live
exploitation without explicit scope and approval; protected mutations follow the
repository approval rules. Report exact checks and remaining risks.
