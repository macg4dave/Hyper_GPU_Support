# Privacy policy

Effective 19 October 2026.

Hyper GPU Support is a Windows desktop application. It is published as open-source software by [macg4dave](https://github.com/macg4dave), who maintains it. This policy covers the application, its installer and its helper programs.

## Summary

The application does not collect, transmit, sell or share personal data. It has no analytics, telemetry, crash reporting, advertising, accounts or automatic update service, and it does not contact the maintainer or any other online service during ordinary use. Everything it records stays on your computer and in the Hyper-V virtual machines you choose to set up.

## Information stored on your computer

The application keeps only what it needs to manage GPU support for your virtual machines:

| Information | Location | Purpose |
| --- | --- | --- |
| Host and VM inventory, GPU details, saved VM profiles, permissions, operation history and recovery records | `%ProgramData%\HyperGpuSupportProduct`, readable only by administrators, SYSTEM and the enrolled user | To apply, verify, repair and safely recover GPU changes |
| Interface preferences, such as the last selected VM and remembered guest operating system details | `%LOCALAPPDATA%\HyperGpuSupport` for each Windows user | To restore your view |
| Guest sign-in details (user name and password) | The current Windows user's Windows Credential Manager | Saved only when you explicitly choose to remember them, so later guest operations do not prompt again |
| Optional diagnostic traces | The installation's `diagnostics` folder, administrator-only | Written only when an administrator enables diagnostics with a protected marker file |
| Install and update archives | `%ProgramData%\HyperGpuSupport-archives`, administrator and SYSTEM only | Preserve recovery evidence before an install, update or uninstall resets app state |

Guest passwords are requested with standard Windows credential prompts. They are never written to the application's own files, logs or command lines.

## Information inside your virtual machines

When you approve a guest operation, the application uses PowerShell Direct over the local Hyper-V connection to sign in to the selected VM. It copies GPU driver files and a small helper there, and runs driver and graphics checks. These files remain in the VM, under `Program Files\HyperGpuSupport\Guest`. Nothing is sent outside your host and its VMs.

## Network use

The application makes no internet connections. You may choose to open the project's GitHub pages or report an issue. Those actions take place in your web browser and are covered by [GitHub's privacy statement](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement). Anything you include in a public issue is public, so do not post passwords, VM disk images or machine-specific files.

Building from source downloads Rust dependencies and build tools from their respective services. That happens only when you build, not when the application runs.

## Code signing

Official release installers are signed through SignPath. During a release build, SignPath receives the project's build executables and GitHub build metadata, but never any user data. See [SignPath's privacy policy](https://about.signpath.io/privacy-policy) and the [code signing policy](code-signing.md).

## Your control over the data

- **Delete profile** in the application removes its saved records for one VM.
- Saved guest sign-in details can be removed in Windows **Credential Manager** under *Windows Credentials*.
- Installing or updating resets app-owned state, including profiles, history, preferences and remembered guest sign-in details for the user running Setup. Protected state is archived first.
- Uninstalling through Windows **Installed apps** removes the application and its protected state. It keeps per-user preferences, saved guest credentials and install archives, and you can delete these yourself.

Because no data reaches the maintainer, there is nothing for the maintainer to access, correct or delete on your behalf.

## Changes and contact

Changes to this policy are published in this file and recorded in the repository history. For questions, [open an issue](https://github.com/macg4dave/Hyper_GPU_Support/issues).
