using System;
using System.Collections.ObjectModel;
using System.Linq;

namespace GensysMock;

public sealed class Activity
{
  public DateTime At { get; set; }
  public string Kind { get; set; } = "";
  public string Note { get; set; } = "";
}

public sealed class Ticket
{
  public int Number { get; set; }
  public string Customer { get; set; } = "";
  public string Contact { get; set; } = "";
  public string Subject { get; set; } = "";
  public string Status { get; set; } = "";
  public string Priority { get; set; } = "";
  public string Description { get; set; } = "";
  public ObservableCollection<Activity> Activities { get; } = new();

  /// <summary>Title of the ticket's details window.</summary>
  public string DetailsTitle =>
    $"Ticket details for ticket: {Number} — {Customer} — Gensys";
}

public static class TicketStore
{
  private static readonly (string Customer, string Contact, string Subject)[] Seeds =
  {
    ("ACME B.V.", "J. de Vries", "Printer on 2nd floor offline"),
    ("Contoso", "M. Jansen", "Cannot log in to VPN"),
    ("Fabrikam", "S. Bakker", "Outlook keeps asking for password"),
    ("Northwind", "L. Visser", "New laptop for onboarding"),
    ("Tailspin", "R. Smit", "Shared drive permissions"),
    ("Wingtip", "E. Meijer", "Teams calls drop after 5 minutes"),
    ("Litware", "K. Mulder", "Invoice export fails"),
    ("Adventure Works", "T. de Boer", "Phone number change"),
    ("Woodgrove", "A. Dekker", "Backup job failed overnight"),
    ("Proseware", "N. Peters", "Request: second monitor"),
  };

  private static readonly string[] Statuses = { "New", "In progress", "Waiting on customer", "Scheduled" };
  private static readonly string[] Priorities = { "Low", "Normal", "High", "Urgent" };

  public static ObservableCollection<Ticket> All { get; } = new(
    Seeds.Select((seed, index) =>
    {
      var ticket = new Ticket
      {
        Number = 4711 + index * 13,
        Customer = seed.Customer,
        Contact = seed.Contact,
        Subject = seed.Subject,
        Status = Statuses[index % Statuses.Length],
        Priority = Priorities[index % Priorities.Length],
        Description = $"{seed.Contact} reports: {seed.Subject.ToLowerInvariant()}.\n"
          + "Please check and get back to the customer.",
      };

      ticket.Activities.Add(new Activity
      {
        At = DateTime.Now.AddHours(-index - 2),
        Kind = "Call",
        Note = "Ticket created by phone.",
      });

      return ticket;
    })
  );
}
