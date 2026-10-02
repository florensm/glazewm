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

## Tab bar

Each stack has a tab bar, a rounded strip with one tab per window; the
active tab is highlighted, and the highlight slides when it changes.

- Click a tab to show it; scroll the mouse wheel over the bar to switch to
  the next or previous tab.
- Close a tab's window with its close button (shown on the active and the
  hovered tab by default) or a middle click. This sends it a normal
  `WM_CLOSE`.
- Drag a tab sideways to reorder it; drag it well above or below the bar
  to take its window out of the stack.
- Right-click a tab for a menu with "Close" and "Remove from stack".
- Tabs show their window's icon. Icons that apps set at runtime (as WPF
  apps do) are fetched in the background, so a busy app never stalls the
  bar; until then the window class icon is shown.

The bar sits directly behind the stack's active window in z-order, so
windows that cover the stack cover its tab bar too.

## Config

```yaml
stack:
  # Height of the tab bar (0px = no tab bar).
  tab_bar_height: "28px"
  # "top" | "bottom"
  tab_bar_position: "top"
  # Colors accept a hex value, "accent", or a palette file like borders:
  # { file: "path/to/yasb_colors.css", key: "--yasb-accent-light1" }
  tab_bar_background: "#1f1f1f"
  tab_bar_opacity: "92%"
  tab_active_background: "#3a3a3a"
  tab_hover_background: "#2c2c2c"
  tab_inactive_background: "#00000000"
  tab_text_color: "#ffffff"
  tab_inactive_text_color: "#a0a0a0"
  tab_font_family: "Segoe UI"
  tab_font_size: "12px"
  tab_corner_radius: "8px"
  # Tabs share the bar; they scroll once narrower than `tab_min_width`, and
  # show only their icon below ~60px.
  tab_min_width: "48px"
  # Widest a tab gets (0px = no limit).
  tab_max_width: "0px"
  show_tab_icons: true
  # Prefix titles with their position ("1. Title").
  show_tab_numbers: false
  # "hover" (active and hovered tab) | "always" | "never"
  tab_close_button: "hover"
  # Regex replacements applied to tab titles, in order.
  tab_title_overrides:
    - regex: " - Mozilla Firefox$"
      replace: ""
```

All sizes scale with the monitor's DPI. The tab bar is only drawn on
Windows.

### New windows and popups

```yaml
stack:
  # A new tiling window opened while a stacked window is focused is tiled
  # next to the stack. Set to true to open it inside the stack instead.
  new_windows_join_focused_stack: false
  # Popups (windows with an owner) of an app that has windows in a stack
  # open floating instead of being tiled.
  float_owned_popups: true
```

## Auto-stacking

`stack.auto_stack` puts matching windows into a named stack as they open:

```yaml
stack:
  auto_stack:
    - name: "details"
      match:
        - window_process: { equals: "MyApp" }
          window_title: { regex: "^Details for" }
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
