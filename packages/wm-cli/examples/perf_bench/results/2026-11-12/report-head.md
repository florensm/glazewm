# Peek drawing benchmark: tryout/stand-ins vs origin/main

Desktop PC, 2026-11-12, run unattended 21:05–{{END}}.

## Bottom line

Based mainly on the controlled runs (all four builds interleaved, fixed
tile order, 4 rounds, with and without CPU load). The requested runs
agree, with more noise.

- **Smoother where peeks draw**: `resize` in `default`/`morepeeks`, and
  `float` in `morepeeks` only. Frame p90 roughly halves (7–12 → 4–5.5 ms).
  Interval p90 drops from 8–11 ms (1.4–1.9 vblanks) to 6–7 ms. On
  `resize` this is beyond noise against `main` in every run set. On
  `float` the old engine varies a lot between rounds, so `morepeeks` has
  the lower median everywhere but separates in 11 of 16 comparisons.
- **Not faster.** On `move`, `resize` and (`morepeeks`) `float`, peek
  modes answer the command that starts an animation 30–64% later than
  `nopeeks` and 54–91% later than `main`. On `resize` that is 130–160 ms vs 90–114 ms vs 76–85 ms; on `move`,
  24–26 vs 18–19 vs 16–17 ms. They also use 15–55% more WM time per frame
  (`tick`), plus `peek_frame` (3.1–3.8 ms/frame on `resize`/`float`).
- **One long stall instead of several.** A peek-mode `resize` burst has
  a single 74–95 ms gap between frames. In that frame the real windows
  are repositioned synchronously while peeks are up. The old engine has
  several 40–65 ms gaps. The single gap is probably the hand-over at the
  end of the motion rather than mid-motion, but that needs DWM capture
  to confirm.
- **`move` and `minimize`: little to choose.** `move` runs at the
  refresh rate in every mode, and peeks add 6–10 ms before it starts.
  On `minimize` the old engine in the new build (`nopeeks`) is rougher
  than `main` in 3 of 4 run sets (frame p90 +45–55%), and peeks bring
  it back to `main`'s level.
- **`switch` doesn't use peeks.** The tryout build halves `main`'s frame
  p90 and its longest stall (~45 vs 90–100 ms), but answers 46–75%
  later.
- **The tryout build is slower to answer even with peeks off.**
  `nopeeks` takes 17–75% longer than `main` on `move`, `resize` and
  `switch`, and on `float` at load 0, while being as smooth or
  smoother. That cost is separate from peek drawing and worth a look.
- `-CpuLoad 8` adds up to ~25% to command times (mostly 5–20%, in
  every build) and changes no verdict.

## Setup

- **Machine**: Ryzen 7 3700X (8C/16T), RTX 3070, one 3440×1440 monitor
  at 174 Hz (5.75 ms frame budget), Windows 11 Pro. Display sleep is off
  on AC, so the monitor stayed on throughout.
- **Builds** (release, profiler on via `GLAZEWM_PERF=1`):
  - `main`: `origin/main` `cfd3cef9`, built in `..\glazewm-baseline`.
  - `nopeeks` / `default` / `morepeeks`: `tryout/stand-ins` `65a60de3`
    with `GLAZEWM_ISOLATE` = `nopeeks` / unset / `morepeeks`.
  - Both binaries contain the frame profiler. Every run of both builds
    produced one report per burst (9–11 per run, see raw files).
- **Windows**: whatever was open, 7 tiles on workspace 1: Windows
  Terminal, chrome (Helium, the target), Notepad, Everything, Explorer,
  and 2× ApplicationFrameHost (Settings, Microsoft Store). The UWP
  frames, Explorer, Everything and Notepad repeatedly block synchronous
  repositions (8–800 ms, `errors.log`). That cost lands in every mode,
  and it is why commands here take ~50–150 ms instead of ~15–30 ms with
  3 windows.
- **Scenarios**: `move`, `float`, `minimize`, `resize`, `switch`, 10
  bursts each, 1.2 s apart, 3 rounds. `window_minimize` is off in your
  config, so `minimize` measures the neighbours re-tiling around chrome.
  `switch` moved every second tile to workspace 2 and switched 1↔2.
- **Tile order flips on every restart.** The WM is restarted per mode
  and round, and each restart flips the tiles between two orders: chrome
  leftmost, or rightmost. Chrome's neighbours, and so the workload,
  depend on that. In the requested runs this is the main noise source.
  The expansion runs fix the order after each restart. See "Side
  finding" near the end.
- **CPU load**: `-CpuLoad 8` = 8 busy-loop jobs on a 16-thread CPU. That
  only half-loads it, so the effect is modest (commands mostly +5–20%).

### Deviations from the instructions

1. **perf_bench for `main`.** The tryout's `WorkspaceDto` has two
   required fields (`isSpecial`, `layout`) that `main` doesn't send.
   The tryout-built perf_bench therefore can't parse `main`'s workspace
   queries, which breaks `restore-layout` and `switch`. For `main` I
   copied the tryout's perf_bench source unchanged (`diff -r` clean) into
   the baseline worktree and built it there, against `main`'s IPC types.
   I also added `serde_yaml` to that worktree's `wm-cli` dev-dependencies.
   The measurement and report parsing code is identical.
2. **Config for `main`.** `main` has no `include:`, so it would have
   silently run with default animations and effects. It ran with a
   single-file translation of your config: same animation durations,
   easings and thresholds, gaps, borders, 90% transparency, backdrops,
   square corners, rules and workspaces. Keys `main` doesn't have were
   dropped: `scroll`, `focus_change`, `particles`, border
   `highlight`/`rim`/`glow`, `special_workspace`, `layouts`. Startup and
   shutdown commands were emptied. The tryout build therefore also runs
   the 150 ms `focus_change` fade, which `main` lacks.
3. **Baseline watcher.** I also built `glazewm-watcher` in the baseline,
   so it starts the way the tryout build does.
4. **Paths.** `C:\Users\FMN` doesn't exist on this PC, so everything is
   under `C:\Users\googlers` (repo `C:\Users\googlers\glazewm`, this
   report in `C:\Users\googlers\.glzr\glazewm`).
5. **GlazeWM wasn't running** when I started (20:59). The script needs a
   running WM to restore afterwards, so I first started the tryout build
   the usual way (`target\release\glazewm.exe start`, no env vars). It
   is running that way now, with all windows on workspace 1.
6. **Old results kept.** The earlier `isolate-bench.txt` (a 20:48 run
   of `nopeeks`/`default`, resize+move) is saved as
   `isolate-bench.prev-2026-11-12T2048.txt` next to this report.
7. The repo had two untracked files (`config.yaml`, `wallpaper-probe.ps1`,
   from October) and no tracked changes. Neither file is a build input,
   so I went ahead.
8. **Step 4 ran my own script.** The `main` runs used my copy of
   `isolate_modes.ps1`'s loop, with a per-mode exe, perf_bench and
   config. It records failures and keeps going, and logs the tile order.
   It is in the scratch folder (`bench-run.ps1`). Nothing in the repo
   changed, nothing was committed.
9. **Expansion.** After the requested runs I ran all four builds
   interleaved: first with tile order as restarted (the "Side finding"
   run), then twice with a fixed tile order, with and without CPU load.
   A CPU-load interleaved run without fixed order was stopped after a
   minute, once the order flip showed up. Stopping it also killed its WM,
   so I restarted yours and restored the layout by hand within seconds.
10. **Layout at the end.** `restore-layout` puts every window back on
    its workspace and refocuses the workspace that had focus. It doesn't
    restore tile order within a workspace, and neither does
    `isolate_modes.ps1`. All 7 windows are on workspace 1, as before, in
    whatever order the last restart produced.

### Reading the numbers

- Cells are the **median over rounds**, with the round range (min–max)
  in small print. Each round's value is perf_bench's own median over
  its ~10 burst reports. Lower is better, except interval p50, which is
  the refresh period (5.71–5.72 ms everywhere: every mode ticks once per
  vblank).
- **interval p90** is the smoothness number: 5.75 means even frames at
  174 Hz, and ~9–11 ms means roughly every tenth frame missed a vblank.
- **input->first frame** is only recorded on the old drawing path. Only
  that path calls `perf::note_animation_start`
  (`animation/manager.rs:2691`), so peek-drawn animations and workspace
  switches never pair an input with a frame. Those cells are **n/a**,
  not zero. **command p50** is the closest stand-in: the time until the
  WM answers the command, after its sync has run. Where both are
  measured, the first frame follows the answer by 1–25 ms (50–75 ms in
  `minimize`).
- **peek_frame** is not nested in `tick`, and it includes the wait for
  the WM thread. It is not a cost to add to `tick`; it reads 0 where no
  peeks drew.
- **tick ms/frame** is a mean, so the few long first frames of each
  burst dominate it. **frames >2x budget** (>11.5 ms) is 0–2 per burst
  in `move` and 2–8 in the other scenarios, in every mode. These are
  likely those same start-up frames.
- **Interval tails** (the tables after each results set) come straight
  from `perf.log`, mapped to each run by time. They are the longest gaps
  between WM frames per burst. A peek-mode `resize` burst has a single
  74–95 ms gap (p99 = max), against several 40–65 ms gaps for the old
  engine. In that gap's frame the WM repositions the real windows
  synchronously while peeks are up (`peek_frame` > `platform_sync` >
  `rd_apply`, then `uncloak`), blocked on slow apps. The `uncloak`
  suggests it is the hand-over from peeks to real windows at the end of
  the motion, but the profiler doesn't place it in time. `perf_bench
  --dwm` would show what DWM displayed, but it needs an elevated shell,
  which this session isn't. I didn't create an elevated scheduled task
  without asking.
- **Noise rule**: a difference counts only if the two modes' round
  ranges don't overlap and it is ≥5%. With 3 rounds against 3, separation
  by pure chance happens 10% of the time, so some flags among ~100
  comparisons are noise. The check: `morepeeks` vs `default` runs
  identical code outside `float`. 3 of its 40 comparisons there were
  flagged in the requested runs (9–20%), and 1 of 40 in the controlled
  runs (4 rounds, under 3% expected).
- `main` ran in its own block after the three tryout modes, as the
  instructions asked, so drift over time could bias `main` vs the rest.
  The interleaved runs below check this.
