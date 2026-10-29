# Workspace-switch GPU cost of the scene's parts, on a GPU slowed to about
# an integrated one. Run from an elevated PowerShell (PresentMon and
# nvidia-smi clock locks need it):
#
#   .\scene-bench.ps1 -Scene <glazewm.exe> -PerfBench <perf_bench.exe> `
#     [-Layouts <surrogate build glazewm.exe>] [-Out <dir>]
#
# One WM runs per config; the parts left out change between cells through
# the `GLAZEWM_SCENE_DIAG` flags file, so no WM is restarted per cell.
# Clocks are always reset on exit.
param(
  [Parameter(Mandatory)] [string] $Scene,
  [Parameter(Mandatory)] [string] $PerfBench,
  [string] $Layouts,
  [string] $Out = (Join-Path $env:USERPROFILE 'gw-bench\results_lite'),
  [int] $GraphicsMhz = 450,
  [int] $MemoryMhz = 810,
  [int] $Rounds = 2
)
$ErrorActionPreference = 'Stop'

$configs = Join-Path $PSScriptRoot 'configs'
$smi = Join-Path $env:windir 'System32\nvidia-smi.exe'
$flagsFile = Join-Path $Out 'scene-diag.txt'

$isAdmin = [Security.Principal.WindowsPrincipal]::new(
  [Security.Principal.WindowsIdentity]::GetCurrent()
).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) { throw 'Run from an elevated PowerShell.' }

# Cells per config: a label and the parts it leaves out.
$matrix = [ordered]@{
  base = [ordered]@{
    full = ''
    'no-stills' = 'no-stills'
    'no-frames' = 'no-frames'
    'no-warm-up-lift' = 'no-warm-up no-lift'
    'movers' = 'no-stills no-frames no-warm-up no-lift'
    'empty' = 'no-stills no-items no-warm-up no-lift'
  }
  laptop = [ordered]@{
    full = ''
    'no-backdrops' = 'no-backdrops'
    'movers' = 'no-stills no-frames no-backdrops no-warm-up no-lift'
  }
  rounded = [ordered]@{
    full = ''
    'no-round-clips' = 'no-round-clips'
  }
}

New-Item -ItemType Directory -Force $Out | Out-Null
$log = Join-Path $Out 'progress.log'
function Write-Log($text) {
  $line = "$(Get-Date -Format HH:mm:ss) $text"
  $line | Out-File -Append $log
  Write-Output $line
}

function Stop-Wm($exe) {
  if (-not $exe) { return }
  & $exe command wm-exit *> $null
  $name = [IO.Path]::GetFileNameWithoutExtension($exe)
  for ($i = 0; $i -lt 20 -and (Get-Process $name -ErrorAction SilentlyContinue); $i++) {
    Start-Sleep -Milliseconds 250
  }
  # Only if it didn't exit: a 2nd WM would fight it over the windows.
  Get-Process $name -ErrorAction SilentlyContinue | Stop-Process -Force
}

function Start-Wm($exe, $config) {
  $env:GLAZEWM_PERF = '1'
  $env:GLAZEWM_SCENE_DIAG = $flagsFile
  Start-Process $exe -ArgumentList 'start', '--config', (Join-Path $configs "$config.yaml")
  Start-Sleep -Seconds 5
}

# Puts 3 Explorer windows on each of workspaces 1 and 2.
function Set-Layout($exe) {
  $ids = @()
  for ($i = 0; $i -lt 25 -and $ids.Count -lt 6; $i++) {
    $windows = (& $exe query windows | ConvertFrom-Json).data.windows
    $ids = @($windows | Where-Object {
      $_.processName -eq 'explorer' -and $_.state.type -eq 'tiling'
    } | Select-Object -ExpandProperty id)
    Start-Sleep -Milliseconds 400
  }
  for ($i = 0; $i -lt $ids.Count; $i++) {
    $workspace = if ($i -lt 3) { '1' } else { '2' }
    & $exe command --id $ids[$i] move --workspace $workspace *> $null
  }
  Start-Sleep -Milliseconds 600
  & $exe command focus --workspace 1 *> $null
  Start-Sleep -Milliseconds 600
  return $ids.Count
}

function Invoke-Cell($dir, $label) {
  $p = Start-Process $PerfBench -PassThru -NoNewWindow `
    -RedirectStandardOutput (Join-Path $dir "$label.txt") `
    -RedirectStandardError (Join-Path $dir "$label.err") `
    -ArgumentList '--scenario', 'switch', '--between', '1,2', '--bursts', '10',
      '--settle-ms', '2000', '--dwm', '--frames', '--label', $label
  if (-not $p.WaitForExit(90000)) {
    $p.Kill()
    'TIMEOUT' | Out-File (Join-Path $dir "$label.timeout")
  }
}

function Invoke-Config($exe, $build, $config, $cells) {
  $dir = Join-Path $Out "$build\$config"
  New-Item -ItemType Directory -Force $dir | Out-Null
  '' | Out-File -Encoding ascii $flagsFile
  Start-Wm $exe $config
  try {
    $managed = Set-Layout $exe
    Write-Log "$build/$config managed=$managed"
    for ($round = 1; $round -le $Rounds; $round++) {
      foreach ($cell in $cells.GetEnumerator()) {
        $cell.Value | Out-File -Encoding ascii $flagsFile
        Start-Sleep -Milliseconds 500
        Invoke-Cell $dir "$($cell.Key)_$round"
        Write-Log "$build/$config/$($cell.Key) round $round"
      }
    }
  }
  finally {
    Stop-Wm $exe
  }
}

$shell = New-Object -ComObject Shell.Application
if (@($shell.Windows()).Count -lt 6) {
  foreach ($folder in 'C:\', 'C:\Windows', 'C:\Program Files', 'C:\Users',
    $env:USERPROFILE, $Out) {
    Start-Process explorer.exe -ArgumentList $folder
    Start-Sleep -Milliseconds 700
  }
  Start-Sleep -Seconds 2
}

try {
  & $smi "--lock-gpu-clocks=$GraphicsMhz,$GraphicsMhz" *>> $log
  & $smi "--lock-memory-clocks=$MemoryMhz,$MemoryMhz" *>> $log
  & $smi --query-gpu=clocks.gr,clocks.mem --format=csv *>> $log
  Start-Sleep -Seconds 2

  foreach ($config in $matrix.GetEnumerator()) {
    Invoke-Config $Scene 'scene' $config.Key $config.Value
  }
  if ($Layouts) {
    foreach ($config in $matrix.Keys) {
      Invoke-Config $Layouts 'layouts' $config ([ordered]@{ full = '' })
    }
  }
}
finally {
  & $smi --reset-gpu-clocks *>> $log
  & $smi --reset-memory-clocks *>> $log
  Write-Log 'done'
}
