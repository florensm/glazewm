# WPF test window for GlazeWM color themes. Runs on the WPF that ships with
# Windows (Windows PowerShell 5.1 or PowerShell 7), no SDK needed.
#
#   powershell -ExecutionPolicy Bypass -File tools\color-theme-showcase\Showcase.ps1
#
# -ImagePath shows a photo of your own on the Images tab instead of the
# drawn stand-in portrait.

param([string] $ImagePath)

$ErrorActionPreference = 'Stop'

# WPF needs a single-threaded apartment.
if ([Threading.Thread]::CurrentThread.GetApartmentState() -ne 'STA') {
  $arguments = @('-STA', '-ExecutionPolicy', 'Bypass', '-File', $PSCommandPath)
  if ($ImagePath) { $arguments += @('-ImagePath', $ImagePath) }
  powershell.exe @arguments
  return
}

Add-Type -AssemblyName PresentationFramework, PresentationCore, WindowsBase

# Every window title starts with this, so one window rule matches them all.
$TitlePrefix = 'Color Theme Showcase'

function Read-Xaml([string] $FileName) {
  $path = Join-Path $PSScriptRoot $FileName
  $reader = [Xml.XmlReader]::Create($path)
  try {
    [Windows.Markup.XamlReader]::Load($reader)
  } finally {
    $reader.Dispose()
  }
}

function Show-Dialog([string] $Kind, $Owner, [switch] $Modal) {
  $dialog = Read-Xaml 'Dialog.xaml'
  $dialog.Title = "$TitlePrefix - $Kind"

  if ($Owner) {
    $dialog.Owner = $Owner
  } else {
    $dialog.WindowStartupLocation = 'CenterScreen'
  }

  # `$this` is the clicked button; handlers can't close over `$dialog`, as
  # `GetNewClosure` would hide this script's functions from them.
  $dialog.FindName('Nested').Add_Click({
    Show-Dialog 'Nested dialog' ([Windows.Window]::GetWindow($this)) -Modal
  })
  $dialog.FindName('Ok').Add_Click({ [Windows.Window]::GetWindow($this).Close() })
  $dialog.FindName('Cancel').Add_Click({ [Windows.Window]::GetWindow($this).Close() })

  if ($Modal) {
    [void] $dialog.ShowDialog()
  } else {
    $dialog.Show()
  }
}

$window = Read-Xaml 'MainWindow.xaml'
$window.Title = $TitlePrefix

function Get-Control([string] $Name) {
  $control = $window.FindName($Name)
  if (-not $control) { throw "No control named '$Name' in MainWindow.xaml." }
  $control
}

# Gray ramp: white to black in 11 steps.
$ramp = Get-Control 'GrayRamp'
$swatchStyle = $window.FindResource('Swatch')
for ($i = 0; $i -lt 11; $i++) {
  $level = [byte][Math]::Round(255 * (10 - $i) / 10)
  $hex = '#{0:x2}{0:x2}{0:x2}' -f $level
  $swatch = New-Object Windows.Controls.Border
  $swatch.Style = $swatchStyle
  $swatch.Background = New-Object Windows.Media.SolidColorBrush (
    [Windows.Media.Color]::FromRgb($level, $level, $level))
  $swatch.ToolTip = $hex
  [void] $ramp.Children.Add($swatch)
}

$people = New-Object Data.DataTable
foreach ($column in 'Name', 'Role', 'City') { [void] $people.Columns.Add($column) }
[void] $people.Columns.Add('Age', [int])
@(
  @('Ada Lovelace', 'Analyst', 'London', 36),
  @('Alan Turing', 'Researcher', 'Manchester', 41),
  @('Grace Hopper', 'Engineer', 'New York', 85),
  @('Edsger Dijkstra', 'Professor', 'Austin', 72),
  @('Barbara Liskov', 'Professor', 'Boston', 84),
  @('Donald Knuth', 'Author', 'Stanford', 86),
  @('Margaret Hamilton', 'Director', 'Cambridge', 88),
  @('Linus Torvalds', 'Maintainer', 'Portland', 54)
) | ForEach-Object { [void] $people.Rows.Add($_) }
(Get-Control 'People').ItemsSource = $people.DefaultView

if ($ImagePath) {
  $photo = [Windows.Media.Imaging.BitmapImage]::new()
  $photo.BeginInit()
  $photo.UriSource = [Uri]::new((Resolve-Path -LiteralPath $ImagePath).Path)
  $photo.CacheOption = 'OnLoad'
  $photo.EndInit()

  # The cast unwraps PowerShell's `PSObject`, which WPF can't use as an
  # image once stored in a resource dictionary.
  $window.Resources['Portrait'] = [Windows.Media.ImageSource] $photo
}

(Get-Control 'PeopleWithAvatars').ItemsSource = @(
  'Ada Lovelace', 'Alan Turing', 'Grace Hopper', 'Edsger Dijkstra',
  'Barbara Liskov', 'Donald Knuth', 'Margaret Hamilton', 'Linus Torvalds',
  'Katherine Johnson', 'Dennis Ritchie', 'Frances Allen', 'Ken Thompson'
)

# Repaints every second, so the capture keeps delivering frames even when
# nothing else changes.
$clockItem = Get-Control 'ClockItem'
$clockItem.Content = (Get-Date).ToString('HH:mm:ss')
$clock = New-Object Windows.Threading.DispatcherTimer
$clock.Interval = [TimeSpan]::FromSeconds(1)
$clock.Add_Tick({ $clockItem.Content = (Get-Date).ToString('HH:mm:ss') })
$clock.Start()

$sizeItem = Get-Control 'SizeItem'
$window.Add_SizeChanged({
  $sizeItem.Content = '{0} x {1}' -f [int]$_.NewSize.Width, [int]$_.NewSize.Height
})

$statusBar = Get-Control 'MainStatusBar'
(Get-Control 'ToggleStatusBar').Add_Click({
  $statusBar.Visibility = if ($this.IsChecked) { 'Visible' } else { 'Collapsed' }
})

# App-initiated resizes, as opposed to ones GlazeWM makes.
(Get-Control 'Grow').Add_Click({
  $window.Width += 100
  $window.Height += 100
})
(Get-Control 'Shrink').Add_Click({
  $window.Width = [Math]::Max(300, $window.Width - 100)
  $window.Height = [Math]::Max(200, $window.Height - 100)
})

(Get-Control 'ModalDialog').Add_Click({ Show-Dialog 'Modal dialog' $window -Modal })
(Get-Control 'ModelessDialog').Add_Click({ Show-Dialog 'Modeless dialog' $window })
(Get-Control 'UnownedWindow').Add_Click({ Show-Dialog 'Unowned window' $null })

(Get-Control 'ShowMessageBox').Add_Click({
  [void] [Windows.MessageBox]::Show($window,
    'A native message box owned by the main window.',
    "$TitlePrefix - Message box", 'OKCancel', 'Information')
})
(Get-Control 'About').Add_Click({
  [void] [Windows.MessageBox]::Show($window,
    'Test window for GlazeWM color themes.', "$TitlePrefix - About")
})

$openFile = {
  $fileDialog = New-Object Microsoft.Win32.OpenFileDialog
  $fileDialog.Title = "$TitlePrefix - Open file"
  [void] $fileDialog.ShowDialog($window)
}
(Get-Control 'OpenFile').Add_Click($openFile)
(Get-Control 'OpenFileButton').Add_Click($openFile)
(Get-Control 'Exit').Add_Click({ $window.Close() })

[void] $window.ShowDialog()
$clock.Stop()
