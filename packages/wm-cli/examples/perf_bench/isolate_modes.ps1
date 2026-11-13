<#
.SYNOPSIS
Compares the ways moving windows are drawn (`GLAZEWM_ISOLATE` modes) on
this PC, with nothing to set up.

.DESCRIPTION
Builds the WM and `perf_bench`, starts the `-Launch` programs, then for
each mode restarts GlazeWM with the frame profiler on and that
`GLAZEWM_ISOLATE` value, puts every window back on its workspace,
optionally puts the focused workspace's tiles in a fixed order, and runs
the same benchmark scenarios on it. Modes are run
in turn, round after round, so drift over the session (a browser warming
up, thermals) spreads over all of them rather than favouring the first.
Then it restarts GlazeWM the way it was running, restores the layout and
closes the windows it started.

The modes compared by default:

- `nopeeks`: the old drawing, through `ResizeSession`.
- `default`: the new drawing, pinned peeks (`GLAZEWM_ISOLATE` unset).
- `peeksfirst`: the new drawing, with peek frames drawn before the main
  loop's catch-up.

Results are printed, one table per scenario, and saved to
~/.glzr/glazewm/isolate-bench.txt. Lower is better in every column, except
`frames`; `interval p50 ms` should match the refresh period, and the gap to
`interval p90 ms` shows how evenly frames land. Each mode's results are
preceded by its workspace's window order, which a restart can change.

Desktop PC only: it restarts the WM, starts programs and drives bursts of
window moves over IPC (see CLAUDE.md's build targets).

.PARAMETER Target
Process name of the tiled window to animate, e.g. `chrome` or `notepad`.
It needs a tiled neighbour on its workspace for `move` and `resize`.

.PARAMETER Modes
`GLAZEWM_ISOLATE` values to compare, `default` leaving it unset. A mode
of several flags joins them with `+`, e.g. `nopeeks+peeksfirst`.

.PARAMETER Scenarios
`perf_bench` scenarios to run. `minimize` is not run by default: with
`window_minimize` off it only measures neighbours re-tiling, as `move` and
`resize` do.

.PARAMETER Load
IPC queries queued as each animation starts (`perf_bench --load`), so the
main loop has a backlog to catch up on between frames. `peeksfirst` only
differs from `default` with one; compare those two with e.g. `-Load 40`.

.PARAMETER CpuLoad
Busy-loop threads kept running while measuring, at normal priority, so the
WM competes for the CPU and works through its backlog slower, as on a
busy or weaker machine. `[Environment]::ProcessorCount` loads every core.

.PARAMETER TileOrder
Process names, left to right, to put the focused workspace's tiles in after
every restart, once the layout is restored. A restart can otherwise
reorder them, changing the target's neighbours between modes. Of several
windows of one process, the lowest handle goes first; names with no window
are skipped, and tiles not named stay right of the rest. Use it with
`-Launch` to fix both which windows the target sits among and their order;
Windows Terminal's process name is `WindowsTerminal`.

.PARAMETER Launch
Programs started once, before the first round, so every run measures the
same windows; `none` starts nothing. They open on the focused workspace.
`notepad`, `explorer`, `wt` and `chrome` (with `--new-window`) each open a
new window; any other name is started as given, its window expected from
the process named after the file. The windows of those processes that the
WM manages within 20 seconds, and didn't before, are closed at the end
through the WM's `close`; no other window is.

.PARAMETER KeepLaunched
Leaves the `-Launch` windows open at the end.

.EXAMPLE
powershell -ExecutionPolicy Bypass -File packages\wm-cli\examples\perf_bench\isolate_modes.ps1 -Target chrome

.EXAMPLE
powershell -ExecutionPolicy Bypass -File packages\wm-cli\examples\perf_bench\isolate_modes.ps1 -Target chrome -Modes default,peeksfirst -Scenarios move,resize -Load 40

.EXAMPLE
powershell -ExecutionPolicy Bypass -File packages\wm-cli\examples\perf_bench\isolate_modes.ps1 -Target chrome -Modes default,peeksfirst -Scenarios move,resize -Load 400 -CpuLoad 16

.EXAMPLE
powershell -ExecutionPolicy Bypass -File packages\wm-cli\examples\perf_bench\isolate_modes.ps1 -Target notepad -Scenarios resize -Launch notepad,explorer,chrome -TileOrder chrome,notepad,explorer

.EXAMPLE
powershell -ExecutionPolicy Bypass -File packages\wm-cli\examples\perf_bench\isolate_modes.ps1 -Target chrome -Launch none
#>
param(
  [Parameter(Mandatory)] [string] $Target,
  [string[]] $Modes = @('nopeeks', 'default', 'peeksfirst'),
  [string[]] $Scenarios = @('move', 'float', 'resize', 'switch'),
  [int] $Rounds = 2,
  [int] $Bursts = 10,
  [int] $Load = 0,
  [int] $CpuLoad = 0,
  [string[]] $TileOrder = @(),
  [string[]] $Launch = @('notepad', 'explorer', 'wt', 'chrome'),
  [switch] $KeepLaunched
)

$ErrorActionPreference = 'Stop'
Set-Location (Resolve-Path (Join-Path $PSScriptRoot '..\..\..\..'))

# `powershell -File` binds `-Modes a,b` as the one string 'a,b'.
$Modes = @($Modes -split '\s*,\s*' | Where-Object { $_ })
$Scenarios = @($Scenarios -split '\s*,\s*' | Where-Object { $_ })
$TileOrder = @($TileOrder -split '\s*,\s*' | Where-Object { $_ })
$Launch = @($Launch -split '\s*,\s*' | Where-Object { $_ -and $_ -ne 'none' })

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

# Process names of the focused workspace's windows, in tree order.
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

# Moves the focused workspace's tiles into `$Order`, by process name (see
# `-TileOrder`).
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

# The WM's managed windows, keyed by handle as `[int64]`, or `$null` if the
# query fails.
function Get-ManagedWindows([string] $Cli) {
  $ErrorActionPreference = 'Continue'
  $json = & $Cli query windows 2>$null | Out-String
  if ($LASTEXITCODE) { return $null }
  $windows = @{}
  foreach ($window in ($json | ConvertFrom-Json).data.windows) {
    $windows[[int64] $window.handle] = $window
  }
  $windows
}

# Starts each of `$Names` once (see `-Launch`) and waits up to 20 seconds
# for the WM to manage their windows. Returns the new windows' handles.
function Start-Launched([string] $Cli, [string[]] $Names) {
  # `Owner` is the window's process: launchers hand off to it and exit.
  $known = @{
    notepad  = @{ FilePath = 'notepad.exe'; Owner = 'notepad' }
    explorer = @{ FilePath = 'explorer.exe'; Owner = 'explorer' }
    wt       = @{ FilePath = 'wt.exe'; Owner = 'WindowsTerminal' }
    chrome   = @{ FilePath = 'chrome.exe'; Arguments = '--new-window'; Owner = 'chrome' }
  }
  $before = Get-ManagedWindows $Cli
  if ($null -eq $before) { throw 'Querying the windows failed.' }

  $started = @()
  foreach ($name in $Names) {
    try {
      $spec = $known[$name]
      if (-not $spec) {
        $spec = @{ FilePath = $name; Owner = [IO.Path]::GetFileNameWithoutExtension($name) }
      }
      $startArgs = @{ FilePath = $spec.FilePath; PassThru = $true; ErrorAction = 'Stop' }
      if ($spec.Arguments) { $startArgs.ArgumentList = $spec.Arguments }
      $process = Start-Process @startArgs
      Write-Host "  Started '$name' (process $($process.Id))."
      $started += [pscustomobject]@{ Name = $name; Owner = $spec.Owner }
    } catch {
      Write-Warning "Could not start '$name', skipping it: $($_.Exception.Message)"
    }
  }

  $owners = @($started | ForEach-Object { $_.Owner })
  $new = @()
  $pending = @($started)
  for ($i = 0; $i -lt 40 -and $pending.Count -gt 0; $i++) {
    Start-Sleep -Milliseconds 500
    $windows = Get-ManagedWindows $Cli
    if ($null -eq $windows) { continue }
    $new = @($windows.Values | Where-Object {
      -not $before.ContainsKey([int64] $_.handle) -and $owners -contains $_.processName
    })
    $newOwners = @($new | ForEach-Object { $_.processName })
    $pending = @($started | Where-Object { $newOwners -notcontains $_.Owner })
  }
  foreach ($program in $pending) {
    Write-Warning "No window of '$($program.Name)' appeared within 20 seconds."
  }
  $new | ForEach-Object { [int64] $_.handle }
}

# Closes the windows of `$Handles` still open, through the WM's `close`.
# Container ids change on a restart, so each is looked up by handle.
function Close-Launched([string] $Cli, [int64[]] $Handles) {
  $ErrorActionPreference = 'Continue'
  $windows = Get-ManagedWindows $Cli
  if ($null -eq $windows) {
    Write-Warning 'Querying the windows failed; the started ones stay open.'
    return
  }
  foreach ($handle in $Handles) {
    $window = $windows[$handle]
    if (-not $window) { continue }
    & $Cli command --id $window.id close *> $null
    if ($LASTEXITCODE) {
      Write-Warning "Closing the $($window.processName) window $handle failed."
    }
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

# One job per busy thread, each its own process, for the whole run.
$cpuJobs = @()
# Handles of the windows `-Launch` opened: container ids change on a restart.
$launched = @()
$layoutSaved = $false

try {
  if ($Launch) {
    Write-Host "Starting $($Launch -join ', ')..."
    $launched = @(Start-Launched $wm $Launch)
  }

  & $bench save-layout $layoutFile
  if ($LASTEXITCODE) { throw 'Saving the window layout failed.' }
  $layoutSaved = $true

  $header = & $bench summary-header
  # Table rows per scenario, in the order they ran.
  $rows = [ordered]@{}
  foreach ($scenario in $Scenarios) { $rows[$scenario] = @() }

  Set-Content -Path $resultsFile -Value (
    "Isolate benchmark, $(Get-Date -Format s), target $Target, " +
    "$Rounds round(s) of $Bursts bursts per scenario and mode, load $Load, " +
    "CPU load $CpuLoad thread(s) of $([Environment]::ProcessorCount), tile " +
    "order $(if ($TileOrder) { $TileOrder -join ',' } else { 'as restarted' }), " +
    "launched $(if ($Launch) { $Launch -join ',' } else { 'none' }) " +
    "($($launched.Count) window(s))"
  )

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
      if ($TileOrder) { Set-TileOrder $TileOrder }
      Start-Sleep -Seconds 2
      $tileLine = "tiles $mode #$round $(Get-TileOrder $wm)"
      Write-Host "  $tileLine"
      $tileLine | Add-Content -Path $resultsFile

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

  if ($layoutSaved) {
    Write-Host 'Restarting GlazeWM as it was...'
    $benchWm = Get-WmProcess
    if ($benchWm) { Stop-Wm $wm $benchWm }
    Start-Wm $userExe $userArgs $false ''
    Wait-WmReady $wm
    & $bench restore-layout $layoutFile
  }

  if ($launched.Count -gt 0 -and -not $KeepLaunched) {
    Write-Host "Closing the $($launched.Count) window(s) it started..."
    Close-Launched $wm $launched
  }
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
