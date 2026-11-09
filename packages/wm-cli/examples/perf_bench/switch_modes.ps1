<#
.SYNOPSIS
Measures every workspace-switch style on this PC, with nothing to set up.

.DESCRIPTION
Builds the WM and `perf_bench`, restarts GlazeWM with the frame profiler on
and a copy of your config, and runs the switch benchmark once per style.
Then it restarts GlazeWM the way it was running and puts every window back
on its workspace. Your config files are only read; the copies are deleted
at the end.

Results are printed and saved to ~/.glzr/glazewm/switch-bench.txt.

Desktop PC only: it restarts the WM and drives bursts of workspace
switches over IPC (see CLAUDE.md's build targets).

.EXAMPLE
powershell -ExecutionPolicy Bypass -File packages\wm-cli\examples\perf_bench\switch_modes.ps1
#>
param(
  [string[]] $Styles = @('off', 'cut', 'drift', 'soft_zoom', 'slide'),
  [int] $Bursts = 10
)

$ErrorActionPreference = 'Stop'
Set-Location (Resolve-Path (Join-Path $PSScriptRoot '..\..\..\..'))

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

Write-Host 'Building...'
cargo build --release -p wm
if ($LASTEXITCODE) { throw 'Building the WM failed.' }
cargo build --release -p wm-cli --example perf_bench
if ($LASTEXITCODE) { throw 'Building perf_bench failed.' }

$wm = (Resolve-Path 'target\release\glazewm.exe').Path
$bench = (Resolve-Path 'target\release\examples\perf_bench.exe').Path
$dataDir = Join-Path $HOME '.glzr\glazewm'
$layoutFile = Join-Path $dataDir 'switch-bench-layout.txt'
$resultsFile = Join-Path $dataDir 'switch-bench.txt'

$userWm = Get-WmProcess
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

& $bench save-layout $layoutFile
if ($LASTEXITCODE) { throw 'Saving the window layout failed.' }
$benchConfig = & $bench write-switch-config $Styles[0] @configArgs
if ($LASTEXITCODE) { throw 'Writing the config copy failed.' }

Set-Content -Path $resultsFile -Value (
  "Switch benchmark, $(Get-Date -Format s), $Bursts bursts per style"
)

Write-Host 'Restarting GlazeWM with the profiler on...'
Stop-Wm $wm $userWm
$env:GLAZEWM_PERF = '1'
Start-Process -FilePath $wm -ArgumentList "start --config `"$benchConfig`""
Remove-Item Env:GLAZEWM_PERF

try {
  Wait-WmReady $wm

  foreach ($style in $Styles) {
    Write-Host "Measuring '$style'..."
    & $bench write-switch-config $style @configArgs | Out-Null
    & $wm command wm-reload-config | Out-Null
    Start-Sleep -Seconds 1
    & $bench --scenario switch --bursts $Bursts --label $style |
      Tee-Object -FilePath $resultsFile -Append
  }
} finally {
  Write-Host 'Restarting GlazeWM as it was...'
  $benchWm = Get-WmProcess
  if ($benchWm) { Stop-Wm $wm $benchWm }
  Start-Process -FilePath $userExe -ArgumentList $userArgs
  Wait-WmReady $wm
  & $bench restore-layout $layoutFile
  & $bench clean-switch-config @configArgs
}

Write-Host "Done. Results are in $resultsFile"
