<#
.SYNOPSIS
isolate_modes.ps1's measurement loop, with `main` (origin/main, built in
..\glazewm-baseline) available as a mode beside the tryout build's
GLAZEWM_ISOLATE modes. Modes are run in turn, round after round.

Unlike isolate_modes.ps1, a failing scenario is recorded and skipped rather
than ending the run, and the WM is always restarted as it was at the end.
#>
param(
  [Parameter(Mandatory)] [string] $Target,
  [Parameter(Mandatory)] [string] $ResultsFile,
  [string[]] $Modes = @('main'),
  [string[]] $Scenarios = @('move', 'float', 'minimize', 'resize', 'switch'),
  [int] $Rounds = 3,
  [int] $Bursts = 10,
  [int] $Load = 0,
  [int] $CpuLoad = 0,
  # Process names, left to right, to arrange the focused workspace's tiles
  # in after every restart. A restart otherwise alternates between two
  # orders, which with an even number of modes locks each mode to one.
  [string[]] $TileOrder = @(),
  # Only checks the arguments and the builds, starting nothing.
  [switch] $DryRun
)

$ErrorActionPreference = 'Stop'

# `powershell -File` binds `-Modes a,b` as the one string 'a,b'.
$Modes = @($Modes -split '\s*,\s*' | Where-Object { $_ })
$Scenarios = @($Scenarios -split '\s*,\s*' | Where-Object { $_ })
$TileOrder = @($TileOrder -split '\s*,\s*' | Where-Object { $_ })

$repo = 'C:\Users\googlers\glazewm'
$baseRepo = 'C:\Users\googlers\glazewm-baseline'
$current = @{
  Exe    = "$repo\target\release\glazewm.exe"
  Bench  = "$repo\target\release\examples\perf_bench.exe"
  Config = ''
}
$baseline = @{
  Exe    = "$baseRepo\target\release\glazewm.exe"
  # The tryout branch's perf_bench source, built against origin/main's IPC
  # types: the tryout build's `WorkspaceDto` has fields origin/main lacks.
  Bench  = "$baseRepo\target\release\examples\perf_bench.exe"
  Config = Join-Path $PSScriptRoot 'baseline-config.yaml'
}
$layoutFile = Join-Path $PSScriptRoot 'layout.txt'

function Get-Build([string] $Mode) {
  if ($Mode -eq 'main') { $baseline } else { $current }
}

function Get-Isolate([string] $Mode) {
  if ($Mode -in 'main', 'default') { '' } else { $Mode -replace '\+', ',' }
}

# Arguments of a process's command line, without its executable.
function Get-Arguments([string] $CommandLine) {
  $CommandLine -replace '^\s*("[^"]*"|\S+)\s*', ''
}

# Running WMs: started with no arguments or with `start`, unlike the
# short-lived CLI calls the same executable serves.
function Get-WmProcesses {
  @(Get-CimInstance Win32_Process -Filter "Name = 'glazewm.exe'" |
    Where-Object {
      $arguments = Get-Arguments $_.CommandLine
      $arguments -eq '' -or $arguments -match '^start\b'
    })
}

function Wait-WmReady([string] $Cli) {
  $ErrorActionPreference = 'Continue'
  for ($i = 0; $i -lt 60; $i++) {
    # A GUI app: only with its output piped does PowerShell wait for it and
    # set `$LASTEXITCODE`.
    & $Cli query monitors 2>$null | Out-Null
    if ($LASTEXITCODE -eq 0) { return }
    Start-Sleep -Milliseconds 500
  }
  throw 'GlazeWM did not start answering within 30 seconds.'
}

# Asks every running WM to exit and waits for them to end.
function Stop-AllWm {
  $ErrorActionPreference = 'Continue'
  foreach ($process in Get-WmProcesses) {
    & $process.ExecutablePath command wm-exit 2>$null | Out-Null
    Wait-Process -Id $process.ProcessId -Timeout 20 -ErrorAction SilentlyContinue
    if (Get-Process -Id $process.ProcessId -ErrorAction SilentlyContinue) {
      Stop-Process -Id $process.ProcessId -Force
      Write-Host "  GlazeWM $($process.ProcessId) did not exit; killed it."
    }
  }
}

function Start-Wm([string] $Exe, [string] $Arguments, [bool] $Perf, [string] $Isolate) {
  $env:GLAZEWM_PERF = $(if ($Perf) { '1' } else { $null })
  $env:GLAZEWM_ISOLATE = $(if ($Isolate) { $Isolate } else { $null })
  try {
    Start-Process -FilePath $Exe -ArgumentList $Arguments
  } finally {
    Remove-Item Env:GLAZEWM_PERF, Env:GLAZEWM_ISOLATE -ErrorAction SilentlyContinue
  }
}

# Process names of the displayed workspace's windows, in tree order. A
# restart can reorder tiles, which changes the target's neighbours.
function Get-TileOrder([string] $Cli) {
  $ErrorActionPreference = 'Continue'
  $json = & $Cli query workspaces 2>$null | Out-String
  $workspace = ($json | ConvertFrom-Json).data.workspaces |
    Where-Object { $_.hasFocus } | Select-Object -First 1
  $names = New-Object System.Collections.Generic.List[string]
  $walk = {
    param($children)
    foreach ($child in $children) {
      if ($child.type -eq 'window') { $names.Add($child.processName) }
      elseif ($child.children) { & $walk $child.children }
    }
  }
  & $walk $workspace.children
  "ws $($workspace.name): $($names -join ', ')"
}

# Sends `$Message` over the WM's IPC socket and returns its response. One
# connection serves many commands, rather than a CLI process each.
function Invoke-WmMessage($Socket, [string] $Message) {
  $none = [Threading.CancellationToken]::None
  $bytes = [Text.Encoding]::UTF8.GetBytes($Message)
  $Socket.SendAsync([ArraySegment[byte]]::new($bytes),
    [Net.WebSockets.WebSocketMessageType]::Text, $true, $none).Wait()
  $buffer = New-Object byte[] 65536
  while ($true) {
    $stream = New-Object IO.MemoryStream
    do {
      $result = $Socket.ReceiveAsync([ArraySegment[byte]]::new($buffer), $none).Result
      $stream.Write($buffer, 0, $result.Count)
    } while (-not $result.EndOfMessage)
    $response = [Text.Encoding]::UTF8.GetString($stream.ToArray()) | ConvertFrom-Json
    if ($response.messageType -eq 'client_response' -and
        $response.clientMessage -eq $Message) {
      if (-not $response.success) { throw "'$Message' failed: $($response.error)" }
      return $response
    }
  }
}

# Tiled windows of the focused workspace, in tree order.
function Get-Tiles($Socket) {
  $workspace = (Invoke-WmMessage $Socket 'query workspaces').data.workspaces |
    Where-Object { $_.hasFocus } | Select-Object -First 1
  $tiles = New-Object System.Collections.Generic.List[object]
  $walk = {
    param($children)
    foreach ($child in $children) {
      if ($child.type -eq 'window') {
        if ($child.state.type -eq 'tiling') { $tiles.Add($child) }
      } elseif ($child.children) {
        & $walk $child.children
      }
    }
  }
  & $walk $workspace.children
  $tiles
}

# Moves the focused workspace's tiles into `$Order`, by process name. Of
# several windows of one process, the lowest handle goes first. Names with
# no window are skipped, and windows not named stay right of the rest.
function Set-TileOrder([string[]] $Order) {
  $none = [Threading.CancellationToken]::None
  $socket = New-Object System.Net.WebSockets.ClientWebSocket
  $socket.ConnectAsync([Uri] 'ws://127.0.0.1:6123', $none).Wait()
  try {
    $placed = 0
    foreach ($name in $Order) {
      $tiles = @(Get-Tiles $socket)
      $match = @(for ($k = $placed; $k -lt $tiles.Count; $k++) {
        if ($tiles[$k].processName -eq $name) {
          [pscustomobject]@{ Index = $k; Id = $tiles[$k].id; Handle = $tiles[$k].handle }
        }
      }) | Sort-Object Handle | Select-Object -First 1
      if (-not $match) { continue }
      for ($k = $match.Index; $k -gt $placed; $k--) {
        Invoke-WmMessage $socket "command --id $($match.Id) move --direction left" | Out-Null
      }
      $placed++
    }
  } finally {
    $socket.CloseAsync(
      [Net.WebSockets.WebSocketCloseStatus]::NormalClosure, '', $none).Wait()
  }
}

function Add-Result([string] $Line) {
  Write-Host $Line
  $Line | Add-Content -Path $ResultsFile
}

foreach ($path in $current.Exe, $current.Bench) {
  if (-not (Test-Path $path)) { throw "Missing $path." }
}
if ($Modes -contains 'main') {
  foreach ($path in $baseline.Exe, $baseline.Bench, $baseline.Config) {
    if (-not (Test-Path $path)) { throw "Missing $path." }
  }
}

$userWm = @(Get-WmProcesses) | Select-Object -First 1
if (-not $userWm) { throw 'Start GlazeWM as usual first.' }
$userExe = $userWm.ExecutablePath
$userArgs = Get-Arguments $userWm.CommandLine
if ($userArgs -eq '') { $userArgs = 'start' }

Write-Host "Modes: $($Modes -join ', '); scenarios: $($Scenarios -join ', ');"
Write-Host "rounds $Rounds, bursts $Bursts, load $Load, CPU load $CpuLoad;"
Write-Host "tile order: $(if ($TileOrder) { $TileOrder -join ', ' } else { 'as restarted' });"
Write-Host "user WM: $userExe $userArgs; results: $ResultsFile"
if ($DryRun) { return }

& $current.Bench save-layout $layoutFile
if ($LASTEXITCODE) { throw 'Saving the window layout failed.' }

$header = & $current.Bench summary-header
$rows = [ordered]@{}
foreach ($scenario in $Scenarios) { $rows[$scenario] = @() }

Set-Content -Path $ResultsFile -Value (
  "Bench run, $(Get-Date -Format s), modes $($Modes -join ','), target " +
  "$Target, $Rounds round(s) of $Bursts bursts per scenario and mode, " +
  "load $Load, CPU load $CpuLoad thread(s) of $([Environment]::ProcessorCount)"
)

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
      $build = Get-Build $mode
      $isolate = Get-Isolate $mode
      $label = "$mode #$round"
      Write-Host "Round $round/${Rounds}: restarting GlazeWM as '$mode'..."

      try {
        Stop-AllWm
        $arguments = 'start'
        if ($build.Config) { $arguments += " --config `"$($build.Config)`"" }
        Start-Wm $build.Exe $arguments $true $isolate
        Wait-WmReady $build.Exe
        $count = @(Get-WmProcesses).Count
        if ($count -ne 1) { throw "$count GlazeWM processes are running." }

        & $build.Bench restore-layout $layoutFile
        if ($LASTEXITCODE) { throw 'Restoring the window layout failed.' }
        if ($TileOrder) { Set-TileOrder $TileOrder }
        Start-Sleep -Seconds 2
        Add-Result "tiles $label $(Get-TileOrder $build.Exe)"
      } catch {
        Add-Result "FAILED $label start: $_"
        continue
      }

      foreach ($scenario in $Scenarios) {
        Write-Host "  Measuring '$scenario'..."
        $ErrorActionPreference = 'Continue'
        $output = & $build.Bench --scenario $scenario --target $Target `
          --bursts $Bursts --load $Load --label $label 2>&1 |
          ForEach-Object { "$_" }
        $exitCode = $LASTEXITCODE
        $ErrorActionPreference = 'Stop'
        $output | Add-Content -Path $ResultsFile
        if ($exitCode) {
          Add-Result "FAILED $label $scenario (exit $exitCode)"
          continue
        }
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
  Stop-AllWm
  Start-Wm $userExe $userArgs $false ''
  Wait-WmReady $userExe
  & $current.Bench restore-layout $layoutFile
}

$tables = foreach ($scenario in $Scenarios) {
  ''
  "## $scenario"
  ''
  $header
  foreach ($mode in $Modes) {
    $rows[$scenario] | Where-Object { $_ -like "| $mode #*" }
  }
}
$tables | Add-Content -Path $ResultsFile
$tables | Write-Host

Write-Host "Done. Results are in $ResultsFile"
