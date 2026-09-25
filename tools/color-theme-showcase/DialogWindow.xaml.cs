using System.Windows;

namespace ColorThemeShowcase;

public partial class DialogWindow : Window
{
    public DialogWindow() => InitializeComponent();

    private void Nested_Click(object sender, RoutedEventArgs e) =>
        new DialogWindow { Owner = this, Title = "Nested dialog" }.ShowDialog();

    private void Close_Click(object sender, RoutedEventArgs e) => Close();
}
