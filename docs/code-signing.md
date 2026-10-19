# Code signing policy

Every official GitHub Release must contain a Windows x64 installer signed through SignPath Foundation. Local Cargo builds and local installers remain available unsigned for testing and for users building the source themselves. Signing failures never produce an unsigned official release. GitHub's automatically generated source archives contain source code and do not carry Authenticode signatures.

**Foundation approval is pending.** This repository does not claim an approved subscription or certificate. Do not publish the first official release until the Foundation accepts this project and the complete signing workflow succeeds. The Foundation currently requires a project to have already been released in the form being signed; ask the Foundation how this project can qualify while maintaining its signed-only release policy. Do not publish an unsigned bootstrap release to work around that requirement. Acceptance is the Foundation's decision. See its [conditions](https://signpath.org/terms.html) and [application](https://signpath.org/apply.html).

After acceptance, the required acknowledgement is: “Free code signing provided by [SignPath.io](https://about.signpath.io), certificate by [SignPath Foundation](https://signpath.org).” This acknowledgement is conditional until the service has actually been granted.

## People, review and privacy

The repository owner, [macg4dave](https://github.com/macg4dave), is the initial author, reviewer and proposed signing approver. Before enabling production signing, the owner must confirm these assignments in SignPath; additional trusted maintainers must be named here before receiving these roles. External contributions require maintainer review. Each signing request requires manual approval by a trusted maintainer in SignPath, including both signing passes for a release. Require multi-factor authentication for every maintainer's GitHub and SignPath access. Protect release tags and the default branch; require review of source, dependencies and build/workflow changes before tagging.

The application collects and transmits no personal data; see the [Privacy policy](privacy.md). SignPath receives build executables and GitHub origin metadata; see [SignPath's privacy policy](https://about.signpath.io/privacy-policy).

## Foundation eligibility and configuration

Before applying, review the current [Foundation conditions](https://signpath.org/terms.html) against the actual release, rather than treating this page as proof of eligibility:

- Confirm the MIT project licence and every dependency licence are acceptable. Slint is used under its Royalty-free Desktop, Mobile and Web Applications License option; preserve its required attribution and ask the Foundation explicitly whether the full Slint licensing arrangement satisfies its open-source and dual-licensing conditions. The package licence files are the authoritative inventory.
- Disclose the repository's modified, vendored `i-slint-backend-winit` repaint backport. Obtain a Foundation decision about upstream/fork requirements and signing the statically linked result. Do not claim to have met the visible-fork or upstream-signed-build conditions without that decision.
- Disclose Inno Setup's generated setup/uninstaller runtime and seek approval to sign these generated executables. Only this project's build outputs are submitted; third-party build tools are never submitted for signing.
- Confirm the project's functionality, system-change warning, install/update reset behaviour, uninstall instructions, [privacy policy](privacy.md), maintainer roles and downloadable release form meet the conditions. Maintain the existing explicit approval and recovery boundaries for VM/guest actions.

Create a SignPath project connected to the repository through the SignPath GitHub App and the predefined GitHub.com trusted build system. Production policies must restrict the repository, approved workflow and release tags, require manual maintainer approval, use the Foundation production certificate and timestamp signatures. All jobs leading to signing run on GitHub-hosted runners. [SignPath verifies build origin from GitHub artifacts and workflow metadata](https://docs.signpath.io/trusted-build-systems/github).

Import the reviewed artifact configurations [windows-payload](../scripts/release/signpath-payload.xml) and [windows-setup](../scripts/release/signpath-setup.xml) using those exact slugs. Their required `version` parameter constrains ProductName to `Hyper GPU Support` and ProductVersion to the Cargo version. The first configuration signs the five exact project executable paths and exactly one generated uninstaller; the second signs only the versioned setup executable. Have the Foundation review and accept both configurations and their licensing before production use. Inno pads some version-resource strings with spaces; confirm the service accepts these exact restrictions against actual generated artifacts during onboarding. Local verification trims that padding while preserving the expected values. Do not weaken production metadata restrictions to bypass a rejected request.

Create a GitHub environment named `signpath-release`, limited to trusted maintainers and protected `v*` release tags. Configure:

| Name | Kind | Value |
| --- | --- | --- |
| `SIGNPATH_API_TOKEN` | Environment secret | Token for the SignPath CI submitter, scoped to this project/policy |
| `SIGNPATH_ORGANIZATION_ID` | Environment variable | Foundation-approved SignPath organization ID |
| `SIGNPATH_PROJECT_SLUG` | Environment variable | Approved project slug |
| `SIGNPATH_SIGNING_POLICY_SLUG` | Environment variable | Production policy requiring manual approval |
| `SIGNPATH_EXPECTED_PUBLISHER` | Environment variable | Exact complete certificate Subject string supplied by SignPath, including ordering and punctuation |

Do not put API tokens, user passwords or private keys in source, workflow inputs, issues or chat. The certificate private key stays with SignPath. Missing configuration, refusal, approval timeout, signature failure or changed payload bytes stops the workflow. Workflow permissions are read-only; it cannot create or modify a GitHub Release.

## Preparing and publishing an update

1. Review the source and dependency/licence inventory, update the Cargo package versions together, run the checks, and create the existing release tag `v<Cargo version>` using your normal reviewed Git process. Installer versions currently use three numeric components (for example `v0.1.0`); prerelease/build suffixes are rejected. Ordinary local builds use no signing account; see [the build instructions](../README.md#install).
2. Run **Prepare signed Windows release** manually with that existing tag selected as both the workflow ref and its `tag` input. With GitHub CLI, for example: `gh workflow run signed-release.yml --ref v0.1.0 -f tag=v0.1.0`. The workflow requires GitHub's event ref, event commit, checked-out tag commit and Cargo version to agree, so SignPath origin verification refers to the actual source being built. It provisions hash-pinned portable Inno Setup 6.7.3 and innounp 2.67.11, then runs the shared build script to build release binaries and prepare Inno's version-specific uninstaller cache.
3. Review the first SignPath request's repository, tag, commit, version, source changes and artifact paths. Approve the five project executables and generated uninstaller only after this review. The workflow verifies trusted Authenticode signatures, exact publisher, Code Signing and Time Stamping EKUs, timestamps, versions, and byte preservation outside Authenticode fields.
4. Inno packages those signed inputs and cached signed uninstaller. Review and approve the second request for the outer setup executable. The workflow verifies its signature and preserved compiler output, extracts the installer without executing it, checks every normal and temporary payload hash, rechecks the signed uninstaller cache supplied to the pinned compiler, and only then writes the final setup checksum. Inno retains its internal uninstaller FileVersion; its ProductVersion matches the application. The compiler validates the cache identity and the build records its unchanged hash. The pinned unpacker cannot expose Inno's runtime used to reconstruct the uninstaller, so the installed uninstaller's signature still requires separate live acceptance.
5. Download the successful workflow artifact named `signed-release-<version>-windows-x64`. Upload only its signed `HyperGpuSupport-<version>-windows-x64-setup.exe` and `SHA256SUMS.txt` to the matching GitHub Release, using GitHub's normal manual release UI. Recheck the downloaded setup with `Get-AuthenticodeSignature` and its checksum before publishing. Include a link to this Code signing policy and the acknowledgement above in the release notes after approval.

Artifacts named `transient-*-signing-input` are intermediate signing inputs, retained for one day; they are never release assets and may contain unsigned executables. A denied or failed request has no `signed-release-*` artifact. Delete or let those transient artifacts expire; restart with a fresh workflow run after resolving the cause. Do not upload a failed run's setup manually. Actual installation, upgrade, uninstall, UAC, alternate-administrator identity, runner enrollment and VM/guest acceptance require their own explicit authorisation and evidence; packaging success does not establish those outcomes.

For local release-tool regression checks, run `powershell -NoProfile -File scripts/release/test-release.ps1`. The checks use isolated temporary fixtures and do not contact SignPath, run Setup or access live app state.
