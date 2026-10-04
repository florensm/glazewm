using System;
using System.IO;
using System.Linq;
using System.Windows;
using System.Windows.Threading;

namespace TicketDeskMock;

/// <summary>Command-line options, mainly for scripted tests.</summary>
public sealed class Options
{
  /// <summary>Number of tickets opened on start.</summary>
  public int OpenTickets { get; set; }

  /// <summary>Show ticket windows untitled first, titling them a moment
  /// later, as some WPF apps do.</summary>
  public bool LateTitles { get; set; }

  /// <summary>File whose appearance makes the first ticket open its
  /// "Add Activity" dialog, then is deleted.</summary>
  public string? DialogTrigger { get; set; }

  public static Options Parse(string[] args)
  {
    var options = new Options();

    for (var i = 0; i < args.Length; i++)
    {
      switch (args[i])
      {
        case "--tickets" when i + 1 < args.Length:
          options.OpenTickets = int.Parse(args[++i]);
          break;
        case "--late-titles":
          options.LateTitles = true;
          break;
        case "--dialog-trigger" when i + 1 < args.Length:
          options.DialogTrigger = Path.GetFullPath(args[++i]);
          break;
        default:
          MessageBox.Show(
            $"Unknown option '{args[i]}'.\n\n"
              + "Options: --tickets <n>, --late-titles, --dialog-trigger <file>",
            "TicketDesk"
          );
          Environment.Exit(1);
          break;
      }
    }

    return options;
  }
}

public static class Program
{
  [STAThread]
  public static void Main(string[] args)
  {
    var options = Options.Parse(args);
    var app = new Application { ShutdownMode = ShutdownMode.OnMainWindowClose };

    var main = new MainWindow(options);
    app.MainWindow = main;
    main.Show();

    foreach (var ticket in TicketStore.All.Take(options.OpenTickets))
    {
      main.OpenTicket(ticket);
    }

    if (options.DialogTrigger is { } trigger)
    {
      WatchDialogTrigger(main, trigger);
    }

    app.Run();
  }

  /// <summary>Opens the first open ticket's "Add Activity" dialog whenever
  /// <paramref name="trigger"/> appears.</summary>
  private static void WatchDialogTrigger(MainWindow main, string trigger)
  {
    var timer = new DispatcherTimer { Interval = TimeSpan.FromMilliseconds(300) };

    timer.Tick += (_, _) =>
    {
      if (!File.Exists(trigger))
      {
        return;
      }

      File.Delete(trigger);
      main.OpenTicketWindows.FirstOrDefault()?.ShowAddActivity();
    };

    timer.Start();
  }
}
