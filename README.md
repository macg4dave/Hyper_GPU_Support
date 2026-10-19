<div align="center">

<!-- Optional logo: docs/images/logo.png -->

# Hyper GPU Support

**GPU support for your Hyper-V virtual machines.**

[![Windows](https://img.shields.io/badge/Windows-11-0078D4?style=for-the-badge&logo=windows11&logoColor=white)](#requirements)
[![NVIDIA](https://img.shields.io/badge/NVIDIA-target-76B900?style=for-the-badge&logo=nvidia&logoColor=white)](#requirements)
[![Licence: MIT](https://img.shields.io/badge/licence-MIT-4f46e5?style=for-the-badge)](LICENSE)

[Features](#features) &nbsp;·&nbsp;
[Getting started](#getting-started) &nbsp;·&nbsp;
[FAQ](#frequently-asked-questions) &nbsp;·&nbsp;
[Report a problem](https://github.com/macg4dave/Hyper_GPU_Support/issues)

<br />

<img src="docs/images/hero.png" alt="Hyper GPU Support dashboard with GPU support enabled and GPU memory set for a Windows 11 VM" width="820" />

</div>

<br />

## Overview

Hyper-V can share your computer's graphics card with virtual machines through GPU partitioning (GPU-PV).
Hyper GPU Support brings GPU configuration, guest driver setup and graphics checks into one Windows app.

<br />

## Features

<table>
<tr>
<td width="50%" valign="top">

### Simple controls

Turn GPU support on or off for an eligible VM, and choose its graphics memory.

</td>
<td width="50%" valign="top">

### Guided first-time setup

First setup saves and confirms permission for the exact VM/GPU pair, then configures it, copies driver files and checks hardware graphics.

</td>
</tr>
<tr>
<td width="50%" valign="top">

### Built-in safeguards

Actions start after fresh checks, including any required VM starts and normal shutdowns. Uncertain results block further changes until recovery is resolved. The app never forces a VM off or restarts your computer.

</td>
<td width="50%" valign="top">

### Checking and repair

Check driver files and hardware graphics, reinstall drivers or detach GPU support through <b>More actions</b>.

</td>
</tr>
<tr>
<td width="50%" valign="top">

### Your sign-in stays yours

VM sign-in details are used only for the steps you approve, and saved in Windows Credential Manager only if you choose <b>Remember</b>.

</td>
<td width="50%" valign="top">

### Designed for Windows

Light, dark and system themes, optional help text, keyboard controls and accessibility labels. British English is currently the only bundled language.

</td>
</tr>
</table>

<br />

## Screenshots

<div align="center">

<p><strong>Before setup</strong></p>
<img src="docs/images/before%20setup.png" alt="A Windows 11 VM before GPU setup, with the Start GPU setup button" width="720" />

<br /><br />

<p><strong>First-time setup progress</strong></p>
<img src="docs/images/settingup.png" alt="Applying GPU changes, with each setup step and its progress" width="720" />

<br /><br />

<p><strong>Setup result</strong></p>
<img src="docs/images/setupdone.png" alt="Changes applied summary showing GPU support enabled and driver setup and checks completed" width="720" />

<br /><br />

<p><strong>Changing GPU memory</strong></p>
<img src="docs/images/changea.png" alt="Changes applied summary after changing GPU memory to 5 GB" width="720" />

<br /><br />

<p><strong>The GPU working inside the VM</strong></p>
<img src="docs/images/hypervfurmark.png" alt="FurMark running on the NVIDIA GeForce RTX 5060 inside a Hyper-V Windows 11 VM" width="720" />

</div>

<br />

## Requirements

The current target configuration is:

- An x64 Windows 11 Pro, Enterprise or Education host with Hyper-V enabled
- An NVIDIA GPU with its host driver installed
- An existing Generation 2 Windows 11 VM

> [!NOTE]
> Automatic guest driver setup is implemented for NVIDIA GPUs only. Windows-version compatibility is not currently checked; an advisory warning does not establish that another guest OS is supported.

<br />

## Getting started

### Install

Install from source on Windows. You need Git, [Rust](https://rustup.rs), and the Visual Studio C++ build tools with a Windows SDK for the `x86_64-pc-windows-msvc` target. Rustup uses the version pinned in [rust-toolchain.toml](rust-toolchain.toml).

Close the app and let any operation finish before installing or updating.

```powershell
git clone https://github.com/macg4dave/Hyper_GPU_Support.git
cd Hyper_GPU_Support
cargo run -p app-install
```

The installer builds the app, shows its plan and asks you to type `yes`, then asks for administrator permission. Add `-- --yes` to skip the typed confirmation.

> [!WARNING]
> Installing or updating resets app-owned state, including saved profiles, history, recovery records, protected registration, preferences and remembered guest sign-in details. Protected state is archived by default. The installer does not change VMs, disks or guest files. See [Install details](#install-details).

Then launch the installed app from PowerShell:

```powershell
& "$env:ProgramFiles\HyperGpuSupport\hyper-gpu-support.exe"
```

### Set up a VM

1. **Save work inside the VM.** Setup can start and shut down the VM. It must be running or turned off, rather than saved or paused.
2. **Select the VM and GPU** on the Dashboard, turn on GPU support and choose how much graphics memory to assign.
3. **Select Start GPU setup.** Fresh checks prepare the request, the app saves and confirms permission for that VM/GPU pair, then continues into GPU setup. Approve Windows administrator prompts when shown and use the local Windows sign-in prompt for the guest account. **Remember** is optional.
4. **Wait for the final result.** Successful first setup confirms driver files and hardware graphics, leaves the VM off and saves your requested settings. Start the VM in Hyper-V Manager when ready. A completed file transfer alone is not a successful setup result.

> [!TIP]
> After successful first setup, use **Apply** for memory and on/off changes. Actions begin after fresh checks without a separate review or second confirmation; they include their required power changes. Ordinary changes do not repeat guest setup or graphics verification unless needed, and leave the VM off when complete.

<br />

## Frequently asked questions

<details>
<summary><b>Will this change anything else about my VM?</b></summary>
<br />
GPU setup can change the VM's MMIO ranges, guest cache setting and automatic-checkpoint policy, as well as its GPU attachment and allocation. Operations can start and normally shut down the VM, and driver setup writes files inside the guest. Disk configuration, system RAM amounts, CPU counts and networking are outside the app's scope.
</details>

<details>
<summary><b>What happens if an operation is interrupted?</b></summary>
<br />
An interrupted or uncertain change retains a recovery record. Open <b>More actions &gt; Review recovery</b> to inspect the outcome and available recovery actions. A hold tied to one VM normally blocks that VM; shared, unreadable or unknown-scope records can block all new changes. Uncertain work is never repeated automatically.
</details>

<details>
<summary><b>Can I remove GPU support later?</b></summary>
<br />
Yes. <b>More actions &gt; Remove GPU support</b> detaches the app's GPU partition, restores compatibility values it still owns and leaves the VM off. It keeps guest driver files, permission and history. <b>Delete profile</b> separately removes the app's records without detaching the GPU or changing the VM.
</details>

<details>
<summary><b>Does the app store my VM sign-in details?</b></summary>
<br />
Only if you select <b>Remember</b>. They are then stored in Windows Credential Manager and are available to your Windows account only. Otherwise they are used only for the steps you approved.
</details>

<details>
<summary><b>Can several VMs share one graphics card?</b></summary>
<br />
The app allows multiple VM profiles to use the same physical GPU. Allocations are manual: it does not balance them or warn when the combined requested memory exceeds the card's capacity. The VRAM range does not guarantee enforced isolation between VMs.
</details>

<details>
<summary><b>Is the app available in my language?</b></summary>
<br />
British English (<code>en-GB</code>) is currently the only bundled language. The Settings language control can follow Windows or select a compiled language; with the current catalogue, both resolve to British English. See <a href="resources/i18n/README.md">UI text and languages</a> for the catalogue format.
</details>

<br />

## Support and feedback

If you find a problem or have a suggestion, please [open an issue](https://github.com/macg4dave/Hyper_GPU_Support/issues). Reports from hardware and configurations that have not been tested yet are particularly helpful.

<br />

---

<div align="center">

### Technical information

<sub>Additional detail for advanced users and contributors.</sub>

</div>

<details>
<summary><b>All actions in detail</b></summary>
<br />

| Action | What it does |
| --- | --- |
| **Start GPU setup** | Saves and confirms permission for the exact VM/GPU pair, assigns the partition, copies and checks drivers inside the VM, tests Direct3D 11, then leaves the VM off and saves requested settings. |
| **Apply** | Applies the requested GPU toggle or allocation after fresh checks. Ordinary changes use host readback and leave the VM off; required driver work can also trigger guest setup and graphics checks. |
| **Verify GPU setup** | Checks driver files, the setup record, device status and hardware graphics without reinstalling drivers. An off VM is temporarily started and returned to off; a running VM stays running after successful verification. |
| **Repair / Reinstall** | Runs the full signed driver setup again, even if earlier checks passed, then checks graphics and turns the VM off. |
| **Remove GPU support** | Detaches the app's GPU partition, restores compatibility values it still owns and leaves the VM off. Keeps driver files in the VM, permission and history. No guest sign-in needed. |
| **Remove partial GPU setup** | When available in recovery, removes the interrupted attachment, restores its recorded original compatibility settings and leaves the VM off. Keeps guest files and failure evidence. |
| **Delete profile** | Deletes the app's saved settings, permission, history and recovery records for one VM. Doesn't touch the VM, its power state, GPU assignment, disks or guest files. |
| **Review recovery** | Checks an interrupted or uncertain outcome and offers the recovery actions supported by the recorded and current state. Unresolved uncertainty keeps the hold. |

Setup, Apply, Verify, Repair / Reinstall, Remove GPU support and Delete profile start after fresh checks without a separate review or confirmation screen. Windows sign-in and administrator prompts remain separate. Recovery offers its own actions based on the recorded state. Most maintenance actions are in **More actions**; availability depends on the selected VM and any recovery hold.

**Advanced allocation** sets exact Minimum / Optimal / Maximum VRAM values in bytes. It preserves existing values exactly (including values below 1 GB) and keeps any saved compute, encode and decode settings.

The VRAM range is a Hyper-V allocation setting and doesn't guarantee isolation between VMs. The app doesn't balance memory automatically or warn when a GPU is fully allocated. It doesn't check which Windows version the VM runs; instead it shows a non-blocking warning before GPU installation, driver refresh or Repair / Reinstall.

</details>

<details>
<summary><b>Safety and recovery</b></summary>
<br />

- Protected changes are recorded before effects begin. Interrupted or uncertain results retain a recovery hold. A known VM hold normally blocks that VM; shared or unreadable recovery can block all changes.
- Recovery checks the recorded result without repeating Apply, and only enters the VM if needed.
- The app never repeats an uncertain operation automatically, never forces a VM off and never restarts the host.
- Starting the app only checks for unfinished work; it never repairs or repeats GPU operations.
- **Delete profile** explicitly discards that VM's app records, including its recovery records. It does not resolve or undo the VM's actual GPU state.
- If a change succeeds but its settings can't be saved, saving can be retried on its own.
- Failures keep the original error alongside freshly read power and recovery facts.
- The host's GPU configuration is compared with saved settings, and differences are reported before any repair or removal.

</details>

<details>
<summary><b>Security</b></summary>
<br />

- Protected work runs in a separate installed runner, started from a scheduled task. The GUI runs without elevation; first-time protected setup uses a separate UAC-approved process.
- Installed executables are checked against pinned SHA-256 hashes. Protected policy, path and access checks restrict execution, and local pipe connections authenticate their peers.
- Driver payloads are checked through embedded signatures or catalogue membership. Guest helpers and probes are checked against approved hashes. An invalid guest reply counts as a communication failure.
- Guest credentials are used only for approved steps, and stored in Windows Credential Manager (current user only) only when **Remember** is selected.

</details>

<details>
<summary><b>Command line</b></summary>
<br />

Launching `hyper-gpu-support.exe` without arguments opens the GUI. Use the GUI for ordinary GPU operations. These commands remain for existing records and recovery; `--help` lists their syntax:

| Command | Purpose |
| --- | --- |
| `install --config FILE` | Protected-runner installation/enrolment for existing configuration; use `app-install` below for app installs and updates |
| `import --config FILE` | Import settings into missing per-VM configuration files (never approves or applies anything) |
| `save-only --operation ID` | Retry saving a recorded, verified result |
| `reconcile --operation ID [--approve-shutdown]` | Check a recorded result without repeating Apply |
| `import-reconcile --operation ID` | Resume an interrupted import |
| `native-review --job REFERENCE` | Review a recorded Hyper-V job |
| `native-reconcile --review TOKEN` | Approve clearing a reviewed Hyper-V job record |

Configuration files use TOML schema 2 or 3; see [config/product.example.toml](config/product.example.toml).

</details>

<a name="install-details"></a>
<details>
<summary><b>Install details</b></summary>
<br />

`cargo run -p app-install` is the only supported way to install or update. Run it from the repository root with the app closed and no operation running:

```powershell
cargo run -p app-install            # build, review the plan, type "yes", approve UAC
cargo run -p app-install -- --yes   # skips the typed confirmation only
```

The tool builds the release binaries (`--skip-build` reuses the existing build), then checks that the app and its helpers are closed. Before resetting anything, the elevated phase checks again and refuses if a product scheduled task is running. It does not stop an active operation for you.

After the UAC prompt it archives protected state from `%ProgramData%\HyperGpuSupportProduct` and the legacy `%ProgramData%\HyperGpuSupport` to `%USERPROFILE%\HyperGpuSupport-archives\install-<time>`. If archiving fails, nothing is removed. `--no-archive` explicitly skips this evidence copy.

It then removes the previous app installation, protected product registration, scheduled tasks and runtime state; installs the four executables to `%ProgramFiles%\HyperGpuSupport`; and checks their SHA-256 hashes against the build. After that succeeds, it clears `%LOCALAPPDATA%\HyperGpuSupport` and remembered guest credentials for the current user. VMs, guest files and the configured `data_root` are not changed.

First-time setup in the app enrols the protected runner again. This reset happens during installs and updates, not ordinary launches.

</details>

<details>
<summary><b>Contributing</b></summary>
<br />

| Document | Purpose |
| --- | --- |
| [UI text and languages](resources/i18n/README.md) | Compiled wording catalogue and its format |
| [AGENTS.md](AGENTS.md) | Instructions for Codex and Claude Code |

The app is written in Rust and the interface is built with [Slint](https://slint.dev). GUI text uses a compiled catalogue, with British English as the reference. Backend and worker diagnostics remain English.

Image spaces are marked above. Put the optional logo and real app screenshots in [docs/images/](docs/images/) using the indicated filenames, then replace the matching placeholders with image links.

Please keep passwords, driver payloads, VM disks and machine-specific evidence out of Git.

</details>

<br />

<div align="center">
<sub>Released under the <a href="LICENSE">MIT licence</a>.</sub>
</div>
