# Overview presets

Ready-made looks for the overview (`toggle-overview`). Each file is a whole
`overview:` section, to use in place of your `overview.yaml`.

To try one, copy it next to your `config.yaml` and point `include` at it
instead of `overview.yaml`, then reload the config:

```yaml
include:
  - "mission-control.yaml"   # was "overview.yaml"
```

Only one can be included at a time, as no setting may be set twice.
Anything a preset leaves out keeps its default (see `docs/overview.md`).

| Preset | Look |
| --- | --- |
| `mission-control.yaml` | Nothing but the workspace: windows at their own sizes on a rounded slab of wallpaper, like macOS. |
| `edge-to-edge.yaml` | Exposé in an even grid with as little around the windows as can be, so each shows largest. |
| `frosted.yaml` | Activities over a light blur, with a glow around the picked window. |
| `accent-frame.yaml` | Activities framed in your Windows accent color. |
| `niri-column.yaml` | Every workspace stacked top to bottom, over a heavy blur, fading in. |
| `paper.yaml` | A light overview for light-mode desktops. |
| `flat-and-fast.yaml` | No rounding or shadow, and nothing animates. |
| `playful.yaml` | Bouncy springs, confetti, embers and a breathing glow. |
