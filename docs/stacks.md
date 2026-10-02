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
| `move-to-stack --name <name>` | Move the focused window into the named stack, creating it if needed. |
| `cycle-stack-focus [--prev]` | Focus the next (or previous) tab, wrapping around. |
| `focus-stack-index --index <n>` | Focus the tab at zero-based index `n`. |

Moving or resizing a window in a stack moves or resizes the stack as a
whole. Directional focus enters a stack through its active tab.

An unnamed stack is removed once it holds a single window. A named stack
(`move-to-stack`) is kept until it is empty, so it stays a target for
windows that open later.

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
