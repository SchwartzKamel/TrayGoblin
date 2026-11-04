<#
.SYNOPSIS
    Collects genuine INTERACTIVE Windows/WSL attention observations. Not automation.
.DESCRIPTION
    Use synthetic activity only. No capability answer, mock, offline check or Linux
    run becomes a passing case. Timings are conservative human-observed bounds:
    start the stopwatch BEFORE triggering a transition; stop AFTER observing it.
.PARAMETER PollIntervalMs
    Expected candidate cadence, NOT an override. Must equal the effective config
    pollIntervalMs / poll_interval_ms (500-10000 integer ms; omitted/null = 1000).
    Supply it explicitly for nondefault cadence; mismatches/duplicates are rejected
    before collecting observations or writing evidence.
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory)][string] $ExecutablePath,
    [Parameter(Mandatory)][string] $EvidencePath,
    [string] $BuildSha,
    [string] $ConfigurationPath = "$env:APPDATA/TrayGoblin/config.json",
    [ValidateRange(500, 10000)][int] $PollIntervalMs = 1000,
    [ValidateRange(2, 1024)][int] $SyntheticSessionCount = 4
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
if ([Environment]::OSVersion.Platform -ne [PlatformID]::Win32NT) {
    throw 'Windows interactive host required; Linux/offline results cannot satisfy M6.'
}
if (-not [Environment]::UserInteractive -or [Console]::IsInputRedirected) {
    throw 'An interactive Windows console and real observations are required.'
}
$exe = (Resolve-Path -LiteralPath $ExecutablePath).Path
$digest = (Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash.ToLowerInvariant()
$manifestPath = Join-Path (Split-Path -Parent $exe) 'resources/build.json'
$dirty = $null
if (Test-Path -LiteralPath $manifestPath) {
    $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
    if ($manifest.executableSha256 -ne $digest) { throw 'Packaged build identity does not match executable SHA256.' }
    if ($BuildSha -and $BuildSha -ne $manifest.buildCommit) { throw 'BuildSha differs from packaged identity.' }
    $BuildSha = $manifest.buildCommit
    $dirty = $manifest.worktreeDirty
}
if ($BuildSha -notmatch '^[0-9a-f]{40}$') { throw 'Use the packaged build.json or supply the actual -BuildSha (40 hex characters).' }
$running = @(Get-Process -Name ([IO.Path]::GetFileNameWithoutExtension($exe)) -ErrorAction SilentlyContinue)
if ($running.Count -ne 1 -or -not $running[0].Path -or
    -not [string]::Equals([IO.Path]::GetFullPath($running[0].Path), $exe, [StringComparison]::OrdinalIgnoreCase)) {
    throw 'Start exactly one candidate from ExecutablePath; mismatched/ambiguous processes are rejected.'
}
$target = $running[0]
$sourceKinds = @('copilot', 'pi')
$sourceCount = 2
if (Test-Path -LiteralPath $ConfigurationPath) {
    $configJson = Get-Content -LiteralPath $ConfigurationPath -Raw
    # Match src/config.rs Option<u64> + serde alias/default/range rules. Inspect
    # raw JSON first: ConvertFrom-Json collapses duplicate keys and [int] coerces
    # strings/fractions, either of which could certify the wrong timing bound.
    $document = [System.Text.Json.JsonDocument]::Parse($configJson)
    try {
        if ($document.RootElement.ValueKind -ne [System.Text.Json.JsonValueKind]::Object) {
            throw 'Candidate configuration must be a JSON object.'
        }
        $intervals = @($document.RootElement.EnumerateObject() | Where-Object { $_.Name -cin @('pollIntervalMs', 'poll_interval_ms') })
        if ($intervals.Count -gt 1) { throw 'Duplicate pollIntervalMs / poll_interval_ms configuration is invalid.' }
        [ulong] $effectivePollIntervalMs = 1000
        if ($intervals.Count -eq 1 -and $intervals[0].Value.ValueKind -ne [System.Text.Json.JsonValueKind]::Null) {
            $value = $intervals[0].Value
            if ($value.ValueKind -ne [System.Text.Json.JsonValueKind]::Number -or -not $value.TryGetUInt64([ref] $effectivePollIntervalMs)) {
                throw 'Configured poll interval must be an unsigned integer in milliseconds (or null for default 1000).'
            }
        }
        if ($effectivePollIntervalMs -lt 500 -or $effectivePollIntervalMs -gt 10000) {
            throw 'Configured poll interval must be within 500-10000 milliseconds.'
        }
        if ($effectivePollIntervalMs -ne $PollIntervalMs) {
            throw "PollIntervalMs does not match the effective candidate configuration ($effectivePollIntervalMs ms); pass the matching -PollIntervalMs."
        }
    } finally { $document.Dispose() }
    # Keep case-distinct unknown fields harmless, as in the runtime. Cadence
    # duplicates have already been rejected before this dictionary conversion.
    $config = $configJson | ConvertFrom-Json -AsHashtable
    if ($config.Keys -ccontains 'sources') {
        $sourceKinds = @($config.sources | ForEach-Object { $_.kind } | Sort-Object -Unique)
        $sourceCount = @($config.sources).Count
        $wslSources = @($config.sources | Where-Object { $_.path -like '\\wsl*' })
        if ($wslSources.Count -eq 0) { throw 'Configure actual WSL UNC sources before collecting this field journey.' }
    } else { throw 'Explicit native and WSL sources required for this field journey.' }
} else { throw 'Configure actual native+WSL sources before running the field collector.' }
if ($sourceKinds -notcontains 'pi' -or $sourceKinds -notcontains 'copilot') { throw 'Both pi and Copilot sources are required.' }
$wslVersion = (& wsl.exe --version 2>&1 | Out-String).Trim()
if ($LASTEXITCODE -ne 0) { throw 'Could not obtain actual WSL version; update WSL and retry.' }
$os = Get-CimInstance Win32_OperatingSystem
Write-Host 'Prepare actual native+WSL pi and Copilot sessions using synthetic activity only.'
$observedCount = 0
$loadText = Read-Host 'Count actual live sessions in the persistent tray menu; enter that integer (not a capability answer)'
if (-not [int]::TryParse($loadText, [ref] $observedCount) -or $observedCount -ne $SyntheticSessionCount) {
    throw 'Observed load differs from requested SyntheticSessionCount; prepare that load or explicitly set the actual count.'
}
$report = [ordered]@{
    evidenceTier = 'interactive-windows-wsl-observations'; startedUtc = [DateTime]::UtcNow.ToString('o')
    buildSha = $BuildSha; worktreeDirty = $dirty; executableSha256 = $digest
    pid = $target.Id; executablePath = $exe; processStartUtc = $target.StartTime.ToUniversalTime().ToString('o')
    windows = @{ caption = $os.Caption; version = $os.Version; build = $os.BuildNumber }
    wslVersion = $wslVersion; sourceKinds = $sourceKinds; sourceCount = $sourceCount
    syntheticSessionCount = $observedCount; loadEvidence = 'operator-counted-actual-live-menu-rows'; pollIntervalMs = $PollIntervalMs
    timingMethod = 'stopwatch-before-trigger-to-human-observation-conservative-upper-bound'
    cases = @(); completed = $false; passed = $false
}
$parent = Split-Path -Parent ([IO.Path]::GetFullPath($EvidencePath))
New-Item -ItemType Directory -Path $parent -Force | Out-Null
function Save-Evidence {
    $report | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $EvidencePath -Encoding utf8
}
function Observe-Case {
    param([string] $Id, [string] $Action, [int] $MaximumMs = 0, [int] $MinimumMs = 0)
    $current = @(Get-Process -Name ([IO.Path]::GetFileNameWithoutExtension($exe)) -ErrorAction SilentlyContinue)
    if ($current.Count -ne 1 -or -not $current[0].Path -or
        -not [string]::Equals($current[0].Path, $exe, [StringComparison]::OrdinalIgnoreCase)) {
        throw 'Candidate PID/path unavailable or mismatched before field case.'
    }
    if ((Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash.ToLowerInvariant() -ne $digest) { throw 'Candidate digest changed.' }
    Write-Host "`n[$Id] $Action"
    Write-Host 'Use actual native/WSL sessions and synthetic activity. No mock/capability answer counts.'
    [void](Read-Host 'Prepare first. Press Enter BEFORE triggering the action (starts the conservative timer)')
    $utc = [DateTime]::UtcNow.ToString('o')
    $watch = [Diagnostics.Stopwatch]::StartNew()
    if ($MinimumMs -gt 0) { Start-Sleep -Milliseconds $MinimumMs }
    [void](Read-Host 'Press Enter AFTER observing the requested outcome or failure')
    $watch.Stop()
    do { $outcome = Read-Host 'Actual observed result: type PASS or FAIL (never inferred from capability)' } while ($outcome -cnotin @('PASS', 'FAIL'))
    $elapsed = [math]::Round($watch.Elapsed.TotalMilliseconds, 2)
    $timingPassed = ($MaximumMs -eq 0 -or $elapsed -le $MaximumMs) -and $elapsed -ge $MinimumMs
    $report.cases += [ordered]@{ id = $Id; startedUtc = $utc; observedUtc = [DateTime]::UtcNow.ToString('o')
        pid = $current[0].Id; executablePath = $exe; executableSha256 = $digest
        operatorObserved = $outcome; elapsedMs = $elapsed; maximumMs = $MaximumMs; minimumMs = $MinimumMs
        timingPassed = $timingPassed; passed = ($outcome -ceq 'PASS' -and $timingPassed) }
    Save-Evidence
}
Save-Evidence
$deadline = 2 * $PollIntervalMs + 500
foreach ($origin in @('native', 'wsl')) {
    foreach ($dialog in @('confirm', 'select', 'input', 'editor', 'custom', 'manual-settings', 'manual-compaction')) {
        Observe-Case "$origin-$dialog-attention" "Open a REAL $origin pi $dialog blocking dialog; observe correct label/source/IDs and Input required in the tray/notification." $deadline
        Observe-Case "$origin-$dialog-clear" 'Resolve/reject that dialog and observe prior state restored without a duplicate notification.' $deadline
    }
    Observe-Case "$origin-final-error" "Cause a genuine FINAL settled $origin pi run error, not a recovered tool failure; observe one attributed error." $deadline
    Observe-Case "$origin-error-deduplicated" 'Leave that acknowledged error unchanged; no duplicate notification throughout THREE intervals.' 0 (3 * $PollIntervalMs)
    Observe-Case "$origin-error-dialog" 'Open/answer a real dialog over that error; verify SAME error restored without error replay.'
    Observe-Case "$origin-new-work-clear" 'Start new real pi work; verify previous error clears.' $deadline
    Observe-Case "$origin-completion-quiet" 'Complete successfully; verify no notification throughout at least THREE polling intervals.' 0 (3 * $PollIntervalMs)
    Observe-Case "$origin-abort-recovered-quiet" 'Abort a run and recover a tool failure in subsequent real work; no attention unless an explicit dialog opens.' 0 (3 * $PollIntervalMs)
    Observe-Case "$origin-copilot-persisted-error" "Append a NEW persisted session error through actual $origin Copilot; observe one attributed error. Historical/tool-only/input prompts do NOT qualify." $deadline
    Observe-Case "$origin-copilot-new-turn-clear" 'Start a genuine new Copilot turn and verify the persisted session error clears.' $deadline
}
Observe-Case 'same-directory-identity' 'Keep TWO actual same-directory pi sessions alive with distinct manual labels/session+instance IDs; identify each independently.'
Observe-Case 'suppression-persistent-list' 'Enable Windows notification suppression/Focus Assist; trigger actual input/error and verify persistent attention-first menu/counts despite suppression.'
Observe-Case 'pi-reload' 'Reload pi; old owned status disappears, new instance is healthy; a NEW real dialog alerts once.' $deadline
Observe-Case 'pi-crash-stale' 'Force-exit a real publisher while waiting; input becomes Unavailable after heartbeat expiry, never phantom waiting.' (10000 + 2 * $PollIntervalMs)
Observe-Case 'explorer-no-replay' 'Restart Explorer AFTER acknowledged attention; tray re-registers and persistent list remains, with NO replay of the same error/dialog.' 0 (3 * $PollIntervalMs)
Observe-Case 'wsl-shutdown-after-read' 'Immediately after a successful WSL source read, shut down that WSL distro; observe actual unavailable diagnostics, not falsely healthy input.' (10000 + 2 * $PollIntervalMs)
Observe-Case 'native-isolation' 'While WSL is shut down/stalled, trigger native activity; verify native monitoring continues for at least THREE ticks and tray remains responsive.' 0 (3 * $PollIntervalMs)
Observe-Case 'unc-wake-observed' 'With WSL stopped, allow configured UNC polling; inspect actual WSL state. Record whether UNC access wakes/keeps it alive in the next question (either behavior is allowed).'
do { $wake = Read-Host 'Actual UNC behavior observed: type WOKE, STAYED-STOPPED or FAIL' } while ($wake -cnotin @('WOKE', 'STAYED-STOPPED', 'FAIL'))
$report.uncWakeObservation = $wake
Save-Evidence
Observe-Case 'wsl-recovery' 'Intentionally recover WSL. After successful file access, observe healthy current state and a NEW real attention episode.' $deadline
Observe-Case 'source-removal-restart-no-reads' 'Remove WSL source from config, Quit/restart TrayGoblin, use an actual Windows file-access trace to confirm NO subsequent reads of that UNC root; native source still works.'
Observe-Case 'clean-quit' 'Choose Quit; icon and candidate process disappear promptly, with no background collector left.'
$report.completed = $true
$report.finishedUtc = [DateTime]::UtcNow.ToString('o')
$report.passed = @($report.cases | Where-Object { -not $_.passed }).Count -eq 0 -and $wake -cne 'FAIL'
if ((Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash.ToLowerInvariant() -ne $digest) { throw 'Executable changed during collection.' }
Save-Evidence
if (-not $report.passed) { throw 'One or more actual field observations/timings failed; preserve this report and investigate.' }
Write-Host 'Interactive observations recorded. Independent candidate review and matching performance report are still required.'
