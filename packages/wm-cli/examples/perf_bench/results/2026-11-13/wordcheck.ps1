<#
Per-word manual check. Restarts the WM (target\release) with GLAZEWM_PERF=1
and GLAZEWM_ISOLATE=$Word, restores the saved layout and tile order, then:

- check:  keybound resize (+2%), move left, move right, resize -2%; then
          new errors.log lines, window states and one settled screenshot.
- visual: one step pair ($Scenario) on $Config (slow motions), with one
          screenshot $ShotDelayMs after the first or second step's answer.

Leaves the bench WM running; the caller restarts the user's WM at the end.
#>
param(
  [Parameter(Mandatory)] [string] $Word,
  [Parameter(Mandatory)] [string] $Out,
  [ValidateSet('check', 'visual')] [string] $Kind = 'check',
  [string] $Config = '',
  [ValidateSet('resize', 'move')] [string] $Scenario = 'resize',
  [ValidateSet('first', 'second')] [string] $ShotOn = 'first',
  [int] $ShotDelayMs = 0,
  [int] $SettleMs = 1000,
  [int] $OrderWaitMs = 2000,
  [string] $TileOrder = 'Notepad,WindowsTerminal,WindowsTerminal,chrome,chrome,FPilot'
)

$ErrorActionPreference = 'Stop'
$repo = 'C:\Users\googlers\glazewm'
$wm = "$repo\target\release\glazewm.exe"
$bench = "$repo\target\release\examples\perf_bench.exe"
$scratch = Split-Path $PSCommandPath
$layoutFile = Join-Path $scratch 'layout.txt'
$errorsLog = Join-Path $HOME '.glzr\glazewm\errors.log'
$tag = ($Word -replace '\+', '_') + "-$Kind"
New-Item -ItemType Directory -Force $Out | Out-Null

function Get-Arguments([string] $CommandLine) {
  $CommandLine -replace '^\s*("[^"]*"|\S+)\s*', ''
}

function Get-WmProcesses {
  @(Get-CimInstance Win32_Process -Filter "Name = 'glazewm.exe'" |
    Where-Object {
      $arguments = Get-Arguments $_.CommandLine
      $arguments -eq '' -or $arguments -match '^start\b'
    })
}

function Wait-WmReady {
  $ErrorActionPreference = 'Continue'
  for ($i = 0; $i -lt 60; $i++) {
    & $wm query monitors 2>$null | Out-Null
    if ($LASTEXITCODE -eq 0) { return }
    Start-Sleep -Milliseconds 500
  }
  throw 'GlazeWM did not start answering within 30 seconds.'
}

function Stop-AllWm {
  $ErrorActionPreference = 'Continue'
  foreach ($process in Get-WmProcesses) {
    & $process.ExecutablePath command wm-exit 2>$null | Out-Null
    Wait-Process -Id $process.ProcessId -Timeout 20 -ErrorAction SilentlyContinue
    if (Get-Process -Id $process.ProcessId -ErrorAction SilentlyContinue) {
      throw "GlazeWM $($process.ProcessId) did not exit."
    }
  }
}

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

function Set-TileOrder($Socket, [string[]] $Order) {
  $placed = 0
  foreach ($name in $Order) {
    $tiles = @(Get-Tiles $Socket)
    $match = @(for ($k = $placed; $k -lt $tiles.Count; $k++) {
      if ($tiles[$k].processName -eq $name) {
        [pscustomobject]@{ Index = $k; Id = $tiles[$k].id; Handle = $tiles[$k].handle }
      }
    }) | Sort-Object Handle | Select-Object -First 1
    if (-not $match) { continue }
    for ($k = $match.Index; $k -gt $placed; $k--) {
      Invoke-WmMessage $Socket "command --id $($match.Id) move --direction left" | Out-Null
    }
    $placed++
  }
}

function Save-Screen([string] $Path) {
  Add-Type -AssemblyName System.Windows.Forms, System.Drawing
  $bounds = [Windows.Forms.Screen]::PrimaryScreen.Bounds
  $bitmap = New-Object Drawing.Bitmap $bounds.Width, $bounds.Height
  $graphics = [Drawing.Graphics]::FromImage($bitmap)
  $graphics.CopyFromScreen($bounds.Location, [Drawing.Point]::Empty, $bounds.Size,
    [Drawing.CopyPixelOperation]::SourceCopy)
  $bitmap.Save($Path, [Drawing.Imaging.ImageFormat]::Png)
  $graphics.Dispose()
  $bitmap.Dispose()
}

Add-Type -AssemblyName System.Windows.Forms, System.Drawing
$log = New-Object System.Collections.Generic.List[string]
function Note([string] $Line) { Write-Host $Line; $log.Add($Line) }

Stop-AllWm
$env:GLAZEWM_PERF = '1'
$env:GLAZEWM_ISOLATE = $(if ($Word -eq 'default') { $null } else { $Word -replace '\+', ',' })
$arguments = 'start'
if ($Config) { $arguments += " --config `"$Config`"" }
try {
  Start-Process -FilePath $wm -ArgumentList $arguments
} finally {
  Remove-Item Env:GLAZEWM_PERF, Env:GLAZEWM_ISOLATE -ErrorAction SilentlyContinue
}
Wait-WmReady
if (@(Get-WmProcesses).Count -ne 1) { throw 'Not exactly one GlazeWM is running.' }
& $bench restore-layout $layoutFile | Out-Null
if ($LASTEXITCODE) { throw 'Restoring the layout failed.' }

$none = [Threading.CancellationToken]::None
$socket = New-Object System.Net.WebSockets.ClientWebSocket
$socket.ConnectAsync([Uri] 'ws://127.0.0.1:6123', $none).Wait()
Set-TileOrder $socket ($TileOrder -split ',')
Start-Sleep -Milliseconds $OrderWaitMs
$tiles = @(Get-Tiles $socket)
Note "$tag tiles: $(($tiles | ForEach-Object { $_.processName }) -join ', ')"
$target = $tiles | Where-Object { $_.processName -eq 'chrome' } | Select-Object -First 1
$errStart = (Get-Item $errorsLog).Length

$clock = [Diagnostics.Stopwatch]::StartNew()
function Step([string] $Command) {
  $sent = $clock.Elapsed.TotalMilliseconds
  Invoke-WmMessage $socket "command --id $($target.id) $Command" | Out-Null
  $answered = $clock.Elapsed.TotalMilliseconds
  Note ("{0,8:N0} ms  {1} (answered in {2:N1} ms)" -f $sent, $Command, ($answered - $sent))
}

if ($Kind -eq 'check') {
  foreach ($command in 'resize --width +2%', 'move --direction left',
      'move --direction right', 'resize --width -2%') {
    Step $command
    Start-Sleep -Milliseconds $SettleMs
  }
  Start-Sleep -Milliseconds 500
  Save-Screen (Join-Path $Out "$tag-settled.png")
  Note ("{0,8:N0} ms  settled screenshot" -f $clock.Elapsed.TotalMilliseconds)
} else {
  $pair = if ($Scenario -eq 'resize') {
    'resize --width +25%', 'resize --width -25%'
  } else {
    'move --direction left', 'move --direction right'
  }
  Step $pair[0]
  if ($ShotOn -eq 'first') {
    Start-Sleep -Milliseconds $ShotDelayMs
    Save-Screen (Join-Path $Out "$tag-shot.png")
    Note ("{0,8:N0} ms  screenshot" -f $clock.Elapsed.TotalMilliseconds)
  }
  Start-Sleep -Milliseconds $SettleMs
  Step $pair[1]
  if ($ShotOn -eq 'second') {
    Start-Sleep -Milliseconds $ShotDelayMs
    Save-Screen (Join-Path $Out "$tag-shot.png")
    Note ("{0,8:N0} ms  screenshot" -f $clock.Elapsed.TotalMilliseconds)
  }
  Start-Sleep -Milliseconds $SettleMs
}

# Where every window on the workspace ended up.
foreach ($tile in @(Get-Tiles $socket)) {
  Note ("  {0,-16} {1,-8} x={2,5} w={3,5} y={4,4} h={5,4} {6}" -f $tile.processName,
    $tile.displayState, $tile.x, $tile.width, $tile.y, $tile.height, $tile.handle)
}
$socket.CloseAsync([Net.WebSockets.WebSocketCloseStatus]::NormalClosure, '', $none).Wait()

$stream = [IO.File]::Open($errorsLog, 'Open', 'Read', 'ReadWrite')
$stream.Seek($errStart, 'Begin') | Out-Null
$new = (New-Object IO.StreamReader($stream)).ReadToEnd()
$stream.Close()
$lines = @($new -split "`r?`n" | Where-Object { $_ })
$lines | Set-Content (Join-Path $Out "$tag-errors.txt")
Note "errors.log: $($lines.Count) new line(s)"
$lines | ForEach-Object {
  $_ -replace '\x1b\[[0-9;]*m', '' -replace '^\S+\s+', ''
} | ForEach-Object { $_ -replace '0x[0-9a-f]+|\d+', '#' } | Group-Object |
  Sort-Object Count -Descending |
  ForEach-Object { Note ("  {0}x {1}" -f $_.Count, $_.Name.Substring(0, [Math]::Min(200, $_.Name.Length))) }
$log | Set-Content (Join-Path $Out "$tag-log.txt")
