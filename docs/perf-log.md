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
build (counters stay ~0 across a second), so it cannot be used. Captured
through PresentMon instead with `perf_bench --dwm`; see "DWM side".

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
the same calls take ~0.2 ms total per session.

**Correction (see "Edge sampling deferred to idle" below):** the first
explanation written here blamed the fill's composition tree. Wrong:
forcing a hidden, or visible but empty, fill tree onto stretch surrogates
changed nothing. The cause is the edge-color screen sampling, which the
fill mode started for every session.

## Phase 2.1 tried: pinned borders for move/resize (rejected)

Border overlays of move/resize sessions pinned to the monitor work area
for the session (ring moved by composition offset), reusing the
workspace-switch `pin_or_slide`. Window set from here on: Helium,
3x File Pilot, Windows Terminal, Settings (6 tiled). Back to back, two
rounds:

| config | build | tick | p90 | sess_ovl | ovl_region | ->1st frame (resize / relayout / move) |
|---|---|---|---|---|---|---|
| A (fill) | tracked | 2.00 | 4.4 | 0.73 | 0.66 | 93 / 80-84 / 32-33 |
| A (fill) | pinned | 1.60 | 3.0-3.4 | 0.05 | 0.05 | 101-106 / 89 / 33-39 |
| A (stretch) | tracked | 2.0-2.2 | 4.4-4.8 | 0.68-0.75 | 0.62-0.69 | 31-33 / 26-28 / 12-14 |
| A (stretch) | pinned | 1.6 | 2.4-3.4 | 0.04 | 0.05 | 44-47 / 42 / 15-16 |

Per frame it does what it should (tick -20%, p90 -30%, overlay stages
gone), but pinning costs at the start: the relayout's
`session_overlays` grows 7 -> 20 ms (each border window is hidden,
resized to the monitor and re-shown), against a first tick that only
gets 4 ms faster. Net +12 ms before motion, which on this machine
matters more than 0.4 ms/frame. Misses the gate (>= 25% tick drop, no
regressions); reverted. Worth revisiting for the laptop, where
per-frame cost dominated, ideally with the pin made at rest so it is not
on the keypress path.

Phase 2.2/2.3 (backdrop and ring drawn inside the surrogate) rested on a
worry that composition content on the surrogate is what made fill slow.
Tested: a hidden, and a visible but empty, `SurrogateFill` tree forced
onto stretch surrogates changed nothing (resize 31-35 ms either way). So
that worry is cleared.

## Edge sampling deferred to idle

The real cause of fill's slow start: every session started
`sample_edge_color_async`, two GPU->CPU `BitBlt` screen readbacks on a
background thread, right at animation start. They stall DWM, so every
DWM call the WM makes next (surrogate setup, flush, cloak) waits. With
sampling simply switched off (experiment), fill matched stretch.

Kept the sampling (its color is only used by the *next* session anyway)
but queued per window and run once animations are idle, reading the
window's current rect and skipping hidden/cloaked/minimized windows.
Verified the samples still land (24 colors over 4 bursts of 6 windows).

| run | scenario | ->1st frame | flush out | cloak out | begin out | tick |
|---|---|---|---|---|---|---|
| fill before | resize | 93-96 | 26-32 | 26-29 | 51-53 | 2.0 |
| fill before | relayout | 79 | 30-32 | 23-24 | 47-49 | 2.3 |
| fill before | move | 29-32 | 22-25 | 19-22 | 5.5-5.9 | 0.6 |
| fill deferred | resize | 24-29 | 17-19 | 13-14 | 1.9 | 2.0-2.1 |
| fill deferred | relayout | 21 | 18-19 | 13-14 | 1.2-1.3 | 2.3 |
| fill deferred | move | 12-14 | 11-12 | 7-9 | 0.5 | 0.6 |
| stretch deferred | resize | 32 | 16-21 | 15 | 1.3 | 2.0-2.1 |
| stretch deferred | relayout | 27-28 | 17-20 | 12-13 | 0.7-1.3 | 2.5-3.0 |
| stretch deferred | move | 13 | 12 | 8-9 | 0.3-0.5 | 0.6-0.8 |

Fill start latency -70% (resize 95 -> 26 ms, relayout 79 -> 21 ms). Fill
and stretch now start equally fast; the remaining difference between
them is only how the content looks mid-resize.

Unrelated, seen in both builds and both modes: the `float` scenario
sometimes runs ~270 frames per burst instead of ~63 (an animation or
settle keeps ticking ~1.5 s). Not investigated yet.

## Profiler fix: outside-frame section double counted

`roll_up_frame` left the per-frame accumulators set, so the next
`start_frame` added the previous frame's stages to the outside-frame
section too. Every "flush out" / "cloak out" / "begin out" column above
therefore includes in-frame calls of the same stage. `session_begin`
only runs in the relayout, so its column is right; `dwm_flush` and
`cloak` are inflated by a few in-frame calls. The latency, tick and
timeline numbers were never affected, and every comparison was between
builds with the same instrumentation, so the conclusions stand.

## Phase 3 tried: overlay re-anchoring batched into one transaction (rejected)

The relayout re-stacks every visible border/backdrop window behind its
new surrogate with its own blocking `SetWindowPos`. Tried queuing those
restacks into the pass's `DeferWindowPos` batch instead. Back to back,
two rounds, input -> first frame:

| config | before | batched |
|---|---|---|
| fill resize / relayout / move | 32 / 20 / 12 ms | 32 / 19 / 11-13 ms |
| stretch resize / relayout / move | 32-34 / 27 / 13 ms | 32-33 / 27-29 / 12-13 ms |

The pre-cloak `DwmFlush` got shorter (outside-frame flush 17-21 -> 4-8
ms) by exactly what the rest of the path got longer: DWM absorbs a cost
per window touched, however the calls are grouped (the old notes'
finding again). Reverted. The pre-cloak flush deferral (Phase 3) would
most likely move time the same way, so it was not attempted.

## Phase 2.2/2.3 ceiling, and the border region dropped during animations

2.2/2.3 would hide the overlay windows for a session and draw backdrop
and ring inside the surrogate. Config C (`overlay_tracking: none`) hides
them too, so C minus A is the most 2.2/2.3 can win. Measured first,
6 windows, back to back:

| run | scenario | tick | p90 | sess_ovl | batch | ovl_region | ->1st frame |
|---|---|---|---|---|---|---|---|
| A (fill) | resize / relayout | 1.98 / 2.32 | 4.28 / 4.69 | 0.73 / 0.77 | 0.59 / 0.62 | 0.66 / 0.69 | 32 / 21 |
| C (fill) | resize / relayout | 1.25 / 1.33 | 1.43 / 1.40 | 0 / 0 | 0.35 / 0.35 | 0.03 / 0.04 | 26 / 19 |
| AS (stretch) | resize / relayout | 2.02 / 2.51 | 4.36 / 5.37 | 0.70 / 0.90 | 0.57 / 0.76 | 0.61 / 0.80 | 34 / 28 |
| CS (stretch) | resize / relayout | 1.31 / 1.36 | 1.57 / 1.66 | 0 / 0 | 0.34 / 0.34 | 0.02 / 0.03 | 29 / 23 |

Hiding the overlays costs nothing at the start (latency slightly lower),
unlike the pin in 2.1. But almost all of `sess_ovl` is `ovl_region`: the
border's `SetWindowRgn`, reshaped on every resize frame. That region is
for hit-testing only (keeps the overlay out of `WindowFromPoint`), and
behind a surrogate it changes nothing, since the surrogate covers the
hole and is just as unanswering.

Kept: the border drops its region while anchored behind a surrogate
(detected by window class, cached per anchor) and restores it on the
first placement behind anything else. Covers all three session-tracking
paths (relayout, close, fade tail) without caller changes. Checked with a
3 s resize: all 6 visible borders region-less mid-animation, complex
region again after.

| run | scenario | tick | p90 | sess_ovl | batch | ovl_region | ->1st frame |
|---|---|---|---|---|---|---|---|
| A before | resize / relayout | 1.97 / 2.15 | 4.41 / 4.66 | 0.74 / 0.74 | 0.60 / 0.61 | 0.63 / 0.66 | 32 / 21 |
| A after | resize / relayout | 1.55 / 1.65 | 2.59 / 2.74 | 0.03 / 0.03 | 0.74 / 0.69 | 0.02 / 0.05 | 28 / 20 |
| AS before | resize / relayout | 2.20 / 2.51 | 4.78 / 5.16 | 0.77 / 0.84 | 0.58 / 0.68 | 0.70 / 0.74 | 34 / 26 |
| AS after | resize / relayout | 1.60 / 1.83 | 3.06 / 3.18 | 0.03 / 0.03 | 0.75 / 0.81 | 0.03 / 0.04 | 31 / 27 |

Tick -21 to -27%, p90 -35 to -41%, start unchanged or better; move
unaffected (a translation never reshaped the region). Part of the saving
reappears in `batch_commit` (+0.15 ms): DWM absorbs some of the shape
change at commit instead.

What is left for 2.2/2.3 is the gap to C: ~0.3 ms tick and p90 ~2.7 ->
~1.5 ms, i.e. two overlay windows per session in the batch. Not started:

- 2.3 (ring) needs the surrogate outset by the border width to have room
  for the ring. DWM's corner rounding then applies to the outer edge, and
  the thumbnail inside is (as far as known, not verified) square-cornered,
  since rounding is why the surrogate gets a corner preference at all.
  Composition content draws under the thumbnail, so the ring cannot mask
  those corners.
- 2.2 (backdrop) has no such problem but needs one `DesktopWindowTarget`
  per surrogate shared by fill and backdrop, and a hand-over to the real
  backdrop window before the surrogate's fade-out. Alone worth ~0.15-0.2
  ms/frame, under the 25% gate.

## Phase 3 leftovers: fade-out, z-settle, `float` run-on

Of a burst's 63 ticks, 28 move windows (`platform_sync`); the other 35
are the tail (100 ms `SESSION_FADE_OUT`, then the 200 ms overlay
z-settle), ~15 ms in all, after motion has ended. Shortening the fade or
throttling the settle would only idle the timer sooner; frame pacing and
start latency are unaffected. Not changed. The fade length is a visual
choice (shadow/late-repaint blend), not a cost.

`float` run-on: not reproduced in 24 bursts (fill and stretch, 63-64
frames each). The old run-on reports (09:38-09:42) show 272 ticks of
which only 55 ran `platform_sync`, the rest cleanup-only: a z-settle
extended to its 2 s `OVERLAY_Z_SETTLE_MAX` cap. Floating windows are
`shown_on_top` in the bench configs, so the likely trigger is a band
change (`is_topmost` vs `shown_on_top`) that never landed, keeping the
settle alive. Cheap (cleanup ticks ~0.1 ms) and bounded by the cap. The
settle now logs a warning, once, when it reaches the cap, naming the
window, process and band mismatch, so the next occurrence explains itself.

## DWM side (PresentMon)

`perf_bench --dwm` tracks `dwm.exe` through the PresentMon service's API
(PresentMon 2.6 installed; the service needs no elevation, unlike the
console app's own ETW session). Per burst it takes the longest run of
displayed DWM frames (split at gaps > 60 ms, which drops idle-desktop
presents) and reports missed vblanks (frames shown for more than one
refresh), the display interval p90 and DWM's GPU busy p90. The run
includes the fade tail, ~45 vblanks in all.

Chrome target, medians of 10 bursts; last three columns DWM-side:

| config | scenario | tick | p90 | ->1st frame | missed vblanks | disp int p90 | DWM GPU p90 |
|---|---|---|---|---|---|---|---|
| A | resize | 1.69 | 2.93 | 31 | 23 | 17.2 | 6.7 |
| A | relayout | 1.84 | 3.18 | 21 | 19 | 11.5 | 2.1 |
| A | move | 0.94 | 1.41 | 14 | 9 | 11.4 | 3.4 |
| C | resize | 1.30 | 1.41 | 28 | 15 | 11.5 | 5.8 |
| C | relayout | 1.43 | 1.76 | 19 | 15 | 11.4 | 1.5 |
| T (transparency only) | resize | 1.14 | 2.31 | 21 | 14 | 11.5 | 5.0 |
| T | relayout | 1.07 | 1.60 | 15 | 16 | 11.4 | 2.7 |
| T | move | 0.35 | 0.57 | 10 | 8 | 11.4 | 4.8 |
| P (no effects) | resize | 0.85 | 1.49 | 23 | 13 | 11.4 | 3.6 |
| P | relayout | 0.95 | 1.45 | 14 | 8 | 11.4 | 0.5 |
| P | move | 0.31 | 0.56 | 8 | 6 | 5.7 | 1.9 |

(C move came out at 19 / 22.9 / 14.7, an outlier against every other
move run; left out. P ticks ~81 frames per burst instead of ~63, not
looked into.)

- The WM ticks every vblank, but DWM does not show a new frame every
  vblank. With all effects (A) a resize burst loses ~23 of ~45 vblanks;
  even with none (P), 6-13.
- DWM's GPU work is what runs over: A resize p90 6.7 ms against a 5.7 ms
  budget. Per-burst captures (Terminal as target, which is harsher) put
  DWM's GPU busy at 6-8 ms median in the slow bursts, which then run at
  1/2 or 1/3 rate for the whole animation, against ~1.4 ms in good ones.
  Over 2 rounds each, bursts at half rate or worse: P 0/10, T 0/10,
  C 3-5/10, A 8/11. Transparency on every window is the largest DWM
  cost (everything under each window must be composed); the tracked
  overlays add to it (C vs A), which is the DWM-side case for 2.2/2.3.
- Two stalls in every burst regardless of effects, seen by lining up a
  temporary per-tick WM trace (QPC-stamped) with the DWM presents:
  2-4 vblanks at the start (the known session setup), and 5-6 vblanks
  (~25-30 ms) right after the hand-back uncloaks the real windows,
  while the WM's ticks cost 0.5 ms. DWM takes in the uncloaked windows,
  and the last motion frame lands late. The uncloak fires once the
  motion completes, and two frames before the end the default curve
  (`cubic_bezier(0.2, 0, 0, 1)`) is already at 99.8%, under 2 px from
  the final rect, so the stall holds back a finished picture rather than
  motion. Not worth changing.
- Not settled: even P spends DWM GPU p90 3.6 ms on a resize vs 1.9 ms on
  a move. Which part of composition costs what (thumbnails, rounded
  corners, geometry clips, the backdrop's layers) needs a GPU profiler
  on DWM (GPUView/PIX), which needs elevation. Drawing backdrop or ring
  inside the surrogate (2.2/2.3) keeps the same pixels to blend, so if
  the cost is overdraw rather than window count, it would win little on
  the DWM side; unproven either way.

## Where things stand (6 windows, borders + backdrop on all)

- Pacing: frame interval p50 5.71 ms, p90 5.74 ms; one ~10 ms interval
  per burst (the slow first frame misses one vblank).
- Tick ~1.6 ms of a 5.7 ms budget on resize/relayout (was ~2 ms before
  the border region was dropped during animations), p90 ~2.6-3.2 ms.
- Input -> first frame: fill 20-32 ms, stretch 27-34 ms, move ~12 ms
  (from 55-77 ms at baseline, with the first frame then at 81% progress).
- What remains of the start is DWM absorbing per-window changes
  (surrogate setup, cloaks, overlay restacks, first commit); grouping
  calls differently does not reduce it, only fewer windows does.

## Status against the plan (handoff, 2026-10-09)

Plan text: the `perf/lean-animations` plan (Phases 0-4, DComp spike).
Branch `perf/lean-animations` off local `dcomp` (2 commits ahead of
`origin/feat/dcomp`). Machine notes and method at the top of this file.

| Item | Status |
|---|---|
| Phase 0 baseline, bench tool, latency/interval/timeline profiling | Done |
| Phase 0 DWM-side timing (PresentMon) | Done: `perf_bench --dwm`, through the PresentMon service (no elevation) |
| Outside the plan: 120 Hz tick cap | Removed (87.5 -> 175 fps) |
| Outside the plan: animation clock started in the relayout | Fixed (first frame 0.81 -> ~0.01 progress) |
| Outside the plan: edge sampling at session start | Moved to idle (fill start 95 -> 26 ms) |
| Phase 1 stretch | Done as `animations.window_resize.style: fill \| stretch` |
| Phase 1 remove fill + edge sampling | Skipped by owner's decision; fill stays an option |
| Phase 2.1 pinned borders for move/resize | Tried, rejected (+12 ms start for -0.4 ms/frame) |
| Phase 2.2/2.3 ceiling measured (config C) | Done: tick -37%, no start cost |
| Outside the plan: border hit-test region dropped while behind a surrogate | Done (tick -21 to -27%, p90 -35 to -41%); most of the 2.2/2.3 ceiling |
| Phase 2.2 backdrop drawn in the surrogate | Not started; ~0.15-0.2 ms/frame left, under the gate alone |
| Phase 2.3 ring drawn in the surrogate | Not started; blocked on square thumbnail corners in an outset surrogate (unverified) |
| Phase 3 pre-cloak `DwmFlush` deferral, cloak staggering | Not started; batching overlay restacks showed time only moves |
| Phase 3 shorter `SESSION_FADE_OUT`, throttled z-settle | Measured, not changed: tail is ~15 ms of cheap ticks after motion ends |
| `float` run-on | Not reproduced (24 bursts); z-settle cap now logged with cause |
| Phase 3 overlay restacks batched into one transaction | Tried, rejected (no latency change) |
| Phase 4 / DComp spike | Not started. PresentMon now shows DWM's GPU time, not window count or the WM, limits pacing with all effects; documented composition APIs cannot host another window's content (thumbnails are `HWND`-based), so a spike needs an owner decision on undocumented DWM APIs |
| DWM hand-back stall (~25-30 ms after uncloak) | Measured, not changed: lands on a finished frame (<2 px left) |

Known issues: `float` bench scenario intermittently ticks ~1.5 s instead
of ~0.35 s (not reproduced under logging); toggling a workspace's tiling
direction queues no redraw (pre-existing, bench works around it).

Suggested next order: re-run the bench with `--dwm` on the laptop,
where per-frame cost was the original problem; a GPU profile of DWM
(GPUView/PIX, elevated) during an A resize to find what its 6-8 ms is
spent on; then decide on 2.2/2.3 and the DComp spike from that.

Tooling for the next session: bench configs `~/.glzr/glazewm/bench-A.yaml`
(fill), `bench-AS.yaml` (stretch), `bench-B/C.yaml` (overlay tracking), `bench-CS.yaml` (stretch, no
tracking), `bench-SLOW.yaml` (3 s resize, for mid-animation checks),
`bench-T.yaml` (transparency only), `bench-P.yaml` (no effects); add
`--dwm` to the bench for DWM-side columns;
run `GLAZEWM_PERF=1 glazewm.exe start --config <cfg>`, then
`cargo run -p wm-cli --release --example perf_bench -- --scenario <resize|float|relayout|move> --target chrome --bursts 10 --label <x>`.
Only one WM instance at a time (a second start pops a fatal-error dialog).
