<#
.SYNOPSIS
    Verifies the per-user installer, uninstaller, and performance script.

.DESCRIPTION
    Runs install.ps1 and uninstall.ps1 against a sandbox profile so the real
    user profile is never touched, and statically checks all three shipped
    scripts for elevation, machine-wide paths, and configuration-preservation
    regressions.

    Windows-shortcut behaviour is exercised only on Windows; on other hosts
    those checks are reported as skipped instead of silently passing.

.EXAMPLE
    pwsh -NoProfile -File scripts/test-installer.ps1
#>
[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$script:RepositoryRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$script:InstallScript = Join-Path $script:RepositoryRoot 'install.ps1'
$script:UninstallScript = Join-Path $script:RepositoryRoot 'uninstall.ps1'
$script:MeasureScript = Join-Path $PSScriptRoot 'measure-performance.ps1'
# A space in the sandbox name proves every path in the scripts stays quoted.
$script:SandboxRoot = Join-Path $script:RepositoryRoot '.installer-test/Tray Goblin Sandbox'
$script:Failures = New-Object System.Collections.Generic.List[string]
$script:Skipped = New-Object System.Collections.Generic.List[string]
$script:Passed = 0

function Test-IsWindowsHost {
    if ($null -ne (Get-Variable -Name 'IsWindows' -Scope Global -ErrorAction SilentlyContinue)) {
        return [bool] $IsWindows
    }

    return $true
}

function Get-ShellPath {
    $current = Get-Process -Id $PID
    if ($current.Path) {
        return $current.Path
    }

    throw 'The PowerShell host executable could not be resolved.'
}

function Invoke-Check {
    param(
        [Parameter(Mandatory)][string] $Name,
        [Parameter(Mandatory)][scriptblock] $Body
    )

    try {
        & $Body
        $script:Passed++
        Write-Host "  PASS  $Name"
    } catch {
        $script:Failures.Add("$Name -> $($_.Exception.Message)")
        Write-Host "  FAIL  $Name"
        Write-Host "        $($_.Exception.Message)"
    }
}

function Skip-Check {
    param(
        [Parameter(Mandatory)][string] $Name,
        [Parameter(Mandatory)][string] $Reason
    )

    $script:Skipped.Add("$Name -> $Reason")
    Write-Host "  SKIP  $Name ($Reason)"
}

function Assert-True {
    param(
        [Parameter(Mandatory)][bool] $Condition,
        [Parameter(Mandatory)][string] $Message
    )

    if (-not $Condition) {
        throw $Message
    }
}

function Invoke-Script {
    param(
        [Parameter(Mandatory)][string] $Path,
        [string[]] $ScriptArguments = @()
    )

    $shell = Get-ShellPath
    $output = & $shell -NoProfile -File $Path @ScriptArguments 2>&1
    return [pscustomobject]@{
        ExitCode = $LASTEXITCODE
        Output   = ($output | Out-String)
    }
}

function New-Sandbox {
    param([Parameter(Mandatory)][string] $Name)

    $root = Join-Path $script:SandboxRoot $Name
    if (Test-Path -LiteralPath $root) {
        Remove-Item -LiteralPath $root -Recurse -Force
    }

    $payload = Join-Path $root 'payload'
    $localAppData = Join-Path $root 'Local'
    $appData = Join-Path $root 'Roaming'
    $startup = Join-Path $root 'Startup'

    foreach ($directory in @($payload, $localAppData, $appData, $startup)) {
        New-Item -ItemType Directory -Path $directory -Force | Out-Null
    }

    Set-Content -LiteralPath (Join-Path $payload 'tray-goblin.exe') -Value 'test payload' -NoNewline
    Copy-Item -LiteralPath $script:UninstallScript -Destination (Join-Path $payload 'uninstall.ps1') -Force
    Copy-Item -LiteralPath $script:InstallScript -Destination (Join-Path $payload 'install.ps1') -Force

    return [pscustomobject]@{
        Root         = $root
        Payload      = $payload
        LocalAppData = $localAppData
        AppData      = $appData
        Startup      = $startup
        InstallRoot  = (Join-Path (Join-Path $localAppData 'Programs') 'TrayGoblin')
        ConfigRoot   = (Join-Path $appData 'TrayGoblin')
    }
}

function Invoke-Install {
    param(
        [Parameter(Mandatory)] $Sandbox,
        [string[]] $ExtraArguments = @()
    )

    $previousLocal = $env:LOCALAPPDATA
    $previousRoaming = $env:APPDATA
    try {
        $env:LOCALAPPDATA = $Sandbox.LocalAppData
        $env:APPDATA = $Sandbox.AppData
        $arguments = @('-SourcePath', $Sandbox.Payload, '-StartupDirectory', $Sandbox.Startup, '-NoLaunch') + $ExtraArguments
        return Invoke-Script -Path $script:InstallScript -ScriptArguments $arguments
    } finally {
        $env:LOCALAPPDATA = $previousLocal
        $env:APPDATA = $previousRoaming
    }
}

function Invoke-Uninstall {
    param(
        [Parameter(Mandatory)] $Sandbox,
        [string[]] $ExtraArguments = @()
    )

    $previousLocal = $env:LOCALAPPDATA
    $previousRoaming = $env:APPDATA
    try {
        $env:LOCALAPPDATA = $Sandbox.LocalAppData
        $env:APPDATA = $Sandbox.AppData
        $arguments = @('-StartupDirectory', $Sandbox.Startup) + $ExtraArguments
        return Invoke-Script -Path $script:UninstallScript -ScriptArguments $arguments
    } finally {
        $env:LOCALAPPDATA = $previousLocal
        $env:APPDATA = $previousRoaming
    }
}

function New-Configuration {
    param([Parameter(Mandatory)] $Sandbox)

    New-Item -ItemType Directory -Path $Sandbox.ConfigRoot -Force | Out-Null
    $path = Join-Path $Sandbox.ConfigRoot 'config.json'
    Set-Content -LiteralPath $path -Value '{"pollIntervalMs":2000}' -NoNewline
    return $path
}

Write-Host '[TrayGoblin] Installer verification'

if (Test-Path -LiteralPath $script:SandboxRoot) {
    Remove-Item -LiteralPath $script:SandboxRoot -Recurse -Force
}

$onWindowsHost = Test-IsWindowsHost
$allScripts = @($script:InstallScript, $script:UninstallScript, $script:MeasureScript)

foreach ($path in $allScripts) {
    Assert-True -Condition (Test-Path -LiteralPath $path -PathType Leaf) -Message "Required script '$path' is missing."
}

Write-Host 'Static checks'

Invoke-Check -Name 'every shipped script parses' -Body {
    foreach ($path in $allScripts) {
        $errors = $null
        [System.Management.Automation.Language.Parser]::ParseFile($path, [ref] $null, [ref] $errors) | Out-Null
        Assert-True -Condition ($null -eq $errors -or $errors.Count -eq 0) `
            -Message "$([System.IO.Path]::GetFileName($path)) has $($errors.Count) parse error(s)."
    }
}

Invoke-Check -Name 'no script requests elevation or machine-wide paths' -Body {
    $forbidden = @(
        'RunAsAdministrator',
        '-Verb\s+RunAs',
        'runas\.exe',
        'HKLM:',
        'HKEY_LOCAL_MACHINE',
        'ProgramFiles',
        'Program Files',
        'CurrentVersion\\Run'
    )

    foreach ($path in $allScripts) {
        $text = Get-Content -LiteralPath $path -Raw
        foreach ($pattern in $forbidden) {
            Assert-True -Condition ($text -notmatch $pattern) `
                -Message "$([System.IO.Path]::GetFileName($path)) references '$pattern', which would need elevation or leave the user profile."
        }
    }
}

Invoke-Check -Name 'install defaults below LocalAppData' -Body {
    $text = Get-Content -LiteralPath $script:InstallScript -Raw
    Assert-True -Condition ($text -match 'LOCALAPPDATA') -Message 'install.ps1 does not derive its default folder from LOCALAPPDATA.'
    Assert-True -Condition ($text -match "'Programs'") -Message 'install.ps1 does not install below a per-user Programs folder.'
}

Invoke-Check -Name 'uninstall keeps configuration unless opted in' -Body {
    $text = Get-Content -LiteralPath $script:UninstallScript -Raw
    Assert-True -Condition ($text -match '\[switch\]\s*\$RemoveConfiguration') `
        -Message 'uninstall.ps1 does not expose -RemoveConfiguration as an opt-in switch.'
    Assert-True -Condition ($text -notmatch '\$RemoveConfiguration\s*=\s*\$true') `
        -Message 'uninstall.ps1 defaults -RemoveConfiguration to on.'
}

Invoke-Check -Name 'performance script states the documented thresholds' -Body {
    $text = Get-Content -LiteralPath $script:MeasureScript -Raw
    Assert-True -Condition ($text -match '\$MaxWorkingSetMb\s*=\s*50') -Message 'measure-performance.ps1 does not default to a 50 MB working-set budget.'
    Assert-True -Condition ($text -match '\$MaxIdleCpuPercent\s*=\s*5') -Message 'measure-performance.ps1 does not default to a 5% CPU budget.'
    Assert-True -Condition ($text -match '\$DurationSeconds\s*=\s*30') -Message 'measure-performance.ps1 does not default to a 30-second window.'
    Assert-True -Condition ($text -match 'peakWorkingSetMb' -and $text -match 'averageCpuPercent') `
        -Message 'measure-performance.ps1 does not report both working set and CPU.'
}

Invoke-Check -Name 'performance output stays content-free' -Body {
    $text = Get-Content -LiteralPath $script:MeasureScript -Raw
    foreach ($forbidden in @('session-state', 'events.jsonl', 'workspace.yaml', 'CommandLine', 'prompt')) {
        Assert-True -Condition ($text -notmatch [regex]::Escape($forbidden)) `
            -Message "measure-performance.ps1 references '$forbidden', which risks reporting session content."
    }
}

Write-Host 'Behaviour checks'

Invoke-Check -Name 'install copies the payload below LocalAppData without elevation' -Body {
    $sandbox = New-Sandbox -Name 'install'
    $result = Invoke-Install -Sandbox $sandbox
    Assert-True -Condition ($result.ExitCode -eq 0) -Message "install.ps1 exited $($result.ExitCode): $($result.Output)"

    $installed = Join-Path $sandbox.InstallRoot 'tray-goblin.exe'
    Assert-True -Condition (Test-Path -LiteralPath $installed -PathType Leaf) -Message "'$installed' was not created."
    Assert-True -Condition (Test-Path -LiteralPath (Join-Path $sandbox.InstallRoot '.tray-goblin-install') -PathType Leaf) `
        -Message 'the installation ownership marker was not created.'
    Assert-True -Condition (Test-Path -LiteralPath (Join-Path $sandbox.InstallRoot 'uninstall.ps1') -PathType Leaf) `
        -Message 'uninstall.ps1 was not installed alongside the executable.'
    Assert-True -Condition ($installed.StartsWith($sandbox.LocalAppData)) -Message 'the install folder is not below LocalAppData.'
}

Invoke-Check -Name 'install is idempotent and refreshes the payload' -Body {
    $sandbox = New-Sandbox -Name 'idempotent'
    $first = Invoke-Install -Sandbox $sandbox
    Assert-True -Condition ($first.ExitCode -eq 0) -Message "the first install exited $($first.ExitCode): $($first.Output)"

    Set-Content -LiteralPath (Join-Path $sandbox.Payload 'tray-goblin.exe') -Value 'updated payload' -NoNewline
    $second = Invoke-Install -Sandbox $sandbox
    Assert-True -Condition ($second.ExitCode -eq 0) -Message "the second install exited $($second.ExitCode): $($second.Output)"

    $content = Get-Content -LiteralPath (Join-Path $sandbox.InstallRoot 'tray-goblin.exe') -Raw
    Assert-True -Condition ($content -eq 'updated payload') -Message 'a repeated install did not replace the executable.'
}

Invoke-Check -Name 'install repairs an interrupted marked installation' -Body {
    $sandbox = New-Sandbox -Name 'interrupted-install'
    New-Item -ItemType Directory -Path $sandbox.InstallRoot -Force | Out-Null
    [System.IO.File]::WriteAllText(
        (Join-Path $sandbox.InstallRoot '.tray-goblin-install'),
        'TrayGoblin per-user installation',
        [System.Text.UTF8Encoding]::new($false)
    )
    Set-Content -LiteralPath (Join-Path $sandbox.InstallRoot 'partial.tmp') -Value 'interrupted' -NoNewline

    $result = Invoke-Install -Sandbox $sandbox

    Assert-True -Condition ($result.ExitCode -eq 0) -Message "install.ps1 could not repair an interrupted install: $($result.Output)"
    Assert-True -Condition (Test-Path -LiteralPath (Join-Path $sandbox.InstallRoot 'tray-goblin.exe') -PathType Leaf) `
        -Message 'the repaired install is missing the executable.'
}

if ($onWindowsHost) {
    Invoke-Check -Name 'install creates and uninstall removes the Startup shortcut' -Body {
        $sandbox = New-Sandbox -Name 'shortcut'
        $installResult = Invoke-Install -Sandbox $sandbox
        Assert-True -Condition ($installResult.ExitCode -eq 0) -Message "install.ps1 exited $($installResult.ExitCode): $($installResult.Output)"

        $shortcut = Join-Path $sandbox.Startup 'TrayGoblin.lnk'
        Assert-True -Condition (Test-Path -LiteralPath $shortcut -PathType Leaf) -Message 'the Startup shortcut was not created.'

        $shell = New-Object -ComObject 'WScript.Shell'
        $target = $shell.CreateShortcut($shortcut).TargetPath
        Assert-True -Condition ($target -eq (Join-Path $sandbox.InstallRoot 'tray-goblin.exe')) `
            -Message "the Startup shortcut points at '$target'."

        $uninstallResult = Invoke-Uninstall -Sandbox $sandbox
        Assert-True -Condition ($uninstallResult.ExitCode -eq 0) -Message "uninstall.ps1 exited $($uninstallResult.ExitCode): $($uninstallResult.Output)"
        Assert-True -Condition (-not (Test-Path -LiteralPath $shortcut)) -Message 'the Startup shortcut survived uninstall.'
    }
} else {
    Skip-Check -Name 'install creates and uninstall removes the Startup shortcut' -Reason 'Windows shortcuts require a Windows host'
}

Invoke-Check -Name 'install honours -NoStartup' -Body {
    $sandbox = New-Sandbox -Name 'no-startup'
    $result = Invoke-Install -Sandbox $sandbox -ExtraArguments @('-NoStartup')
    Assert-True -Condition ($result.ExitCode -eq 0) -Message "install.ps1 exited $($result.ExitCode): $($result.Output)"
    Assert-True -Condition (-not (Test-Path -LiteralPath (Join-Path $sandbox.Startup 'TrayGoblin.lnk'))) `
        -Message '-NoStartup still created a Startup shortcut.'
}

Invoke-Check -Name 'uninstall removes the installation but keeps configuration' -Body {
    $sandbox = New-Sandbox -Name 'preserve-config'
    $installResult = Invoke-Install -Sandbox $sandbox
    Assert-True -Condition ($installResult.ExitCode -eq 0) -Message "install.ps1 exited $($installResult.ExitCode): $($installResult.Output)"

    $configPath = New-Configuration -Sandbox $sandbox
    $result = Invoke-Uninstall -Sandbox $sandbox
    Assert-True -Condition ($result.ExitCode -eq 0) -Message "uninstall.ps1 exited $($result.ExitCode): $($result.Output)"
    Assert-True -Condition (-not (Test-Path -LiteralPath $sandbox.InstallRoot)) -Message 'the installation folder survived uninstall.'
    Assert-True -Condition (Test-Path -LiteralPath $configPath -PathType Leaf) -Message 'uninstall deleted user configuration by default.'
    Assert-True -Condition ((Get-Content -LiteralPath $configPath -Raw) -eq '{"pollIntervalMs":2000}') -Message 'uninstall modified user configuration.'
}

Invoke-Check -Name 'uninstall -RemoveConfiguration deletes configuration on request' -Body {
    $sandbox = New-Sandbox -Name 'remove-config'
    $installResult = Invoke-Install -Sandbox $sandbox
    Assert-True -Condition ($installResult.ExitCode -eq 0) -Message "install.ps1 exited $($installResult.ExitCode): $($installResult.Output)"

    New-Configuration -Sandbox $sandbox | Out-Null
    $result = Invoke-Uninstall -Sandbox $sandbox -ExtraArguments @('-RemoveConfiguration')
    Assert-True -Condition ($result.ExitCode -eq 0) -Message "uninstall.ps1 exited $($result.ExitCode): $($result.Output)"
    Assert-True -Condition (-not (Test-Path -LiteralPath $sandbox.ConfigRoot)) -Message 'the configuration folder survived an explicit removal request.'
}

Invoke-Check -Name 'uninstall removes only config.json from a non-empty configuration folder' -Body {
    $sandbox = New-Sandbox -Name 'remove-config-only'
    $installResult = Invoke-Install -Sandbox $sandbox
    Assert-True -Condition ($installResult.ExitCode -eq 0) -Message "install.ps1 exited $($installResult.ExitCode): $($installResult.Output)"

    $configPath = New-Configuration -Sandbox $sandbox
    $unrelated = Join-Path $sandbox.ConfigRoot 'operator-notes.txt'
    Set-Content -LiteralPath $unrelated -Value 'keep me' -NoNewline

    $result = Invoke-Uninstall -Sandbox $sandbox -ExtraArguments @('-RemoveConfiguration')

    Assert-True -Condition ($result.ExitCode -eq 0) -Message "uninstall.ps1 exited $($result.ExitCode): $($result.Output)"
    Assert-True -Condition (-not (Test-Path -LiteralPath $configPath)) -Message 'config.json survived an explicit removal request.'
    Assert-True -Condition (Test-Path -LiteralPath $unrelated -PathType Leaf) -Message 'uninstall deleted an unrelated configuration-root file.'
}

Invoke-Check -Name 'uninstall is idempotent when nothing is installed' -Body {
    $sandbox = New-Sandbox -Name 'uninstall-twice'
    Invoke-Install -Sandbox $sandbox | Out-Null
    $first = Invoke-Uninstall -Sandbox $sandbox
    $second = Invoke-Uninstall -Sandbox $sandbox
    Assert-True -Condition ($first.ExitCode -eq 0) -Message "the first uninstall exited $($first.ExitCode): $($first.Output)"
    Assert-True -Condition ($second.ExitCode -eq 0) -Message "a repeated uninstall exited $($second.ExitCode): $($second.Output)"
    Assert-True -Condition ($second.Output -match 'Nothing to remove') -Message 'a repeated uninstall did not report that nothing was left to remove.'
}

Invoke-Check -Name 'uninstall resumes an empty directory left by a partial removal' -Body {
    $sandbox = New-Sandbox -Name 'resume-empty-uninstall'
    New-Item -ItemType Directory -Path $sandbox.InstallRoot -Force | Out-Null

    $result = Invoke-Uninstall -Sandbox $sandbox

    Assert-True -Condition ($result.ExitCode -eq 0) -Message "uninstall.ps1 could not resume an empty partial removal: $($result.Output)"
    Assert-True -Condition (-not (Test-Path -LiteralPath $sandbox.InstallRoot)) -Message 'the empty partial installation directory survived retry.'
}

Invoke-Check -Name 'uninstall refuses an unmarked folder and still removes Startup registration' -Body {
    $sandbox = New-Sandbox -Name 'unmarked-root'
    New-Item -ItemType Directory -Path $sandbox.InstallRoot -Force | Out-Null
    $unrelated = Join-Path $sandbox.InstallRoot 'unrelated-data.txt'
    Set-Content -LiteralPath $unrelated -Value 'keep me' -NoNewline
    $shortcut = Join-Path $sandbox.Startup 'TrayGoblin.lnk'
    Set-Content -LiteralPath $shortcut -Value 'test shortcut' -NoNewline

    $result = Invoke-Uninstall -Sandbox $sandbox

    Assert-True -Condition ($result.ExitCode -ne 0) -Message 'uninstall accepted a folder without the ownership marker.'
    Assert-True -Condition (Test-Path -LiteralPath $unrelated -PathType Leaf) -Message 'uninstall deleted unrelated data from an unmarked folder.'
    Assert-True -Condition (-not (Test-Path -LiteralPath $shortcut)) -Message 'uninstall left the Startup shortcut after refusing folder deletion.'
    Assert-True -Condition ($result.Output -match 'not marked') -Message "the refusal was not actionable: $($result.Output)"
}

Invoke-Check -Name 'install fails actionably when the payload is incomplete' -Body {
    $sandbox = New-Sandbox -Name 'missing-payload'
    Remove-Item -LiteralPath (Join-Path $sandbox.Payload 'tray-goblin.exe') -Force

    $result = Invoke-Install -Sandbox $sandbox
    Assert-True -Condition ($result.ExitCode -ne 0) -Message 'install.ps1 reported success without an executable to install.'
    Assert-True -Condition ($result.Output -match 'tray-goblin\.exe') -Message "the failure did not name the missing file: $($result.Output)"
    Assert-True -Condition ($result.Output -match 'release ZIP') -Message "the failure did not explain how to recover: $($result.Output)"
    Assert-True -Condition (-not (Test-Path -LiteralPath $sandbox.InstallRoot)) -Message 'a failed install still created the installation folder.'
}

Invoke-Check -Name 'install refuses a missing payload folder' -Body {
    $sandbox = New-Sandbox -Name 'missing-folder'
    $missing = Join-Path $sandbox.Root 'not there'
    $result = Invoke-Script -Path $script:InstallScript -ScriptArguments @('-SourcePath', $missing, '-InstallRoot', $sandbox.InstallRoot, '-NoStartup', '-NoLaunch')
    Assert-True -Condition ($result.ExitCode -ne 0) -Message 'install.ps1 accepted a payload folder that does not exist.'
    Assert-True -Condition ($result.Output -match 'does not exist') -Message "the failure was not actionable: $($result.Output)"
}

Invoke-Check -Name 'install refuses to install onto its own payload folder' -Body {
    $sandbox = New-Sandbox -Name 'same-folder'
    $result = Invoke-Script -Path $script:InstallScript -ScriptArguments @('-SourcePath', $sandbox.Payload, '-InstallRoot', $sandbox.Payload, '-NoStartup', '-NoLaunch')
    Assert-True -Condition ($result.ExitCode -ne 0) -Message 'install.ps1 installed a folder onto itself.'
    Assert-True -Condition ($result.Output -match 'same') -Message "the failure was not actionable: $($result.Output)"
}

if (Test-Path -LiteralPath $script:SandboxRoot) {
    Remove-Item -LiteralPath (Join-Path $script:RepositoryRoot '.installer-test') -Recurse -Force
}

Write-Host ''
Write-Host "[TrayGoblin] $script:Passed check(s) passed, $($script:Failures.Count) failed, $($script:Skipped.Count) skipped."

foreach ($skip in $script:Skipped) {
    Write-Host "  skipped: $skip"
}

if ($script:Failures.Count -gt 0) {
    foreach ($failure in $script:Failures) {
        Write-Host "  failed: $failure"
    }
    [Console]::Error.WriteLine('error: installer verification failed. Fix the checks listed above before packaging a release.')
    exit 1
}

exit 0
