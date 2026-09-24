---
agent: agent
description: Review and harden host, guest, API, driver, and parsing boundaries
---

Review or harden the requested area.

Check for:

- untrusted API, guest, file, path, process, and device inputs;
- overflow, truncation, alignment, range, lifetime, and encoding errors;
- command injection, unsafe path handling, excessive privileges, and secret leaks;
- insecure loopback/API ownership assumptions and unexpected network exposure;
- weakened VM isolation, signing, Secure Boot, driver, or host boundaries;
- dependency, downloaded artifact, vendored code, and binary provenance risks;
- denial-of-service inputs, crashes, unchecked assumptions, and unsafe code.

Keep fixes minimal, add regression coverage for each fixed issue, preserve
compatibility where possible, and update affected security/contract docs. Do not
perform live exploitation or protected host/VM mutation without explicit scope
and approval.

