# Local development data

This tree is for machine-local Windows media, VM disks, driver inputs and test
evidence. Git tracks only this guide and the empty-directory markers. Everything
else below `data/` is ignored; still inspect staged files before every commit.

| Path | Purpose |
|---|---|
| `iso/original/` | Official installation media, retained byte-for-byte and recorded by source, edition/build and SHA-256. |
| `iso/modified/` | Evidence-triggered experiments only. The project baseline does not modify Windows installation media. |
| `images/golden/` | Shut-down, generalized and immutable parent VHDX files, versioned in stable subdirectories. |
| `images/disposable/` | Writable differencing VHDX files for the single enrolled test slot. |
| `drivers/host/` | Host driver installers/packages retained as provenance inputs, never release payloads. |
| `drivers/guest/` | Guest runtime/driver staging inputs and extracted payloads. |
| `staging/` | Regenerable temporary preparation files. |
| `logs/` | Machine-local diagnostic output. |
| `test-results/` | Hardware reports, manifests and workload evidence before redacted results are promoted to `docs/evidence/`. |

The repository-relative default root is `data/`. Large installations should put
the entire tree on suitable external storage and provide its absolute root in the
future versioned project configuration; individual leaf paths should derive from
that one root. Do not use a junction or symlink as an implicit relocation because
the privileged runner must validate one canonical parent/child/result root and
reject reparse-point escapes. Until the configuration/runner work exists, each
hardware task records the actual absolute roots it used.

Never place credentials, activation keys, certificates, signing keys, production
data, proprietary OS images or redistributable driver assumptions in Git. Cleanup
may delete `staging/`, logs, test results and an enrolled disposable child within
the authorized test mechanism. It must never delete or write an ISO in
`iso/original/` or a sealed parent in `images/golden/`.
