# GlazeWM

A tiling window manager for Windows, with an overview of every workspace and window. Animation terms (stand-in, peek, surrogate, hand-over, hand-back, companion) are defined at the top of `packages/wm/src/animation/mod.rs` and aren't repeated here.

## Overview

**Exposé**:
The overview layout that spreads the shown workspace's windows out so each can be seen and picked, minimized windows and every tab of a stack included.
_Avoid_: Mission Control, spread view

**Placeholder**:
What the overview shows in place of a minimized window, which has nothing to show of its own: a gray, slightly see-through window with its app's icon and title.
_Avoid_: Stand-in (that means any drawing of a window in place of the real one), ghost, tile
