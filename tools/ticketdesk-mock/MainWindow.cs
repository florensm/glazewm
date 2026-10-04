using System.Collections.Generic;
using System.Linq;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Data;
using System.Windows.Input;
using System.Windows.Media;

namespace TicketDeskMock;

/// <summary>The app's main window: the ticket queue.</summary>
public sealed class MainWindow : Window
{
  private readonly Options _options;
  private readonly ListView _queue;
  private readonly CheckBox _lateTitles;
  private readonly Dictionary<int, TicketWindow> _open = new();

  public IEnumerable<TicketWindow> OpenTicketWindows => _open.Values;

  public MainWindow(Options options)
  {
    _options = options;

    Title = "TicketDesk";
    Width = 1000;
    Height = 600;
    Background = Ui.Window;

    _queue = new ListView
    {
      ItemsSource = TicketStore.All,
      View = Ui.Columns(
        ("Ticket", nameof(Ticket.Number), 70),
        ("Customer", nameof(Ticket.Customer), 150),
        ("Subject", nameof(Ticket.Subject), 330),
        ("Status", nameof(Ticket.Status), 150),
        ("Priority", nameof(Ticket.Priority), 90)
      ),
      Margin = new Thickness(12, 0, 12, 12),
    };
    _queue.MouseDoubleClick += (_, _) => OpenSelected();
    _queue.KeyDown += (_, e) =>
    {
      if (e.Key == Key.Enter) OpenSelected();
    };

    _lateTitles = new CheckBox
    {
      Content = "Titles arrive late",
      IsChecked = options.LateTitles,
      VerticalAlignment = VerticalAlignment.Center,
      Margin = new Thickness(16, 0, 0, 0),
      ToolTip = "Show ticket windows untitled first and title them a moment later, as some WPF apps do.",
    };

    var toolbar = new StackPanel { Orientation = Orientation.Horizontal, Margin = new Thickness(12) };
    toolbar.Children.Add(Ui.Button("Open ticket", OpenSelected, primary: true));
    toolbar.Children.Add(Ui.Button("Open 3 tickets", () => OpenMany(3)));
    toolbar.Children.Add(Ui.Button("Open all", () => OpenMany(TicketStore.All.Count)));
    toolbar.Children.Add(_lateTitles);

    var header = new TextBlock
    {
      Text = "Ticket queue",
      FontSize = 20,
      FontWeight = FontWeights.SemiBold,
      Foreground = Ui.Accent,
      Margin = new Thickness(12, 12, 12, 0),
    };

    var layout = new DockPanel();
    DockPanel.SetDock(header, Dock.Top);
    DockPanel.SetDock(toolbar, Dock.Top);
    layout.Children.Add(header);
    layout.Children.Add(toolbar);
    layout.Children.Add(_queue);
    Content = layout;
  }

  /// <summary>Opens the details window of <paramref name="ticket"/>, or
  /// brings it forward if it's already open.</summary>
  public void OpenTicket(Ticket ticket)
  {
    if (_open.TryGetValue(ticket.Number, out var existing))
    {
      existing.Activate();
      return;
    }

    var window = new TicketWindow(ticket, _lateTitles.IsChecked == true);
    window.Closed += (_, _) => _open.Remove(ticket.Number);
    _open[ticket.Number] = window;
    window.Show();
  }

  private void OpenSelected()
  {
    if (_queue.SelectedItem is Ticket ticket)
    {
      OpenTicket(ticket);
    }
  }

  private void OpenMany(int count)
  {
    foreach (var ticket in TicketStore.All.Where(t => !_open.ContainsKey(t.Number)).Take(count).ToList())
    {
      OpenTicket(ticket);
    }
  }
}

/// <summary>Shared look of the app's controls.</summary>
public static class Ui
{
  public static readonly Brush Window = new SolidColorBrush(Color.FromRgb(0xF4, 0xF6, 0xF8));
  public static readonly Brush Panel = Brushes.White;
  public static readonly Brush Accent = new SolidColorBrush(Color.FromRgb(0x1F, 0x5F, 0xAD));
  public static readonly Brush Muted = new SolidColorBrush(Color.FromRgb(0x5F, 0x6B, 0x7A));

  public static Button Button(string text, System.Action onClick, bool primary = false)
  {
    var button = new Button
    {
      Content = text,
      Padding = new Thickness(14, 5, 14, 5),
      Margin = new Thickness(0, 0, 8, 0),
      MinWidth = 90,
    };

    if (primary)
    {
      button.Background = Accent;
      button.Foreground = Brushes.White;
      button.BorderBrush = Accent;
    }

    button.Click += (_, _) => onClick();
    return button;
  }

  public static GridView Columns(params (string Header, string Path, double Width)[] columns)
  {
    var view = new GridView();

    foreach (var (header, path, width) in columns)
    {
      view.Columns.Add(new GridViewColumn
      {
        Header = header,
        DisplayMemberBinding = new Binding(path),
        Width = width,
      });
    }

    return view;
  }

  public static TextBlock Label(string text) => new()
  {
    Text = text,
    Foreground = Muted,
    Margin = new Thickness(0, 6, 12, 6),
    VerticalAlignment = VerticalAlignment.Center,
  };
}
