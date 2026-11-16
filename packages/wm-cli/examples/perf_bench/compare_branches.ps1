<#
.SYNOPSIS
Compares this checkout's WM with another branch's (`-Against`, default
`origin/main`) on this PC, with nothing to set up.

.DESCRIPTION
Builds the WM and `perf_bench` here, and the WM of `-Against` from a
`git worktree` of its own with its own target directory, both under
`target\compare\`. The worktree is removed once built; its target
directory is kept, so the next run only rebuilds what changed.

Then it starts the `-Launch` programs and, for each CPU load, round and
borders/backdrops setting, runs the two builds back to back, which one
goes first alternating: each restarts GlazeWM with the frame profiler on
(`GLAZEWM_PERF=1`) and a copy of your config that draws borders and
backdrops as asked (and animates minimize), puts every window back on its
workspace, optionally puts the focused workspace's tiles in a fixed order,
and runs the same `perf_bench` scenarios on it. This build's `perf_bench`
drives both: it reads IPC answers without this branch's types.

An error stops the run. Whether it ends or stops, it then stops the CPU
load, restarts GlazeWM the way it was running, restores the layout, closes
the windows it started and deletes the config copies.

Results go to `-ResultsFile`: each run's output, then per scenario a table
of both builds side by side (`this / against`, medians over rounds, an em
dash where a build has no such stage), then every column per setting. The raw
summary rows are kept next to it, in `<name>-rows.md`. Lower is better in
every column except `frames`; `interval p50 ms` should match the refresh
period, and the gap to `interval p90 ms` shows how evenly frames land.

Desktop PC only: it restarts the WM, starts programs and drives bursts of
window moves over IPC (see CLAUDE.md's build targets).

.PARAMETER Target
Process name of the tiled window to animate, e.g. `chrome` or `notepad`.
It needs a tiled neighbour on its workspace for `move` and `resize`.

.PARAMETER Against
Git ref of the build compared with this checkout. Not fetched: run
`git fetch` first for an up-to-date `origin/main`.

.PARAMETER Scenarios
`perf_bench` scenarios to run. `minimize` stands in for opening and
closing a window: `window_minimize` drives the same entry and exit
animations as `window_open` and `window_close`, without starting a
process per burst. `switch` uses another workspace with windows on the
target's monitor, or moves every second tile of the focused workspace to
the next one to make it.

.PARAMETER Load
IPC queries queued as each animation starts (`perf_bench --load`), so the
main loop has a backlog to catch up on between frames.

.PARAMETER CpuLoad
Busy-loop threads kept running while measuring, at normal priority, so the
WM competes for the CPU, as on a busy or weaker machine. Several values,
e.g. `0,8`, each measure every round and setting; the threads are started
once per value.

.PARAMETER Backdrops
How the config copies draw backdrops, each value measured: `config` (as
your config does), `on` or `off`, e.g. `on,off`.

.PARAMETER Borders
How the config copies draw borders, as `-Backdrops`. Every pair of the
two is measured.

.PARAMETER Dwm
Also reports what DWM displayed per burst (`perf_bench --dwm`), through
the `PresentMon` service, which must be installed.

.PARAMETER TileOrder
Process names, left to right, to put the focused workspace's tiles in after
every restart, once the layout is restored. A restart can otherwise
reorder them, changing the target's neighbours between builds. Of several
windows of one process, the lowest handle goes first; names with no window
are skipped, and tiles not named stay right of the rest. Use it with
`-Launch` to fix both which windows the target sits among and their order;
Windows Terminal's process name is `WindowsTerminal`.

.PARAMETER Launch
Programs started once, before the first round, so every run measures the
same windows; `none` starts nothing. They open on the focused workspace.
`notepad`, `explorer`, `wt` and `chrome` (with `--new-window`) each open a
new window; `explorer`'s is shown by the file manager registered for
folders, if not Explorer (e.g. `FPilot`). Any other name is started as
given, its window expected from the process named after the file. The
windows of those processes that the WM manages within 20 seconds, and
didn't before, are closed at the end through the WM's `close`; no other
window is.

.PARAMETER KeepLaunched
Leaves the `-Launch` windows open at the end.

.PARAMETER ResultsFile
File the results are written to, replacing it. Elsewhere than the default
when an editor holds ~/.glzr/glazewm's files open: Zed's handles make
`Set-Content` and `Add-Content` fail there.

.EXAMPLE
powershell -ExecutionPolicy Bypass -File packages\wm-cli\examples\perf_bench\compare_branches.ps1 -Target chrome

.EXAMPLE
powershell -ExecutionPolicy Bypass -File packages\wm-cli\examples\perf_bench\compare_branches.ps1 -Target chrome -CpuLoad 0,8 -Backdrops on,off -Borders on,off

.EXAMPLE
powershell -ExecutionPolicy Bypass -File packages\wm-cli\examples\perf_bench\compare_branches.ps1 -Target chrome -Scenarios move,resize -Dwm

.EXAMPLE
powershell -ExecutionPolicy Bypass -File packages\wm-cli\examples\perf_bench\compare_branches.ps1 -Target notepad -Against tryout/one-surface -Launch notepad,explorer,chrome -TileOrder chrome,notepad,explorer
#>
param(
  [Parameter(Mandatory)] [string] $Target,
  [string] $Against = 'origin/main',
  [string[]] $Scenarios = @('move', 'resize', 'switch', 'minimize'),
  [int] $Rounds = 3,
  [int] $Bursts = 10,
  [int] $Load = 0,
  # Strings: `powershell -File` binds `-CpuLoad 0,8` as the one string '0,8'.
  [string[]] $CpuLoad = @('0'),
  [string[]] $Backdrops = @('config'),
  [string[]] $Borders = @('config'),
  [switch] $Dwm,
  [string[]] $TileOrder = @(),
  [string[]] $Launch = @('notepad', 'explorer', 'wt', 'chrome'),
  [switch] $KeepLaunched,
  [string] $ResultsFile = (Join-Path $HOME '.glzr\glazewm\compare-bench.md')
)

$ErrorActionPreference = 'Stop'
# Relative to where it was started from, not the repository root.
$ResultsFile = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($ResultsFile)
$repo = (Resolve-Path (Join-Path $PSScriptRoot '..\..\..\..')).Path
Set-Location $repo

# `powershell -File` binds `-Scenarios a,b` as the one string 'a,b'.
function Split-List([string[]] $Values) {
  @($Values -split '\s*,\s*' | Where-Object { $_ })
}

$Scenarios = Split-List $Scenarios
$Backdrops = Split-List $Backdrops
$Borders = Split-List $Borders
$TileOrder = Split-List $TileOrder
$Launch = @(Split-List $Launch | Where-Object { $_ -ne 'none' })
$cpuLoads = @(foreach ($value in (Split-List $CpuLoad)) {
  $threads = 0
  if (-not [int]::TryParse($value, [ref] $threads) -or $threads -lt 0) {
    throw "-CpuLoad takes thread counts, not '$value'."
  }
  $threads
})
foreach ($setting in @($Backdrops) + @($Borders)) {
  if ($setting -notin 'config', 'on', 'off') {
    throw "-Backdrops and -Borders take config, on or off, not '$setting'."
  }
}
if (-not $Scenarios -or -not $cpuLoads -or -not $Backdrops -or -not $Borders) {
  throw '-Scenarios, -CpuLoad, -Backdrops and -Borders each need a value.'
}

# Runs `$Command`, a native command, and throws if it fails. Returns its
# output.
function Invoke-Checked([string] $What, [scriptblock] $Command) {
  & $Command
  if ($LASTEXITCODE) { throw "$What failed (exit code $LASTEXITCODE)." }
}

# Arguments of a process's command line, without its executable.
function Get-Arguments([string] $CommandLine) {
  $CommandLine -replace '^\s*("[^"]*"|\S+)\s*', ''
}

# Running WMs, of either build: started with no arguments or with `start`,
# unlike the short-lived CLI calls the same executable serves.
function Get-WmProcesses {
  @(Get-CimInstance Win32_Process -Filter "Name = 'glazewm.exe'" |
    Where-Object {
      $arguments = Get-Arguments $_.CommandLine
      $arguments -eq '' -or $arguments -match '^start\b'
    })
}

# Waits until a WM answers queries.
function Wait-WmReady([string] $Cli) {
  # Windows PowerShell turns a native command's redirected stderr into a
  # terminating error under 'Stop'.
  $ErrorActionPreference = 'Continue'
  for ($i = 0; $i -lt 60; $i++) {
    # The executable is a GUI app: only with its output piped does
    # PowerShell wait for it and set `$LASTEXITCODE`, not with `*> $null`.
    & $Cli query monitors 2>$null | Out-Null
    if ($LASTEXITCODE -eq 0) { return }
    Start-Sleep -Milliseconds 500
  }
  throw 'GlazeWM did not start answering within 30 seconds.'
}

# Asks every running WM to exit, each through its own executable, and
# waits for them to end.
function Stop-AllWm([string] $FallbackCli) {
  foreach ($process in Get-WmProcesses) {
    $cli = $(if ($process.ExecutablePath) { $process.ExecutablePath } else { $FallbackCli })
    $ErrorActionPreference = 'Continue'
    & $cli command wm-exit 2>$null | Out-Null
    $ErrorActionPreference = 'Stop'
    Wait-Process -Id $process.ProcessId -Timeout 20 -ErrorAction SilentlyContinue
    if (Get-Process -Id $process.ProcessId -ErrorAction SilentlyContinue) {
      throw "GlazeWM ($($process.ProcessId)) did not exit within 20 seconds."
    }
  }
}

# Starts `$Exe` with `$Arguments`, the profiler on if `$Perf`. The variable
# is only set for the started process.
function Start-Wm([string] $Exe, [string] $Arguments, [bool] $Perf) {
  $env:GLAZEWM_PERF = $(if ($Perf) { '1' } else { $null })
  try {
    Start-Process -FilePath $Exe -ArgumentList $Arguments
  } finally {
    Remove-Item Env:GLAZEWM_PERF -ErrorAction SilentlyContinue
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

# Process that shows the window `explorer.exe` opens: a file manager
# registered for folders' `open` verb (e.g. File Pilot) takes it over.
function Get-FolderOwner {
  $command = (Get-ItemProperty -ErrorAction SilentlyContinue `
    'Registry::HKEY_CLASSES_ROOT\Directory\shell\open\command').'(default)'
  if ($command -match '^\s*"?([^"]+?\.exe)\b') {
    return [IO.Path]::GetFileNameWithoutExtension($Matches[1])
  }
  'explorer'
}

# Starts each of `$Names` once (see `-Launch`) and waits up to 20 seconds
# for the WM to manage their windows. Returns the new windows' handles.
function Start-Launched([string] $Cli, [string[]] $Names) {
  # `Owner` is the window's process: launchers hand off to it and exit.
  $known = @{
    notepad  = @{ FilePath = 'notepad.exe'; Owner = 'notepad' }
    explorer = @{ FilePath = 'explorer.exe'; Owner = Get-FolderOwner }
    # Without `-w new`, a `windowingBehavior` setting can open a tab instead.
    wt       = @{ FilePath = 'wt.exe'; Arguments = '-w new'; Owner = 'WindowsTerminal' }
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
    & $Cli command --id $window.id close 2>$null | Out-Null
    if ($LASTEXITCODE) {
      Write-Warning "Closing the $($window.processName) window $handle failed."
    }
  }
}

# Puts every window back on its saved workspace, then the focused
# workspace's tiles in `$Order`, and lets the moves settle. A restart
# manages every window onto its monitor's displayed workspace.
function Reset-Layout([string] $Bench, [string] $LayoutFile, [string[]] $Order) {
  Invoke-Checked 'Restoring the window layout' { & $Bench restore-layout $LayoutFile }
  if ($Order) { Set-TileOrder $Order }
  Start-Sleep -Seconds 2
}

# Removes the other build's worktree, also one left by an interrupted run.
function Remove-AgainstWorktree([string] $Path) {
  $ErrorActionPreference = 'Continue'
  if (Test-Path $Path) {
    git worktree remove --force $Path
    if (Test-Path $Path) { Remove-Item -Recurse -Force $Path }
  }
  git worktree prune
}

# Builds the WM at `$Commit` in a worktree at `$Source`, into `$TargetDir`,
# and removes the worktree again.
function Build-Against([string] $Commit, [string] $Source, [string] $TargetDir) {
  Remove-AgainstWorktree $Source
  Invoke-Checked "Checking out $Commit" { git worktree add --detach $Source $Commit }
  $savedTarget = $env:CARGO_TARGET_DIR
  $env:CARGO_TARGET_DIR = $TargetDir
  Push-Location $Source
  try {
    Invoke-Checked 'Building the other WM' { cargo build --release -p wm -p wm-watcher }
  } finally {
    Pop-Location
    $env:CARGO_TARGET_DIR = $savedTarget
    Remove-AgainstWorktree $Source
  }
}

# One job per busy thread, each its own process.
function Start-CpuLoad([int] $Threads) {
  @(for ($i = 0; $i -lt $Threads; $i++) {
    Start-Job -ScriptBlock { while ($true) {} }
  })
}

function Stop-CpuLoad($Jobs) {
  if ($Jobs) {
    $Jobs | Stop-Job
    $Jobs | Remove-Job -Force
  }
}

$userWm = @(Get-WmProcesses) | Select-Object -First 1
if (-not $userWm) {
  throw 'Start GlazeWM as usual first: this restarts it the way it runs.'
}
$userExe = $userWm.ExecutablePath
$userArgs = Get-Arguments $userWm.CommandLine
if ($userArgs -eq '') { $userArgs = 'start' }

# The copies are made of the config the running WM reads.
$configArgs = @()
if ($userArgs -match '(?:-c|--config)[\s=]+(?:"([^"]+)"|(\S+))') {
  $configArgs = @($(if ($Matches[1]) { $Matches[1] } else { $Matches[2] }))
}

$thisCommit = Invoke-Checked 'Reading HEAD' { git rev-parse --short HEAD }
$thisDirty = [bool] (Invoke-Checked 'Reading the status' { git status --porcelain --untracked-files=no })
$againstCommit = Invoke-Checked "Resolving '$Against'" { git rev-parse --verify --quiet "$Against^{commit}" }

$targetDir = $(if ($env:CARGO_TARGET_DIR) {
  $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($env:CARGO_TARGET_DIR)
} else {
  Join-Path $repo 'target'
})
$againstSource = Join-Path $targetDir 'compare\src'
$againstTarget = Join-Path $targetDir 'compare\target'

Write-Host "Building this checkout ($thisCommit$(if ($thisDirty) { ', modified' }))..."
Invoke-Checked 'Building the WM' { cargo build --release -p wm -p wm-watcher }
Invoke-Checked 'Building perf_bench' { cargo build --release -p wm-cli --example perf_bench }
Write-Host "Building '$Against' ($($againstCommit.Substring(0, 7)))..."
Build-Against $againstCommit $againstSource $againstTarget

$wm = Join-Path $targetDir 'release\glazewm.exe'
$bench = Join-Path $targetDir 'release\examples\perf_bench.exe'
$builds = @(
  [pscustomobject]@{ Name = 'this'; Exe = $wm },
  [pscustomobject]@{ Name = 'against'; Exe = (Join-Path $againstTarget 'release\glazewm.exe') }
)

$known = @(Invoke-Checked 'Listing the scenarios' { & $bench scenarios })
foreach ($scenario in $Scenarios) {
  if ($scenario -notin $known) {
    throw "Unknown scenario '$scenario'. Options: $($known -join ', ')."
  }
}

$dwmArgs = @($(if ($Dwm) { '--dwm' }))
$dataDir = Join-Path $HOME '.glzr\glazewm'
$layoutFile = Join-Path $dataDir 'compare-bench-layout.txt'
$rowsFile = Join-Path (Split-Path $ResultsFile) (
  [IO.Path]::GetFileNameWithoutExtension($ResultsFile) + '-rows.md')
$effectSets = @(foreach ($backdrop in $Backdrops) {
  foreach ($border in $Borders) {
    [pscustomobject]@{
      Name = "bd-$backdrop bo-$border"
      Tag = "bd${backdrop}bo${border}"
      Backdrops = $backdrop
      Borders = $border
      Config = $null
    }
  }
})

$cpuJobs = @()
# Handles of the windows `-Launch` opened: container ids change on a restart.
$launched = @()
$layoutSaved = $false
$configsWritten = $false
$wmRestarted = $false
$completed = $false

try {
  if ($Launch) {
    Write-Host "Starting $($Launch -join ', ')..."
    $launched = @(Start-Launched $wm $Launch)
  }

  Invoke-Checked 'Saving the window layout' { & $bench save-layout $layoutFile }
  $layoutSaved = $true

  $configsWritten = $true
  foreach ($set in $effectSets) {
    $set.Config = Invoke-Checked 'Writing a config copy' {
      & $bench write-bench-config $set.Tag $set.Borders $set.Backdrops @configArgs
    }
  }

  Set-Content -Path $ResultsFile -Encoding UTF8 -Value (
    "Branch comparison, $(Get-Date -Format s): this = $thisCommit" +
    "$(if ($thisDirty) { ' (modified)' }), against = $Against " +
    "($($againstCommit.Substring(0, 7))). Target $Target, $Rounds round(s) " +
    "of $Bursts bursts per scenario, build and setting, IPC load $Load, " +
    "CPU load $($cpuLoads -join ',') thread(s) of " +
    "$([Environment]::ProcessorCount), $(if ($Dwm) { 'with' } else { 'without' }) " +
    "DWM timing, tile order " +
    "$(if ($TileOrder) { $TileOrder -join ',' } else { 'as restarted' }), " +
    "launched $(if ($Launch) { $Launch -join ',' } else { 'none' }) " +
    "($($launched.Count) window(s))."
  )
  Invoke-Checked 'Writing the summary header' { & $bench summary-header @dwmArgs } |
    Set-Content -Path $rowsFile -Encoding UTF8

  # Which build goes first, alternating so neither always runs on the
  # fresher system.
  $slot = 0
  foreach ($threads in $cpuLoads) {
    if ($threads -gt 0) {
      Write-Host "Starting $threads CPU load thread(s)..."
      $cpuJobs = Start-CpuLoad $threads
    }

    for ($round = 1; $round -le $Rounds; $round++) {
      foreach ($set in $effectSets) {
        $pair = $(if ($slot % 2) { @($builds[1], $builds[0]) } else { $builds })
        $slot++

        foreach ($build in $pair) {
          $setting = "cpu$threads $($set.Name)"
          $label = "$($build.Name) $setting #$round"
          Write-Host "Round $round/$Rounds, ${setting}: restarting GlazeWM as '$($build.Name)'..."
          $wmRestarted = $true
          Stop-AllWm $wm
          Start-Wm $build.Exe "start --config `"$($set.Config)`"" $true
          Wait-WmReady $build.Exe

          Reset-Layout $bench $layoutFile $TileOrder
          $tileLine = "tiles $label $(Get-TileOrder $build.Exe)"
          Write-Host "  $tileLine"
          $tileLine | Add-Content -Path $ResultsFile -Encoding UTF8

          foreach ($scenario in $Scenarios) {
            Write-Host "  Measuring '$scenario'..."
            $output = Invoke-Checked "Scenario '$scenario'" {
              & $bench --scenario $scenario --target $Target --bursts $Bursts `
                --load $Load --label $label @dwmArgs
            }
            $output | Add-Content -Path $ResultsFile -Encoding UTF8
            $output | Where-Object { $_ -like '| *' } |
              Add-Content -Path $rowsFile -Encoding UTF8

            # `switch` may have moved tiles to another workspace to switch to.
            if ($scenario -eq 'switch') { Reset-Layout $bench $layoutFile $TileOrder }
          }
        }
      }
    }

    Stop-CpuLoad $cpuJobs
    $cpuJobs = @()
  }
  $completed = $true
} finally {
  # Each step on its own, so one failing doesn't skip the rest, nor hide
  # the error that stopped the run.
  try {
    if ($cpuJobs) {
      Write-Host 'Stopping the CPU load...'
      Stop-CpuLoad $cpuJobs
    }
  } catch { Write-Warning "Stopping the CPU load failed: $($_.Exception.Message)" }

  try {
    if ($wmRestarted) {
      Write-Host 'Restarting GlazeWM as it was...'
      Stop-AllWm $wm
      Start-Wm $userExe $userArgs $false
      Wait-WmReady $userExe
      if ($layoutSaved) {
        Invoke-Checked 'Restoring the window layout' { & $bench restore-layout $layoutFile }
      }
    }
  } catch { Write-Warning "Restoring GlazeWM failed: $($_.Exception.Message)" }

  try {
    if ($launched.Count -gt 0 -and -not $KeepLaunched) {
      Write-Host "Closing the $($launched.Count) window(s) it started..."
      Close-Launched $wm $launched
    }
  } catch { Write-Warning "Closing the started windows failed: $($_.Exception.Message)" }

  try {
    if ($configsWritten) {
      Invoke-Checked 'Deleting the config copies' { & $bench clean-bench-config @configArgs }
    }
  } catch { Write-Warning $_.Exception.Message }
}

if ($completed) {
  $before = @(Get-Content -Path $ResultsFile -Encoding UTF8).Count
  Invoke-Checked 'Comparing the builds' {
    & $bench compare $rowsFile $ResultsFile 'this' 'against'
  }
  Get-Content -Path $ResultsFile -Encoding UTF8 |
    Select-Object -Skip $before | Write-Host
  Write-Host "Done. Results are in $ResultsFile, rows in $rowsFile"
}
