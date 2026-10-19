# Release packaging

Each official Windows x64 release provides two unsigned downloads:

| Download | Contents and use |
| --- | --- |
| `HyperGpuSupport-<version>-windows-x64-unsigned-setup.exe` | Inno Setup installer for installing, updating and uninstalling the application |
| `HyperGpuSupport-<version>-windows-x64-unsigned-binaries.zip` | The same four product executables, maintenance helper, executable hash manifest and licence notices, at the ZIP root |

SignPath Foundation declined the project. Releases no longer depend on Foundation approval, a signing account or signing credentials. Neither the executables nor the generated installer/uninstaller carry this project's code-signing signature. Windows may display an unknown-publisher warning. SHA-256 checksums detect changed downloads; they do not establish a certified publisher identity.

Use Setup to install the application and maintain its protected registration, files, shortcuts and uninstall entry. Extracting the binaries ZIP does not install the application, enrol its runner or reset app data. The ZIP's `app-install.exe` is the installer maintenance helper and source-checkout launcher; it is not a replacement for Setup. Keep the included licence notices with redistributed binaries. GitHub's automatic source archives contain source, rather than these compiled packages.

## Shared build

Local builds and GitHub Actions use [build-installer.ps1](../scripts/build-installer.ps1). It reads the package version and output directory from Cargo and uses the pinned Rust toolchain. Prerequisites and the source-install launcher are documented in [Install](../README.md#install).

```powershell
powershell -NoProfile -File scripts/build-installer.ps1
# If Inno Setup 6.7.3 is not found automatically:
powershell -NoProfile -File scripts/build-installer.ps1 -IsccPath 'C:\path\to\ISCC.exe'
```

The default build runs formatting, strict Clippy, workspace tests and isolated packaging/release regression checks, builds the five release executables, gathers the enabled Windows dependency licences and compiles Setup. It produces both packages and `SHA256SUMS.txt` in `dist/`. The ZIP's internal `SHA256SUMS.txt` covers its five executables; the external file covers the two downloadable packages. The `BuildPayload` and `Package` stages support staged tooling and fixtures. `-SkipBuild` and `-SkipChecks` are local development shortcuts; the official workflow uses the full checks.

Building and extracting packages does not run Setup or modify installed application state. Installation still refuses active work, archives protected state before resetting, cleans the initiating user's preferences and saved credentials, and leaves VM/guest state untouched. See [Install details](../README.md#install-details).

## Preparing and publishing an update

1. Review the source and [third-party notices](../licenses/THIRD_PARTY_NOTICES.md), update the product and maintenance-helper Cargo versions together, and create the reviewed tag `v<Cargo version>` through your normal Git process. Only numeric three-part versions such as `v0.1.0` are supported.
2. Run **Prepare unsigned Windows release** manually with that existing tag selected as both the workflow ref and its `tag` input. For example: `gh workflow run release.yml --ref v0.1.0 -f tag=v0.1.0`. The workflow requires the event ref, event commit, checked-out tag commit and Cargo version to agree. It uses a GitHub-hosted Windows runner and hash-pinned portable Inno Setup 6.7.3 and innounp 2.67.11. No SignPath secrets, variables or GitHub environment are required.
3. The workflow builds both packages from the same payload, checks product/version metadata and unsigned status, and extracts the ZIP and installer without running them. Every normal and temporary installer payload file must match the staged files. Missing files, unexpected files, changed bytes, invalid versions or compilation failures stop preparation before any release-ready artifact is uploaded.
4. Download the successful workflow artifact `unsigned-release-<version>-windows-x64`. It contains the two downloads and a supplementary `SHA256SUMS.txt`. Check both hashes after downloading:

   ```powershell
   Get-FileHash -Algorithm SHA256 .\HyperGpuSupport-0.1.0-windows-x64-unsigned-setup.exe
   Get-FileHash -Algorithm SHA256 .\HyperGpuSupport-0.1.0-windows-x64-unsigned-binaries.zip
   ```

5. Manually attach the two packages to the matching GitHub Release. Copy both checksum lines into its release notes, state that the downloads are unsigned, and link this page. Review the tag and attached files before publishing. The workflow has read-only repository permissions and does not create, upload to or publish a GitHub Release. Do not use packages from a failed run.

Protect release tags and the default branch, and review source, dependency and workflow changes before tagging. The repository owner, [macg4dave](https://github.com/macg4dave), maintains and publishes releases. The [Privacy policy](privacy.md) describes the application's local data handling.

## Validation boundaries

Run isolated tooling checks with:

```powershell
powershell -NoProfile -File scripts/testing/test-installer.ps1 -IsccPath 'C:\path\to\ISCC.exe'
powershell -NoProfile -File scripts/release/test-release.ps1
```

These checks do not run Setup or access live app state. Installer compilation and archive inspection do not establish installation, upgrade, uninstall, UAC, alternate-administrator identity, runner enrolment or VM/guest acceptance. Those remain separate explicitly authorised steps.
