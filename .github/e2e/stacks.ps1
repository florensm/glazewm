# End-to-end test of stacks on real Windows: starts GlazeWM, opens the
# ticket test app and drives the stack through the CLI, checking the
# window tree after each step. Writes screenshots, tree dumps and the WM
# log to `e2e-out`, and exits non-zero if a check failed.
param([string]$Bin = 'target/debug')

$ErrorActionPreference = 'Stop'

$Root = Split-Path -Parent $MyInvocation.MyCommand.Path
$Out = Join-Path (Get-Location) 'e2e-out'
New-Item -ItemType Directory -Force $Out | Out-Null

$Wm = Resolve-Path (Join-Path $Bin 'glazewm.exe')
$Cli = Resolve-Path (Join-Path $Bin 'glazewm-cli.exe')
$Trigger = Join-Path $Out 'open-dialog.trigger'

Add-Type -AssemblyName System.Windows.Forms, System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class Native {
  [DllImport("user32.dll")] public static extern bool IsWindowEnabled(IntPtr hwnd);
  [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr hwnd);
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint flags, int dx, int dy, uint data, UIntPtr extra);
}
'@

$Failures = New-Object System.Collections.Generic.List[string]
$Step = 0
$CliTimeouts = 0

function Check([bool]$Condition, [string]$Message) {
  if ($Condition) {
    Write-Host "PASS  $Message"
  } else {
    Write-Host "FAIL  $Message" -ForegroundColor Red
    $Failures.Add($Message)
  }
}

# Runs the CLI with a timeout, so a WM that stops answering fails the
# check instead of hanging the job. Returns its parsed JSON output.
function Invoke-Cli([string[]]$Arguments, [int]$TimeoutSeconds = 15) {
  $info = New-Object Diagnostics.ProcessStartInfo
  $info.FileName = $Cli
  $info.Arguments = ($Arguments | ForEach-Object { '"' + $_ + '"' }) -join ' '
  $info.RedirectStandardOutput = $true
  $info.RedirectStandardError = $true
  $info.UseShellExecute = $false
  $info.CreateNoWindow = $true

  $process = [Diagnostics.Process]::Start($info)
  $output = $process.StandardOutput.ReadToEndAsync()

  if (-not $process.WaitForExit($TimeoutSeconds * 1000)) {
    $process.Kill()
    Write-Host "CLI timed out: $($Arguments -join ' ')" -ForegroundColor Yellow
    $script:CliTimeouts++
    return $null
  }

  $raw = $output.Result
  if (-not $raw -or -not $raw.Trim()) { return $null }

  try { return $raw | ConvertFrom-Json } catch { return $null }
}

function Send-WmCommand([string]$Id, [string]$Command) {
  $words = @('command', '--id', $Id) + ($Command -split ' ')
  $result = Invoke-Cli $words
  Check ($null -ne $result -and $result.success) "command '$Command' succeeds"
  Start-Sleep -Milliseconds 1500
}

function Get-Workspaces {
  $result = Invoke-Cli @('query', 'workspaces')
  if ($null -eq $result) { return @() }
  return $result.data.workspaces
}

function Get-Descendants($Container) {
  foreach ($child in $Container.children) {
    $child
    Get-Descendants $child
  }
}

function Get-All { Get-Workspaces | ForEach-Object { $_; Get-Descendants $_ } }

function Get-Tickets {
  Get-All | Where-Object { $_.type -eq 'window' -and $_.title -like 'Ticket details for*' }
}

function Get-TicketStack {
  $tickets = @(Get-Tickets)
  $parentIds = @($tickets | ForEach-Object parentId | Sort-Object -Unique)
  Get-All | Where-Object { $_.type -eq 'stack' -and $parentIds -contains $_.id } |
    Select-Object -First 1
}

function Save-State([string]$Name) {
  $script:Step++
  $prefix = '{0:d2}-{1}' -f $script:Step, $Name
  (Invoke-Cli @('query', 'workspaces')) | ConvertTo-Json -Depth 50 |
    Set-Content (Join-Path $Out "$prefix.json")

  # A screenshot is a nice-to-have; a runner without a desktop has none.
  try {
    $bounds = [Windows.Forms.SystemInformation]::VirtualScreen
    $bitmap = New-Object Drawing.Bitmap $bounds.Width, $bounds.Height
    $graphics = [Drawing.Graphics]::FromImage($bitmap)
    $graphics.CopyFromScreen($bounds.Location, [Drawing.Point]::Empty, $bounds.Size)
    $bitmap.Save((Join-Path $Out "$prefix.png"), [Drawing.Imaging.ImageFormat]::Png)
    $graphics.Dispose()
    $bitmap.Dispose()
  } catch {
    Write-Host "Screenshot '$prefix' failed: $_"
  }
}

function Wait-Until([scriptblock]$Condition, [int]$Seconds = 15) {
  $deadline = (Get-Date).AddSeconds($Seconds)
  while ((Get-Date) -lt $deadline) {
    if (& $Condition) { return $true }
    Start-Sleep -Milliseconds 500
  }
  return $false
}

# Mouse input on the tab bar, which sits in the top `$TabBarHeight` pixels
# of a tiling stack and splits its width evenly between the tabs.
$TabBarHeight = 30
$MouseLeftDown = 0x2; $MouseLeftUp = 0x4; $MouseMiddleDown = 0x20; $MouseMiddleUp = 0x40

function Get-TabPoint($Stack, [int]$Index) {
  $count = @($Stack.children).Count
  $x = [int]($Stack.x + $Stack.width * (2 * $Index + 1) / (2 * $count))
  $y = [int]($Stack.y + $TabBarHeight / 2)
  return @($x, $y)
}

function Send-Mouse([uint32]$Flags) {
  [Native]::mouse_event($Flags, 0, 0, 0, [UIntPtr]::Zero)
  Start-Sleep -Milliseconds 80
}

function Click-Tab($Stack, [int]$Index, [switch]$Middle) {
  $x, $y = Get-TabPoint $Stack $Index
  [void][Native]::SetCursorPos($x, $y)
  Start-Sleep -Milliseconds 200
  if ($Middle) {
    Send-Mouse $MouseMiddleDown; Send-Mouse $MouseMiddleUp
  } else {
    Send-Mouse $MouseLeftDown; Send-Mouse $MouseLeftUp
  }
  Start-Sleep -Milliseconds 1500
}

# Drags tab `$Index` straight down by `$Distance` pixels, in a few steps.
function Drag-Tab($Stack, [int]$Index, [int]$Distance) {
  $x, $y = Get-TabPoint $Stack $Index
  [void][Native]::SetCursorPos($x, $y)
  Start-Sleep -Milliseconds 200
  Send-Mouse $MouseLeftDown
  foreach ($step in 1..8) {
    [void][Native]::SetCursorPos($x, $y + [int]($Distance * $step / 8))
    Start-Sleep -Milliseconds 60
  }
  Send-Mouse $MouseLeftUp
  Start-Sleep -Milliseconds 1500
}

function Get-ShownTab($Stack) {
  @($Stack.children | Where-Object { $_.displayState -in 'shown', 'showing' })[0]
}

function States($Stack) { @($Stack.children | ForEach-Object { $_.state.type } | Sort-Object -Unique) }

$wmProcess = Start-Process $Wm -ArgumentList 'start', '--config', (Join-Path $Root 'config.yaml') `
  -RedirectStandardOutput (Join-Path $Out 'wm.log') -RedirectStandardError (Join-Path $Out 'wm.err.log') `
  -PassThru -WindowStyle Hidden
$appProcess = $null

try {
  $ready = Wait-Until { $null -ne (Invoke-Cli @('query', 'monitors')) } 60
  Check $ready 'GlazeWM starts and answers IPC'
  if (-not $ready) { throw 'GlazeWM did not start.' }

  # Opened after the WM, so they are auto-stacked as they open.
  $appProcess = Start-Process powershell.exe -PassThru -ArgumentList `
    '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', (Join-Path $Root 'ticket-app.ps1'), '-Trigger', $Trigger

  $stacked = Wait-Until { $s = Get-TicketStack; $s -and @($s.children).Count -eq 3 } 30
  Check $stacked 'auto-stack: 3 ticket windows open into one stack'
  Save-State 'auto-stacked'
  if (-not $stacked) { throw 'Tickets were not stacked.' }

  $stack = Get-TicketStack
  $shown = @($stack.children | Where-Object { $_.displayState -in 'shown', 'showing' })
  Check ($shown.Count -eq 1) 'only the active tab is shown'
  $tab = $stack.children[0].id
  $wsWidth = (Get-Workspaces | Select-Object -First 1).width

  # Cycling shows another tab.
  $activeBefore = $shown[0].id
  Send-WmCommand $tab 'cycle-stack-focus'
  $activeAfter = @((Get-TicketStack).children | Where-Object { $_.displayState -in 'shown', 'showing' })
  Check ($activeAfter.Count -eq 1 -and $activeAfter[0].id -ne $activeBefore) 'cycle-stack-focus shows the next tab'

  # Clicking a tab shows it.
  $stack = Get-TicketStack
  $target = @($stack.children | Where-Object id -ne (Get-ShownTab $stack).id)[0]
  $targetIndex = [array]::IndexOf(@($stack.children | ForEach-Object id), $target.id)
  Click-Tab $stack $targetIndex
  Check ((Get-ShownTab (Get-TicketStack)).id -eq $target.id) 'clicking a tab shows it'
  Save-State 'tab-clicked'

  # Dragging a tab off the bar floats its window out of the stack.
  $stack = Get-TicketStack
  $dragged = $stack.children[2]
  Drag-Tab $stack 2 250
  $draggedNow = Get-Tickets | Where-Object id -eq $dragged.id
  Check ($draggedNow.state.type -eq 'floating' -and @((Get-TicketStack).children).Count -eq 2) 'dragging a tab off the bar floats it out'
  Save-State 'tab-dragged-off'
  Send-WmCommand $dragged.id 'toggle-floating'
  Check (@((Get-TicketStack).children).Count -eq 3) 'the dragged-off window tiles back into the stack'

  # Floating floats the whole stack, with its tab bar on screen.
  Send-WmCommand $tab 'toggle-floating --centered'
  $stack = Get-TicketStack
  Check (((States $stack) -join ',') -eq 'floating') 'toggle-floating floats every tab'
  $workspace = Get-Workspaces | Select-Object -First 1
  Check ($stack.y -ge $workspace.y) 'floating stack keeps its tab bar on screen'
  Save-State 'floating'

  Send-WmCommand $tab 'toggle-floating'
  $stack = Get-TicketStack
  Check (((States $stack) -join ',') -eq 'tiling') 'toggle-floating again tiles every tab'

  # Fullscreen applies to the whole stack.
  Send-WmCommand $tab 'toggle-fullscreen'
  Check (((States (Get-TicketStack)) -join ',') -eq 'fullscreen') 'toggle-fullscreen makes every tab fullscreen'
  Save-State 'fullscreen'
  Send-WmCommand $tab 'toggle-fullscreen'
  Check (((States (Get-TicketStack)) -join ',') -eq 'tiling') 'toggle-fullscreen again tiles every tab'

  # Minimizing minimizes the stack; only the active window natively.
  $active = @((Get-TicketStack).children | Where-Object { $_.displayState -in 'shown', 'showing' })[0]
  Send-WmCommand $active.id 'toggle-minimized'
  $minimized = Wait-Until { ((States (Get-TicketStack)) -join ',') -eq 'minimized' } 10
  Check $minimized 'minimizing a tab minimizes the whole stack'
  $hiddenIconic = @((Get-TicketStack).children | Where-Object { $_.id -ne $active.id } |
      Where-Object { [Native]::IsIconic([IntPtr]$_.handle) })
  Check ($hiddenIconic.Count -eq 0) 'hidden tabs are not natively minimized'
  Send-WmCommand $active.id 'toggle-minimized'
  $restored = Wait-Until { ((States (Get-TicketStack)) -join ',') -eq 'tiling' } 10
  Check $restored 'restoring brings the whole stack back'

  # A single tab floats out and goes back in.
  $outId = (Get-TicketStack).children[1].id
  Send-WmCommand $outId 'float-out-of-stack'
  $out = Get-Tickets | Where-Object id -eq $outId
  Check ($out.state.type -eq 'floating' -and @((Get-TicketStack).children).Count -eq 2) 'float-out-of-stack takes one tab out, floating'
  Send-WmCommand $outId 'toggle-floating'
  Check (@((Get-TicketStack).children).Count -eq 3) 'toggle-floating puts the tab back into its stack'

  # Stacked windows stay usable while the app shows a blocking dialog.
  New-Item -ItemType File $Trigger | Out-Null
  $dialogOpen = Wait-Until { Get-All | Where-Object { $_.type -eq 'window' -and $_.title -eq 'Add Activity' } } 15
  Check $dialogOpen 'the app opens its blocking dialog'
  Start-Sleep -Seconds 2
  $dialog = Get-All | Where-Object { $_.type -eq 'window' -and $_.title -eq 'Add Activity' }
  if ($dialog) {
    Check ($dialog.state.type -eq 'floating') 'the dialog opens floating, outside the stack'
    $disabled = @((Get-TicketStack).children | Where-Object { -not [Native]::IsWindowEnabled([IntPtr]$_.handle) })
    Check ($disabled.Count -eq 0) 'stay-interactive keeps stacked tickets enabled'
    Save-State 'dialog'
    Send-WmCommand $dialog.id 'close'
  }

  # Middle-clicking a tab closes its window and keeps the layout.
  $before = Get-TicketStack
  Click-Tab $before 0 -Middle
  $after = Get-TicketStack
  Check (@($after.children).Count -eq 2) 'middle-clicking a tab closes only that tab'
  Check ($after.width -eq $before.width -and $after.x -eq $before.x) 'closing a tab keeps the stack size'
  Save-State 'after-close'

  Check (-not $wmProcess.HasExited) 'GlazeWM is still running'
  Check ($CliTimeouts -eq 0) "GlazeWM answered every CLI call ($CliTimeouts timed out)"
}
catch {
  Check $false "test aborted: $_"
}
finally {
  Invoke-Cli @('command', 'wm-exit') | Out-Null
  Start-Sleep -Seconds 2
  if (-not $wmProcess.HasExited) { Stop-Process -Id $wmProcess.Id -Force }
  if ($appProcess -and -not $appProcess.HasExited) { Stop-Process -Id $appProcess.Id -Force }

  $log = Get-Content (Join-Path $Out 'wm.log'), (Join-Path $Out 'wm.err.log') -ErrorAction SilentlyContinue
  Check (-not ($log -match 'panicked')) 'GlazeWM log has no panic'
}

Write-Host ''
if ($Failures.Count) {
  Write-Host "$($Failures.Count) check(s) failed:" -ForegroundColor Red
  $Failures | ForEach-Object { Write-Host "  - $_" }
  exit 1
}
Write-Host 'All stack checks passed.' -ForegroundColor Green
