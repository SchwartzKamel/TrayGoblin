<#
.SYNOPSIS
    Installs TrayGoblin for the current user only.

.DESCRIPTION
    Copies the portable payload below %LOCALAPPDATA%\Programs\TrayGoblin and
    registers a per-user Startup shortcut. The script never requires
    administrator rights, never writes outside the current user's profile, and
    never inspects Copilot session content.

    Re-running the script is safe: existing files are replaced and the Startup
    shortcut is rewritten in place.

.PARAMETER SourcePath
    Folder holding the extracted release payload. Defaults to the folder that
    contains this script, which is where the release ZIP places it.

.PARAMETER InstallRoot
    Per-user installation folder. Defaults to
    "%LOCALAPPDATA%\Programs\TrayGoblin".

.PARAMETER StartupDirectory
    Per-user Startup folder that receives the shortcut. Defaults to the
    current user's Startup folder.

.PARAMETER NoStartup
    Installs the files without creating a Startup shortcut.

.PARAMETER NoLaunch
    Installs without starting TrayGoblin afterwards.

.EXAMPLE
    powershell -NoProfile -ExecutionPolicy Bypass -File .\install.ps1
#>
[CmdletBinding()]
param(
    [string] $SourcePath,
    [string] $InstallRoot,
    [string] $StartupDirectory,
    [switch] $NoStartup,
    [switch] $NoLaunch
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$script:ApplicationName = 'TrayGoblin'
$script:ExecutableName = 'tray-goblin.exe'
$script:ShortcutName = 'TrayGoblin.lnk'
$script:InstallMarkerName = '.tray-goblin-install'
$script:InstallMarkerContent = 'TrayGoblin per-user installation'
$script:OptionalPayload = @('uninstall.ps1', 'install.ps1', 'LICENSE', 'README.txt')

function Write-Step {
    param([Parameter(Mandatory)][string] $Message)
    Write-Host "[$script:ApplicationName] $Message"
}

function Stop-InstallScript {
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

function Test-IsWindowsHost {
    if ($null -ne (Get-Variable -Name 'IsWindows' -Scope Global -ErrorAction SilentlyContinue)) {
        return [bool] $IsWindows
    }

    # Windows PowerShell 5.1 has no $IsWindows automatic variable.
    return $true
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
        Stop-InstallScript -Message 'LOCALAPPDATA is not set, so the per-user installation folder cannot be resolved.' `
            -Remedy 'Re-run with -InstallRoot "<folder below your user profile>".'
    }

    return (Join-Path (Join-Path $localAppData 'Programs') $script:ApplicationName)
}

function Resolve-StartupDirectory {
    param([string] $Requested)

    if ($Requested) {
        return $Requested
    }

    return [Environment]::GetFolderPath('Startup')
}

function Stop-RunningInstance {
    param([Parameter(Mandatory)][string] $ExecutablePath)

    if (-not (Test-Path -LiteralPath $ExecutablePath)) {
        return
    }

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
            # Processes owned by other users may not expose their executable
            # path. They are unrelated to this per-user installation.
            continue
        }
    }

    if ($running.Count -eq 0) {
        return
    }

    Write-Step "Stopping $($running.Count) running instance(s) before replacing files."
    foreach ($process in $running) {
        try {
            Stop-Process -Id $process.Id -ErrorAction Stop
        } catch {
            if (-not (Get-Process -Id $process.Id -ErrorAction SilentlyContinue)) {
                continue
            }
            Stop-InstallScript -Message "a running $script:ApplicationName process could not be stopped." `
                -Remedy 'Quit TrayGoblin from its tray menu and run this installer again.'
        }
    }

    # Give Windows a moment to release the executable's file lock.
    Start-Sleep -Milliseconds 500
}

function New-StartupShortcut {
    param(
        [Parameter(Mandatory)][string] $ShortcutPath,
        [Parameter(Mandatory)][string] $TargetPath,
        [Parameter(Mandatory)][string] $WorkingDirectory
    )

    $shell = $null
    try {
        $shell = New-Object -ComObject 'WScript.Shell'
    } catch {
        Stop-InstallScript -Message 'the Windows Script Host component required to create a Startup shortcut is unavailable.' `
            -Remedy 'Re-run with -NoStartup, then start TrayGoblin manually or add your own Startup entry.'
    }

    try {
        $shortcut = $shell.CreateShortcut($ShortcutPath)
        $shortcut.TargetPath = $TargetPath
        $shortcut.WorkingDirectory = $WorkingDirectory
        $shortcut.Description = "$script:ApplicationName - Copilot CLI status in the notification area"
        $shortcut.WindowStyle = 1
        $shortcut.Save()
    } catch {
        Stop-InstallScript -Message 'the Startup shortcut could not be written.' `
            -Remedy "Check that `"$ShortcutPath`" is writable, or re-run with -NoStartup."
    }
}

if (-not $SourcePath) {
    $SourcePath = if ($PSScriptRoot) { $PSScriptRoot } else { (Get-Location).Path }
}

if (-not (Test-Path -LiteralPath $SourcePath -PathType Container)) {
    Stop-InstallScript -Message "the payload folder `"$SourcePath`" does not exist." `
        -Remedy 'Extract the release ZIP first, then run install.ps1 from the extracted folder.'
}

$SourcePath = (Resolve-Path -LiteralPath $SourcePath).Path
$sourceExecutable = Join-Path $SourcePath $script:ExecutableName

if (-not (Test-Path -LiteralPath $sourceExecutable -PathType Leaf)) {
    Stop-InstallScript -Message "`"$script:ExecutableName`" was not found in `"$SourcePath`"." `
        -Remedy 'Run install.ps1 from the folder produced by extracting the release ZIP.'
}

$InstallRoot = [System.IO.Path]::GetFullPath((Resolve-InstallRoot -Requested $InstallRoot))
$InstallRoot = $InstallRoot.TrimEnd([System.IO.Path]::DirectorySeparatorChar, [System.IO.Path]::AltDirectorySeparatorChar)
$installedExecutable = Join-Path $InstallRoot $script:ExecutableName
$installMarker = Join-Path $InstallRoot $script:InstallMarkerName

$comparison = if (Test-IsWindowsHost) { 'OrdinalIgnoreCase' } else { 'Ordinal' }
if ([string]::Equals($SourcePath.TrimEnd([System.IO.Path]::DirectorySeparatorChar), $InstallRoot, $comparison)) {
    Stop-InstallScript -Message 'the payload folder and the installation folder are the same.' `
        -Remedy 'Extract the release ZIP to a separate folder before installing.'
}

Write-Step "Installing to `"$InstallRoot`" (no administrator rights required)."

if (Test-Path -LiteralPath $InstallRoot -PathType Container) {
    $existingEntries = @(Get-ChildItem -LiteralPath $InstallRoot -Force)
    if ($existingEntries.Count -gt 0) {
        $validMarker = $false
        if (Test-Path -LiteralPath $installMarker -PathType Leaf) {
            try {
                $validMarker = [System.IO.File]::ReadAllText($installMarker) -eq $script:InstallMarkerContent
            } catch {
                $validMarker = $false
            }
        }

        if (-not $validMarker) {
            Stop-InstallScript -Message "the installation folder `"$InstallRoot`" contains files not owned by TrayGoblin." `
                -Remedy 'Choose an empty folder, or remove the unrelated files before installing.'
        }
    }
}

Stop-RunningInstance -ExecutablePath $installedExecutable

try {
    if (-not (Test-Path -LiteralPath $InstallRoot -PathType Container)) {
        New-Item -ItemType Directory -Path $InstallRoot -Force | Out-Null
    }
} catch {
    Stop-InstallScript -Message "the installation folder `"$InstallRoot`" could not be created." `
        -Remedy 'Choose a writable folder below your user profile with -InstallRoot "<folder>".'
}

try {
    [System.IO.File]::WriteAllText(
        $installMarker,
        $script:InstallMarkerContent,
        [System.Text.UTF8Encoding]::new($false)
    )
} catch {
    Stop-InstallScript -Message 'the installation ownership marker could not be written.' `
        -Remedy 'Confirm the installation folder is writable, then run the installer again.'
}

try {
    Copy-Item -LiteralPath $sourceExecutable -Destination $installedExecutable -Force
} catch {
    Stop-InstallScript -Message "`"$script:ExecutableName`" could not be copied to `"$InstallRoot`"." `
        -Remedy 'Quit any running TrayGoblin instance, then run this installer again.'
}

foreach ($name in $script:OptionalPayload) {
    $candidate = Join-Path $SourcePath $name
    if (Test-Path -LiteralPath $candidate -PathType Leaf) {
        try {
            Copy-Item -LiteralPath $candidate -Destination (Join-Path $InstallRoot $name) -Force
        } catch {
            Stop-InstallScript -Message "`"$name`" could not be copied to the installation folder." `
                -Remedy 'Confirm the installation folder is writable, then run the installer again.'
        }
    }
}

Write-Step "Installed `"$script:ExecutableName`"."

$shortcutPath = $null
if ($NoStartup) {
    Write-Step 'Skipping the Startup shortcut because -NoStartup was supplied.'
} else {
    $StartupDirectory = Resolve-StartupDirectory -Requested $StartupDirectory

    if (-not $StartupDirectory) {
        if (Test-IsWindowsHost) {
            Stop-InstallScript -Message 'the per-user Startup folder could not be resolved.' `
                -Remedy 'Re-run with -StartupDirectory "<folder>" or with -NoStartup.'
        }

        Write-Step 'Skipping the Startup shortcut: this host has no Windows Startup folder.'
    } else {
        if (-not (Test-Path -LiteralPath $StartupDirectory -PathType Container)) {
            New-Item -ItemType Directory -Path $StartupDirectory -Force | Out-Null
        }

        $shortcutPath = Join-Path $StartupDirectory $script:ShortcutName

        if (Test-IsWindowsHost) {
            New-StartupShortcut -ShortcutPath $shortcutPath -TargetPath $installedExecutable -WorkingDirectory $InstallRoot
            Write-Step "Startup shortcut written to `"$shortcutPath`"."
        } else {
            $shortcutPath = $null
            Write-Step 'Skipping the Startup shortcut: Windows shortcuts can only be created on Windows.'
        }
    }
}

if ($NoLaunch) {
    Write-Step 'Skipping launch because -NoLaunch was supplied.'
} elseif (Test-IsWindowsHost) {
    try {
        Start-Process -FilePath $installedExecutable -WorkingDirectory $InstallRoot | Out-Null
        Write-Step 'Started TrayGoblin. Look for the goblin icon in the notification area.'
    } catch {
        Write-Warning "TrayGoblin was installed but could not be started automatically. Start `"$installedExecutable`" manually."
    }
} else {
    Write-Step 'Skipping launch: the notification-area shell only runs on Windows.'
}

Write-Step 'Install complete. Run uninstall.ps1 to remove it; your configuration is kept unless you pass -RemoveConfiguration.'
exit 0
