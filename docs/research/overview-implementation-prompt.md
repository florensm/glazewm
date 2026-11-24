# Prompt: build the new overview options into GlazeWM

A hand-off for Claude Code, run in this repo on the desktop PC. Start it with:

> Read `docs/research/overview-implementation-prompt.md` and carry it out,
> phase by phase. Ask me before anything it says to ask about.

Everything below is the prompt.

---

You're implementing a set of overview options that were designed and tried
out in browser mock-ups. The design work is done and agreed; this is the
build. Work in phases, each one complete (code, tests, docs, sample config)
and committed before the next. Plan each phase before writing code, and
review it after.

## 1. Read first

In this order:

1. `CLAUDE.md`: code style, comments, error handling, endpoint safety and
   **build targets**. Follow it exactly.
2. `docs/agents/adr/0001-overview-uses-dwm-thumbnails-only.md`: **windows
   are shown with DWM thumbnails only.** No `Windows.Graphics.Capture`, no
   screen grabs, no undocumented DWM calls. Nothing in this prompt needs
   them. 3D looks are faked with layout (position, size, opacity, order).
3. `docs/features/overview.md`: what the overview does today, every option.
   You'll update it in every phase.
4. `docs/research/overview-ux.md`: why these changes, with sources. Read
   "GlazeWM today, measured", "Why it zooms out too much",
   "What people find useful" and "Recommendations".
5. `docs/research/overview-perf.md`: the performance changes (phase 6).
6. `docs/research/overview-ux/config.html`: **the spec.** Open it in a
   browser. Its `TABS` array lists every option this prompt adds, with its
   YAML key, type, range, default and meaning; its preview shows what each
   does. Options without `isNew` exist already: don't rename them. When this
   prompt and the builder disagree, ask.

Other mock-ups, for feel only: `playground.html` (zoom and easing),
`lab.html` (search modes, hints, peek, hold to pick, minimized shelf),
`layouts.html` (cluster vs rows vs grid), `experiments3.html` (zoom levels).
Their JS (`stage.js`, `layouts.js`) is a sketch, not code to port: the real
layout code is Rust in `wm-platform/src/overview/`.

## 2. Branches: ask before touching any

`CLAUDE.md` says to develop on `tryout/<name>` and cut `test/<name>` from
it. For this feature the newest overview code is on
`origin/test/overview-config`, 37 commits ahead of
`origin/tryout/overview-config`, an older, diverged copy whose 6 own
commits are superseded: ignore them. Test branches lack the benchmark and
automation files `CLAUDE.md` says test builds strip; as `CLAUDE.md` says,
those live on `origin/tryout/one-surface`.

The user sets the branch up (or tells you to). The steps:

```bash
git fetch origin test/overview-config tryout/one-surface
git checkout -b tryout/overview-styling origin/test/overview-config
git checkout origin/tryout/one-surface -- packages/wm-cli/examples/perf_bench packages/wm-platform/examples/uncloak_all.rs resources/scripts/package.ps1
```

Then put back `wm-cli`'s `[dev-dependencies]` and
`[target.'cfg(windows)'.dev-dependencies]` tables in
`packages/wm-cli/Cargo.toml`, copied from the same file on
`origin/tryout/one-surface` (only those two tables; keep the rest of this
branch's file), run `cargo build` to update `Cargo.lock`, and
`cargo test --workspace`. `perf_bench` was written against the
one-surface branch: if it no longer builds here, stop and ask rather than
change it. Commit as `chore: restore benchmarks for the tryout branch`.

Develop on `tryout/overview-styling`. Cut `test/overview-styling` from it
per `CLAUDE.md` when the user wants a laptop build. Never push to a test
branch except by cutting it. Commit at the end of each phase; push only
when the user asks.

## 3. Ground rules for this work

- **No new Windows APIs.** Everything here is layout, input handling and
  config on top of what the overview already calls. If you find you need
  one, stop and ask: it must be called out for Defender (see `CLAUDE.md`).
- **Defaults keep today's behaviour.** Every new option's default is what
  the overview does now (the builder's `def` values). The proposed values go
  in the sample config and the user's own config, and a separate final
  commit changes defaults only if the user agrees (phase 7).
- **Config:** types and parsing in `packages/wm-common/src/parsed_config.rs`
  (the overview structs there and in
  `packages/wm-platform/src/overview/model.rs`); add a parse test per new
  option, including the bad-value case. Values must survive a config reload.
- **Docs:** every option you add goes into the right table in
  `docs/features/overview.md`, worded like the rows around it, and into
  `resources/assets/sample-config/overview.yaml` only if the sample sets it.
- **Tests:** unit tests next to the code, as `expose.rs` and `layout.rs`
  have. Pure layout maths must be tested, not eyeballed. Run
  `cargo test --workspace` and `cargo clippy --workspace` every phase.
- **Keys:** typing searches, so new commands go on keys that type nothing or
  on `Alt` chords. `Alt`+`H`/`J`/`K`/`L`, `Alt`+digits and `Alt`+`-` are
  taken, and `Tab` switches layouts. A user's own keybindings win (see
  "Your own keybindings" in the feature doc).
- **Endpoint safety:** verify by hand on the desktop with a couple of real
  windows; no screenshot bursts, no synthesized input loops. Profile only
  with `GLAZEWM_PERF` on the desktop, never on the laptop.

## 4. Where things are

- `packages/wm-platform/src/overview/expose.rs`: the exposé cluster
  (`spread`, `LARGEST = 0.85`).
- `.../overview/layout.rs`: card metrics, the activities row and scene
  (`activities`, margins 48 px, `ROW_HEIGHT`, `COMPACT_ROW_HEIGHT`,
  `SceneChrome`), the stack (`stack`, `STACK_MARGIN`, `STACK_GAP = 28`),
  zoom timing, `CHROME_FROM = 0.8`, springs.
- `.../overview/native/scenes.rs`: spreading a scene's windows,
  `spread_windows`, search matches spread alone, `ripple_delays`.
- `.../overview/native/screen.rs`: per-monitor screen: thumbnails
  registered (`register`), placed each frame (`place`), `frame_padding`.
- `.../overview/native/mod.rs`: the overview thread, `tick`, open and close.
- `.../overview/state.rs`: selection, keyboard, search (`query`, matching).
- `.../overview/chrome.rs`: drawn pictures: names, search box, labels.
- `.../overview/native/pacing.rs`: frame timing, cards drawn per frame.
- `packages/wm/src/overview/`: the WM side (`frame` builds what it shows,
  `actions` carries out picks). Keybindings reach the overview through the
  WM's keyboard hook (`wm.rs`, see "How it works" in the feature doc).
- `packages/wm-common/src/parsed_config.rs`: `OverviewConfig` and friends,
  `SpringParams` (default ζ 1, k 800, ε 1e-4).

Grep rather than trust line numbers.

## 5. Phases

Each phase: plan, build, test, update docs and sample config, review,
commit. Show the user the plan before phases 2, 4 and 5.

### Phase 1: sizes and motion as options

Today these are constants. Make them options, defaults unchanged:

| Key | Type, range | Default | Replaces |
| --- | --- | --- | --- |
| `expose.largest` | 0.6 to 1.0 | 0.85 | `expose.rs` `LARGEST` |
| `expose.margin` | px, 8 to 120 | 48 | the 48 px margin in `layout.rs` `activities` for the exposé |
| `activities.margin` | px, 8 to 120 | 48 | the same margin, for the activities |
| `animation.chrome_from` | 0 to 0.95 | 0.8 | `layout.rs` `CHROME_FROM` |
| `animation.open_easing` | easing, optional | unset: `animation.easing` | the open uses it; the close keeps `animation.easing` |

Also, with no option: **settle the zoom on half a physical pixel** instead
of ε = 1e-4 of the way (`overview-perf.md` §1): for a spring zoom, end when
the largest remaining distance of anything moving is under 0.5 px. Test that
the last frame lands exactly on target, and that a `spring(1, 800)` zoom of
600 px ends near 335 ms instead of 415 ms.

### Phase 2: the workspace row on any edge

| Key | Values | Default |
| --- | --- | --- |
| `workspace_row` | `compact`, `shown`, `hidden` | `compact` (exists; applies to exposé and activities) |
| `expose.workspace_row` | `inherit`, `compact`, `shown`, `hidden` | `inherit` |
| `expose.workspace_row_position` | `top`, `bottom`, `left`, `right` | `top` |
| `activities.workspace_row_position` | same | `top` |

Bottom: the row sits above the search line, labels under the miniatures.
Left and right: miniatures in a column, labels under each, the scene takes
the rest of the width. Everything that assumes a top row must follow:
`SceneChrome`, the scene rect, the search box and spotlight positions,
arrow-key navigation between the row and the windows (with the row on the
left, `←` from the windows' left edge goes to the row), the zoom's
open/close (the row fades in from its own edge), the **+** card, multiple
monitors, and the special workspace's card. Test each position's rects.

### Phase 3: the exposé's windows

| Key | Values | Default | Meaning |
| --- | --- | --- | --- |
| `expose.arrange` | `cluster`, `grid`, `rows` | `cluster` | Cluster as today. Grid: even cells in reading order, each window fit to its cell. Rows: GNOME's (`UnalignedLayoutStrategy`, ported in `docs/research/overview-ux/layouts.js` `rows`, with its constants and the 1.5× bump for short windows; cite GNOME's file in a comment). |
| `expose.small_window_boost` | 0 to 0.6 | 0 | Cluster only: before clustering, grow each window about its centre by `1 + boost × (1 − min(1, area / (half the work area)))`, then shrink each result back to its true aspect inside its slot. Small windows grow relative to big ones; positions are kept. |
| `expose.readable_floor` | 0 to 0.7 | 0 (off) | If the arrangement's smallest window scale is under this, use rows instead. |
| `expose.titles` | `hover`, `always` | `hover` | Titles on every window, not just the selected one. |
| `expose.app_icons` | bool | false | The app's icon large in the middle of each window, as minimized windows already show it. |
| `expose.minimized` | `inline`, `shelf` | `inline` | `shelf`: minimized windows in a row of small previews along the bottom of the room, under a "Minimized" label, the rest arranged above. Still pickable, searchable, draggable. |

Cluster, grid and rows must all keep the invariants `expose.rs` tests:
within the room, at most their own size, a stack's tabs apart, centred.

### Phase 4: search and keys in the exposé

| Key | Values | Default | Meaning |
| --- | --- | --- | --- |
| `expose.search_mode` | `spread`, `dim`, `shelf` | `spread` | `spread` as today. `dim`: nothing moves; non-matches fade to a quarter where they are (the switching studies favour keeping positions). `shelf`: matches gather in the room, non-matches go to the bottom shelf (shared with `expose.minimized: shelf`). |
| `expose.hints` | `off`, `alt` | `off` | While `Alt` is held, a letter on every window, in reading order, matches first; `Alt`+letter picks it. Letters from `asdfgqwertyuiopzxcvbnm` (no `h j k l`). |
| `expose.peek_key` | `none`, `ctrl` | `none` | Holding it enlarges the selected window to about 84% of the screen, the rest dimmed; letting go puts it back. |
| `expose.release_picks` | bool | false | If the overview was opened by a keybinding whose modifier is still held, letting go of that modifier picks the selected window, once the selection has moved (like Alt+Tab). Needs the WM's keyboard hook to tell the overview when the modifier is released. |

Ask the user how hints and `release_picks` should behave when both use
`Alt`. Keep the existing search behaviour (letters in order, Enter, Esc,
Backspace) in every mode.

### Phase 5: activities and stack

| Key | Values | Default | Meaning |
| --- | --- | --- | --- |
| `stack.direction` | `vertical`, `horizontal` | `vertical` | Horizontal: workspaces side by side; wheel and `←`/`→` move along. Update arrow handling and monitor crossing to match. |
| `stack.gap` | px, 0 to 200 | 28 | `STACK_GAP`. |
| `stack.names` | `beside`, `above`, `below`, `hidden` | `beside` | Where each workspace's name goes. |
| `stack.neighbour_opacity` | 0.2 to 1 | 1 | Opacity of workspaces other than the selected one. |
| `stack.neighbour_scale` | 0.7 to 1 | 1 | Their scale relative to the selected one. |
| `stack.open_style` | `inherit`, `zoom`, `fade`, `none`, `cascade` | `inherit` | The stack's own open/close. `cascade`: each workspace starts its zoom a step after the one nearer the selected (about 0.16 of the way per step, up to 3 steps). |

The stack's glide between workspaces already follows `animation.switch_ms`
and `animation.bounce`; check it does, and say so in the doc.

### Phase 6: lighter (from `overview-perf.md`)

In order; measure each before and after with `GLAZEWM_PERF` on the desktop
(time to first frame, frame times, missed frames), and report the numbers:

1. Register thumbnails in stages on open: the shown workspace's previews
   before the first frame, the row, cards and other scenes over the next
   frames, as `DrawBudget` does for drawing.
2. Fewer surfaces in motion: the border and backdrop overlay thumbnails fade
   in at rest instead of moving with the zoom. Ask the user to look at it.
3. Keep drawn card pictures between opens; redraw only those whose content
   changed.
4. Only if measurements show it: the framed backdrop scaled rather than
   resized each frame (`composition.rs` `set_layout`).

### Phase 7: defaults and the user's config

Ask the user whether to make the proposed values the defaults:
`expose.largest: 0.94`, `expose.margin: 32`, `wallpaper.padding: 16`,
`expose.gap: 24`, `animation.open_easing: "spring(0.85, 520)"`,
`animation.chrome_from: 0.6`, `effects.window_lift: 0.04`,
`effects.dim: 0.12`, `effects.stagger_ms: 10`. A full-screen window then
ends at about 0.80× instead of 0.69× (see the research). Either way, set the
sample config to the proposed values, and offer to write this to the user's
config (their "Bottom thumbnails" setup from the builder):

```yaml
overview:
  workspace_row: "shown"
  expose:
    workspace_row_position: "bottom"
    gap: 24
    largest: 0.94
    margin: 32
    search_mode: "shelf"
  activities:
    workspace_row_position: "bottom"
  wallpaper:
    padding: 16
  animation:
    open_easing: "spring(0.85, 520)"
    chrome_from: 0.6
  effects:
    window_lift: 0.04
    dim: 0.12
    stagger_ms: 10
```

### Later, only if asked

**Zoom levels** (`experiments3.html` #1): the wheel zooms continuously from
the desktop, to the exposé, to every workspace side by side. The user liked
it; propose a design for it after phase 5, don't build it unasked.

## 6. Out of scope

The other experiments (flick ring, carousel, gravity search, timeline,
pinned shelf, sound badges and so on), anything using screen capture or 3D
transforms of windows (ADR-0001), launching apps from the search (the user
has their own plan), and YASB integration (another repo).

## 7. Each phase's report

End each phase with: what changed (files), the options added with their
defaults, tests added and their result, `cargo clippy` clean, what to look
at by hand and how, anything left or uncertain, and the commit hash.
