# Special workspace

A scratch workspace for parking windows you want close at hand but out of
the way. There is one special workspace, shared by all monitors. Summoning
it pulls it onto the focused monitor, and leaving it takes you back to the
workspace you were on.

It works like any other workspace (tiling, stacks, floating windows,
layouts), with these differences:

- It is configured in its own `special_workspace` section, not in
  `workspaces`.
- Workspace cycling (`--next-active-workspace`, `--prev-workspace`,
  `--next-active-workspace-on-monitor`, `--next-empty-workspace` and so on)
  skips it. Cycling from inside it goes to the first workspace (next) or
  the last one (previous).
- It is never remembered as the recent workspace, so
  `focus --recent-workspace` and `toggle_workspace_on_refocus` work as if
  it weren't there.
- It always sorts after every other workspace.
- Like a dynamic workspace, it is removed once it is empty and you
  leave it.
- `update-workspace-config` can't change it. Edit the config instead.

## Config

```yaml
special_workspace:
  # Name to address it by (e.g. `focus --workspace special`). Must not be
  # numeric, nor the name of a workspace in `workspaces`.
  name: "special"

  # Name shown for it (e.g. in the overview). Defaults to `name`.
  # display_name: "Scratch"

  # Its layout: "scrolling" (default), "split" or "dwindle".
  layout: "scrolling"
```

A config with an invalid name fails to load (or to reload).

## Commands

| Command | Behaviour |
| --- | --- |
| `focus --special-workspace` (`alt+-` in the sample config) | If the special workspace isn't focused, move it to the focused monitor (creating it if needed) and focus it. If it is focused, go back to the workspace this monitor showed most recently before it, or else to the recent workspace. |
| `move --special-workspace` (`alt+shift+-` in the sample config) | Move the focused window to the special workspace, keeping focus where it is. If the window is already in it, move the window back to the workspace `focus --special-workspace` would return to. |

Moving a window to an inactive special workspace creates it on the focused
monitor. If it is already active on another monitor, the window goes there
and the special workspace stays put until you summon it.

## IPC

Workspaces carry an `isSpecial` flag in query responses and events.

## Overview

The overview shows the special workspace on every monitor, set apart from
the other workspaces. See [Overview](./overview.md#special-workspace).
