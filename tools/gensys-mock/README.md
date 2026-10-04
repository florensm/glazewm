# Gensys stand-in

A small WPF app that behaves like the ticketing app GlazeWM's stacks were
built for, to try stacks out without the real thing:

- The main window, **Gensys**, lists a ticket queue. Double-click a ticket
  (or use **Open ticket** / **Open 3 tickets**) to open its window, titled
  `Ticket details for ticket: 4711 — ACME B.V. — Gensys`.
- **Add Activity…** in a ticket opens a modal dialog. Like any WPF
  `ShowDialog`, it disables all the app's other windows until it closes.
- **Open ticket popup** opens a small non-modal `Ticket: 4711` window owned
  by the ticket.
- **Titles arrive late** shows ticket windows untitled first, titling them
  0.7 s later, as some WPF apps do.

The executable is named `Gensys.exe`, so rules matching the real app's
process name apply to it. It runs on Windows 10/11 as is (.NET Framework
4.8).

## Getting it

Download the `gensys-mock` artifact of a "Stack e2e (Windows)" run, or
build it with the .NET SDK:

```sh
dotnet build tools/gensys-mock -c Release -o mock
```

## Command-line options

| Option | Effect |
| --- | --- |
| `--tickets <n>` | Open the first `n` tickets on start. |
| `--late-titles` | Start with **Titles arrive late** checked. |
| `--dialog-trigger <file>` | Open the first ticket's "Add Activity" dialog whenever `<file>` appears (used by the end-to-end test). |

## GlazeWM config to try it with

```yaml
window_rules:
  # Keep the stacked tickets usable while "Add Activity" is open.
  - commands: ["stay-interactive"]
    match:
      - window_process: { equals: "Gensys" }

stack:
  auto_stack:
    - name: "tickets"
      match:
        - window_process: { equals: "Gensys" }
          window_title: { regex: "^Ticket details for" }
  tab_title_overrides:
    # Shows "4711 — ACME B.V." on the tab.
    - regex: "^Ticket details for ticket: "
      replace: ""
    - regex: " — Gensys$"
      replace: ""
```
