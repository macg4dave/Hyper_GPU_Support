# Security and recovery review

Follow [AGENTS.md](../../AGENTS.md).

Trace untrusted input through authority, fixed elevation, peer authentication, artifact/path/ACL checks, fresh approval, locks, and effects. Examine admission, partial failure, disconnect, independent readback, publication, and recovery. Preserve separate setup/enrollment/effect/import domains; intent grants no authority. Check credential lifetime and secret-free diagnostics.

Report concrete source-backed findings and missing negative tests. Source review does not authorize installed IPC probes, protected writes, guest access, or live fault injection.
