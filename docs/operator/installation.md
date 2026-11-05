# Installation

**Audience:** Operators

TrayGoblin is packaged as an UNPUBLISHED Windows x86-64 ZIP, installed per-user.
No administrator rights are required. Default paths stay below your profile.
Implementation/cross-build validation is not real Windows/WSL release evidence.

Requirements:

- Windows 10 or Windows 11, x86-64
- Pi 1.4.0 with the explicit opt-in integration, and/or Copilot CLI with local
  `%USERPROFILE%\.copilot\session-state`; WSL is optional and explicitly configured
- Windows PowerShell 5.1 or PowerShell 7+

## 1. Obtain the archive

Official releases are built and published manually from a clean tagged commit;
there is no auto-update. Local manual builds remain authoritative, with no CI
prerequisite. Ways to get the archive:

- Download `tray-goblin-0.1.0-windows-x86_64.zip` and its `.sha256` file from a published GitHub
  release for this repository, if one exists for the version you want.
- Optional **test ZIP**, when available: sign in to GitHub, open this repository's
  **Actions → Windows test build**, select a successful main/manual run for the
  source commit you want, then download **tray-goblin-windows-x86_64** under
  **Artifacts**. Extract that outer artifact ZIP to obtain the inner versioned ZIP
  and sibling `.zip.sha256`; verify those below before extracting/installing the
  inner ZIP. Artifacts expire after 14 days; if absent/expired, build locally.
  These are unsigned, unpublished test builds, **not GitHub Releases**. A green
  run proves Rust tests and Windows-target compilation/package only, not real
  pi-host/model, interactive Windows/WSL or performance behavior. The full manual
  attention demo and M6 field/resource checks remain independent and required.
- Build it yourself from a checkout. On a Linux or WSL host with the toolchain described in
  [Development](../agent/development.md):

  ```bash
  bash scripts/package-release.sh 0.1.0
  ```

  The archive and checksum are written to `dist/`.

No official release is published yet; no actual CI download is claimed until CI2
verifies a GitHub run. See [`README.md`](../../README.md#project-status) for current
status. If no test artifact is available, use the build-it-yourself path above.

## 2. Verify the checksum before extracting

In PowerShell, from the folder holding the downloaded files:

```powershell
Get-FileHash .\tray-goblin-0.1.0-windows-x86_64.zip -Algorithm SHA256
Get-Content .\tray-goblin-0.1.0-windows-x86_64.zip.sha256
```

The hash printed by `Get-FileHash` must match the hash in the `.sha256` file. If it does not,
delete the download and obtain the archive again. Do not install an archive that fails this check.

## 3. Extract and install

Extract the ZIP to a scratch folder such as `%USERPROFILE%\Downloads\tray-goblin`, then run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\install.ps1
```

The installer:

1. Validates the payload and writes ownership marker `.tray-goblin-install`
   BEFORE copying, preserving repair ownership after a partial installation.
2. Copies `tray-goblin.exe`, scripts/license/README, `integrations/pi/` and
   `resources/` to `%LOCALAPPDATA%\Programs\TrayGoblin`. Resources include complete
   standalone setup, an example config, build/executable identity and both
   Windows collectors; no source checkout is needed for those instructions.
3. Creates `TrayGoblin.lnk` in your per-user Startup folder.
4. Starts TrayGoblin, so the icon appears immediately.

Re-running the installer is safe: it stops only the instance running from the installed path,
replaces the files, and rewrites the shortcut. It never silently edits pi settings.
The extracted folder can be removed after installation. To opt in persistently:

```powershell
$tg = Join-Path $env:LOCALAPPDATA 'Programs\TrayGoblin'
pi install "$tg\integrations\pi"
```

Restart/reload pi. Local packages reference that installed path (pi does not copy
it). For WSL durable-copy/opt-in, labels and source paths see installed
`resources/SETUP.md` ([source](../../assets/attention/SETUP.md)) and
[First run](first-run.md). Never register a temporary extraction directory.

`%LOCALAPPDATA%` is your per-user program-data location; `%APPDATA%` is the separate per-user
configuration location. The standard command uses all defaults. The options below are only for
custom installation or testing needs.

### Installer options

| Option | Effect |
|---|---|
| `-SourcePath <folder>` | Payload folder to install from. Defaults to the folder holding `install.ps1`. |
| `-InstallRoot <folder>` | Alternate per-user installation folder. |
| `-StartupDirectory <folder>` | Alternate Startup folder for the shortcut. |
| `-NoStartup` | Install without creating a Startup shortcut. |
| `-NoLaunch` | Install without starting TrayGoblin afterwards. |

The installer stops with an actionable message when the payload folder and installation folder are
the same, when `tray-goblin.exe` is missing from the payload, or when the target folder already
contains files that TrayGoblin does not own.

## 4. Confirm it is running

Look for the goblin icon in the notification area. Continue with [First run](first-run.md) to
interpret what it shows, and see [Troubleshooting](troubleshooting.md) if no icon appears.

If this archive is a release candidate, installation is only the first Operator check. Continue
through the [Manual Windows checklist](../agent/testing.md#manual-windows-checklist), record the
results and performance numbers, and return that evidence to the Agent preparing the release.

## Upgrading

Extract the newer archive to a scratch folder and run `install.ps1` again. Your configuration in
`%APPDATA%\TrayGoblin\config.json` is untouched by installation.

## Uninstalling

Remove the explicit pi opt-in FIRST and reload/quit pi, then use the persistent
uninstaller even if the extracted folder is gone:

```powershell
$tg = Join-Path $env:LOCALAPPDATA 'Programs\TrayGoblin'
pi remove "$tg\integrations\pi"
powershell -NoProfile -ExecutionPolicy Bypass -File "$tg\uninstall.ps1"
```

In WSL, `pi remove "$HOME/.local/share/traygoblin/pi"` and reload/quit before
removing that separate durable copy. Uninstall never edits pi settings or deletes
producer status roots on your behalf.

The uninstaller removes the Startup shortcut first, stops the instance running from the installed
path, then deletes the installation folder. Your configuration is preserved by default and the
script prints where it was kept. To remove it as well:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File "$tg\uninstall.ps1" -RemoveConfiguration
```

`-RemoveConfiguration` deletes `config.json`, and removes `%APPDATA%\TrayGoblin` only when nothing
else remains in it. Any other files you put there are kept.

The uninstaller is safe to re-run. A missing installation folder, shortcut, or configuration folder
is reported and treated as success. If the installation folder is not marked as a TrayGoblin
installation, the script stops instead of deleting files it does not own.

| Option | Effect |
|---|---|
| `-InstallRoot <folder>` | Installation folder to remove. |
| `-StartupDirectory <folder>` | Startup folder holding the shortcut. |
| `-ConfigurationRoot <folder>` | Configuration folder. Defaults to `%APPDATA%\TrayGoblin`. |
| `-RemoveConfiguration` | Also delete `config.json`. Off by default. |

## What installation never does

- It never requires or requests elevation.
- It never writes to `%ProgramFiles%`, `HKEY_LOCAL_MACHINE`, or any machine-wide location.
- It never changes Copilot CLI configuration, telemetry settings, or session data.
- It never registers/removes pi packages silently. Only explicit `pi install` /
  `pi remove` commands change pi settings; integration/resources persist in the
  marked install root until uninstall.

## Related

- [First run](first-run.md)
- [Configuration](configuration.md)
- [Troubleshooting](troubleshooting.md)
- [Manual release](../manual-release.md) for how the archive you install was produced
