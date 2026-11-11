<#
.SYNOPSIS
Compares the ways moving windows are drawn (`GLAZEWM_ISOLATE` modes) on
this PC, with nothing to set up.

.DESCRIPTION
Builds the WM and `perf_bench`, then for each mode restarts GlazeWM with the
frame profiler on and that `GLAZEWM_ISOLATE` value, puts every window back
on its workspace, and runs the same benchmark scenarios on it. Modes are run
in turn, round after round, so drift over the session (a browser warming
up, thermals) spreads over all of them rather than favouring the first.
Then it restarts GlazeWM the way it was running and restores the layout.

The modes compared by default:

- `nopeeks`: the old drawing, through `ResizeSession`.
- `default`: the new drawing, pinned peeks (`GLAZEWM_ISOLATE` unset).
- `peeksfirst`: the new drawing, with peek frames drawn before the main
  loop's catch-up.

Results are printed, one table per scenario, and saved to
~/.glzr/glazewm/isolate-bench.txt. Lower is better in every column, except
`frames`; `interval p50 ms` should match the refresh period, and the gap to
`interval p90 ms` shows how evenly frames land.

Desktop PC only: it restarts the WM and drives bursts of window moves over
IPC (see CLAUDE.md's build targets).

.PARAMETER Target
Process name of the tiled window to animate, e.g. `chrome` or `notepad`.
It needs a tiled neighbour on its workspace for `move` and `resize`.

.PARAMETER Modes
`GLAZEWM_ISOLATE` values to compare, `default` leaving it unset. A mode
of several flags joins them with `+`, e.g. `nopeeks+peeksfirst`.

.PARAMETER Load
IPC queries queued as each animation starts (`perf_bench --load`), so the
main loop has a backlog to catch up on between frames. `peeksfirst` only
differs from `default` with one; compare those two with e.g. `-Load 40`.

.PARAMETER CpuLoad
Busy-loop threads kept running while measuring, at normal priority, so the
WM competes for the CPU and works through its backlog slower, as on a
busy or weaker machine. `[Environment]::ProcessorCount` loads every core.

.EXAMPLE
powershell -ExecutionPolicy Bypass -File packages\wm-cli\examples\perf_bench\isolate_modes.ps1 -Target chrome

.EXAMPLE
powershell -ExecutionPolicy Bypass -File packages\wm-cli\examples\perf_bench\isolate_modes.ps1 -Target chrome -Modes default,peeksfirst -Scenarios move,resize -Load 40

.EXAMPLE
powershell -ExecutionPolicy Bypass -File packages\wm-cli\examples\perf_bench\isolate_modes.ps1 -Target chrome -Modes default,peeksfirst -Scenarios move,resize -Load 400 -CpuLoad 16
#>
param(
  [Parameter(Mandatory)] [string] $Target,
  [string[]] $Modes = @('nopeeks', 'default', 'peeksfirst'),
  [string[]] $Scenarios = @('move', 'float', 'minimize', 'resize', 'switch'),
  [int] $Rounds = 2,
  [int] $Bursts = 10,
  [int] $Load = 0,
  [int] $CpuLoad = 0
)

$ErrorActionPreference = 'Stop'
Set-Location (Resolve-Path (Join-Path $PSScriptRoot '..\..\..\..'))

# `powershell -File` binds `-Modes a,b` as the one string 'a,b'.
$Modes = @($Modes -split '\s*,\s*' | Where-Object { $_ })
$Scenarios = @($Scenarios -split '\s*,\s*' | Where-Object { $_ })

# Arguments of a process's command line, without its executable.
function Get-Arguments([string] $CommandLine) {
  $CommandLine -replace '^\s*("[^"]*"|\S+)\s*', ''
}

# The running WM: started with no arguments or with `start`, unlike the
# short-lived CLI calls the same executable serves.
function Get-WmProcess {
  Get-CimInstance Win32_Process -Filter "Name = 'glazewm.exe'" |
    Where-Object {
      $arguments = Get-Arguments $_.CommandLine
      $arguments -eq '' -or $arguments -match '^start\b'
    } |
    Select-Object -First 1
}

# Waits until a WM answers queries.
function Wait-WmReady([string] $Cli) {
  # Windows PowerShell turns a native command's redirected stderr into a
  # terminating error under 'Stop'.
  $ErrorActionPreference = 'Continue'
  for ($i = 0; $i -lt 60; $i++) {
    & $Cli query monitors *> $null
    if ($LASTEXITCODE -eq 0) { return }
    Start-Sleep -Milliseconds 500
  }
  throw 'GlazeWM did not start answering within 30 seconds.'
}

# Asks the WM to exit and waits for its process to end.
function Stop-Wm([string] $Cli, $Process) {
  $ErrorActionPreference = 'Continue'
  & $Cli command wm-exit *> $null
  Wait-Process -Id $Process.ProcessId -Timeout 20 -ErrorAction SilentlyContinue
  if (Get-Process -Id $Process.ProcessId -ErrorAction SilentlyContinue) {
    throw 'GlazeWM did not exit within 20 seconds.'
  }
}

# Starts `$Exe` with `$Arguments`, the profiler on if `$Perf`, and
# `GLAZEWM_ISOLATE` set to `$Isolate` unless it is empty. The variables
# are only set for the started process.
function Start-Wm([string] $Exe, [string] $Arguments, [bool] $Perf, [string] $Isolate) {
  $env:GLAZEWM_PERF = $(if ($Perf) { '1' } else { $null })
  $env:GLAZEWM_ISOLATE = $(if ($Isolate) { $Isolate } else { $null })
  try {
    Start-Process -FilePath $Exe -ArgumentList $Arguments
  } finally {
    Remove-Item Env:GLAZEWM_PERF, Env:GLAZEWM_ISOLATE -ErrorAction SilentlyContinue
  }
}

Write-Host 'Building...'
cargo build --release -p wm
if ($LASTEXITCODE) { throw 'Building the WM failed.' }
cargo build --release -p wm-cli --example perf_bench
if ($LASTEXITCODE) { throw 'Building perf_bench failed.' }

$wm = (Resolve-Path 'target\release\glazewm.exe').Path
$bench = (Resolve-Path 'target\release\examples\perf_bench.exe').Path
$dataDir = Join-Path $HOME '.glzr\glazewm'
$layoutFile = Join-Path $dataDir 'isolate-bench-layout.txt'
$resultsFile = Join-Path $dataDir 'isolate-bench.txt'

$userWm = Get-WmProcess
if (-not $userWm) {
  throw 'Start GlazeWM as usual first: this restarts it the way it runs.'
}
$userExe = $userWm.ExecutablePath
$userArgs = Get-Arguments $userWm.CommandLine
if ($userArgs -eq '') { $userArgs = 'start' }

& $bench save-layout $layoutFile
if ($LASTEXITCODE) { throw 'Saving the window layout failed.' }

$header = & $bench summary-header
# Table rows per scenario, in the order they ran.
$rows = [ordered]@{}
foreach ($scenario in $Scenarios) { $rows[$scenario] = @() }

Set-Content -Path $resultsFile -Value (
  "Isolate benchmark, $(Get-Date -Format s), target $Target, " +
  "$Rounds round(s) of $Bursts bursts per scenario and mode, load $Load, " +
  "CPU load $CpuLoad thread(s) of $([Environment]::ProcessorCount)"
)

# One job per busy thread, each its own process, for the whole run.
$cpuJobs = @()

try {
  if ($CpuLoad -gt 0) {
    Write-Host "Starting $CpuLoad CPU load thread(s)..."
    $cpuJobs = @(1..$CpuLoad | ForEach-Object {
      Start-Job -ScriptBlock { while ($true) {} }
    })
  }

  for ($round = 1; $round -le $Rounds; $round++) {
    foreach ($mode in $Modes) {
      $isolate = $(if ($mode -eq 'default') { '' } else { $mode -replace '\+', ',' })
      Write-Host "Round $round/${Rounds}: restarting GlazeWM as '$mode'..."
      $running = Get-WmProcess
      if ($running) { Stop-Wm $wm $running }
      Start-Wm $wm $userArgs $true $isolate
      Wait-WmReady $wm

      # A restart manages every window onto its monitor's displayed
      # workspace, so each mode starts from the same layout.
      & $bench restore-layout $layoutFile
      if ($LASTEXITCODE) { throw 'Restoring the window layout failed.' }
      Start-Sleep -Seconds 2

      foreach ($scenario in $Scenarios) {
        Write-Host "  Measuring '$scenario'..."
        $output = & $bench --scenario $scenario --target $Target `
          --bursts $Bursts --load $Load --label "$mode #$round"
        if ($LASTEXITCODE) { throw "Scenario '$scenario' failed." }
        $output | Add-Content -Path $resultsFile
        $rows[$scenario] += $output | Where-Object { $_ -like '| *' }
      }
    }
  }
} finally {
  if ($cpuJobs) {
    Write-Host 'Stopping the CPU load...'
    $cpuJobs | Stop-Job
    $cpuJobs | Remove-Job -Force
  }

  Write-Host 'Restarting GlazeWM as it was...'
  $benchWm = Get-WmProcess
  if ($benchWm) { Stop-Wm $wm $benchWm }
  Start-Wm $userExe $userArgs $false ''
  Wait-WmReady $wm
  & $bench restore-layout $layoutFile
}

$tables = foreach ($scenario in $Scenarios) {
  ''
  "## $scenario"
  ''
  $header
  # Grouped by mode, so a mode's rounds sit together.
  foreach ($mode in $Modes) {
    $rows[$scenario] | Where-Object { $_ -like "| $mode #*" }
  }
}
$tables | Add-Content -Path $resultsFile
$tables | Write-Host

Write-Host "Done. Results are in $resultsFile"
