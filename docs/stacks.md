# Stacks

A stack shows several windows in one tile, one at a time, with a tab bar to
switch between them. The windows stay normal top-level windows managed by
GlazeWM: only the active tab is shown, the other tabs are cloaked.

## Commands

These cover everyday use and are bound in the sample config:

| Command | Behaviour |
| --- | --- |
| `toggle-stack` | Wrap the focused window in a new stack, or take it out of its stack. A floating window gets a floating stack. |
| `stack-absorb-neighbor --direction <dir>` | Pull the neighbouring window in `<dir>` into the focused window's stack. A neighbouring stack is merged in. |
| `cycle-stack-focus [--prev]` | Focus the next (or previous) tab, wrapping around. |
| `float-out-of-stack` | Take the focused window out of its stack as a floating window. |

With the mouse, the tab bar does the same: click a tab, drag a tab off the
bar, drop a window onto a bar, and drag tabs to reorder them.

These are available for your own bindings or scripts:

| Command | Behaviour |
| --- | --- |
| `focus-stack-index --index <n>` | Focus the tab at zero-based index `n`. |
| `move-stack-tab [--prev]` | Move the active tab one position right (or left), wrapping around. |
| `move-to-stack --name <name>` | Move the focused window into the named stack, on any workspace. Creates the stack in place if it doesn't exist. |
| `stack-all` | Put all tiling windows of the workspace into one stack (the focused window's). |
| `unstack-all` | Take apart every stack on the workspace. Windows of a floating stack are cascaded. |

Moving or resizing a window in a stack moves or resizes the stack as a
whole. Directional focus enters a stack through its active tab.

Closing or removing a tab never changes the stack's size. An unnamed stack
is removed once it holds a single window, which then takes over the
stack's whole slot. A named stack is kept until it is empty, so it stays a
target for windows that open later.

### Coming from other window managers

| GlazeWM | komorebi | Hyprland | i3 / sway |
| --- | --- | --- | --- |
| `toggle-stack` | `stack` / `unstack` | `togglegroup` | `layout tabbed` / `layout toggle split` |
| `stack-absorb-neighbor --direction <dir>` | `stack <dir>` | `moveintogroup <dir>` | `move <dir>` into a tabbed container |
| `float-out-of-stack` | `unstack` | `moveoutofgroup` | `move` out + `floating enable` |
| `cycle-stack-focus [--prev]` | `cycle-stack next/previous` | `changegroupactive f/b` | `focus left/right` |
| `focus-stack-index --index <n>` | `focus-stack-window <n>` | `changegroupactive <n>` | — |
| `move-stack-tab [--prev]` | — | `movegroupwindow f/b` | `move left/right` |
| `stack-all` / `unstack-all` | `stack-all` / `unstack-all` | — | — |
| `stack.new_tab_position: after_active` | — | `group:insert_after_current` | — |

## A stack acts as one window

All windows of a stack share one state, so commands and events that change
a window's state change the whole stack:

- `toggle-floating` floats the stack, tab bar included, at the size of its
  active window. Toggled again, the stack tiles back into its old slot.
- Dragging a stacked window, or resizing it, drags or resizes the whole
  stack, whether it is tiling or floating. A dragged tiling stack drops
  into the layout like a single window.
- `toggle-fullscreen` (or maximizing a stacked window) makes the stack
  fullscreen, without its tab bar; tabs can still be switched with
  `cycle-stack-focus`.
- Minimizing a stacked window minimizes the stack. Restoring it, or
  activating any of its windows from the taskbar, restores the stack.
- A fullscreen stack moved to another monitor goes as a whole.

To take a single window out, use `float-out-of-stack`, the tab's "Float
window" menu item, or drag its tab off the bar. A window floated out of a
tiling stack goes back into it with `toggle-floating` (or next to the
window left over, if the stack was removed when it had one window left).

Drop a window onto a stack's tab bar (by moving it, not resizing it) to add
it to the stack; a floating window joins a floating stack as is. Dropping a stacked window there brings
its whole stack along.

## Tab bar

Each stack has a tab bar, a rounded strip with one tab per window; the
active tab is highlighted, and the highlight slides when it changes.

- Click a tab to show it; scroll the mouse wheel over the bar to switch to
  the next or previous tab.
- Close a tab's window with its close button (shown on the active and the
  hovered tab by default) or a middle click. This sends it a normal
  `WM_CLOSE`.
- Drag a tab sideways to reorder it; drag it well above or below the bar
  to take its window out of the stack, floating where you let go.
- Right-click a tab for a menu with "Close", "Float window" and "Remove
  from stack".
- Hovering a tab whose title is cut off (or that only shows its icon)
  shows the full title in a tooltip.
- A tab whose window requests attention (e.g. flashes in the taskbar) is
  highlighted with `tab_urgent_background` until the window is shown.
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
  tab_urgent_background: "#8a5a00"
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
  # Where windows added to a stack go among its tabs: "end" or
  # "after_active" (right after the active tab).
  new_tab_position: "end"
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

## Staying usable while a popup is open

Apps such as WPF apps disable all their windows while a blocking popup
(e.g. "Add Activity") is open, including the ones in a stack. Mark an
app's windows with the `stay-interactive` window rule command to keep its
stacked windows usable meanwhile:

```yaml
window_rules:
  - commands: ["stay-interactive"]
    match:
      - window_process: { equals: "MyApp" }
```

- When a window of the app is shown or focused while its marked stacked
  windows are disabled, they are re-enabled. Only windows that are in a
  stack are affected.
- The popup is made floating and shown on top, so it stays visible when
  you click back into a stacked window.
- If the app keeps disabling the windows (e.g. while busy), they are left
  disabled after three attempts per popup.
- Once the popup closes, the app re-enables its windows itself.

The app doesn't expect input in its other windows while the popup is
open, so actions there that conflict with the popup may misbehave. Only
enable this for apps you've checked.
