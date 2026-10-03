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
}
'@

$Failures = New-Object System.Collections.Generic.List[string]
$Step = 0

function Check([bool]$Condition, [string]$Message) {
  if ($Condition) {
    Write-Host "PASS  $Message"
  } else {
    Write-Host "FAIL  $Message" -ForegroundColor Red
    $Failures.Add($Message)
  }
}

function Invoke-Cli([string[]]$Arguments) {
  $raw = & $Cli @Arguments 2>$null | Out-String
  if (-not $raw.Trim()) { return $null }
  return $raw | ConvertFrom-Json
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

  # Closing a tab keeps the layout.
  $before = Get-TicketStack
  Send-WmCommand $before.children[0].id 'close'
  $after = Get-TicketStack
  Check (@($after.children).Count -eq 2) 'closing a tab removes only that tab'
  Check ($after.width -eq $before.width -and $after.x -eq $before.x) 'closing a tab keeps the stack size'
  Save-State 'after-close'

  Check (-not $wmProcess.HasExited) 'GlazeWM is still running'
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
