# Installation

**Audience:** Operators

TrayGoblin ships as a portable Windows x86-64 ZIP that installs into your own user profile. No
administrator rights are required at any point, and nothing is written outside your profile.

Requirements:

- Windows 10 or Windows 11, x86-64
- GitHub Copilot CLI installed and used at least once, so
  `%USERPROFILE%\.copilot\session-state` exists
- Windows PowerShell 5.1 or PowerShell 7+

## 1. Obtain the archive

Releases are built and published manually from a clean tagged commit; there is no CI pipeline and
no auto-update. Two supported ways to get the archive:

- Download `tray-goblin-0.1.0-windows-x86_64.zip` and its `.sha256` file from a published GitHub
  release for this repository, if one exists for the version you want.
- Build it yourself from a checkout. On a Linux or WSL host with the toolchain described in
  [Development](../agent/development.md):

  ```bash
  bash scripts/package-release.sh 0.1.0
  ```

  The archive and checksum are written to `dist/`.

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

1. Copies `tray-goblin.exe`, `install.ps1`, `uninstall.ps1`, `LICENSE`, and `README.txt` to
   `%LOCALAPPDATA%\Programs\TrayGoblin`.
2. Writes an ownership marker file, `.tray-goblin-install`, so a later repair or uninstall never
   deletes a folder it does not own.
3. Creates `TrayGoblin.lnk` in your per-user Startup folder.
4. Starts TrayGoblin, so the icon appears immediately.

Re-running the installer is safe: it stops only the instance running from the installed path,
replaces the files, and rewrites the shortcut.

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

## Upgrading

Extract the newer archive to a scratch folder and run `install.ps1` again. Your configuration in
`%APPDATA%\TrayGoblin\config.json` is untouched by installation.

## Uninstalling

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\uninstall.ps1
```

The uninstaller removes the Startup shortcut first, stops the instance running from the installed
path, then deletes the installation folder. Your configuration is preserved by default and the
script prints where it was kept. To remove it as well:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\uninstall.ps1 -RemoveConfiguration
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

## Related

- [First run](first-run.md)
- [Configuration](configuration.md)
- [Troubleshooting](troubleshooting.md)
- [Manual release](../manual-release.md) for how the archive you install was produced
