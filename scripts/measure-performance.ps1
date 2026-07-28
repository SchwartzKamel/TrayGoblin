<#
.SYNOPSIS
    Measures TrayGoblin's idle working set and CPU use against the release
    thresholds.

.DESCRIPTION
    Samples the running TrayGoblin process for a fixed window and fails when
    the peak working set exceeds 50 MB or the average CPU use exceeds 5%.
    Output is content-free: only the process name, sample counts, and resource
    numbers are reported. Copilot session state is never read.

    Run this on Windows 10 or 11 before promoting a preview build to stable.

.PARAMETER Path
    Executable to start when no TrayGoblin process is already running. The
    started process is stopped again when the measurement finishes.

.PARAMETER ProcessName
    Process name to measure. Defaults to "tray-goblin".

.PARAMETER DurationSeconds
    Length of the measurement window. Defaults to 30 seconds.

.PARAMETER SampleIntervalSeconds
    Seconds between samples. Defaults to 1 second.

.PARAMETER MaxWorkingSetMb
    Working-set budget in MB. Defaults to 50.

.PARAMETER MaxIdleCpuPercent
    Idle CPU budget in percent of one machine's total CPU capacity. Defaults
    to 5.

.PARAMETER JsonPath
    Optional path for a content-free JSON report.

.EXAMPLE
    pwsh -NoProfile -File scripts/measure-performance.ps1 -DurationSeconds 30
#>
[CmdletBinding()]
param(
    [string] $Path,
    [string] $ProcessName = 'tray-goblin',
    [ValidateRange(5, 3600)]
    [int] $DurationSeconds = 30,
    [ValidateRange(1, 60)]
    [int] $SampleIntervalSeconds = 1,
    [ValidateRange(1, 4096)]
    [int] $MaxWorkingSetMb = 50,
    [ValidateRange(1, 100)]
    [int] $MaxIdleCpuPercent = 5,
    [string] $JsonPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$script:ApplicationName = 'TrayGoblin'

function Write-Step {
    param([Parameter(Mandatory)][string] $Message)
    Write-Host "[$script:ApplicationName] $Message"
}

function Stop-MeasurementScript {
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

    return $true
}

function Resolve-DefaultExecutable {
    $localAppData = $env:LOCALAPPDATA
    if (-not $localAppData) {
        $localAppData = [Environment]::GetFolderPath('LocalApplicationData')
    }

    if (-not $localAppData) {
        return $null
    }

    return (Join-Path (Join-Path (Join-Path $localAppData 'Programs') $script:ApplicationName) 'tray-goblin.exe')
}

if (-not (Test-IsWindowsHost)) {
    Write-Warning 'The 50 MB / 5% budget is specified for Windows 10 and 11; results from another host are indicative only.'
}

$startedProcess = $null
$existing = @(Get-Process -Name $ProcessName -ErrorAction SilentlyContinue)

if ($existing.Count -gt 1) {
    Stop-MeasurementScript -Message "$($existing.Count) processes named `"$ProcessName`" are running, so the measurement would be ambiguous." `
        -Remedy 'Quit the extra instances from their tray menus and run this script again.'
}

if ($existing.Count -eq 1) {
    $target = $existing[0]
    Write-Step "Measuring the running `"$ProcessName`" process."
} else {
    if (-not $Path) {
        $Path = Resolve-DefaultExecutable
    }

    if (-not $Path -or -not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        Stop-MeasurementScript -Message "no `"$ProcessName`" process is running and no executable was found to start." `
            -Remedy 'Start TrayGoblin first, or re-run with -Path "<path to tray-goblin.exe>".'
    }

    $Path = (Resolve-Path -LiteralPath $Path).Path
    Write-Step 'Starting a temporary instance for measurement.'

    try {
        $startedProcess = Start-Process -FilePath $Path -PassThru
    } catch {
        Stop-MeasurementScript -Message 'the executable could not be started for measurement.' `
            -Remedy "Confirm that `"$Path`" is the Windows TrayGoblin build and can run on this host."
    }

    # Allow the tray shell to register its icon and settle before sampling.
    Start-Sleep -Seconds 3
    $target = $startedProcess
}

try {
    $target.Refresh()
    if ($target.HasExited) {
        Stop-MeasurementScript -Message "the `"$ProcessName`" process exited before the measurement started." `
            -Remedy 'Start TrayGoblin, confirm the tray icon appears, then run this script again.'
    }

    $processorCount = [Environment]::ProcessorCount
    $startCpu = $target.TotalProcessorTime
    $startClock = [System.Diagnostics.Stopwatch]::StartNew()

    $peakWorkingSetBytes = $target.WorkingSet64
    $sampleCount = 1
    $deadline = (Get-Date).AddSeconds($DurationSeconds)

    Write-Step "Sampling for $DurationSeconds second(s) every $SampleIntervalSeconds second(s)."

    while ((Get-Date) -lt $deadline) {
        Start-Sleep -Seconds $SampleIntervalSeconds

        $target.Refresh()
        if ($target.HasExited) {
            Stop-MeasurementScript -Message "the `"$ProcessName`" process exited during the measurement window." `
                -Remedy 'Investigate why TrayGoblin stopped, then repeat the measurement.'
        }

        if ($target.WorkingSet64 -gt $peakWorkingSetBytes) {
            $peakWorkingSetBytes = $target.WorkingSet64
        }

        $sampleCount++
    }

    $startClock.Stop()
    $target.Refresh()
    $cpuSeconds = ($target.TotalProcessorTime - $startCpu).TotalSeconds
    $elapsedSeconds = $startClock.Elapsed.TotalSeconds
} finally {
    if ($startedProcess -and -not $startedProcess.HasExited) {
        Stop-Process -Id $startedProcess.Id -ErrorAction SilentlyContinue
    }
}

if ($elapsedSeconds -le 0) {
    Stop-MeasurementScript -Message 'the measurement window recorded no elapsed time.' `
        -Remedy 'Re-run with a longer -DurationSeconds value.'
}

$peakWorkingSetMb = [math]::Round($peakWorkingSetBytes / 1MB, 2)
$averageCpuPercent = [math]::Round((100.0 * $cpuSeconds) / ($elapsedSeconds * $processorCount), 2)

$workingSetPassed = $peakWorkingSetMb -le $MaxWorkingSetMb
$cpuPassed = $averageCpuPercent -le $MaxIdleCpuPercent
$passed = $workingSetPassed -and $cpuPassed

$report = [ordered]@{
    process             = $ProcessName
    durationSeconds     = [math]::Round($elapsedSeconds, 2)
    samples             = $sampleCount
    logicalProcessors   = $processorCount
    peakWorkingSetMb    = $peakWorkingSetMb
    maxWorkingSetMb     = $MaxWorkingSetMb
    workingSetPassed    = $workingSetPassed
    averageCpuPercent   = $averageCpuPercent
    maxIdleCpuPercent   = $MaxIdleCpuPercent
    cpuPassed           = $cpuPassed
    passed              = $passed
}

Write-Step ("Peak working set: {0} MB (budget {1} MB) - {2}" -f $peakWorkingSetMb, $MaxWorkingSetMb, $(if ($workingSetPassed) { 'PASS' } else { 'FAIL' }))
Write-Step ("Average CPU: {0}% of {1} logical processor(s) (budget {2}%) - {3}" -f $averageCpuPercent, $processorCount, $MaxIdleCpuPercent, $(if ($cpuPassed) { 'PASS' } else { 'FAIL' }))
Write-Step ("Samples: {0} over {1} second(s)." -f $sampleCount, [math]::Round($elapsedSeconds, 2))

if ($JsonPath) {
    try {
        $parent = Split-Path -Parent $JsonPath
        if ($parent -and -not (Test-Path -LiteralPath $parent -PathType Container)) {
            New-Item -ItemType Directory -Path $parent -Force | Out-Null
        }
        ($report | ConvertTo-Json -Depth 3) | Set-Content -LiteralPath $JsonPath -Encoding utf8
        Write-Step "Wrote the content-free report to `"$JsonPath`"."
    } catch {
        Stop-MeasurementScript -Message "the report could not be written to `"$JsonPath`"." `
            -Remedy 'Choose a writable path for -JsonPath.'
    }
}

if (-not $passed) {
    Stop-MeasurementScript -Message 'the performance budget was exceeded, so this build must not be promoted to stable.' `
        -Remedy 'Re-measure on an idle machine; if it still fails, profile polling and icon work before releasing.'
}

Write-Step 'Performance budget satisfied.'
exit 0
