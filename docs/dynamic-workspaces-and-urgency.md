# Dynamic workspaces & window urgency

Two independent features, both opt-in and both off by default.

- **Dynamic workspaces** — workspaces created on demand beyond the ones
  declared in `workspaces`, destroyed once empty.
- **Window urgency** — a flag on windows that asked for attention while in
  the background, readable over IPC so a status bar can surface it.

They're documented together because they share a motivation: the WM should
tell you what happened rather than reorganise your screen on its own.

---

## Dynamic workspaces

### Config

```yaml
general:
  dynamic_workspaces: true
```

With it enabled, a workspace can be activated under a name that isn't in
the `workspaces` config. It gets a synthesized config named with the lowest
unused positive integer, `keep_alive: false`, and no monitor binding — so
it's destroyed as soon as it's empty and no longer displayed, like any
other non-`keep_alive` workspace.

Configured workspaces are always preferred. With `1`–`9` declared, the
first dynamic workspace is `10`, and only once all nine are in use.

### Commands

| Command | Behaviour |
| --- | --- |
| `focus --next-empty-workspace` | Focus the first workspace with no windows. |
| `move --next-empty-workspace` | Move the focused window there, without following it. |

Target resolution, in order: an empty workspace on the current monitor →
an unused workspace config → an empty workspace on another monitor → a new
dynamic workspace (only if `dynamic_workspaces` is on). If none of those
apply, the command does nothing.

Focusing the next empty workspace while already on an empty one is a no-op.

### Interaction with existing commands

- `focus --next-workspace` / `--prev-workspace` cycle through configured
  workspaces *and* active dynamic ones, in that order.
- `focus --workspace <name>` creates the workspace if the name isn't
  configured and dynamic workspaces are enabled.
- Dynamic workspaces sort after configured ones, ordered by numeric name.
- A config reload leaves dynamic workspaces alone; without the feature, a
  workspace whose config disappeared would be reassigned to a free config.

---

## Window urgency

A window is *urgent* from the moment it asks for attention until it's
focused. The WM never acts on the flag itself — it reports, and the user
(or a status bar) decides.

### Two detectors

| Trigger | How it's detected | Typical cause |
| --- | --- | --- |
| The taskbar button flashes | `HSHELL_FLASH` shell hook | A chat app calling `FlashWindowEx`, or the OS refusing an app's foreground request |
| A background window wins the foreground | The focus event, when `ignore_focus_steal` is on | Opening a link so the browser raises itself |

The first fires whether or not `ignore_focus_steal` is enabled. The second
only exists because the WM is refusing to follow the window.

#### Why a shell hook

`SetWinEventHook`, which the WM uses for every other window event, has no
event for a window requesting attention. The only notification Windows
offers is the shell hook one that drives the taskbar's own flashing. The
event loop's message window is registered with `RegisterShellHookWindow`,
and `HSHELL_FLASH` messages arrive through a window-procedure callback
rather than the hook procedure used elsewhere.

`ChangeWindowMessageFilterEx` allows that message through UIPI, since the
WM is commonly run elevated while the shell is not.

### Config

```yaml
general:
  # Mark a window urgent instead of following it to its workspace.
  ignore_focus_steal: true

  # Stop the taskbar button flashing once the window is flagged.
  suppress_taskbar_flash: true
```

`ignore_focus_steal` also re-hides the window: the OS uncloaks a window as
part of giving it the foreground, so it's queued for redraw and focus is
handed back to the WM's focus target.

`suppress_taskbar_flash` is Windows-only. The flashing can't be prevented,
only ended — the notification *is* "flashing started" — so an auto-hidden
taskbar may still appear briefly before retracting. Leave it off until
something else surfaces urgency, or an urgent window has no visible cue at
all.

### Reading it

`isUrgent` appears on every window in `query windows` and on the windows
nested under `query workspaces`:

```jsonc
{ "type": "window", "processName": "Discord", "isUrgent": true, ... }
```

The `window_urgency_changed` event fires when a window becomes urgent and
when it's cleared:

```sh
glazewm-cli sub --events window_urgency_changed
```

```jsonc
{
  "eventType": "window_urgency_changed",
  "updatedWindow": { "type": "window", "isUrgent": true, ... },
  "workspaceName": "1"
}
```

`workspaceName` is provided because a window's `parentId` is its immediate
parent, which is a split container whenever the window is nested — so it
can't be used to group by workspace on its own.

Repeat alerts re-broadcast rather than being swallowed as "no change",
since a second chat message is a new alert. A window that flashes until
focused has the shell re-notify about once a second, so alerts are
debounced to one per window per three seconds. Clearing only broadcasts on
an actual change, as it runs on every focus change.

### Building a status bar widget

The whole contract is: subscribe, keep a set, clear on focus.

1. `sub --events window_urgency_changed workspace_activated workspace_deactivated`
2. On an urgency event, add or remove `workspaceName` from an urgent set
   based on `updatedWindow.isUrgent`.
3. Seed the set at startup from `query workspaces` by scanning descendants
   for `isUrgent`.

Dynamic workspaces need no widget changes as long as the workspace list is
rendered from `query workspaces` rather than a hardcoded range.

---

## Platform support

| | Windows | macOS |
| --- | --- | --- |
| Dynamic workspaces | yes | yes |
| Urgency via flashing | yes | never fires — no equivalent notification |
| Urgency via focus steal | yes | yes |
| `suppress_taskbar_flash` | yes | no-op |
