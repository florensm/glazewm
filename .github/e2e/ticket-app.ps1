# Test app for the stack end-to-end test: three "Ticket details for N"
# windows in one process, like a WPF app with several ticket windows. Once
# `$Trigger` exists, ticket 1 opens a modal "Add Activity" dialog, which
# disables the other windows of the thread, as WPF's `ShowDialog` does.
param([Parameter(Mandatory)][string]$Trigger)

$Log = "$Trigger.log"
function Write-Log([string]$Message) {
  Add-Content $Log "$(Get-Date -Format o) $Message"
}

Add-Type -AssemblyName System.Windows.Forms
Write-Log "started, trigger: $Trigger"

$forms = foreach ($n in 1..3) {
  $form = New-Object Windows.Forms.Form
  $form.Text = "Ticket details for $n"
  $form.Width = 700
  $form.Height = 450

  $box = New-Object Windows.Forms.TextBox
  $box.Multiline = $true
  $box.Dock = 'Fill'
  $box.Text = "Ticket $n"
  $form.Controls.Add($box)

  $form.Show()
  Start-Sleep -Milliseconds 300
  $form
}

$timer = New-Object Windows.Forms.Timer
$timer.Interval = 500
$timer.Add_Tick({
  try {
    if (Test-Path -LiteralPath $script:Trigger) {
      Remove-Item -LiteralPath $script:Trigger
      Write-Log 'opening dialog'
      $dialog = New-Object Windows.Forms.Form
      $dialog.Text = 'Add Activity'
      $dialog.Width = 320
      $dialog.Height = 200
      $dialog.StartPosition = 'CenterParent'
      [void]$dialog.ShowDialog($script:forms[0])
      Write-Log 'dialog closed'
    }
  } catch {
    Write-Log "tick failed: $_"
  }
})
$timer.Start()
Write-Log "showing $(@($forms).Count) tickets"

# Runs until the process is stopped, whichever windows get closed.
[Windows.Forms.Application]::Run((New-Object Windows.Forms.ApplicationContext))
