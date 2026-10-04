using System;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Threading;

namespace GensysMock;

/// <summary>A ticket's details window, titled "Ticket details for ...",
/// which is what GlazeWM stacks.</summary>
public sealed class TicketWindow : Window
{
  private readonly Ticket _ticket;
  private TicketPopup? _popup;

  public TicketWindow(Ticket ticket, bool lateTitle)
  {
    _ticket = ticket;

    Width = 900;
    Height = 650;
    Background = Ui.Window;
    Title = lateTitle ? "" : ticket.DetailsTitle;

    if (lateTitle)
    {
      // Like apps that show a window before giving it its title.
      var timer = new DispatcherTimer { Interval = TimeSpan.FromMilliseconds(700) };
      timer.Tick += (_, _) =>
      {
        timer.Stop();
        Title = ticket.DetailsTitle;
      };
      Loaded += (_, _) => timer.Start();
    }

    Content = BuildContent();
  }

  /// <summary>Opens the modal "Add Activity" dialog. Like any WPF
  /// <c>ShowDialog</c>, it disables all the app's other windows until it
  /// closes.</summary>
  public void ShowAddActivity()
  {
    var dialog = new AddActivityDialog(_ticket) { Owner = this };

    if (dialog.ShowDialog() == true && dialog.Result is { } activity)
    {
      _ticket.Activities.Insert(0, activity);
    }
  }

  private UIElement BuildContent()
  {
    var title = new TextBlock
    {
      Text = $"#{_ticket.Number}  {_ticket.Subject}",
      FontSize = 20,
      FontWeight = FontWeights.SemiBold,
      Foreground = Ui.Accent,
      TextWrapping = TextWrapping.Wrap,
    };

    var fields = new Grid { Margin = new Thickness(0, 12, 0, 12) };
    fields.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
    fields.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
    fields.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
    fields.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });

    AddField(fields, 0, 0, "Customer", new TextBox { Text = _ticket.Customer });
    AddField(fields, 0, 2, "Contact", new TextBox { Text = _ticket.Contact });
    AddField(fields, 1, 0, "Status", Combo(_ticket.Status, "New", "In progress", "Waiting on customer", "Scheduled", "Closed"));
    AddField(fields, 1, 2, "Priority", Combo(_ticket.Priority, "Low", "Normal", "High", "Urgent"));

    var description = new TextBox
    {
      Text = _ticket.Description,
      AcceptsReturn = true,
      TextWrapping = TextWrapping.Wrap,
      MinHeight = 90,
      VerticalScrollBarVisibility = ScrollBarVisibility.Auto,
    };

    var activities = new ListView
    {
      ItemsSource = _ticket.Activities,
      View = Ui.Columns(
        ("When", nameof(Activity.At), 150),
        ("Type", nameof(Activity.Kind), 90),
        ("Note", nameof(Activity.Note), 520)
      ),
      MinHeight = 140,
    };

    var buttons = new StackPanel
    {
      Orientation = Orientation.Horizontal,
      Margin = new Thickness(0, 12, 0, 0),
    };
    buttons.Children.Add(Ui.Button("Add Activity…", ShowAddActivity, primary: true));
    buttons.Children.Add(Ui.Button("Open ticket popup", ShowPopup));
    buttons.Children.Add(Ui.Button("Close", Close));

    var panel = new DockPanel { Margin = new Thickness(16) };
    foreach (var (element, label) in new (UIElement, string?)[]
    {
      (title, null),
      (fields, null),
      (description, "Description"),
      (buttons, null),
    })
    {
      if (label is not null)
      {
        var caption = Ui.Label(label);
        DockPanel.SetDock(caption, Dock.Top);
        panel.Children.Add(caption);
      }

      DockPanel.SetDock(element, element == buttons ? Dock.Bottom : Dock.Top);
      panel.Children.Add(element);
    }

    var activitiesCaption = Ui.Label("Activities");
    DockPanel.SetDock(activitiesCaption, Dock.Top);
    panel.Children.Add(activitiesCaption);
    panel.Children.Add(activities);

    return new Border { Background = Ui.Panel, Margin = new Thickness(12), Child = panel };
  }

  /// <summary>Shows a small, non-modal popup owned by this window.</summary>
  private void ShowPopup()
  {
    if (_popup is { IsLoaded: true })
    {
      _popup.Activate();
      return;
    }

    _popup = new TicketPopup(_ticket) { Owner = this };
    _popup.Show();
  }

  private static ComboBox Combo(string selected, params string[] items)
  {
    var combo = new ComboBox { ItemsSource = items, SelectedItem = selected };
    return combo;
  }

  private static void AddField(Grid grid, int row, int column, string label, Control input)
  {
    while (grid.RowDefinitions.Count <= row)
    {
      grid.RowDefinitions.Add(new RowDefinition { Height = GridLength.Auto });
    }

    var caption = Ui.Label(label);
    Grid.SetRow(caption, row);
    Grid.SetColumn(caption, column);
    grid.Children.Add(caption);

    input.Margin = new Thickness(0, 4, 16, 4);
    Grid.SetRow(input, row);
    Grid.SetColumn(input, column + 1);
    grid.Children.Add(input);
  }
}

/// <summary>Small non-modal summary popup, titled "Ticket: ...".</summary>
public sealed class TicketPopup : Window
{
  public TicketPopup(Ticket ticket)
  {
    Title = $"Ticket: {ticket.Number}";
    Width = 360;
    Height = 220;
    ShowInTaskbar = false;
    WindowStartupLocation = WindowStartupLocation.CenterOwner;
    Background = Ui.Window;

    var text = new TextBlock
    {
      Text = $"#{ticket.Number}\n{ticket.Customer} — {ticket.Contact}\n\n{ticket.Subject}\nStatus: {ticket.Status}",
      TextWrapping = TextWrapping.Wrap,
      Margin = new Thickness(16),
    };

    Content = text;
  }
}

/// <summary>The modal "Add Activity" dialog.</summary>
public sealed class AddActivityDialog : Window
{
  private readonly ComboBox _kind;
  private readonly TextBox _note;

  public Activity? Result { get; private set; }

  public AddActivityDialog(Ticket ticket)
  {
    Title = "Add Activity";
    Width = 440;
    Height = 300;
    ResizeMode = ResizeMode.NoResize;
    ShowInTaskbar = false;
    WindowStartupLocation = WindowStartupLocation.CenterOwner;
    Background = Ui.Window;

    _kind = new ComboBox
    {
      ItemsSource = new[] { "Call", "E-mail", "Note", "Visit" },
      SelectedIndex = 0,
      Margin = new Thickness(0, 4, 0, 8),
    };

    _note = new TextBox
    {
      AcceptsReturn = true,
      TextWrapping = TextWrapping.Wrap,
      Height = 90,
      Margin = new Thickness(0, 4, 0, 8),
    };

    var buttons = new StackPanel
    {
      Orientation = Orientation.Horizontal,
      HorizontalAlignment = HorizontalAlignment.Right,
    };
    buttons.Children.Add(Ui.Button("Save", Save, primary: true));
    buttons.Children.Add(Ui.Button("Cancel", () => DialogResult = false));

    var panel = new StackPanel { Margin = new Thickness(16) };
    panel.Children.Add(new TextBlock
    {
      Text = $"New activity for ticket #{ticket.Number}",
      FontWeight = FontWeights.SemiBold,
      Margin = new Thickness(0, 0, 0, 8),
    });
    panel.Children.Add(Ui.Label("Type"));
    panel.Children.Add(_kind);
    panel.Children.Add(Ui.Label("Note"));
    panel.Children.Add(_note);
    panel.Children.Add(buttons);
    Content = panel;

    Loaded += (_, _) => _note.Focus();
  }

  private void Save()
  {
    Result = new Activity
    {
      At = DateTime.Now,
      Kind = (string)_kind.SelectedItem,
      Note = string.IsNullOrWhiteSpace(_note.Text) ? "(no note)" : _note.Text.Trim(),
    };

    DialogResult = true;
  }
}
