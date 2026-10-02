# Stacks

A stack shows several windows in one tile, one at a time, with a tab bar to
switch between them. The windows stay normal top-level windows managed by
GlazeWM: only the active tab is shown, the other tabs are cloaked.

## Commands

| Command | Behaviour |
| --- | --- |
| `toggle-stack` | Wrap the focused window in a new stack, or take it out of its stack. |
| `stack-insert` | Stack the focused window with the most recently focused other tiling window. |
| `stack-absorb-neighbor --direction <dir>` | Pull the neighbouring window in `<dir>` into the focused window's stack. |
| `move-to-stack --name <name>` | Move the focused window into the named stack, on any workspace. Creates the stack in place if it doesn't exist. |
| `cycle-stack-focus [--prev]` | Focus the next (or previous) tab, wrapping around. |
| `focus-stack-index --index <n>` | Focus the tab at zero-based index `n`. |

Moving or resizing a window in a stack moves or resizes the stack as a
whole. Directional focus enters a stack through its active tab.

Closing or removing a tab never changes the stack's size. An unnamed stack
is removed once it holds a single window, which then takes over the
stack's whole slot. A named stack is kept until it is empty, so it stays a
target for windows that open later.

## Config

```yaml
stack:
  # Height of the tab bar (0px = no tab bar).
  tab_bar_height: "30px"
  # "top" | "bottom"
  tab_bar_position: "top"
  tab_bar_background: "#2d2d2d"
  tab_active_background: "#4a4a6a"
  tab_inactive_background: "#1e1e2e"
  tab_text_color: "#cdd6f4"
```

The tab bar height scales with the monitor's DPI when
`gaps.scale_with_dpi` is enabled. The tab bar is only drawn on Windows.

## Auto-stacking

`stack.auto_stack` puts matching windows into a named stack as they open:

```yaml
stack:
  auto_stack:
    - name: "tickets"
      match:
        - window_process: { equals: "Gensys" }
          window_title: { regex: "^Ticket details for" }
      # Optional: windows that never join, even if they match.
      exclude: []
      # Optional: where to create the stack if it doesn't exist yet.
      # Defaults to where the window would otherwise open.
      workspace: "1"
      # Optional: let windows with an owner window join (default false).
      allow_owned: false
  # How long an untitled window that could still match is held back.
  auto_stack_title_timeout_ms: 1500
```

- There is one stack per name across all workspaces and monitors. A
  matching window joins it wherever it opens.
- A matching window is attached straight into the stack, with no tiling
  step in between, and becomes its active tab. It only takes focus if the
  OS made it the foreground window, i.e. you opened it yourself.
- Some apps (often WPF) show a window before giving it a title. A window
  with an empty title whose process and class match a rule is held back,
  hidden, until it gets a title or `auto_stack_title_timeout_ms` passes.
  It is then stacked or placed normally.
- A window that only gets a matching title after it was placed joins the
  stack once. Reloading the config also applies the rules to windows that
  are already open.
- A window that matches but is never stacked (see below) is logged with
  the reason, e.g. `Not auto-stacking window '...' because it has an owner
  window`.
- Dialogs (`#32770`, modal frames), tool windows and, unless
  `allow_owned` is set, owned windows are never stacked.
- Each window joins at most once: a window you take out of its stack is
  never pulled back in.
- Window rules that would move a stacked window or change its state
  (`set-floating`, `move --workspace`, ...) are skipped when it is
  auto-stacked.
