using System.Windows;
using System.Windows.Controls;
using System.Windows.Media;
using System.Windows.Threading;
using Microsoft.Win32;

namespace ColorThemeShowcase;

public partial class MainWindow : Window
{
    private readonly DispatcherTimer _clock = new() { Interval = TimeSpan.FromSeconds(1) };

    public MainWindow()
    {
        InitializeComponent();

        BuildGrayRamp();
        People.ItemsSource = Person.Samples();

        // Repaints the status bar every second, so the capture keeps
        // delivering frames even when nothing else changes.
        _clock.Tick += (_, _) => ClockItem.Content = DateTime.Now.ToString("HH:mm:ss");
        _clock.Start();
        ClockItem.Content = DateTime.Now.ToString("HH:mm:ss");

        SizeChanged += (_, e) =>
            SizeItem.Content = $"{(int)e.NewSize.Width} x {(int)e.NewSize.Height}";
    }

    private void BuildGrayRamp()
    {
        const int steps = 11;

        for (var i = 0; i < steps; i++)
        {
            var level = (byte)Math.Round(255.0 * (steps - 1 - i) / (steps - 1));
            var color = Color.FromRgb(level, level, level);

            GrayRamp.Children.Add(new Border
            {
                Style = (Style)FindResource("Swatch"),
                Background = new SolidColorBrush(color),
                ToolTip = $"#{level:x2}{level:x2}{level:x2}",
            });
        }
    }

    private void ModalDialog_Click(object sender, RoutedEventArgs e) =>
        new DialogWindow { Owner = this, Title = "Modal dialog" }.ShowDialog();

    private void ModelessDialog_Click(object sender, RoutedEventArgs e) =>
        new DialogWindow { Owner = this, Title = "Modeless dialog" }.Show();

    private void UnownedWindow_Click(object sender, RoutedEventArgs e) =>
        new DialogWindow { Title = "Unowned window" }.Show();

    private void MessageBox_Click(object sender, RoutedEventArgs e) =>
        MessageBox.Show(this, "A native message box owned by the main window.",
            "Message box", MessageBoxButton.OKCancel, MessageBoxImage.Information);

    private void About_Click(object sender, RoutedEventArgs e) =>
        MessageBox.Show(this, "Test window for GlazeWM color themes.", "About");

    private void OpenFile_Click(object sender, RoutedEventArgs e) =>
        new OpenFileDialog { Title = "Common file dialog" }.ShowDialog(this);

    private void Exit_Click(object sender, RoutedEventArgs e) => Close();

    private void ToggleStatusBar_Click(object sender, RoutedEventArgs e) =>
        MainStatusBar.Visibility = ((MenuItem)sender).IsChecked
            ? Visibility.Visible
            : Visibility.Collapsed;

    // App-initiated resizes, as opposed to ones GlazeWM makes.
    private void Grow_Click(object sender, RoutedEventArgs e) => Resize(100);

    private void Shrink_Click(object sender, RoutedEventArgs e) => Resize(-100);

    private void Resize(double delta)
    {
        Width = Math.Max(MinWidth, Width + delta);
        Height = Math.Max(MinHeight, Height + delta);
    }
}

public record Person(string Name, string Role, string City, int Age)
{
    public static IReadOnlyList<Person> Samples() =>
    [
        new("Ada Lovelace", "Analyst", "London", 36),
        new("Alan Turing", "Researcher", "Manchester", 41),
        new("Grace Hopper", "Engineer", "New York", 85),
        new("Edsger Dijkstra", "Professor", "Austin", 72),
        new("Barbara Liskov", "Professor", "Boston", 84),
        new("Donald Knuth", "Author", "Stanford", 86),
        new("Margaret Hamilton", "Director", "Cambridge", 88),
        new("Linus Torvalds", "Maintainer", "Portland", 54),
    ];
}
