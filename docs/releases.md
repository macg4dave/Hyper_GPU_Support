# Release packaging

Each official Windows x64 release provides two unsigned downloads:

| Download | Contents and use |
| --- | --- |
| `HyperGpuSupport-<version>-windows-x64-unsigned-setup.exe` | Inno Setup installer for installing, updating and uninstalling the application |
| `HyperGpuSupport-<version>-windows-x64-unsigned-binaries.zip` | The same four product executables, maintenance helper, executable hash manifest and licence notices, plus `install.txt` with simple installation instructions, at the ZIP root |

SignPath Foundation declined the project. Releases no longer depend on Foundation approval, a signing account or signing credentials. Neither the executables nor the generated installer/uninstaller carry this project's code-signing signature. Windows may display an unknown-publisher warning. SHA-256 checksums detect changed downloads; they do not establish a certified publisher identity.

Use Setup to install the application and maintain its protected registration, files, shortcuts and uninstall entry. The ZIP's [install.txt](../packaging/install.txt) directs users to the matching Setup download. Extracting the binaries ZIP does not install the application, enrol its runner or reset app data. The ZIP's `app-install.exe` is the installer maintenance helper and source-checkout launcher; it is not a replacement for Setup. Keep the included licence notices with redistributed binaries. GitHub's automatic source archives contain source, rather than these compiled packages.

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

Keep a short, user-facing summary for each version in [CHANGELOG.md](../CHANGELOG.md), grouping related work rather than listing commits. Reuse that version's summary in the GitHub release notes. Agent instructions for this process are in the [release prompt](../.github/prompts/release.prompt.md).

1. Review the source and [third-party notices](../licenses/THIRD_PARTY_NOTICES.md), update the product and maintenance-helper Cargo versions together, and create the reviewed tag `v<Cargo version>` through your normal Git process. Only numeric three-part versions such as `v0.5.0` are supported.
2. Run **Prepare unsigned Windows release** manually from the reviewed `main` branch, with the existing release tag as its `tag` input. For example: `gh workflow run release.yml --ref main -f tag=v0.5.0`. The workflow checks out that exact tag separately from its release tooling, verifies its commit and Cargo version, and records both source and workflow commits in the run summary. This allows CI environment repairs without moving a release tag. It uses a GitHub-hosted Windows runner and hash-pinned portable Inno Setup 6.7.3 and innounp 2.67.11. No SignPath secrets, variables or GitHub environment are required.
3. The workflow builds both packages from the same payload, checks product/version metadata and unsigned status, and extracts the ZIP and installer without running them. Every normal and temporary installer payload file must match the staged files. Missing files, unexpected files, changed bytes, invalid versions or compilation failures stop preparation before any release-ready artifact is uploaded.
4. Download the successful workflow artifact `unsigned-release-<version>-windows-x64`. It contains the two downloads and a supplementary `SHA256SUMS.txt`. Check both hashes after downloading:

   ```powershell
   Get-FileHash -Algorithm SHA256 .\HyperGpuSupport-0.5.0-windows-x64-unsigned-setup.exe
   Get-FileHash -Algorithm SHA256 .\HyperGpuSupport-0.5.0-windows-x64-unsigned-binaries.zip
   ```

5. Manually attach the two packages to the matching GitHub Release. Copy both checksum lines into its release notes, state that the downloads are unsigned, and link this page. Review the tag and attached files before publishing. The workflow has read-only repository permissions and does not create, upload to or publish a GitHub Release. Do not use packages from a failed run.

Protect release tags and the default branch, and review source, dependency and workflow changes before tagging. The repository owner, [macg4dave](https://github.com/macg4dave), maintains and publishes releases. The [Privacy policy](privacy.md) describes the application's local data handling.

The hosted runner provisions a disposable standard Windows build account and grants it access only to the source, staging, Rust caches and packaging tools. The full shared build runs in Windows PowerShell 5.1 under that account, preserving the non-hardware tests' ordinary-user requirement and inbox module lookup. Account setup is confined to the disposable GitHub runner; local builds use the current user and do not provision accounts. The workflow removes the account after the build and the runner is discarded after the job.

## Validation boundaries

Run isolated tooling checks with:

```powershell
powershell -NoProfile -File scripts/testing/test-installer.ps1 -IsccPath 'C:\path\to\ISCC.exe'
powershell -NoProfile -File scripts/release/test-release.ps1
```

The installer checks run a small Inno harness with a fake maintenance helper in a temporary folder. It exercises the production helper launcher, including a folder the normal user can read but cannot write, non-zero exit codes and missing executables. It installs nothing and does not access live app state. The package checks also compile Setup and inspect its contents.

Full installation, upgrade, uninstall, UAC, alternate-administrator identity, runner enrolment and VM/guest checks remain separate explicitly authorised steps. Setup now saves a Windows Setup log automatically for diagnosing installation failures.

To test a real installation on a prepared Windows PC, run:

```powershell
powershell -NoProfile -File scripts/testing/test-install-uninstall.ps1 -SetupPath 'C:\build\HyperGpuSupport-0.5.0-windows-x64-unsigned-setup.exe' -PayloadDirectory 'C:\build\payload'
```

This installs the app, checks its file hashes, shortcut and Windows Installed apps entry, runs the generated uninstaller, checks removal of app files and background registration, then reinstalls it. It requests administrator permission and resets app data. Protected backups are retained. It does not launch the app or change VMs or guest files.
