# Animation perf log (`perf/lean-animations`)

Every number measured on this branch. Compare back-to-back runs only;
treat < ~20% as noise.

## Machine

| | |
|---|---|
| CPU | AMD Ryzen 7 3700X (8C/16T) |
| GPU | NVIDIA GeForce RTX 3070 (dGPU), driver 32.0.16.1692 |
| RAM | 16 GB |
| Display | 3440x1440 @ 175 Hz (frame budget 5.71 ms), single monitor |
| Windows | 11 Pro 25H2, build 26200.9457 |

The old notes (`git show 1388e2f0^:RESIZE-PERF-NOTES.md`) were taken on a
laptop; their absolute numbers are not comparable to these.

## Method

- WM: `GLAZEWM_PERF=1 glazewm.exe start --config ~/.glzr/glazewm/bench-<X>.yaml`.
- Driver: `cargo run -p wm-cli --release --example perf_bench -- --scenario <s> --target chrome --bursts 10 --label <l>`.
  Each burst is one profiler report; the table shows medians over bursts.
- Windows: 4 tiled windows on one workspace (Helium/chrome = pinned
  target, Notepad, Explorer, Windows Terminal).
- Scenarios (all reversible, no process spawning):
  - `resize`: target `resize --width +25%` / `-25%` (grow + neighbour shrink).
  - `float`: target `toggle-floating --centered` / `toggle-tiling`
    (neighbours grow / shrink; stands in for close/open).
  - `relayout`: workspace `toggle-tiling-direction` + `wm-redraw` (every
    window moves and resizes in both dimensions).
  - `move`: target `move --direction left` / `right` (pure translation).
- Configs: `resources/assets/sample-config.yaml` with the owner's dcomp
  `window_effects` (borders on all windows, 90% transparency, backdrop on
  focused + other windows), `startup_commands: []`, 150 ms move/resize.
  - A: `overlay_tracking: all`
  - B: `overlay_tracking: focused_only`
  - C: `overlay_tracking: none`

Columns: frames per burst; tick ms/frame (WM-thread cost); frame-time
p90; frame interval p50 (real pacing); `session_overlays`, `batch_commit`,
`ovl_region` ms/frame; `dwm_flush` calls inside frames; `dwm_flush`,
`cloak`, `session_begin` ms outside frames (the relayout before the first
tick); input -> first animation started; input -> end of first frame.

DWM-side frame timing: `DwmGetCompositionTimingInfo` is stubbed on this
build (counters stay ~0 across a second), so it cannot be used. Needs
PresentMon/ETW; not captured yet.

Relayout latency is timed from the `wm-redraw` message, i.e. after the
(instant) direction toggle.

## Phase 0: baseline (parent branch code + profiler, 2026-10-09)

Memory 40% committed, GPU idle. A run first and last to check drift.

| run | scenario | frames | tick | p90 | interval | sess_ovl | batch | ovl_region | flush# | flush out | cloak out | begin out | ->start | ->1st frame |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| base-A | resize | 29 | 2.20 | 9.30 | 11.43 | 0.65 | 0.80 | 0.54 | 2 | 26.7 | 24.2 | 33.7 | 0.17 | 72.4 |
| base-A | float | 29 | 2.27 | 8.80 | 11.43 | 0.60 | 0.71 | 0.52 | 2 | 26.8 | 24.7 | 35.4 | 0.17 | 77.2 |
| base-A | relayout | 31 | 2.37 | 8.71 | 11.43 | 0.67 | 0.80 | 0.60 | 2 | 25.8 | 19.1 | 31.8 | 0.09 | 52.7 |
| base-A | move | 31 | 0.64 | 0.97 | 11.43 | 0.02 | 0.23 | 0.00 | 1 | 33.5 | 30.2 | 5.2 | 0.18 | 38.9 |
| base-B | resize | 29 | 1.60 | 7.48 | 11.43 | 0.19 | 0.52 | 0.18 | 2 | 25.4 | 22.5 | 32.7 | 0.17 | 71.3 |
| base-B | float | 29 | 1.57 | 7.37 | 11.43 | 0.14 | 0.55 | 0.12 | 2 | 26.0 | 23.1 | 36.1 | 0.18 | 67.9 |
| base-B | relayout | 31 | 1.79 | 7.35 | 11.43 | 0.19 | 0.54 | 0.19 | 2 | 25.9 | 20.7 | 29.0 | 0.10 | 52.9 |
| base-B | move | 31 | 0.63 | 0.82 | 11.43 | 0.02 | 0.20 | 0.00 | 1 | 33.7 | 30.5 | 6.6 | 0.19 | 40.1 |
| base-C | resize | 29 | 1.46 | 5.58 | 11.43 | 0.00 | 0.46 | 0.03 | 2 | 28.5 | 24.4 | 34.4 | 0.18 | 67.4 |
| base-C | float | 29 | 1.40 | 7.39 | 11.43 | 0.00 | 0.47 | 0.02 | 2 | 25.9 | 27.5 | 29.7 | 0.21 | 67.8 |
| base-C | relayout | 31 | 1.57 | 6.09 | 11.43 | 0.00 | 0.51 | 0.03 | 2 | 26.4 | 20.8 | 31.5 | 0.11 | 52.6 |
| base-C | move | 31 | 0.58 | 0.56 | 11.43 | 0.00 | 0.12 | 0.00 | 1 | 33.5 | 30.2 | 5.1 | 0.18 | 39.1 |
| base-A (again) | resize | 29 | 2.14 | 9.40 | 11.43 | 0.67 | 0.68 | 0.58 | 2 | 30.0 | 27.5 | 34.2 | 0.18 | 76.9 |
| base-A (again) | float | 29 | 2.11 | 8.41 | 11.43 | 0.62 | 0.68 | 0.53 | 2 | 28.1 | 24.5 | 25.1 | 0.18 | 68.6 |
| base-A (again) | relayout | 31 | 2.48 | 8.93 | 11.43 | 0.66 | 0.74 | 0.58 | 2 | 26.8 | 20.7 | 30.2 | 0.10 | 57.4 |
| base-A (again) | move | 31 | 0.68 | 0.98 | 11.43 | 0.02 | 0.24 | 0.00 | 1 | 34.0 | 30.7 | 5.3 | 0.18 | 36.9 |

Reading:

- On this machine the WM thread is well inside budget: tick 2.2 ms (A)
  vs 1.5 ms (C) per frame. The laptop's 12-200 ms ticks do not reproduce
  here with 4 windows. Overlays (A vs C) cost ~0.7 ms/frame.
- Pacing, not cost, limited smoothness: every config ran at an 11.43 ms
  interval (87.5 fps on a 175 Hz panel), from `MAX_TICK_RATE_HZ = 120`
  dropping every second vblank.
- Keypress -> first motion is 53-77 ms (9-13 frames at 175 Hz) before
  anything moves. Per burst, outside frames: ~27 ms `DwmFlush`, ~20-30 ms
  cloak, ~30-35 ms `session_begin` on resizes. This is the Phase 3 target
  and likely the biggest "feel" gap to Hyprland.

## Tick cap removed (`MAX_TICK_RATE_HZ`)

Back to back, config A then C, 10 bursts each:

| run | scenario | frames | tick | p90 | interval | sess_ovl | batch | ovl_region | flush# | flush out | cloak out | begin out | ->start | ->1st frame |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| nocap-A | resize | 52 | 1.54 | 4.21 | 5.72 | 0.49 | 0.54 | 0.43 | 2 | 30.1 | 27.7 | 34.9 | 0.18 | 71.4 |
| nocap-A | float | 52 | 1.34 | 4.34 | 5.71 | 0.38 | 0.54 | 0.32 | 2 | 28.7 | 26.6 | 33.4 | 0.17 | 67.8 |
| nocap-A | relayout | 55 | 1.67 | 6.30 | 5.71 | 0.52 | 0.61 | 0.47 | 2 | 24.2 | 21.3 | 32.5 | 0.10 | 59.2 |
| nocap-A | move | 59 | 0.49 | 0.92 | 5.71 | 0.01 | 0.25 | 0.00 | 1 | 32.6 | 29.1 | 5.1 | 0.18 | 36.2 |
| base-A | resize | 29 | 2.21 | 9.30 | 11.43 | 0.68 | 0.71 | 0.58 | 2 | 27.1 | 23.7 | 32.2 | 0.17 | 67.5 |
| base-A | float | 29 | 2.01 | 9.27 | 11.43 | 0.53 | 0.68 | 0.46 | 2 | 26.7 | 26.1 | 34.7 | 0.18 | 78.4 |
| base-A | relayout | 31 | 2.41 | 9.13 | 11.43 | 0.67 | 0.79 | 0.59 | 2 | 28.4 | 21.3 | 33.3 | 0.10 | 57.9 |
| base-A | move | 31 | 0.65 | 0.94 | 11.43 | 0.02 | 0.24 | 0.00 | 1 | 33.7 | 30.5 | 4.9 | 0.18 | 37.2 |
| nocap-A | resize | 52 | 1.50 | 3.91 | 5.72 | 0.51 | 0.53 | 0.44 | 2 | 29.6 | 27.9 | 33.0 | 0.17 | 73.3 |
| nocap-A | float | 53 | 1.52 | 5.70 | 5.72 | 0.47 | 0.55 | 0.40 | 2 | 28.5 | 23.2 | 35.1 | 0.18 | 76.4 |
| nocap-A | relayout | 54 | 1.73 | 6.59 | 5.72 | 0.53 | 0.61 | 0.46 | 2 | 26.0 | 20.7 | 32.3 | 0.10 | 58.6 |
| nocap-A | move | 59 | 0.50 | 0.89 | 5.71 | 0.02 | 0.25 | 0.00 | 1 | 34.4 | 30.1 | 4.4 | 0.18 | 37.0 |
| nocap-C | resize | 55 | 0.93 | 3.15 | 5.71 | 0.00 | 0.41 | 0.01 | 2 | 25.7 | 22.7 | 32.7 | 0.18 | 63.6 |
| nocap-C | float | 55 | 0.93 | 2.72 | 5.71 | 0.00 | 0.42 | 0.01 | 2 | 23.9 | 21.6 | 33.0 | 0.18 | 66.0 |
| nocap-C | relayout | 57 | 0.91 | 2.45 | 5.72 | 0.00 | 0.38 | 0.01 | 2 | 25.3 | 21.0 | 31.8 | 0.10 | 56.5 |
| nocap-C | move | 58 | 0.36 | 0.43 | 5.71 | 0.00 | 0.11 | 0.00 | 1 | 33.8 | 30.4 | 5.3 | 0.18 | 39.4 |

Frames per burst 29 -> 52, interval 11.43 -> 5.72 ms (87.5 -> 175 fps),
frame p90 9.3 -> 4.2 ms on resize. Tick cost per frame went down, not up
(fixed per-burst work spread over more frames). Latency unchanged, as
expected. Kept. DWM-side confirmation that every frame is presented is
still pending (PresentMon).

## Animation clock anchored at the first tick, not the relayout

Found while reading the latency timeline: `WindowAnimationState` started
its clock on first evaluation, which is the relayout that creates it, not
the first tick. That relayout (session setup, pre-cloak `DwmFlush`, cloaks)
ran ~70 ms before the first tick, so the first frame that moved anything
was already deep into the 150 ms curve. Measured with a temporary
per-frame log of eased progress (config A, resize, 4 windows animating):

| | first moving frame, eased progress | next frames |
|---|---|---|
| before | 0.808 | 0.825, 0.856, 0.882 ... |
| clock started at first tick | 0.000 (start repeated), then 0.004 | 0.132, ... |
| ... minus one frame period (kept) | 0.011 | 0.132, 0.427, 0.532 ... |

So before this, every move/resize visibly jumped ~80% of the way and then
crawled through the tail. Frames per burst rose 52 -> 62-63 because the
whole curve is now rendered. The start is still uneven (0.132 -> 0.427
across one interval): the first ticks are slow (first `batch_commit`
8.6 ms), which is the next target.

## `window_resize.style: fill | stretch`

`fill` (default) is the existing behaviour: content at real size, gap
strips filled with the sampled edge color, real window resized late.
`stretch` registers the thumbnail with no source rect, so DWM always
draws the whole current window (whatever size the app has reached) scaled
into the animated rect; the real window gets its final size on the first
frame (Hyprland-style); no fill, no edge-color sampling. Checked why the
June stretch mode was removed (`2a51011b`): the commit gives no reason,
and that version scaled a fixed source rect, so it could crop or
oversample while the app resized. This one cannot.

Back to back, config A (`bench-A.yaml` vs `bench-AS.yaml`), two rounds,
10 bursts each:

| run | scenario | frames | tick | p90 | interval | sess_ovl | batch | ovl_region | flush# | flush out | cloak out | begin out | ->start | ->1st frame |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| fill | resize | 62 | 1.65 | 4.02 | 5.72 | 0.59 | 0.60 | 0.51 | 2 | 28.9 | 25.7 | 33.2 | 0.17 | 71.3 |
| fill | float | 63 | 1.65 | 4.66 | 5.71 | 0.54 | 0.63 | 0.48 | 2 | 34.1 | 29.1 | 33.6 | 0.17 | 74.1 |
| fill | relayout | 63 | 1.91 | 5.73 | 5.72 | 0.64 | 0.65 | 0.57 | 2 | 25.4 | 20.8 | 33.1 | 0.11 | 56.8 |
| fill | move | 63 | 0.71 | 1.38 | 5.71 | 0.02 | 0.46 | 0.00 | 1 | 33.7 | 30.2 | 5.4 | 0.20 | 36.9 |
| stretch | resize | 62 | 1.72 | 4.34 | 5.71 | 0.57 | 0.62 | 0.49 | 2 | 17.2 | 13.5 | 0.8 | 0.17 | 35.6 |
| stretch | float | 63 | 1.66 | 4.60 | 5.72 | 0.52 | 0.63 | 0.44 | 2 | 16.7 | 12.2 | 0.8 | 0.18 | 34.9 |
| stretch | relayout | 63 | 1.98 | 5.47 | 5.71 | 0.64 | 0.77 | 0.57 | 2 | 21.0 | 12.3 | 0.5 | 0.10 | 34.3 |
| stretch | move | 63 | 0.71 | 1.40 | 5.71 | 0.02 | 0.45 | 0.00 | 1 | 11.8 | 8.2 | 0.3 | 0.19 | 12.4 |
| fill | resize | 62 | 1.63 | 4.04 | 5.71 | 0.59 | 0.62 | 0.51 | 2 | 28.5 | 24.3 | 36.0 | 0.18 | 71.9 |
| fill | float | 63 | 1.60 | 4.66 | 5.71 | 0.54 | 0.64 | 0.47 | 2 | 29.3 | 27.6 | 35.9 | 0.17 | 77.5 |
| fill | relayout | 63 | 1.82 | 5.56 | 5.71 | 0.63 | 0.65 | 0.55 | 2 | 26.9 | 24.4 | 32.6 | 0.10 | 60.0 |
| fill | move | 64 | 0.70 | 1.37 | 5.71 | 0.02 | 0.46 | 0.00 | 1 | 33.5 | 30.1 | 6.3 | 0.19 | 36.3 |
| stretch | resize | 62 | 1.73 | 4.33 | 5.71 | 0.56 | 0.65 | 0.49 | 2 | 16.1 | 13.2 | 0.9 | 0.19 | 34.3 |
| stretch | float | 62 | 1.64 | 4.49 | 5.71 | 0.50 | 0.63 | 0.44 | 2 | 16.8 | 12.7 | 0.8 | 0.17 | 35.5 |
| stretch | relayout | 63 | 2.04 | 5.52 | 5.71 | 0.64 | 0.84 | 0.57 | 2 | 17.5 | 11.6 | 0.5 | 0.11 | 33.2 |
| stretch | move | 64 | 0.72 | 1.37 | 5.71 | 0.02 | 0.46 | 0.00 | 1 | 12.5 | 8.1 | 0.3 | 0.19 | 11.7 |

- Per-frame cost: identical within noise (tick 1.63-1.65 vs 1.72-1.73
  on resize; the extra destination update per frame is not visible).
- Start latency: halved. Input -> first frame resize 71 -> 35 ms,
  relayout 58 -> 34 ms, move 36 -> 12 ms. `session_begin` 33 -> 0.8 ms:
  in fill mode most of session setup is the gap fill's composition work
  (the fill is created/cleared per session, one blocking hop to the
  composition thread each), and the pre-cloak `DwmFlush` and cloaks get
  cheaper too (flush 29 -> 17 ms, cloak 25 -> 13 ms), presumably because
  DWM has less new composition state to absorb.
- Visual: one screenshot mid-resize at a 3 s duration showed no holes,
  black borders or visible distortion (apps already have their final
  size, so scale factors stay small). Flicker at start/end not judged;
  needs eyes on the real thing.

## High memory (87-88% committed)

Pressure from one process that commits and touches pages up to a target
(`VirtualAlloc` + write per 4 KB page, held while benching). GPU at the
time: 948 MB dedicated, 100 MB shared (RTX 3070, 8 GB). Same build as the
stretch runs, 10 bursts, two rounds:

| run | scenario | frames | tick | p90 | interval | ->1st frame |
|---|---|---|---|---|---|---|
| hi fill | resize | 62 | 1.60 | 3.93 | 5.71 | 72.6 |
| hi fill | float | 63 | 1.66 | 5.37 | 5.71 | 78.5 |
| hi fill | relayout | 63 | 1.81 | 5.79 | 5.72 | 56.9 |
| hi fill | move | 64 | 0.71 | 1.40 | 5.71 | 40.0 |
| hi stretch | resize | 62 | 1.70 | 4.25 | 5.71 | 33.6 |
| hi stretch | float | 62 | 1.80 | 5.16 | 5.72 | 37.2 |
| hi stretch | relayout | 63 | 2.05 | 5.51 | 5.72 | 33.8 |
| hi stretch | move | 63 | 0.69 | 1.36 | 5.71 | 12.2 |
| hi fill | resize | 62 | 1.60 | 3.94 | 5.72 | 71.7 |
| hi fill | float | 63 | 1.59 | 5.73 | 5.72 | 77.8 |
| hi fill | relayout | 63 | 1.84 | 5.85 | 5.72 | 59.1 |
| hi fill | move | 64 | 0.71 | 1.40 | 5.71 | 28.8 |
| hi stretch | resize | 62 | 1.69 | 4.19 | 5.72 | 35.5 |
| hi stretch | float | 63 | 1.64 | 4.42 | 5.71 | 33.1 |
| hi stretch | relayout | 63 | 1.97 | 5.14 | 5.72 | 32.7 |
| hi stretch | move | 64 | 0.71 | 1.37 | 5.71 | 11.4 |

Indistinguishable from the 40% runs on this machine. Either the slowness
under high RAM was app-side (paged-out apps repainting slowly, which this
benchmark's light targets don't show) or it needs VRAM pressure, which an
8 GB dGPU with ~1 GB used doesn't have. Not reproduced; worth re-testing
when it actually happens, with `GLAZEWM_PERF=1` running, and looking at
`rd_apply by process` / `pre_commit` vs `tick`/`batch_commit`.

## Why fill-mode session setup is slow

Temporary timing inside `ResizeSession::begin_impl` / `NativeSurrogate::revive`
(fill, config A, resize, 4 sessions per burst):

| per session, in burst order | 1st | 2nd | 3rd | 4th |
|---|---|---|---|---|
| surrogate revive | 0.5 ms | 4-7 ms | 16-33 ms | 13-17 ms |
| of which fill clear / corner / border attrs | ~0.03 ms each | | | |
| border inset, fill color | ~0.01 ms | | | |

The cost is the revive's `SetWindowPos` (to `HWND_TOP`) and thumbnail
update, growing with each surrogate DWM has to absorb. In stretch mode
the same calls take ~0.2 ms total per session; the difference is that a
fill-mode surrogate carries a `DesktopWindowTarget` composition tree (the
gap fill). So removing the fill (Phase 1) is what buys the latency, not
removing the edge sampling.
