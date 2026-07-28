<#
.SYNOPSIS
    Removes a per-user TrayGoblin installation.

.DESCRIPTION
    Stops any running instance, deletes the per-user installation folder, and
    removes the Startup shortcut. User configuration is preserved by default;
    pass -RemoveConfiguration to delete it as an explicit opt-in.

    The script never requires administrator rights and is safe to re-run: a
    missing installation, shortcut, or configuration folder is reported and
    treated as success.

.PARAMETER InstallRoot
    Per-user installation folder. Defaults to
    "%LOCALAPPDATA%\Programs\TrayGoblin".

.PARAMETER StartupDirectory
    Per-user Startup folder holding the shortcut. Defaults to the current
    user's Startup folder.

.PARAMETER ConfigurationRoot
    Folder holding config.json. Defaults to "%APPDATA%\TrayGoblin".

.PARAMETER RemoveConfiguration
    Also deletes the configuration folder. Off by default.

.EXAMPLE
    powershell -NoProfile -ExecutionPolicy Bypass -File .\uninstall.ps1

.EXAMPLE
    powershell -NoProfile -ExecutionPolicy Bypass -File .\uninstall.ps1 -RemoveConfiguration
#>
[CmdletBinding()]
param(
    [string] $InstallRoot,
    [string] $StartupDirectory,
    [string] $ConfigurationRoot,
    [switch] $RemoveConfiguration
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$script:ApplicationName = 'TrayGoblin'
$script:ExecutableName = 'tray-goblin.exe'
$script:ShortcutName = 'TrayGoblin.lnk'
$script:InstallMarkerName = '.tray-goblin-install'
$script:InstallMarkerContent = 'TrayGoblin per-user installation'

function Write-Step {
    param([Parameter(Mandatory)][string] $Message)
    Write-Host "[$script:ApplicationName] $Message"
}

function Stop-UninstallScript {
    param(
        [Parameter(Mandatory)][string] $Message,
        [string] $Remedy
    )

    $text = "error: $Message"
    if ($Remedy) {
        $text = "$text`n       $Remedy"
    }

    [Console]::Error.WriteLine($text)
    exit 1
}

function Resolve-InstallRoot {
    param([string] $Requested)

    if ($Requested) {
        return $Requested
    }

    $localAppData = $env:LOCALAPPDATA
    if (-not $localAppData) {
        $localAppData = [Environment]::GetFolderPath('LocalApplicationData')
    }

    if (-not $localAppData) {
        Stop-UninstallScript -Message 'LOCALAPPDATA is not set, so the per-user installation folder cannot be resolved.' `
            -Remedy 'Re-run with -InstallRoot "<installation folder>".'
    }

    return (Join-Path (Join-Path $localAppData 'Programs') $script:ApplicationName)
}

function Resolve-ConfigurationRoot {
    param([string] $Requested)

    if ($Requested) {
        return $Requested
    }

    $appData = $env:APPDATA
    if (-not $appData) {
        $appData = [Environment]::GetFolderPath('ApplicationData')
    }

    if (-not $appData) {
        return $null
    }

    return (Join-Path $appData $script:ApplicationName)
}

function Stop-RunningInstance {
    param([Parameter(Mandatory)][string] $ExecutablePath)

    $processName = [System.IO.Path]::GetFileNameWithoutExtension($script:ExecutableName)
    $expectedPath = [System.IO.Path]::GetFullPath($ExecutablePath)
    $running = @()
    foreach ($process in @(Get-Process -Name $processName -ErrorAction SilentlyContinue)) {
        try {
            if ($process.Path -and [string]::Equals(
                [System.IO.Path]::GetFullPath($process.Path),
                $expectedPath,
                [System.StringComparison]::OrdinalIgnoreCase
            )) {
                $running += $process
            }
        } catch {
            continue
        }
    }

    if ($running.Count -eq 0) {
        return
    }

    Write-Step "Stopping $($running.Count) running instance(s)."
    foreach ($process in $running) {
        try {
            Stop-Process -Id $process.Id -ErrorAction Stop
        } catch {
            if (-not (Get-Process -Id $process.Id -ErrorAction SilentlyContinue)) {
                continue
            }
            Stop-UninstallScript -Message "a running $script:ApplicationName process could not be stopped." `
                -Remedy 'Quit TrayGoblin from its tray menu and run this uninstaller again.'
        }
    }

    Start-Sleep -Milliseconds 500
}

$InstallRoot = Resolve-InstallRoot -Requested $InstallRoot
$InstallRoot = [System.IO.Path]::GetFullPath($InstallRoot)
$InstallRoot = $InstallRoot.TrimEnd([System.IO.Path]::DirectorySeparatorChar, [System.IO.Path]::AltDirectorySeparatorChar)
$installedExecutable = Join-Path $InstallRoot $script:ExecutableName
$installMarker = Join-Path $InstallRoot $script:InstallMarkerName

if (-not $StartupDirectory) {
    $StartupDirectory = [Environment]::GetFolderPath('Startup')
}

if ($StartupDirectory) {
    $shortcutPath = Join-Path $StartupDirectory $script:ShortcutName
    if (Test-Path -LiteralPath $shortcutPath -PathType Leaf) {
        try {
            Remove-Item -LiteralPath $shortcutPath -Force
        } catch {
            Stop-UninstallScript -Message "the Startup shortcut `"$shortcutPath`" could not be removed." `
                -Remedy 'Delete the shortcut from your Startup folder manually.'
        }
        Write-Step "Removed the Startup shortcut `"$shortcutPath`"."
    } else {
        Write-Step 'No Startup shortcut was present.'
    }
} else {
    Write-Step 'No Startup folder was resolved; skipping shortcut removal.'
}

if (Test-Path -LiteralPath $InstallRoot -PathType Container) {
    $installEntries = @(Get-ChildItem -LiteralPath $InstallRoot -Force)
    $validMarker = $false
    if (Test-Path -LiteralPath $installMarker -PathType Leaf) {
        try {
            $validMarker = [System.IO.File]::ReadAllText($installMarker) -eq $script:InstallMarkerContent
        } catch {
            $validMarker = $false
        }
    }

    $resumableEmptyDirectory = -not $validMarker -and $installEntries.Count -eq 0
    if (-not $validMarker -and -not $resumableEmptyDirectory) {
        Stop-UninstallScript -Message "the folder `"$InstallRoot`" is not marked as a TrayGoblin installation, so it was not deleted." `
            -Remedy 'Remove the folder manually only after confirming it contains no unrelated files.'
    }

    if ($validMarker) {
        Stop-RunningInstance -ExecutablePath $installedExecutable
    }

    $currentDirectory = [System.IO.Path]::GetFullPath((Get-Location).Path)
    $installPrefix = $InstallRoot + [System.IO.Path]::DirectorySeparatorChar
    if (
        [string]::Equals($currentDirectory, $InstallRoot, [System.StringComparison]::OrdinalIgnoreCase) -or
        $currentDirectory.StartsWith($installPrefix, [System.StringComparison]::OrdinalIgnoreCase)
    ) {
        $temporaryDirectory = [System.IO.Path]::GetTempPath()
        Set-Location $temporaryDirectory
        [System.IO.Directory]::SetCurrentDirectory($temporaryDirectory)
    }

    if ($validMarker) {
        try {
            foreach ($entry in @(Get-ChildItem -LiteralPath $InstallRoot -Force)) {
                if (-not [string]::Equals(
                    $entry.FullName,
                    $installMarker,
                    [System.StringComparison]::OrdinalIgnoreCase
                )) {
                    Remove-Item -LiteralPath $entry.FullName -Recurse -Force
                }
            }
        } catch {
            Stop-UninstallScript -Message "files in the installation folder `"$InstallRoot`" could not be removed." `
                -Remedy 'Close anything using TrayGoblin files and run this uninstaller again; the ownership marker was preserved.'
        }

        try {
            Remove-Item -LiteralPath $installMarker -Force
        } catch {
            Stop-UninstallScript -Message 'the installation ownership marker could not be removed.' `
                -Remedy 'Confirm the installation folder is writable, then run this uninstaller again.'
        }
    }

    try {
        Remove-Item -LiteralPath $InstallRoot -Force
    } catch {
        Stop-UninstallScript -Message "the empty installation folder `"$InstallRoot`" could not be removed." `
            -Remedy 'Close any File Explorer window using the folder and run this uninstaller again.'
    }
    Write-Step "Removed `"$InstallRoot`"."
} else {
    Write-Step "Nothing to remove at `"$InstallRoot`"."
}

$ConfigurationRoot = Resolve-ConfigurationRoot -Requested $ConfigurationRoot

if (-not $ConfigurationRoot) {
    Write-Step 'No configuration folder was resolved; nothing to keep or remove.'
} elseif (-not $RemoveConfiguration) {
    if (Test-Path -LiteralPath $ConfigurationRoot -PathType Container) {
        Write-Step "Kept your configuration in `"$ConfigurationRoot`". Re-run with -RemoveConfiguration to delete it."
    } else {
        Write-Step 'No configuration folder was present.'
    }
} elseif (Test-Path -LiteralPath $ConfigurationRoot -PathType Container) {
    $configurationFile = Join-Path $ConfigurationRoot 'config.json'
    try {
        if (Test-Path -LiteralPath $configurationFile -PathType Leaf) {
            Remove-Item -LiteralPath $configurationFile -Force
        }

        if (@(Get-ChildItem -LiteralPath $ConfigurationRoot -Force).Count -eq 0) {
            Remove-Item -LiteralPath $ConfigurationRoot -Force
            Write-Step "Removed the configuration folder `"$ConfigurationRoot`" as requested."
        } else {
            Write-Step "Removed config.json and kept other files in `"$ConfigurationRoot`"."
        }
    } catch {
        Stop-UninstallScript -Message "configuration could not be removed from `"$ConfigurationRoot`"." `
            -Remedy 'Close any editor holding config.json open and run this uninstaller again.'
    }
} else {
    Write-Step 'No configuration folder was present to remove.'
}

Write-Step 'Uninstall complete.'
exit 0
