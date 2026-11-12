# Peek drawing benchmark: tryout/stand-ins vs origin/main

Desktop PC, 2026-11-12, run unattended 21:05–22:33.

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

# Results: the requested runs

Tryout modes from `isolate_modes.ps1`, then `main` in its own block.

## No CPU load

Median over rounds (up to 3), round range in small print. Lower is better, except interval p50, which is the refresh period.

### move

| metric | main | nopeeks | default | morepeeks |
| --- | ---: | ---: | ---: | ---: |
| interval p50 ms | 5.71 <sub>5.71–5.71</sub> | 5.71 <sub>5.71–5.71</sub> | 5.71 <sub>5.71–5.71</sub> | 5.71 <sub>5.71–5.71</sub> |
| interval p90 ms | 5.75 <sub>5.74–5.75</sub> | 5.76 <sub>5.75–5.76</sub> | 5.77 <sub>5.76–5.77</sub> | 5.77 <sub>5.76–5.77</sub> |
| frame p90 ms | 0.99 <sub>0.98–1.11</sub> | 1.00 <sub>0.99–1.22</sub> | 1.25 <sub>0.91–1.50</sub> | 1.05 <sub>0.90–1.27</sub> |
| frames >2x budget | 0.00 <sub>0.00–2.00</sub> | 1.00 <sub>0.00–1.00</sub> | 1.00 <sub>1.00–2.00</sub> | 1.00 <sub>0.00–1.00</sub> |
| tick ms/frame | 0.65 <sub>0.62–1.24</sub> | 0.82 <sub>0.61–1.09</sub> | 1.25 <sub>0.86–1.52</sub> | 0.96 <sub>0.88–1.22</sub> |
| peek_frame ms/frame | n/a | n/a | 0.51 <sub>0.38–0.61</sub> | 0.42 <sub>0.38–0.52</sub> |
| input->first frame ms | 16.94 <sub>16.47–45.78</sub> | 28.18 <sub>18.76–50.15</sub> | n/a | n/a |
| command p50 ms | 16.22 <sub>15.64–35.81</sub> | 23.95 <sub>18.23–42.35</sub> | 37.30 <sub>22.79–40.78</sub> | 23.99 <sub>23.48–38.14</sub> |

| vs (ranges must not overlap) | default vs main | default vs nopeeks | morepeeks vs main | morepeeks vs nopeeks | morepeeks vs default | nopeeks vs main |
| --- | --- | --- | --- | --- | --- | --- |
| interval p90 ms | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |
| frame p90 ms | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |
| frames >2x budget | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |
| tick ms/frame | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |
| input->first frame ms | n/a | n/a | n/a | n/a | n/a | ≈ |
| command p50 ms | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |

### float

| metric | main | nopeeks | default | morepeeks |
| --- | ---: | ---: | ---: | ---: |
| interval p50 ms | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.72–5.72</sub> |
| interval p90 ms | 10.47 <sub>10.25–10.79</sub> | 9.18 <sub>9.01–10.18</sub> | 9.64 <sub>9.52–10.86</sub> | 6.40 <sub>6.33–6.48</sub> |
| frame p90 ms | 10.18 <sub>10.12–10.71</sub> | 8.93 <sub>8.55–9.39</sub> | 9.44 <sub>6.30–10.56</sub> | 4.38 <sub>4.16–4.89</sub> |
| frames >2x budget | 5.00 <sub>5.00–6.00</sub> | 4.00 <sub>3.00–4.00</sub> | 4.00 <sub>4.00–5.00</sub> | 5.00 <sub>5.00–5.00</sub> |
| tick ms/frame | 3.87 <sub>3.66–3.91</sub> | 3.61 <sub>3.40–3.68</sub> | 4.15 <sub>3.90–4.94</sub> | 5.06 <sub>4.95–5.66</sub> |
| peek_frame ms/frame | n/a | n/a | 0.00 <sub>0.00–0.00</sub> | 3.32 <sub>3.28–3.51</sub> |
| input->first frame ms | 97.94 <sub>97.35–99.84</sub> | 119.26 <sub>119.23–121.69</sub> | n/a | n/a |
| command p50 ms | 79.30 <sub>77.25–81.82</sub> | 102.17 <sub>99.40–113.47</sub> | 99.28 <sub>92.17–111.12</sub> | 133.88 <sub>133.00–143.97</sub> |

| vs (ranges must not overlap) | default vs main | default vs nopeeks | morepeeks vs main | morepeeks vs nopeeks | morepeeks vs default | nopeeks vs main |
| --- | --- | --- | --- | --- | --- | --- |
| interval p90 ms | ≈ | ≈ | better (-39%) | better (-30%) | better (-34%) | better (-12%) |
| frame p90 ms | ≈ | ≈ | better (-57%) | better (-51%) | better (-54%) | better (-12%) |
| frames >2x budget | ≈ | ≈ | ≈ | worse (+25%) | ≈ | better (-20%) |
| tick ms/frame | ≈ | worse (+15%) | worse (+31%) | worse (+40%) | worse (+22%) | ≈ |
| input->first frame ms | n/a | n/a | n/a | n/a | n/a | worse (+22%) |
| command p50 ms | worse (+25%) | ≈ | worse (+69%) | worse (+31%) | worse (+35%) | worse (+29%) |

### minimize

| metric | main | nopeeks | default | morepeeks |
| --- | ---: | ---: | ---: | ---: |
| interval p50 ms | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.72–5.72</sub> | 5.71 <sub>5.71–5.72</sub> |
| interval p90 ms | 5.78 <sub>5.77–5.80</sub> | 6.22 <sub>5.98–6.37</sub> | 5.77 <sub>5.77–5.78</sub> | 5.79 <sub>5.78–5.80</sub> |
| frame p90 ms | 3.86 <sub>3.05–4.45</sub> | 5.66 <sub>4.65–5.73</sub> | 4.02 <sub>3.67–4.05</sub> | 3.76 <sub>3.56–3.90</sub> |
| frames >2x budget | 6.00 <sub>5.00–6.00</sub> | 5.00 <sub>5.00–5.00</sub> | 5.00 <sub>5.00–5.00</sub> | 6.00 <sub>6.00–7.00</sub> |
| tick ms/frame | 1.67 <sub>1.61–1.71</sub> | 1.56 <sub>1.40–1.75</sub> | 1.76 <sub>1.73–2.26</sub> | 2.21 <sub>1.79–2.53</sub> |
| peek_frame ms/frame | n/a | n/a | 1.20 <sub>1.18–1.20</sub> | 1.24 <sub>1.19–1.32</sub> |
| input->first frame ms | 99.00 <sub>97.50–126.44</sub> | 117.13 <sub>114.08–128.89</sub> | n/a | n/a |
| command p50 ms | 46.38 <sub>44.83–52.82</sub> | 62.12 <sub>59.82–63.57</sub> | 62.36 <sub>40.46–66.23</sub> | 68.08 <sub>67.99–68.68</sub> |

| vs (ranges must not overlap) | default vs main | default vs nopeeks | morepeeks vs main | morepeeks vs nopeeks | morepeeks vs default | nopeeks vs main |
| --- | --- | --- | --- | --- | --- | --- |
| interval p90 ms | ≈ | better (-7%) | ≈ | better (-7%) | ≈ | worse (+8%) |
| frame p90 ms | ≈ | better (-29%) | ≈ | better (-34%) | ≈ | worse (+47%) |
| frames >2x budget | ≈ | ≈ | ≈ | worse (+20%) | worse (+20%) | ≈ |
| tick ms/frame | worse (+5%) | ≈ | worse (+32%) | worse (+42%) | ≈ | ≈ |
| input->first frame ms | n/a | n/a | n/a | n/a | n/a | ≈ |
| command p50 ms | ≈ | ≈ | worse (+47%) | worse (+10%) | worse (+9%) | worse (+34%) |

### resize

| metric | main | nopeeks | default | morepeeks |
| --- | ---: | ---: | ---: | ---: |
| interval p50 ms | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.71–5.72</sub> | 5.71 <sub>5.71–5.72</sub> |
| interval p90 ms | 10.22 <sub>9.24–10.44</sub> | 9.24 <sub>9.10–10.66</sub> | 6.77 <sub>5.81–6.96</sub> | 5.98 <sub>5.90–6.34</sub> |
| frame p90 ms | 10.12 <sub>9.10–10.27</sub> | 8.80 <sub>8.14–9.83</sub> | 4.40 <sub>3.92–4.92</sub> | 4.41 <sub>4.24–4.42</sub> |
| frames >2x budget | 5.00 <sub>4.00–5.00</sub> | 4.00 <sub>4.00–4.00</sub> | 5.00 <sub>5.00–5.00</sub> | 5.00 <sub>5.00–5.00</sub> |
| tick ms/frame | 4.14 <sub>3.93–4.31</sub> | 4.05 <sub>3.36–4.15</sub> | 5.62 <sub>5.17–5.93</sub> | 5.18 <sub>5.06–5.41</sub> |
| peek_frame ms/frame | n/a | n/a | 3.37 <sub>3.34–3.42</sub> | 3.36 <sub>3.35–3.40</sub> |
| input->first frame ms | 96.06 <sub>82.04–99.55</sub> | 117.57 <sub>114.07–117.91</sub> | n/a | n/a |
| command p50 ms | 75.03 <sub>73.29–76.37</sub> | 103.26 <sub>97.10–107.89</sub> | 134.21 <sub>133.39–135.46</sub> | 141.68 <sub>125.53–141.77</sub> |

| vs (ranges must not overlap) | default vs main | default vs nopeeks | morepeeks vs main | morepeeks vs nopeeks | morepeeks vs default | nopeeks vs main |
| --- | --- | --- | --- | --- | --- | --- |
| interval p90 ms | better (-34%) | better (-27%) | better (-41%) | better (-35%) | ≈ | ≈ |
| frame p90 ms | better (-57%) | better (-50%) | better (-56%) | better (-50%) | ≈ | ≈ |
| frames >2x budget | ≈ | worse (+25%) | ≈ | worse (+25%) | ≈ | ≈ |
| tick ms/frame | worse (+36%) | worse (+39%) | worse (+25%) | worse (+28%) | ≈ | ≈ |
| input->first frame ms | n/a | n/a | n/a | n/a | n/a | worse (+22%) |
| command p50 ms | worse (+79%) | worse (+30%) | worse (+89%) | worse (+37%) | ≈ | worse (+38%) |

### switch

| metric | main | nopeeks | default | morepeeks |
| --- | ---: | ---: | ---: | ---: |
| interval p50 ms | 5.72 <sub>5.71–5.72</sub> | 5.71 <sub>5.71–5.72</sub> | 5.72 <sub>5.71–5.72</sub> | 5.71 <sub>5.71–5.72</sub> |
| interval p90 ms | 5.75 <sub>5.75–5.76</sub> | 5.76 <sub>5.76–5.79</sub> | 5.77 <sub>5.77–5.78</sub> | 5.77 <sub>5.76–5.77</sub> |
| frame p90 ms | 3.52 <sub>3.34–3.54</sub> | 1.90 <sub>1.84–1.92</sub> | 1.75 <sub>1.69–2.03</sub> | 1.94 <sub>1.55–2.27</sub> |
| frames >2x budget | 3.00 <sub>2.00–3.00</sub> | 3.00 <sub>3.00–3.00</sub> | 3.00 <sub>3.00–4.00</sub> | 4.00 <sub>3.00–4.00</sub> |
| tick ms/frame | 2.50 <sub>2.22–2.51</sub> | 1.95 <sub>1.93–2.05</sub> | 1.98 <sub>1.68–2.09</sub> | 1.83 <sub>1.65–2.17</sub> |
| peek_frame ms/frame | n/a | n/a | 0.00 <sub>0.00–0.00</sub> | 0.00 <sub>0.00–0.00</sub> |
| input->first frame ms | n/a | n/a | n/a | n/a |
| command p50 ms | 50.11 <sub>28.64–51.16</sub> | 66.86 <sub>55.89–77.90</sub> | 68.97 <sub>51.18–80.76</sub> | 70.26 <sub>58.22–71.60</sub> |

| vs (ranges must not overlap) | default vs main | default vs nopeeks | morepeeks vs main | morepeeks vs nopeeks | morepeeks vs default | nopeeks vs main |
| --- | --- | --- | --- | --- | --- | --- |
| interval p90 ms | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |
| frame p90 ms | better (-50%) | ≈ | better (-45%) | ≈ | ≈ | better (-46%) |
| frames >2x budget | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |
| tick ms/frame | better (-21%) | ≈ | better (-27%) | ≈ | ≈ | better (-22%) |
| input->first frame ms | n/a | n/a | n/a | n/a | n/a | n/a |
| command p50 ms | worse (+38%) | ≈ | worse (+40%) | ≈ | ≈ | worse (+33%) |

#### No CPU load: frame interval tails

From `perf.log`: per run, the median over its burst reports of interval p99 and max (ms). Then median over rounds, with the rounds' range. 5.75 ms is one vblank, ~11.5 ms one missed.

| scenario | main p99 | main max | nopeeks p99 | nopeeks max | default p99 | default max | morepeeks p99 | morepeeks max |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| move | 10.7 <sub>9.9–38.1</sub> | 10.7 <sub>9.9–38.1</sub> | 15.7 <sub>11.9–37.4</sub> | 15.7 <sub>11.9–37.4</sub> | 35.6 <sub>12.6–42.1</sub> | 35.6 <sub>12.6–42.1</sub> | 13.1 <sub>12.1–30.5</sub> | 13.1 <sub>12.1–30.5</sub> |
| float | 58.2 <sub>55.9–61.1</sub> | 58.2 <sub>55.9–61.1</sub> | 60.7 <sub>53.9–62.0</sub> | 60.7 <sub>53.9–62.0</sub> | 63.6 <sub>56.7–65.1</sub> | 63.6 <sub>56.7–65.1</sub> | 74.2 <sub>72.7–86.7</sub> | 74.2 <sub>74.2–86.7</sub> |
| minimize | 41.6 <sub>40.1–44.9</sub> | 53.7 <sub>49.0–54.3</sub> | 29.0 <sub>25.2–33.6</sub> | 52.0 <sub>51.4–56.2</sub> | 36.1 <sub>32.7–42.9</sub> | 54.7 <sub>54.5–62.4</sub> | 39.0 <sub>32.3–43.2</sub> | 58.2 <sub>56.7–64.1</sub> |
| resize | 48.8 <sub>48.0–49.6</sub> | 51.4 <sub>49.8–54.8</sub> | 40.1 <sub>39.4–61.9</sub> | 61.9 <sub>57.8–67.9</sub> | 88.8 <sub>79.6–94.2</sub> | 88.8 <sub>79.6–94.2</sub> | 82.3 <sub>77.2–84.7</sub> | 82.3 <sub>77.2–84.7</sub> |
| switch | 83.9 <sub>83.3–90.2</sub> | 94.0 <sub>90.2–95.3</sub> | 45.6 <sub>44.0–46.7</sub> | 45.6 <sub>44.0–46.7</sub> | 44.5 <sub>39.8–49.2</sub> | 44.5 <sub>39.8–49.2</sub> | 39.4 <sub>37.9–47.2</sub> | 47.2 <sub>39.4–48.2</sub> |

## With CPU load (-CpuLoad 8)

Median over rounds (up to 3), round range in small print. Lower is better, except interval p50, which is the refresh period.

### move

| metric | main | nopeeks | default | morepeeks |
| --- | ---: | ---: | ---: | ---: |
| interval p50 ms | 5.71 <sub>5.71–5.72</sub> | 5.71 <sub>5.71–5.72</sub> | 5.71 <sub>5.71–5.71</sub> | 5.71 <sub>5.71–5.71</sub> |
| interval p90 ms | 5.76 <sub>5.75–5.77</sub> | 5.75 <sub>5.74–5.76</sub> | 5.75 <sub>5.75–5.77</sub> | 5.76 <sub>5.76–5.77</sub> |
| frame p90 ms | 1.16 <sub>1.10–1.22</sub> | 1.18 <sub>1.08–1.19</sub> | 1.01 <sub>0.99–1.53</sub> | 1.37 <sub>0.63–1.53</sub> |
| frames >2x budget | 1.00 <sub>0.00–1.00</sub> | 1.00 <sub>0.00–1.00</sub> | 1.00 <sub>1.00–1.00</sub> | 1.00 <sub>1.00–1.00</sub> |
| tick ms/frame | 0.79 <sub>0.74–1.38</sub> | 0.64 <sub>0.61–1.23</sub> | 1.01 <sub>0.97–1.38</sub> | 1.16 <sub>0.92–1.18</sub> |
| peek_frame ms/frame | n/a | n/a | 0.47 <sub>0.41–0.61</sub> | 0.55 <sub>0.46–0.58</sub> |
| input->first frame ms | 17.91 <sub>17.24–49.80</sub> | 20.44 <sub>19.69–50.14</sub> | n/a | n/a |
| command p50 ms | 17.08 <sub>16.42–39.48</sub> | 19.73 <sub>18.99–38.58</sub> | 24.84 <sub>24.47–41.74</sub> | 28.59 <sub>24.62–28.82</sub> |

| vs (ranges must not overlap) | default vs main | default vs nopeeks | morepeeks vs main | morepeeks vs nopeeks | morepeeks vs default | nopeeks vs main |
| --- | --- | --- | --- | --- | --- | --- |
| interval p90 ms | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |
| frame p90 ms | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |
| frames >2x budget | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |
| tick ms/frame | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |
| input->first frame ms | n/a | n/a | n/a | n/a | n/a | ≈ |
| command p50 ms | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |

### float

| metric | main | nopeeks | default | morepeeks |
| --- | ---: | ---: | ---: | ---: |
| interval p50 ms | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.72–5.72</sub> | 5.71 <sub>5.71–5.71</sub> |
| interval p90 ms | 11.07 <sub>11.06–11.53</sub> | 10.17 <sub>10.02–10.58</sub> | 10.78 <sub>9.62–11.18</sub> | 7.03 <sub>6.92–8.27</sub> |
| frame p90 ms | 8.87 <sub>7.96–11.27</sub> | 9.92 <sub>9.57–10.49</sub> | 10.02 <sub>8.77–10.92</sub> | 5.52 <sub>4.84–5.99</sub> |
| frames >2x budget | 6.00 <sub>5.00–6.00</sub> | 4.00 <sub>4.00–5.00</sub> | 5.00 <sub>5.00–5.00</sub> | 5.00 <sub>5.00–5.00</sub> |
| tick ms/frame | 4.25 <sub>3.99–4.34</sub> | 3.82 <sub>3.43–4.34</sub> | 4.47 <sub>4.26–5.08</sub> | 5.83 <sub>5.69–5.90</sub> |
| peek_frame ms/frame | n/a | n/a | 0.00 <sub>0.00–0.00</sub> | 3.68 <sub>3.39–3.73</sub> |
| input->first frame ms | 109.78 <sub>76.03–116.91</sub> | 129.00 <sub>125.91–135.11</sub> | n/a | n/a |
| command p50 ms | 87.41 <sub>85.87–88.50</sub> | 111.66 <sub>111.36–117.42</sub> | 108.57 <sub>96.66–122.85</sub> | 152.81 <sub>145.73–152.90</sub> |

| vs (ranges must not overlap) | default vs main | default vs nopeeks | morepeeks vs main | morepeeks vs nopeeks | morepeeks vs default | nopeeks vs main |
| --- | --- | --- | --- | --- | --- | --- |
| interval p90 ms | ≈ | ≈ | better (-36%) | better (-31%) | better (-35%) | better (-8%) |
| frame p90 ms | ≈ | ≈ | better (-38%) | better (-44%) | better (-45%) | ≈ |
| frames >2x budget | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |
| tick ms/frame | ≈ | ≈ | worse (+37%) | worse (+53%) | worse (+30%) | ≈ |
| input->first frame ms | n/a | n/a | n/a | n/a | n/a | worse (+18%) |
| command p50 ms | worse (+24%) | ≈ | worse (+75%) | worse (+37%) | worse (+41%) | worse (+28%) |

### minimize

| metric | main | nopeeks | default | morepeeks |
| --- | ---: | ---: | ---: | ---: |
| interval p50 ms | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.71–5.72</sub> | 5.72 <sub>5.72–5.72</sub> |
| interval p90 ms | 5.83 <sub>5.82–6.52</sub> | 6.10 <sub>5.82–6.50</sub> | 5.79 <sub>5.78–5.81</sub> | 5.79 <sub>5.78–5.84</sub> |
| frame p90 ms | 5.13 <sub>3.77–6.46</sub> | 5.09 <sub>3.22–5.56</sub> | 4.55 <sub>4.48–4.81</sub> | 4.36 <sub>4.04–4.81</sub> |
| frames >2x budget | 8.00 <sub>7.00–8.00</sub> | 6.00 <sub>5.00–7.00</sub> | 7.00 <sub>7.00–7.00</sub> | 7.00 <sub>6.00–7.00</sub> |
| tick ms/frame | 2.04 <sub>1.88–2.06</sub> | 1.72 <sub>1.65–1.84</sub> | 2.51 <sub>2.09–2.58</sub> | 2.14 <sub>2.09–2.15</sub> |
| peek_frame ms/frame | n/a | n/a | 1.31 <sub>1.24–1.34</sub> | 0.76 <sub>0.00–0.80</sub> |
| input->first frame ms | 127.90 <sub>98.45–128.63</sub> | 126.39 <sub>112.38–138.99</sub> | n/a | n/a |
| command p50 ms | 53.55 <sub>27.24–54.87</sub> | 69.12 <sub>68.87–73.04</sub> | 75.62 <sub>68.57–82.54</sub> | 77.28 <sub>61.09–78.72</sub> |

| vs (ranges must not overlap) | default vs main | default vs nopeeks | morepeeks vs main | morepeeks vs nopeeks | morepeeks vs default | nopeeks vs main |
| --- | --- | --- | --- | --- | --- | --- |
| interval p90 ms | ≈ | better (-5%) | ≈ | ≈ | ≈ | ≈ |
| frame p90 ms | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |
| frames >2x budget | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |
| tick ms/frame | worse (+23%) | worse (+46%) | ≈ | worse (+24%) | ≈ | better (-16%) |
| input->first frame ms | n/a | n/a | n/a | n/a | n/a | ≈ |
| command p50 ms | worse (+41%) | ≈ | worse (+44%) | ≈ | ≈ | worse (+29%) |

### resize

| metric | main | nopeeks | default | morepeeks |
| --- | ---: | ---: | ---: | ---: |
| interval p50 ms | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.71–5.72</sub> | 5.71 <sub>5.71–5.72</sub> |
| interval p90 ms | 11.84 <sub>11.16–12.57</sub> | 10.24 <sub>8.92–10.56</sub> | 6.64 <sub>6.53–7.08</sub> | 6.13 <sub>5.85–7.18</sub> |
| frame p90 ms | 11.75 <sub>10.91–12.50</sub> | 10.00 <sub>8.81–10.14</sub> | 5.38 <sub>5.36–5.43</sub> | 5.15 <sub>4.98–5.22</sub> |
| frames >2x budget | 6.00 <sub>5.00–6.00</sub> | 5.00 <sub>4.00–5.00</sub> | 5.00 <sub>4.00–5.00</sub> | 5.00 <sub>5.00–5.00</sub> |
| tick ms/frame | 4.77 <sub>4.68–4.87</sub> | 4.27 <sub>3.95–4.55</sub> | 6.10 <sub>6.07–6.57</sub> | 6.00 <sub>4.54–6.02</sub> |
| peek_frame ms/frame | n/a | n/a | 3.56 <sub>3.40–3.70</sub> | 3.57 <sub>1.74–3.61</sub> |
| input->first frame ms | 104.99 <sub>95.85–105.06</sub> | 129.74 <sub>117.83–133.20</sub> | n/a | n/a |
| command p50 ms | 82.66 <sub>80.87–84.12</sub> | 108.71 <sub>107.93–110.68</sub> | 151.08 <sub>147.60–159.33</sub> | 146.15 <sub>133.31–149.08</sub> |

| vs (ranges must not overlap) | default vs main | default vs nopeeks | morepeeks vs main | morepeeks vs nopeeks | morepeeks vs default | nopeeks vs main |
| --- | --- | --- | --- | --- | --- | --- |
| interval p90 ms | better (-44%) | better (-35%) | better (-48%) | better (-40%) | ≈ | better (-14%) |
| frame p90 ms | better (-54%) | better (-46%) | better (-56%) | better (-48%) | ≈ | better (-15%) |
| frames >2x budget | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |
| tick ms/frame | worse (+28%) | worse (+43%) | ≈ | ≈ | ≈ | better (-10%) |
| input->first frame ms | n/a | n/a | n/a | n/a | n/a | worse (+24%) |
| command p50 ms | worse (+83%) | worse (+39%) | worse (+77%) | worse (+34%) | ≈ | worse (+32%) |

### switch

| metric | main | nopeeks | default | morepeeks |
| --- | ---: | ---: | ---: | ---: |
| interval p50 ms | 5.71 <sub>5.71–5.72</sub> | 5.71 <sub>5.71–5.72</sub> | 5.72 <sub>5.71–5.72</sub> | 5.72 <sub>5.71–5.72</sub> |
| interval p90 ms | 5.78 <sub>5.77–5.79</sub> | 5.77 <sub>5.76–5.77</sub> | 5.78 <sub>5.77–5.78</sub> | 5.77 <sub>5.77–5.78</sub> |
| frame p90 ms | 4.08 <sub>3.69–4.15</sub> | 2.16 <sub>2.01–2.18</sub> | 2.22 <sub>2.13–2.35</sub> | 2.03 <sub>1.95–2.12</sub> |
| frames >2x budget | 2.00 <sub>2.00–3.00</sub> | 3.00 <sub>3.00–3.00</sub> | 4.00 <sub>3.00–4.00</sub> | 3.00 <sub>3.00–4.00</sub> |
| tick ms/frame | 2.83 <sub>2.49–2.85</sub> | 1.94 <sub>1.89–2.22</sub> | 2.20 <sub>2.03–2.44</sub> | 2.06 <sub>2.01–2.20</sub> |
| peek_frame ms/frame | n/a | n/a | 0.00 <sub>0.00–0.00</sub> | 0.00 <sub>0.00–0.00</sub> |
| input->first frame ms | n/a | n/a | n/a | n/a |
| command p50 ms | 40.88 <sub>31.17–47.70</sub> | 80.86 <sub>73.45–85.57</sub> | 74.70 <sub>61.29–84.62</sub> | 86.06 <sub>55.72–86.08</sub> |

| vs (ranges must not overlap) | default vs main | default vs nopeeks | morepeeks vs main | morepeeks vs nopeeks | morepeeks vs default | nopeeks vs main |
| --- | --- | --- | --- | --- | --- | --- |
| interval p90 ms | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |
| frame p90 ms | better (-46%) | ≈ | better (-50%) | ≈ | better (-9%) | better (-47%) |
| frames >2x budget | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |
| tick ms/frame | better (-22%) | ≈ | better (-27%) | ≈ | ≈ | better (-31%) |
| input->first frame ms | n/a | n/a | n/a | n/a | n/a | n/a |
| command p50 ms | worse (+83%) | ≈ | worse (+111%) | ≈ | ≈ | worse (+98%) |

#### With CPU load: frame interval tails

From `perf.log`: per run, the median over its burst reports of interval p99 and max (ms). Then median over rounds, with the rounds' range. 5.75 ms is one vblank, ~11.5 ms one missed.

| scenario | main p99 | main max | nopeeks p99 | nopeeks max | default p99 | default max | morepeeks p99 | morepeeks max |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| move | 13.2 <sub>13.2–41.7</sub> | 13.2 <sub>13.2–41.7</sub> | 12.8 <sub>11.7–41.3</sub> | 12.8 <sub>11.7–41.3</sub> | 19.0 <sub>14.0–40.6</sub> | 19.0 <sub>14.0–40.6</sub> | 23.2 <sub>21.1–23.4</sub> | 23.2 <sub>21.1–23.4</sub> |
| float | 60.7 <sub>59.4–61.9</sub> | 60.7 <sub>59.4–75.4</sub> | 61.2 <sub>55.4–62.4</sub> | 61.2 <sub>55.4–62.4</sub> | 63.9 <sub>51.0–73.8</sub> | 63.9 <sub>51.0–73.8</sub> | 107.2 <sub>105.8–110.5</sub> | 107.2 <sub>105.8–110.5</sub> |
| minimize | 42.2 <sub>35.5–45.3</sub> | 50.7 <sub>47.6–56.1</sub> | 35.7 <sub>34.0–36.6</sub> | 50.6 <sub>49.9–58.7</sub> | 43.9 <sub>40.7–45.9</sub> | 60.2 <sub>58.0–60.9</sub> | 42.9 <sub>41.4–43.5</sub> | 62.2 <sub>58.6–64.1</sub> |
| resize | 57.8 <sub>51.6–69.1</sub> | 61.3 <sub>57.8–69.1</sub> | 50.0 <sub>49.8–53.2</sub> | 50.0 <sub>49.8–55.6</sub> | 94.7 <sub>89.7–103.0</sub> | 94.7 <sub>89.7–103.0</sub> | 82.6 <sub>64.2–89.4</sub> | 82.6 <sub>71.3–89.4</sub> |
| switch | 94.6 <sub>85.0–106.1</sub> | 99.6 <sub>87.8–108.3</sub> | 45.2 <sub>43.2–50.0</sub> | 45.2 <sub>43.2–50.0</sub> | 52.9 <sub>42.9–57.1</sub> | 53.7 <sub>42.9–57.1</sub> | 48.2 <sub>37.0–57.2</sub> | 48.2 <sub>37.0–57.2</sub> |

# Expansion: all four builds interleaved, fixed tile order

These are the most controlled numbers in this report, and the verdict
leans on them first. They fix the two weaknesses of the requested runs:

- **Drift**: the WM restarts as `main`, `nopeeks`, `default`,
  `morepeeks` in turn, round after round, so drift over the session hits
  every build alike.
- **Tile order**: a restart flips the tiles between two orders, which
  changes chrome's neighbours. An earlier interleaved run, at the end of
  this report, showed this. After every restart the tiles are now
  arranged the same way: Windows Terminal, Notepad, **chrome**,
  Everything, Explorer, Settings, Store. That is 7 `move --direction
  left` commands at most, over one IPC connection. Chrome is never at an
  edge, so every `move` burst animates.

Same scenarios, bursts, windows and target as above, 4 rounds:
`bench-run.ps1 -Modes main,nopeeks,default,morepeeks -Rounds 4
-TileOrder …`, with and without `-CpuLoad 8`. With 4 rounds against 4,
separation by pure chance happens in under 3% of comparisons, against
10% with 3 rounds.

## Fixed tile order, no CPU load

Median over rounds (up to 4), round range in small print. Lower is better, except interval p50, which is the refresh period.

### move

| metric | main | nopeeks | default | morepeeks |
| --- | ---: | ---: | ---: | ---: |
| interval p50 ms | 5.71 <sub>5.71–5.71</sub> | 5.71 <sub>5.71–5.71</sub> | 5.71 <sub>5.71–5.71</sub> | 5.71 <sub>5.71–5.71</sub> |
| interval p90 ms | 5.75 <sub>5.74–5.75</sub> | 5.75 <sub>5.75–5.76</sub> | 5.76 <sub>5.75–5.77</sub> | 5.76 <sub>5.76–5.77</sub> |
| frame p90 ms | 0.95 <sub>0.94–1.06</sub> | 0.95 <sub>0.89–0.96</sub> | 0.66 <sub>0.51–0.76</sub> | 0.56 <sub>0.55–0.61</sub> |
| frames >2x budget | 0.00 <sub>0.00–0.00</sub> | 0.00 <sub>0.00–0.00</sub> | 2.00 <sub>1.00–2.00</sub> | 2.00 <sub>2.00–2.00</sub> |
| tick ms/frame | 0.66 <sub>0.59–0.69</sub> | 0.56 <sub>0.54–0.56</sub> | 0.82 <sub>0.79–0.86</sub> | 0.84 <sub>0.82–0.86</sub> |
| peek_frame ms/frame | n/a | n/a | 0.44 <sub>0.43–0.44</sub> | 0.42 <sub>0.41–0.47</sub> |
| input->first frame ms | 16.48 <sub>15.20–16.92</sub> | 18.99 <sub>18.47–19.40</sub> | n/a | n/a |
| command p50 ms | 15.68 <sub>14.64–16.17</sub> | 18.36 <sub>17.99–18.80</sub> | 24.45 <sub>23.73–25.08</sub> | 24.25 <sub>23.83–25.15</sub> |

| vs (ranges must not overlap) | default vs main | default vs nopeeks | morepeeks vs main | morepeeks vs nopeeks | morepeeks vs default | nopeeks vs main |
| --- | --- | --- | --- | --- | --- | --- |
| interval p90 ms | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |
| frame p90 ms | better (-31%) | better (-31%) | better (-42%) | better (-41%) | ≈ | ≈ |
| frames >2x budget | worse (2.00 vs 0) | worse (2.00 vs 0) | worse (2.00 vs 0) | worse (2.00 vs 0) | ≈ | ≈ |
| tick ms/frame | worse (+25%) | worse (+49%) | worse (+27%) | worse (+51%) | ≈ | better (-16%) |
| input->first frame ms | n/a | n/a | n/a | n/a | n/a | worse (+15%) |
| command p50 ms | worse (+56%) | worse (+33%) | worse (+55%) | worse (+32%) | ≈ | worse (+17%) |

### float

| metric | main | nopeeks | default | morepeeks |
| --- | ---: | ---: | ---: | ---: |
| interval p50 ms | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.71–5.72</sub> | 5.71 <sub>5.71–5.72</sub> |
| interval p90 ms | 8.41 <sub>6.51–10.83</sub> | 8.36 <sub>6.12–9.60</sub> | 9.19 <sub>8.47–9.54</sub> | 6.79 <sub>6.51–7.30</sub> |
| frame p90 ms | 7.11 <sub>6.40–9.83</sub> | 8.00 <sub>4.25–9.50</sub> | 8.53 <sub>7.40–9.25</sub> | 3.98 <sub>3.70–4.43</sub> |
| frames >2x budget | 5.50 <sub>4.00–6.00</sub> | 4.00 <sub>4.00–4.00</sub> | 4.00 <sub>4.00–4.00</sub> | 5.00 <sub>5.00–5.00</sub> |
| tick ms/frame | 3.74 <sub>3.52–3.86</sub> | 3.62 <sub>3.60–3.90</sub> | 4.03 <sub>3.97–4.21</sub> | 5.04 <sub>4.97–5.11</sub> |
| peek_frame ms/frame | n/a | n/a | 0.00 <sub>0.00–0.00</sub> | 3.13 <sub>3.08–3.20</sub> |
| input->first frame ms | 88.37 <sub>74.08–106.79</sub> | 110.36 <sub>84.27–114.45</sub> | n/a | n/a |
| command p50 ms | 74.41 <sub>66.83–84.73</sub> | 105.70 <sub>102.04–107.21</sub> | 99.54 <sub>97.55–104.92</sub> | 137.87 <sub>130.28–140.15</sub> |

| vs (ranges must not overlap) | default vs main | default vs nopeeks | morepeeks vs main | morepeeks vs nopeeks | morepeeks vs default | nopeeks vs main |
| --- | --- | --- | --- | --- | --- | --- |
| interval p90 ms | ≈ | ≈ | ≈ | ≈ | better (-26%) | ≈ |
| frame p90 ms | ≈ | ≈ | better (-44%) | ≈ | better (-53%) | ≈ |
| frames >2x budget | ≈ | ≈ | ≈ | worse (+25%) | worse (+25%) | ≈ |
| tick ms/frame | worse (+8%) | worse (+11%) | worse (+35%) | worse (+39%) | worse (+25%) | ≈ |
| input->first frame ms | n/a | n/a | n/a | n/a | n/a | ≈ |
| command p50 ms | worse (+34%) | ≈ | worse (+85%) | worse (+30%) | worse (+39%) | worse (+42%) |

### minimize

| metric | main | nopeeks | default | morepeeks |
| --- | ---: | ---: | ---: | ---: |
| interval p50 ms | 5.71 <sub>5.71–5.72</sub> | 5.72 <sub>5.71–5.72</sub> | 5.72 <sub>5.71–5.72</sub> | 5.71 <sub>5.71–5.72</sub> |
| interval p90 ms | 5.79 <sub>5.77–5.79</sub> | 6.00 <sub>5.79–6.17</sub> | 5.78 <sub>5.77–5.78</sub> | 5.78 <sub>5.77–5.78</sub> |
| frame p90 ms | 3.50 <sub>3.10–4.33</sub> | 5.43 <sub>5.06–6.00</sub> | 3.63 <sub>3.34–3.87</sub> | 3.58 <sub>3.35–3.74</sub> |
| frames >2x budget | 5.00 <sub>5.00–7.00</sub> | 4.50 <sub>4.00–5.00</sub> | 5.00 <sub>5.00–5.00</sub> | 6.00 <sub>6.00–6.00</sub> |
| tick ms/frame | 1.77 <sub>1.56–1.87</sub> | 1.56 <sub>1.47–1.60</sub> | 1.77 <sub>1.74–1.80</sub> | 1.77 <sub>1.76–1.78</sub> |
| peek_frame ms/frame | n/a | n/a | 0.70 <sub>0.00–0.89</sub> | 0.86 <sub>0.00–1.19</sub> |
| input->first frame ms | 101.12 <sub>97.46–117.15</sub> | 122.73 <sub>109.65–130.30</sub> | n/a | n/a |
| command p50 ms | 47.78 <sub>28.04–49.52</sub> | 57.50 <sub>36.34–59.19</sub> | 67.97 <sub>33.94–69.52</sub> | 64.65 <sub>32.72–68.52</sub> |

| vs (ranges must not overlap) | default vs main | default vs nopeeks | morepeeks vs main | morepeeks vs nopeeks | morepeeks vs default | nopeeks vs main |
| --- | --- | --- | --- | --- | --- | --- |
| interval p90 ms | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |
| frame p90 ms | ≈ | better (-33%) | ≈ | better (-34%) | ≈ | worse (+55%) |
| frames >2x budget | ≈ | ≈ | ≈ | worse (+33%) | worse (+20%) | ≈ |
| tick ms/frame | ≈ | worse (+13%) | ≈ | worse (+13%) | ≈ | ≈ |
| input->first frame ms | n/a | n/a | n/a | n/a | n/a | ≈ |
| command p50 ms | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |

### resize

| metric | main | nopeeks | default | morepeeks |
| --- | ---: | ---: | ---: | ---: |
| interval p50 ms | 5.72 <sub>5.71–5.72</sub> | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.71–5.72</sub> | 5.72 <sub>5.72–5.72</sub> |
| interval p90 ms | 9.16 <sub>9.01–10.21</sub> | 8.03 <sub>6.69–9.18</sub> | 6.55 <sub>5.97–6.96</sub> | 6.82 <sub>5.85–6.96</sub> |
| frame p90 ms | 8.67 <sub>6.41–10.15</sub> | 7.67 <sub>6.53–8.69</sub> | 4.22 <sub>4.00–4.47</sub> | 4.30 <sub>4.27–4.37</sub> |
| frames >2x budget | 5.00 <sub>4.00–5.00</sub> | 4.00 <sub>4.00–4.00</sub> | 5.00 <sub>5.00–5.00</sub> | 5.00 <sub>5.00–5.00</sub> |
| tick ms/frame | 4.35 <sub>3.98–4.59</sub> | 3.77 <sub>3.27–3.97</sub> | 5.42 <sub>4.75–5.56</sub> | 5.10 <sub>4.90–5.48</sub> |
| peek_frame ms/frame | n/a | n/a | 3.38 <sub>3.16–3.46</sub> | 3.29 <sub>3.25–3.40</sub> |
| input->first frame ms | 88.55 <sub>81.09–98.42</sub> | 101.81 <sub>94.27–114.85</sub> | n/a | n/a |
| command p50 ms | 76.21 <sub>70.57–78.32</sub> | 90.07 <sub>87.26–94.09</sub> | 135.34 <sub>133.14–138.25</sub> | 130.36 <sub>125.26–133.10</sub> |

| vs (ranges must not overlap) | default vs main | default vs nopeeks | morepeeks vs main | morepeeks vs nopeeks | morepeeks vs default | nopeeks vs main |
| --- | --- | --- | --- | --- | --- | --- |
| interval p90 ms | better (-28%) | ≈ | better (-26%) | ≈ | ≈ | ≈ |
| frame p90 ms | better (-51%) | better (-45%) | better (-50%) | better (-44%) | ≈ | ≈ |
| frames >2x budget | ≈ | worse (+25%) | ≈ | worse (+25%) | ≈ | ≈ |
| tick ms/frame | worse (+25%) | worse (+44%) | worse (+17%) | worse (+35%) | ≈ | better (-13%) |
| input->first frame ms | n/a | n/a | n/a | n/a | n/a | ≈ |
| command p50 ms | worse (+78%) | worse (+50%) | worse (+71%) | worse (+45%) | ≈ | worse (+18%) |

### switch

| metric | main | nopeeks | default | morepeeks |
| --- | ---: | ---: | ---: | ---: |
| interval p50 ms | 5.71 <sub>5.71–5.71</sub> | 5.72 <sub>5.71–5.72</sub> | 5.72 <sub>5.71–5.72</sub> | 5.72 <sub>5.71–5.72</sub> |
| interval p90 ms | 5.76 <sub>5.76–5.77</sub> | 5.76 <sub>5.76–5.77</sub> | 5.77 <sub>5.77–5.77</sub> | 5.77 <sub>5.77–5.77</sub> |
| frame p90 ms | 3.65 <sub>3.50–3.76</sub> | 1.73 <sub>1.63–1.77</sub> | 1.79 <sub>1.66–1.89</sub> | 1.76 <sub>1.74–1.84</sub> |
| frames >2x budget | 2.00 <sub>2.00–3.00</sub> | 3.00 <sub>3.00–3.00</sub> | 3.00 <sub>3.00–3.00</sub> | 3.00 <sub>3.00–3.00</sub> |
| tick ms/frame | 2.40 <sub>2.34–2.48</sub> | 1.75 <sub>1.70–1.84</sub> | 1.91 <sub>1.86–1.97</sub> | 1.93 <sub>1.92–1.97</sub> |
| peek_frame ms/frame | n/a | n/a | 0.00 <sub>0.00–0.00</sub> | 0.00 <sub>0.00–0.00</sub> |
| input->first frame ms | n/a | n/a | n/a | n/a |
| command p50 ms | 36.68 <sub>31.07–37.93</sub> | 64.20 <sub>53.65–66.29</sub> | 54.34 <sub>52.23–56.00</sub> | 54.36 <sub>52.98–60.92</sub> |

| vs (ranges must not overlap) | default vs main | default vs nopeeks | morepeeks vs main | morepeeks vs nopeeks | morepeeks vs default | nopeeks vs main |
| --- | --- | --- | --- | --- | --- | --- |
| interval p90 ms | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |
| frame p90 ms | better (-51%) | ≈ | better (-52%) | ≈ | ≈ | better (-53%) |
| frames >2x budget | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |
| tick ms/frame | better (-20%) | worse (+9%) | better (-20%) | worse (+10%) | ≈ | better (-27%) |
| input->first frame ms | n/a | n/a | n/a | n/a | n/a | n/a |
| command p50 ms | worse (+48%) | ≈ | worse (+48%) | ≈ | ≈ | worse (+75%) |

#### Fixed tile order, no CPU load: frame interval tails

From `perf.log`: per run, the median over its burst reports of interval p99 and max (ms). Then median over rounds, with the rounds' range. 5.75 ms is one vblank, ~11.5 ms one missed.

| scenario | main p99 | main max | nopeeks p99 | nopeeks max | default p99 | default max | morepeeks p99 | morepeeks max |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| move | 11.4 <sub>10.7–12.0</sub> | 11.4 <sub>10.7–12.0</sub> | 11.6 <sub>11.5–11.8</sub> | 11.6 <sub>11.5–11.8</sub> | 15.0 <sub>14.9–15.2</sub> | 15.0 <sub>14.9–15.2</sub> | 15.2 <sub>15.0–15.4</sub> | 15.2 <sub>15.0–15.4</sub> |
| float | 51.1 <sub>44.8–65.2</sub> | 52.2 <sub>44.8–65.2</sub> | 57.9 <sub>50.6–59.3</sub> | 57.9 <sub>50.6–59.3</sub> | 59.7 <sub>56.9–65.5</sub> | 59.7 <sub>56.9–65.5</sub> | 76.2 <sub>74.0–85.0</sub> | 76.2 <sub>74.0–85.0</sub> |
| minimize | 39.8 <sub>33.2–49.0</sub> | 53.3 <sub>49.9–59.3</sub> | 28.1 <sub>25.5–29.5</sub> | 42.3 <sub>38.8–46.4</sub> | 32.4 <sub>31.4–36.9</sub> | 54.3 <sub>53.6–55.3</sub> | 34.1 <sub>30.8–36.8</sub> | 55.8 <sub>54.9–58.8</sub> |
| resize | 46.9 <sub>43.3–49.0</sub> | 56.3 <sub>47.0–59.9</sub> | 37.9 <sub>34.3–55.9</sub> | 53.0 <sub>40.3–60.2</sub> | 74.5 <sub>73.3–74.8</sub> | 74.5 <sub>73.3–74.8</sub> | 74.1 <sub>73.0–76.6</sub> | 74.1 <sub>73.0–76.6</sub> |
| switch | 87.2 <sub>85.6–88.2</sub> | 89.6 <sub>86.4–90.9</sub> | 41.7 <sub>38.6–44.0</sub> | 41.7 <sub>38.6–44.0</sub> | 43.5 <sub>41.7–44.6</sub> | 43.5 <sub>41.7–44.6</sub> | 44.9 <sub>43.9–45.6</sub> | 44.9 <sub>43.9–45.6</sub> |

## Fixed tile order, with CPU load (-CpuLoad 8)

Median over rounds (up to 4), round range in small print. Lower is better, except interval p50, which is the refresh period.

### move

| metric | main | nopeeks | default | morepeeks |
| --- | ---: | ---: | ---: | ---: |
| interval p50 ms | 5.71 <sub>5.71–5.71</sub> | 5.71 <sub>5.71–5.72</sub> | 5.71 <sub>5.71–5.72</sub> | 5.71 <sub>5.71–5.72</sub> |
| interval p90 ms | 5.76 <sub>5.75–5.77</sub> | 5.76 <sub>5.75–5.76</sub> | 5.75 <sub>5.75–5.76</sub> | 5.76 <sub>5.76–5.77</sub> |
| frame p90 ms | 1.17 <sub>1.12–1.22</sub> | 1.06 <sub>1.01–1.10</sub> | 0.73 <sub>0.63–0.84</sub> | 0.67 <sub>0.64–0.76</sub> |
| frames >2x budget | 1.00 <sub>0.00–1.00</sub> | 1.00 <sub>1.00–1.00</sub> | 1.00 <sub>1.00–2.00</sub> | 1.50 <sub>1.00–2.00</sub> |
| tick ms/frame | 0.77 <sub>0.76–0.82</sub> | 0.63 <sub>0.62–0.69</sub> | 0.96 <sub>0.93–0.98</sub> | 0.95 <sub>0.91–1.01</sub> |
| peek_frame ms/frame | n/a | n/a | 0.41 <sub>0.39–0.45</sub> | 0.45 <sub>0.43–0.46</sub> |
| input->first frame ms | 17.52 <sub>16.94–18.32</sub> | 20.05 <sub>19.87–20.51</sub> | n/a | n/a |
| command p50 ms | 16.55 <sub>16.19–17.57</sub> | 19.36 <sub>19.08–19.70</sub> | 26.08 <sub>24.97–26.61</sub> | 25.45 <sub>25.18–26.17</sub> |

| vs (ranges must not overlap) | default vs main | default vs nopeeks | morepeeks vs main | morepeeks vs nopeeks | morepeeks vs default | nopeeks vs main |
| --- | --- | --- | --- | --- | --- | --- |
| interval p90 ms | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |
| frame p90 ms | better (-38%) | better (-32%) | better (-43%) | better (-38%) | ≈ | better (-9%) |
| frames >2x budget | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |
| tick ms/frame | worse (+25%) | worse (+53%) | worse (+23%) | worse (+51%) | ≈ | better (-18%) |
| input->first frame ms | n/a | n/a | n/a | n/a | n/a | worse (+14%) |
| command p50 ms | worse (+58%) | worse (+35%) | worse (+54%) | worse (+31%) | ≈ | worse (+17%) |

### float

| metric | main | nopeeks | default | morepeeks |
| --- | ---: | ---: | ---: | ---: |
| interval p50 ms | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.72–5.72</sub> | 5.71 <sub>5.71–5.72</sub> |
| interval p90 ms | 10.28 <sub>5.76–10.93</sub> | 9.90 <sub>9.57–10.04</sub> | 8.22 <sub>6.68–10.46</sub> | 7.12 <sub>6.44–7.43</sub> |
| frame p90 ms | 7.84 <sub>4.17–10.23</sub> | 9.80 <sub>9.55–10.02</sub> | 7.38 <sub>4.95–9.86</sub> | 5.02 <sub>4.84–5.10</sub> |
| frames >2x budget | 5.50 <sub>5.00–6.00</sub> | 4.50 <sub>4.00–5.00</sub> | 4.00 <sub>4.00–5.00</sub> | 5.00 <sub>5.00–5.00</sub> |
| tick ms/frame | 4.21 <sub>1.89–4.79</sub> | 4.15 <sub>3.88–4.34</sub> | 4.27 <sub>4.20–4.83</sub> | 5.71 <sub>5.68–6.07</sub> |
| peek_frame ms/frame | n/a | n/a | 0.00 <sub>0.00–0.00</sub> | 3.61 <sub>3.52–3.94</sub> |
| input->first frame ms | 109.20 <sub>77.47–121.14</sub> | 106.39 <sub>91.50–136.61</sub> | n/a | n/a |
| command p50 ms | 91.48 <sub>60.28–97.22</sub> | 99.81 <sub>88.12–120.93</sub> | 112.44 <sub>93.51–121.90</sub> | 163.28 <sub>146.91–165.41</sub> |

| vs (ranges must not overlap) | default vs main | default vs nopeeks | morepeeks vs main | morepeeks vs nopeeks | morepeeks vs default | nopeeks vs main |
| --- | --- | --- | --- | --- | --- | --- |
| interval p90 ms | ≈ | ≈ | ≈ | better (-28%) | ≈ | ≈ |
| frame p90 ms | ≈ | ≈ | ≈ | better (-49%) | ≈ | ≈ |
| frames >2x budget | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |
| tick ms/frame | ≈ | ≈ | worse (+35%) | worse (+38%) | worse (+34%) | ≈ |
| input->first frame ms | n/a | n/a | n/a | n/a | n/a | ≈ |
| command p50 ms | ≈ | ≈ | worse (+78%) | worse (+64%) | worse (+45%) | ≈ |

### minimize

| metric | main | nopeeks | default | morepeeks |
| --- | ---: | ---: | ---: | ---: |
| interval p50 ms | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.71–5.72</sub> |
| interval p90 ms | 5.85 <sub>5.81–6.22</sub> | 6.50 <sub>6.05–6.80</sub> | 5.79 <sub>5.78–5.79</sub> | 5.79 <sub>5.78–5.79</sub> |
| frame p90 ms | 3.88 <sub>3.26–5.22</sub> | 5.65 <sub>4.92–5.95</sub> | 4.12 <sub>3.92–4.37</sub> | 4.12 <sub>3.83–4.38</sub> |
| frames >2x budget | 7.00 <sub>6.00–7.00</sub> | 6.50 <sub>6.00–7.00</sub> | 7.00 <sub>6.00–7.00</sub> | 7.00 <sub>6.00–7.00</sub> |
| tick ms/frame | 2.10 <sub>1.87–2.14</sub> | 1.73 <sub>1.70–1.85</sub> | 1.98 <sub>1.92–2.17</sub> | 2.06 <sub>2.00–2.14</sub> |
| peek_frame ms/frame | n/a | n/a | 1.29 <sub>0.83–1.36</sub> | 1.23 <sub>0.00–1.35</sub> |
| input->first frame ms | 113.29 <sub>104.28–132.98</sub> | 126.62 <sub>115.41–153.98</sub> | n/a | n/a |
| command p50 ms | 56.28 <sub>35.73–58.93</sub> | 68.95 <sub>40.55–70.74</sub> | 77.41 <sub>73.34–78.85</sub> | 72.66 <sub>70.88–76.09</sub> |

| vs (ranges must not overlap) | default vs main | default vs nopeeks | morepeeks vs main | morepeeks vs nopeeks | morepeeks vs default | nopeeks vs main |
| --- | --- | --- | --- | --- | --- | --- |
| interval p90 ms | ≈ | better (-11%) | ≈ | better (-11%) | ≈ | ≈ |
| frame p90 ms | ≈ | better (-27%) | ≈ | better (-27%) | ≈ | ≈ |
| frames >2x budget | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |
| tick ms/frame | ≈ | worse (+14%) | ≈ | worse (+19%) | ≈ | better (-18%) |
| input->first frame ms | n/a | n/a | n/a | n/a | n/a | ≈ |
| command p50 ms | worse (+38%) | worse (+12%) | worse (+29%) | worse (+5%) | ≈ | ≈ |

### resize

| metric | main | nopeeks | default | morepeeks |
| --- | ---: | ---: | ---: | ---: |
| interval p50 ms | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.72–5.72</sub> | 5.71 <sub>5.71–5.71</sub> | 5.71 <sub>5.71–5.72</sub> |
| interval p90 ms | 10.87 <sub>10.61–11.72</sub> | 10.63 <sub>9.70–11.01</sub> | 6.62 <sub>6.36–7.45</sub> | 6.96 <sub>6.54–7.36</sub> |
| frame p90 ms | 10.64 <sub>10.51–11.35</sub> | 10.14 <sub>8.64–10.75</sub> | 5.20 <sub>5.12–5.32</sub> | 5.29 <sub>4.67–5.35</sub> |
| frames >2x budget | 5.00 <sub>4.00–6.00</sub> | 5.00 <sub>5.00–5.00</sub> | 5.00 <sub>5.00–5.00</sub> | 5.00 <sub>5.00–5.00</sub> |
| tick ms/frame | 4.91 <sub>4.34–5.24</sub> | 4.28 <sub>4.13–4.40</sub> | 6.12 <sub>6.06–6.49</sub> | 6.06 <sub>5.36–6.37</sub> |
| peek_frame ms/frame | n/a | n/a | 3.81 <sub>3.43–3.95</sub> | 3.84 <sub>3.73–3.98</sub> |
| input->first frame ms | 105.49 <sub>83.90–118.32</sub> | 128.50 <sub>103.68–133.35</sub> | n/a | n/a |
| command p50 ms | 84.55 <sub>73.77–87.96</sub> | 114.23 <sub>94.37–115.55</sub> | 159.51 <sub>145.15–161.91</sub> | 161.10 <sub>138.30–163.32</sub> |

| vs (ranges must not overlap) | default vs main | default vs nopeeks | morepeeks vs main | morepeeks vs nopeeks | morepeeks vs default | nopeeks vs main |
| --- | --- | --- | --- | --- | --- | --- |
| interval p90 ms | better (-39%) | better (-38%) | better (-36%) | better (-35%) | ≈ | ≈ |
| frame p90 ms | better (-51%) | better (-49%) | better (-50%) | better (-48%) | ≈ | ≈ |
| frames >2x budget | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |
| tick ms/frame | worse (+25%) | worse (+43%) | worse (+23%) | worse (+42%) | ≈ | ≈ |
| input->first frame ms | n/a | n/a | n/a | n/a | n/a | ≈ |
| command p50 ms | worse (+89%) | worse (+40%) | worse (+91%) | worse (+41%) | ≈ | worse (+35%) |

### switch

| metric | main | nopeeks | default | morepeeks |
| --- | ---: | ---: | ---: | ---: |
| interval p50 ms | 5.71 <sub>5.71–5.72</sub> | 5.71 <sub>5.71–5.72</sub> | 5.71 <sub>5.71–5.72</sub> | 5.71 <sub>5.71–5.72</sub> |
| interval p90 ms | 5.78 <sub>5.77–5.79</sub> | 5.77 <sub>5.77–5.77</sub> | 5.77 <sub>5.76–5.78</sub> | 5.78 <sub>5.77–5.78</sub> |
| frame p90 ms | 4.07 <sub>3.97–4.52</sub> | 2.13 <sub>1.98–2.24</sub> | 2.10 <sub>2.08–2.27</sub> | 2.23 <sub>2.19–2.27</sub> |
| frames >2x budget | 2.50 <sub>2.00–3.00</sub> | 3.00 <sub>3.00–4.00</sub> | 3.00 <sub>3.00–3.00</sub> | 3.00 <sub>3.00–3.00</sub> |
| tick ms/frame | 2.77 <sub>2.65–2.91</sub> | 2.01 <sub>1.85–2.19</sub> | 2.10 <sub>1.98–2.14</sub> | 2.15 <sub>2.06–2.21</sub> |
| peek_frame ms/frame | n/a | n/a | 0.00 <sub>0.00–0.00</sub> | 0.00 <sub>0.00–0.00</sub> |
| input->first frame ms | n/a | n/a | n/a | n/a |
| command p50 ms | 38.59 <sub>38.27–39.61</sub> | 60.65 <sub>53.32–66.11</sub> | 59.88 <sub>51.50–90.77</sub> | 56.43 <sub>54.22–57.31</sub> |

| vs (ranges must not overlap) | default vs main | default vs nopeeks | morepeeks vs main | morepeeks vs nopeeks | morepeeks vs default | nopeeks vs main |
| --- | --- | --- | --- | --- | --- | --- |
| interval p90 ms | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |
| frame p90 ms | better (-48%) | ≈ | better (-45%) | ≈ | ≈ | better (-48%) |
| frames >2x budget | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |
| tick ms/frame | better (-24%) | ≈ | better (-23%) | ≈ | ≈ | better (-28%) |
| input->first frame ms | n/a | n/a | n/a | n/a | n/a | n/a |
| command p50 ms | worse (+55%) | ≈ | worse (+46%) | ≈ | ≈ | worse (+57%) |

#### Fixed tile order, with CPU load (-CpuLoad 8): frame interval tails

From `perf.log`: per run, the median over its burst reports of interval p99 and max (ms). Then median over rounds, with the rounds' range. 5.75 ms is one vblank, ~11.5 ms one missed.

| scenario | main p99 | main max | nopeeks p99 | nopeeks max | default p99 | default max | morepeeks p99 | morepeeks max |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| move | 15.1 <sub>12.4–16.4</sub> | 15.1 <sub>12.4–16.4</sub> | 13.7 <sub>13.5–13.7</sub> | 13.7 <sub>13.5–13.7</sub> | 19.2 <sub>17.7–24.8</sub> | 19.2 <sub>17.7–24.8</sub> | 19.0 <sub>17.9–20.7</sub> | 19.0 <sub>17.9–20.7</sub> |
| float | 55.7 <sub>24.1–65.3</sub> | 55.7 <sub>39.2–71.7</sub> | 66.1 <sub>63.0–72.8</sub> | 66.1 <sub>63.0–72.8</sub> | 69.0 <sub>62.3–70.6</sub> | 69.0 <sub>62.3–70.6</sub> | 83.9 <sub>80.3–88.8</sub> | 83.9 <sub>80.3–88.8</sub> |
| minimize | 37.9 <sub>36.1–51.4</sub> | 63.5 <sub>50.0–66.7</sub> | 35.7 <sub>25.7–39.4</sub> | 57.1 <sub>47.1–58.9</sub> | 40.9 <sub>36.8–41.0</sub> | 59.3 <sub>55.1–70.2</sub> | 40.4 <sub>39.4–42.6</sub> | 64.4 <sub>58.1–69.2</sub> |
| resize | 61.1 <sub>44.8–62.6</sub> | 64.4 <sub>55.8–76.4</sub> | 53.9 <sub>39.0–64.4</sub> | 57.8 <sub>47.8–64.4</sub> | 82.8 <sub>80.3–83.7</sub> | 82.9 <sub>80.3–83.7</sub> | 84.3 <sub>79.2–86.4</sub> | 84.3 <sub>79.2–86.4</sub> |
| switch | 94.6 <sub>91.1–98.4</sub> | 101.8 <sub>94.8–108.6</sub> | 46.6 <sub>42.3–48.4</sub> | 46.6 <sub>42.3–48.4</sub> | 47.3 <sub>46.5–48.9</sub> | 47.6 <sub>46.5–48.9</sub> | 49.4 <sub>47.5–50.0</sub> | 49.4 <sub>47.5–50.0</sub> |

# Verdict

"Smoother" means interval p90 and/or frame p90. "Costlier" means
`tick` per frame. "Start" means command p50, the stand-in for
input→first frame. Percentages come from the controlled runs (fixed tile
order, 4 rounds), as load 0 / CPU 8, and are only given where the
rounds separate (≥5%). "Same" means no separation. The requested runs
(3 rounds, tile order flipping) agree on every verdict below, with
wider ranges. On `move` they found no difference at all, because there
the differences are a few ms and the position flip drowned them.

| scenario | `default` vs `main` | `default` vs `nopeeks` | `morepeeks` vs `main` | `morepeeks` vs `nopeeks` |
| --- | --- | --- | --- | --- |
| move | same smoothness (both at refresh; frame p90 −31/−38%), costlier, start +56/+58% | same smoothness (frame p90 −31/−32%), costlier, start +33/+35% | as `default` | as `default` |
| float | same (not on peeks); start +34% at load 0 | same | **smoother** (frame p90 −44% at load 0), costlier, start +85/+78% | **smoother** (−28% / −49% at CPU 8), costlier, start +30/+64% |
| minimize | same; start +38% at CPU 8 | **smoother** (frame p90 −33/−27%), slightly costlier | same; start +29% at CPU 8 | **smoother** (−34/−27%), slightly costlier |
| resize | **smoother** (interval p90 −28/−39%, frame p90 −51/−51%), costlier, start +78/+89% | **smoother** (frame p90 −45/−49%; interval −38% at CPU 8), costlier, start +50/+40% | as `default` (start +71/+91%) | as `default` (start +45/+41%) |
| switch | smoother frame p90 (−51/−48%), cheaper, start +48/+55%. Not peeks: a build difference | same | as `default` | same |

- **move**: every mode stays on the refresh rate (interval p90
  5.75–5.77 ms, longest gap 11–19 ms). The peek modes' frame p90 is
  lower (0.6–0.7 vs 1.0–1.2 ms). They spend more per frame and start
  later: 24–26 ms vs 18–19 (`nopeeks`) and 16–17 (`main`). At load 0
  they had 2 frames >2× budget per burst vs 0, but 1 vs 1 under CPU
  load, so that isn't robust.
- **float**: `default` doesn't draw floating↔tiling on peeks
  (`peek_frame` 0) and behaves like the old engine. `morepeeks` does,
  and its frame p90 stays at 3.7–6.0 ms in every round of every run
  set, while the old engine ranges 4–11 ms round to round. That is why
  only some comparisons separate. The cost is the steepest of all
  scenarios: commands 138/163 ms vs 106/100 (`nopeeks`) and 74/91
  (`main`), and a longest gap of 74–107 ms vs 52–66.
- **minimize** (the neighbours re-tile around chrome): `nopeeks` is the
  rough one (frame p90 5.1–5.7 ms). In the controlled runs `main`,
  `default` and `morepeeks` are all at 3.5–4.1 ms, so peeks fix a
  regression the old engine has in the new build rather than beating
  `main`.
- **resize**: the clearest result, beyond noise in all four run sets.
  Peek modes reach interval p90 6.0–7.0 ms and frame p90 4.2–5.4 ms,
  against 8.0–11.8 / 7.7–11.8 ms for `main`/`nopeeks`. The price is
  `tick` +17–44%, plus `peek_frame` 3.3–3.8 ms. Commands take
  130–160 ms, against 90–114 ms for `nopeeks` and 76–85 ms for `main`.
  The longest gap is one 74–95 ms stall, against several shorter ones
  (see Reading the numbers).
- **switch**: no peeks involved. The tryout build, in every mode, halves
  `main`'s frame p90 (1.7–2.2 vs 3.6–4.1 ms) and its longest gap (42–49
  vs 90–102 ms), and uses 20–28% less `tick`. It answers 46–75% later
  (54–65 vs 37–39 ms).
- **The build without peeks** (`nopeeks` vs `main`): smoothness is the
  same or better (`switch`), except `minimize` (frame p90 +55% at load
  0). Per-frame cost is 13–28% lower. Yet commands take 17–75% longer
  (`move` +17/+17%, `float` +42% at load 0, `resize` +18/+35%,
  `switch` +75/+57%). So part of the peek modes' start delay against
  `main` isn't peek drawing at all.

**Overall**: the new peek drawing is **smoother during the motion**
wherever it is used (`resize` always, `float` with `morepeeks`). The
margin is far beyond the noise: roughly half the frame p90, and interval
p90 from 1.4–2 vblanks to 1.05–1.2. It is **not faster**. It costs more
WM time per frame, and it moves the expensive work (repositioning the
real windows, blocked on slow apps) into one long stall instead of
spreading it over the animation. The command that starts the animation
is answered 30–64% later than with the old engine in the same build, and
54–91% later than `main`. On this desktop that is +40–47 ms on a
`resize` vs `nopeeks` and +54–76 ms vs `main`, with 7 windows, some slow
to reposition. 14–30 ms of the latter is the tryout build even with
peeks off. With fewer or faster windows the stall and the delay should
shrink. On the 60 Hz laptop a 75–95 ms stall is ~5 vblanks rather than
~15, but a slower CPU lengthens it, and the start delay is wall-clock
time either way.

Suggested next checks:

1. Run `perf_bench --dwm` (elevated) on `resize` for `nopeeks` vs
   `default`. It would show whether the 74–95 ms stall is visible on
   screen or falls at the hand-over after the motion.
2. Profile why `nopeeks` answers commands 17–75% slower than `main`.
   It isn't peek drawing, and it is the cheapest part of the start
   delay to win back.

# Side finding: tile order after a restart

This was the first interleaved run (4 rounds, no CPU load, tile order
left as each restart made it). It revealed that the tile order **flips
between two states on every restart**: chrome came up leftmost on
restarts 1, 3, 5, …, and rightmost (or next to it) on 2, 4, 6, ….
With 4 modes in turn, every mode therefore always got the same state:

| mode | chrome's position (of 7) in rounds 1–4 |
| --- | --- |
| `main` | 1, 1, 1, 1 |
| `nopeeks` | 7, 7, 7, 7 |
| `default` | 1, 1, 1, 1 |
| `morepeeks` | 7, 7, 7, 6 |

So only **`main`↔`default`** and **`nopeeks`↔`morepeeks`** compare like
with like in the tables below. The other pairs mostly measure chrome's
position. For example, `default` vs `nopeeks` on `move` shows +118%
`tick`, which comes from chrome's leftmost spot (one burst can't move
and the neighbour differs), not from the drawing. The same-position pairs
agree with everything else in this report. Peeks on `resize`: frame p90
−52% (`default` vs `main`) and −51% (`morepeeks` vs `nopeeks`).
Commands start 32–85% later.

The requested runs had the same flip, unlogged for the tryout modes.
With 3 modes, each mode's 3 rounds got a 2:1 mix of the two states
(`nopeeks`/`morepeeks` one way, `default` the other), and so did the
`main` block. That adds noise and could tilt a median, which is why the
fixed-order runs above take precedence where they disagree.

## Restart-order run, no CPU load

Median over rounds (up to 4), round range in small print. Lower is better, except interval p50, which is the refresh period.

### move

| metric | main | nopeeks | default | morepeeks |
| --- | ---: | ---: | ---: | ---: |
| interval p50 ms | 5.71 <sub>5.71–5.71</sub> | 5.71 <sub>5.71–5.71</sub> | 5.71 <sub>5.71–5.72</sub> | 5.71 <sub>5.71–5.72</sub> |
| interval p90 ms | 5.75 <sub>5.75–5.77</sub> | 5.75 <sub>5.75–5.75</sub> | 5.77 <sub>5.76–5.77</sub> | 5.75 <sub>5.75–5.77</sub> |
| frame p90 ms | 1.12 <sub>1.05–1.32</sub> | 0.94 <sub>0.93–0.96</sub> | 1.18 <sub>1.14–1.26</sub> | 0.93 <sub>0.91–2.93</sub> |
| frames >2x budget | 1.00 <sub>1.00–1.00</sub> | 0.00 <sub>0.00–0.00</sub> | 1.00 <sub>1.00–2.00</sub> | 1.00 <sub>1.00–1.00</sub> |
| tick ms/frame | 0.94 <sub>0.88–1.24</sub> | 0.54 <sub>0.53–0.55</sub> | 1.17 <sub>1.03–1.25</sub> | 0.84 <sub>0.83–1.44</sub> |
| peek_frame ms/frame | n/a | n/a | 0.50 <sub>0.48–0.52</sub> | 0.38 <sub>0.37–0.54</sub> |
| input->first frame ms | 27.77 <sub>25.11–46.70</sub> | 18.56 <sub>18.04–18.85</sub> | n/a | n/a |
| command p50 ms | 22.75 <sub>22.30–37.24</sub> | 17.84 <sub>17.50–18.37</sub> | 32.11 <sub>26.13–37.82</sub> | 23.57 <sub>22.63–32.15</sub> |

| vs (ranges must not overlap) | default vs main | default vs nopeeks | morepeeks vs main | morepeeks vs nopeeks | morepeeks vs default | nopeeks vs main |
| --- | --- | --- | --- | --- | --- | --- |
| interval p90 ms | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |
| frame p90 ms | ≈ | worse (+25%) | ≈ | ≈ | ≈ | better (-16%) |
| frames >2x budget | ≈ | worse (1.00 vs 0) | ≈ | worse (1.00 vs 0) | ≈ | better (-100%) |
| tick ms/frame | ≈ | worse (+118%) | ≈ | worse (+57%) | ≈ | better (-43%) |
| input->first frame ms | n/a | n/a | n/a | n/a | n/a | better (-33%) |
| command p50 ms | ≈ | worse (+80%) | ≈ | worse (+32%) | ≈ | better (-22%) |

### float

| metric | main | nopeeks | default | morepeeks |
| --- | ---: | ---: | ---: | ---: |
| interval p50 ms | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.71–5.72</sub> |
| interval p90 ms | 9.50 <sub>7.81–9.78</sub> | 9.64 <sub>8.55–10.17</sub> | 10.03 <sub>6.99–10.75</sub> | 6.10 <sub>5.80–7.08</sub> |
| frame p90 ms | 8.36 <sub>6.48–9.45</sub> | 8.95 <sub>7.66–9.95</sub> | 9.80 <sub>5.42–10.64</sub> | 4.05 <sub>3.86–4.51</sub> |
| frames >2x budget | 4.00 <sub>4.00–4.00</sub> | 4.00 <sub>4.00–5.00</sub> | 4.50 <sub>4.00–5.00</sub> | 5.00 <sub>5.00–5.00</sub> |
| tick ms/frame | 4.12 <sub>3.59–4.16</sub> | 3.59 <sub>3.46–3.74</sub> | 4.45 <sub>3.69–4.82</sub> | 5.08 <sub>4.95–5.42</sub> |
| peek_frame ms/frame | n/a | n/a | 0.00 <sub>0.00–0.00</sub> | 3.08 <sub>3.02–3.13</sub> |
| input->first frame ms | 85.97 <sub>77.58–109.29</sub> | 112.25 <sub>92.11–121.83</sub> | n/a | n/a |
| command p50 ms | 76.66 <sub>75.65–77.56</sub> | 99.38 <sub>78.51–101.31</sub> | 103.16 <sub>97.29–107.76</sub> | 128.56 <sub>126.46–130.37</sub> |

| vs (ranges must not overlap) | default vs main | default vs nopeeks | morepeeks vs main | morepeeks vs nopeeks | morepeeks vs default | nopeeks vs main |
| --- | --- | --- | --- | --- | --- | --- |
| interval p90 ms | ≈ | ≈ | better (-36%) | better (-37%) | ≈ | ≈ |
| frame p90 ms | ≈ | ≈ | better (-52%) | better (-55%) | better (-59%) | ≈ |
| frames >2x budget | ≈ | ≈ | worse (+25%) | ≈ | ≈ | ≈ |
| tick ms/frame | ≈ | ≈ | worse (+23%) | worse (+42%) | worse (+14%) | ≈ |
| input->first frame ms | n/a | n/a | n/a | n/a | n/a | ≈ |
| command p50 ms | worse (+35%) | ≈ | worse (+68%) | worse (+29%) | worse (+25%) | worse (+30%) |

### minimize

| metric | main | nopeeks | default | morepeeks |
| --- | ---: | ---: | ---: | ---: |
| interval p50 ms | 5.72 <sub>5.71–5.72</sub> | 5.72 <sub>5.71–5.72</sub> | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.71–5.72</sub> |
| interval p90 ms | 5.79 <sub>5.78–5.80</sub> | 5.81 <sub>5.80–6.38</sub> | 5.79 <sub>5.78–5.79</sub> | 5.78 <sub>5.77–5.99</sub> |
| frame p90 ms | 4.46 <sub>3.47–4.64</sub> | 5.21 <sub>4.58–6.07</sub> | 3.85 <sub>3.53–4.19</sub> | 3.98 <sub>3.82–4.14</sub> |
| frames >2x budget | 5.50 <sub>5.00–6.00</sub> | 5.00 <sub>5.00–5.00</sub> | 5.00 <sub>5.00–5.00</sub> | 5.50 <sub>5.00–6.00</sub> |
| tick ms/frame | 1.87 <sub>1.70–1.95</sub> | 1.59 <sub>1.52–1.60</sub> | 1.81 <sub>1.79–1.86</sub> | 2.26 <sub>2.17–4.85</sub> |
| peek_frame ms/frame | n/a | n/a | 1.06 <sub>0.71–1.29</sub> | 0.90 <sub>0.00–2.90</sub> |
| input->first frame ms | 117.72 <sub>102.40–131.58</sub> | 117.79 <sub>109.84–128.40</sub> | n/a | n/a |
| command p50 ms | 46.92 <sub>26.50–48.38</sub> | 54.64 <sub>36.71–60.25</sub> | 67.27 <sub>63.11–69.08</sub> | 71.04 <sub>65.69–119.25</sub> |

| vs (ranges must not overlap) | default vs main | default vs nopeeks | morepeeks vs main | morepeeks vs nopeeks | morepeeks vs default | nopeeks vs main |
| --- | --- | --- | --- | --- | --- | --- |
| interval p90 ms | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |
| frame p90 ms | ≈ | better (-26%) | ≈ | better (-24%) | ≈ | ≈ |
| frames >2x budget | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |
| tick ms/frame | ≈ | worse (+14%) | worse (+21%) | worse (+42%) | worse (+25%) | better (-15%) |
| input->first frame ms | n/a | n/a | n/a | n/a | n/a | ≈ |
| command p50 ms | worse (+43%) | worse (+23%) | worse (+51%) | worse (+30%) | ≈ | ≈ |

### resize

| metric | main | nopeeks | default | morepeeks |
| --- | ---: | ---: | ---: | ---: |
| interval p50 ms | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.72–5.72</sub> | 5.71 <sub>5.71–5.72</sub> | 5.72 <sub>5.71–5.72</sub> |
| interval p90 ms | 9.50 <sub>5.78–11.45</sub> | 9.49 <sub>8.22–9.66</sub> | 6.25 <sub>5.94–7.10</sub> | 6.42 <sub>6.15–6.83</sub> |
| frame p90 ms | 9.23 <sub>5.45–10.64</sub> | 8.85 <sub>7.39–9.58</sub> | 4.47 <sub>4.38–4.52</sub> | 4.32 <sub>3.88–4.40</sub> |
| frames >2x budget | 4.50 <sub>4.00–5.00</sub> | 4.00 <sub>4.00–5.00</sub> | 5.00 <sub>4.00–5.00</sub> | 5.00 <sub>5.00–5.00</sub> |
| tick ms/frame | 4.38 <sub>3.15–4.92</sub> | 3.69 <sub>3.43–3.89</sub> | 5.08 <sub>4.90–5.43</sub> | 5.23 <sub>5.17–5.39</sub> |
| peek_frame ms/frame | n/a | n/a | 3.12 <sub>3.02–3.25</sub> | 3.21 <sub>3.16–3.25</sub> |
| input->first frame ms | 80.31 <sub>73.81–102.09</sub> | 110.67 <sub>93.85–117.72</sub> | n/a | n/a |
| command p50 ms | 72.09 <sub>52.68–81.82</sub> | 95.61 <sub>86.29–106.40</sub> | 133.45 <sub>116.75–136.18</sub> | 125.78 <sub>111.52–133.86</sub> |

| vs (ranges must not overlap) | default vs main | default vs nopeeks | morepeeks vs main | morepeeks vs nopeeks | morepeeks vs default | nopeeks vs main |
| --- | --- | --- | --- | --- | --- | --- |
| interval p90 ms | ≈ | better (-34%) | ≈ | better (-32%) | ≈ | ≈ |
| frame p90 ms | better (-52%) | better (-49%) | better (-53%) | better (-51%) | ≈ | ≈ |
| frames >2x budget | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |
| tick ms/frame | ≈ | worse (+38%) | worse (+20%) | worse (+42%) | ≈ | ≈ |
| input->first frame ms | n/a | n/a | n/a | n/a | n/a | ≈ |
| command p50 ms | worse (+85%) | worse (+40%) | worse (+74%) | worse (+32%) | ≈ | worse (+33%) |

### switch

| metric | main | nopeeks | default | morepeeks |
| --- | ---: | ---: | ---: | ---: |
| interval p50 ms | 5.71 <sub>5.71–5.72</sub> | 5.71 <sub>5.71–5.72</sub> | 5.71 <sub>5.71–5.72</sub> | 5.72 <sub>5.71–5.72</sub> |
| interval p90 ms | 5.75 <sub>5.75–5.76</sub> | 5.77 <sub>5.77–5.77</sub> | 5.77 <sub>5.76–5.78</sub> | 5.77 <sub>5.76–5.77</sub> |
| frame p90 ms | 3.56 <sub>3.39–3.68</sub> | 1.75 <sub>1.56–1.91</sub> | 1.67 <sub>1.60–1.83</sub> | 2.02 <sub>1.78–2.03</sub> |
| frames >2x budget | 3.00 <sub>2.00–3.00</sub> | 3.00 <sub>3.00–3.00</sub> | 3.00 <sub>3.00–4.00</sub> | 4.00 <sub>3.00–4.00</sub> |
| tick ms/frame | 2.30 <sub>2.18–2.42</sub> | 1.81 <sub>1.73–1.97</sub> | 1.67 <sub>1.65–1.73</sub> | 2.17 <sub>1.87–2.21</sub> |
| peek_frame ms/frame | n/a | n/a | 0.00 <sub>0.00–0.00</sub> | 0.00 <sub>0.00–0.00</sub> |
| input->first frame ms | n/a | n/a | n/a | n/a |
| command p50 ms | 46.17 <sub>37.70–51.59</sub> | 66.34 <sub>60.20–78.94</sub> | 72.48 <sub>65.61–74.29</sub> | 65.52 <sub>53.59–68.87</sub> |

| vs (ranges must not overlap) | default vs main | default vs nopeeks | morepeeks vs main | morepeeks vs nopeeks | morepeeks vs default | nopeeks vs main |
| --- | --- | --- | --- | --- | --- | --- |
| interval p90 ms | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |
| frame p90 ms | better (-53%) | ≈ | better (-43%) | ≈ | ≈ | better (-51%) |
| frames >2x budget | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |
| tick ms/frame | better (-27%) | ≈ | ≈ | ≈ | worse (+30%) | better (-22%) |
| input->first frame ms | n/a | n/a | n/a | n/a | n/a | n/a |
| command p50 ms | worse (+57%) | ≈ | worse (+42%) | ≈ | ≈ | worse (+44%) |

# Raw results

## isolate-bench.txt, no CPU load (21:05)

```text
Isolate benchmark, 2026-11-12T21:05:57, target chrome, 3 round(s) of 10 bursts per scenario and mode, load 0, CPU load 0 thread(s) of 16
label=nopeeks #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          23.95
  command p90 ms                          25.60
  frames                                  63.00
  tick ms/frame                            0.82
  frame p90 ms                             1.22
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.03
  batch_commit ms/frame                    0.38
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            7.40
  cloak ms (outside frames)               11.80
  session_begin ms (outside frames)        1.00
  input->anim start ms                     0.30
  input->first frame ms                   28.18
| nopeeks #1 | move | 23.95 | 25.60 | 63.00 | 0.82 | 1.22 | 1.00 | 5.71 | 5.75 | 0.00 | 0.03 | 0.38 | 0.03 | 0.00 | 7.40 | 11.80 | 1.00 | 0.30 | 28.18 |
label=nopeeks #1 scenario=float target=chrome bursts=10 load=0 reports=10 wall=13.0s
  command p50 ms                         113.47
  command p90 ms                         115.82
  frames                                  60.00
  tick ms/frame                            3.40
  frame p90 ms                             8.55
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          9.18
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.07
  batch_commit ms/frame                    1.83
  ovl_region ms/frame                      0.10
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            9.50
  cloak ms (outside frames)               41.90
  session_begin ms (outside frames)        2.80
  input->anim start ms                     0.28
  input->first frame ms                  121.69
| nopeeks #1 | float | 113.47 | 115.82 | 60.00 | 3.40 | 8.55 | 4.00 | 5.72 | 9.18 | 0.00 | 0.07 | 1.83 | 0.10 | 0.00 | 9.50 | 41.90 | 2.80 | 0.28 | 121.69 |
label=nopeeks #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          63.57
  command p90 ms                          70.20
  frames                                 145.00
  tick ms/frame                            1.40
  frame p90 ms                             4.65
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          6.22
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.75
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)           12.00
  cloak ms (outside frames)               42.00
  session_begin ms (outside frames)        3.50
  input->anim start ms                     3.68
  input->first frame ms                  128.89
| nopeeks #1 | minimize | 63.57 | 70.20 | 145.00 | 1.40 | 4.65 | 5.00 | 5.72 | 6.22 | 0.00 | 0.02 | 0.75 | 0.04 | 1.00 | 12.00 | 42.00 | 3.50 | 3.68 | 128.89 |
label=nopeeks #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=13.1s
  command p50 ms                         107.89
  command p90 ms                         110.18
  frames                                  58.00
  tick ms/frame                            4.05
  frame p90 ms                             8.80
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          9.10
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.06
  batch_commit ms/frame                    2.19
  ovl_region ms/frame                      0.12
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           11.20
  cloak ms (outside frames)               40.60
  session_begin ms (outside frames)        2.60
  input->anim start ms                     0.30
  input->first frame ms                  117.91
| nopeeks #1 | resize | 107.89 | 110.18 | 58.00 | 4.05 | 8.80 | 4.00 | 5.72 | 9.10 | 0.00 | 0.06 | 2.19 | 0.12 | 0.00 | 11.20 | 40.60 | 2.60 | 0.30 | 117.91 |
Moved 3 window(s) to workspace '2' to switch to.
label=nopeeks #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          55.89
  command p90 ms                          68.00
  frames                                  99.00
  tick ms/frame                            1.95
  frame p90 ms                             1.92
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            8.20
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| nopeeks #1 | switch | 55.89 | 68.00 | 99.00 | 1.95 | 1.92 | 3.00 | 5.72 | 5.76 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 8.20 | 0.00 | 0.00 | 0.00 | 0.00 |
label=default #1 scenario=move target=chrome bursts=10 load=0 reports=9 wall=12.4s
  command p50 ms                          37.30
  command p90 ms                          39.54
  frames                                  64.00
  tick ms/frame                            1.25
  frame p90 ms                             1.25
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.51
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            9.90
  cloak ms (outside frames)               17.20
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #1 | move | 37.30 | 39.54 | 64.00 | 1.25 | 1.25 | 1.00 | 5.71 | 5.77 | 0.51 | 0.00 | 0.00 | 0.00 | 0.00 | 9.90 | 17.20 | 0.00 | 0.00 | 0.00 |
label=default #1 scenario=float target=chrome bursts=10 load=0 reports=10 wall=13.0s
  command p50 ms                          92.17
  command p90 ms                         111.15
  frames                                  60.00
  tick ms/frame                            4.15
  frame p90 ms                             6.30
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          9.64
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.05
  batch_commit ms/frame                    1.43
  ovl_region ms/frame                      0.15
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           13.00
  cloak ms (outside frames)               33.20
  session_begin ms (outside frames)        2.60
  input->anim start ms                     0.29
  input->first frame ms                  100.58
| default #1 | float | 92.17 | 111.15 | 60.00 | 4.15 | 6.30 | 4.00 | 5.72 | 9.64 | 0.00 | 0.05 | 1.43 | 0.15 | 0.00 | 13.00 | 33.20 | 2.60 | 0.29 | 100.58 |
label=default #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          40.46
  command p90 ms                          72.27
  frames                                 133.00
  tick ms/frame                            1.76
  frame p90 ms                             3.67
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          5.78
  peek_frame ms/frame                      1.18
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.59
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)           11.40
  cloak ms (outside frames)               35.00
  session_begin ms (outside frames)        2.60
  input->anim start ms                    27.48
  input->first frame ms                  176.00
| default #1 | minimize | 40.46 | 72.27 | 133.00 | 1.76 | 3.67 | 5.00 | 5.72 | 5.78 | 1.18 | 0.02 | 0.59 | 0.04 | 1.00 | 11.40 | 35.00 | 2.60 | 27.48 | 176.00 |
label=default #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=13.5s
  command p50 ms                         135.46
  command p90 ms                         173.72
  frames                                  59.00
  tick ms/frame                            5.62
  frame p90 ms                             4.92
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          6.96
  peek_frame ms/frame                      3.34
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.07
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           19.50
  cloak ms (outside frames)               40.60
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #1 | resize | 135.46 | 173.72 | 59.00 | 5.62 | 4.92 | 5.00 | 5.72 | 6.96 | 3.34 | 0.00 | 0.00 | 0.07 | 2.00 | 19.50 | 40.60 | 0.00 | 0.00 | 0.00 |
Moved 3 window(s) to workspace '2' to switch to.
label=default #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.8s
  command p50 ms                          80.76
  command p90 ms                         102.27
  frames                                  99.00
  tick ms/frame                            2.09
  frame p90 ms                             2.03
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          5.78
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            7.10
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #1 | switch | 80.76 | 102.27 | 99.00 | 2.09 | 2.03 | 4.00 | 5.72 | 5.78 | 0.00 | 0.00 | 0.00 | 0.04 | 0.00 | 7.10 | 0.00 | 0.00 | 0.00 | 0.00 |
label=morepeeks #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          23.99
  command p90 ms                          25.17
  frames                                  63.00
  tick ms/frame                            0.96
  frame p90 ms                             1.05
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.42
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            6.50
  cloak ms (outside frames)                9.40
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #1 | move | 23.99 | 25.17 | 63.00 | 0.96 | 1.05 | 1.00 | 5.71 | 5.77 | 0.42 | 0.00 | 0.00 | 0.00 | 0.00 | 6.50 | 9.40 | 0.00 | 0.00 | 0.00 |
label=morepeeks #1 scenario=float target=chrome bursts=10 load=0 reports=10 wall=13.3s
  command p50 ms                         143.97
  command p90 ms                         150.27
  frames                                  56.00
  tick ms/frame                            5.66
  frame p90 ms                             4.89
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          6.40
  peek_frame ms/frame                      3.32
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.09
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           16.40
  cloak ms (outside frames)               37.20
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #1 | float | 143.97 | 150.27 | 56.00 | 5.66 | 4.89 | 5.00 | 5.72 | 6.40 | 3.32 | 0.00 | 0.00 | 0.09 | 2.00 | 16.40 | 37.20 | 0.00 | 0.00 | 0.00 |
label=morepeeks #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          67.99
  command p90 ms                          83.31
  frames                                 128.00
  tick ms/frame                            2.53
  frame p90 ms                             3.90
  frames >2x budget                        7.00
  interval p50 ms                          5.72
  interval p90 ms                          5.80
  peek_frame ms/frame                      1.32
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.62
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)           14.00
  cloak ms (outside frames)               34.90
  session_begin ms (outside frames)        2.70
  input->anim start ms                     3.98
  input->first frame ms                  220.01
| morepeeks #1 | minimize | 67.99 | 83.31 | 128.00 | 2.53 | 3.90 | 7.00 | 5.72 | 5.80 | 1.32 | 0.02 | 0.62 | 0.04 | 1.00 | 14.00 | 34.90 | 2.70 | 3.98 | 220.01 |
label=morepeeks #1 scenario=resize target=chrome bursts=10 load=0 reports=9 wall=13.3s
  command p50 ms                         125.53
  command p90 ms                         146.14
  frames                                  57.00
  tick ms/frame                            5.41
  frame p90 ms                             4.41
  frames >2x budget                        5.00
  interval p50 ms                          5.71
  interval p90 ms                          5.90
  peek_frame ms/frame                      3.40
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.06
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           12.10
  cloak ms (outside frames)               32.40
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #1 | resize | 125.53 | 146.14 | 57.00 | 5.41 | 4.41 | 5.00 | 5.71 | 5.90 | 3.40 | 0.00 | 0.00 | 0.06 | 2.00 | 12.10 | 32.40 | 0.00 | 0.00 | 0.00 |
Moved 3 window(s) to workspace '2' to switch to.
label=morepeeks #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          58.22
  command p90 ms                          74.68
  frames                                  99.00
  tick ms/frame                            1.83
  frame p90 ms                             2.27
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            6.60
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #1 | switch | 58.22 | 74.68 | 99.00 | 1.83 | 2.27 | 4.00 | 5.72 | 5.77 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 6.60 | 0.00 | 0.00 | 0.00 | 0.00 |
label=nopeeks #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          18.23
  command p90 ms                          19.26
  frames                                  62.00
  tick ms/frame                            0.61
  frame p90 ms                             0.99
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.27
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            6.80
  cloak ms (outside frames)               11.50
  session_begin ms (outside frames)        1.00
  input->anim start ms                     0.30
  input->first frame ms                   18.76
| nopeeks #2 | move | 18.23 | 19.26 | 62.00 | 0.61 | 0.99 | 0.00 | 5.71 | 5.76 | 0.00 | 0.02 | 0.27 | 0.02 | 0.00 | 6.80 | 11.50 | 1.00 | 0.30 | 18.76 |
label=nopeeks #2 scenario=float target=chrome bursts=10 load=0 reports=10 wall=13.1s
  command p50 ms                         102.17
  command p90 ms                         114.74
  frames                                  59.00
  tick ms/frame                            3.68
  frame p90 ms                             9.39
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                         10.18
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.06
  batch_commit ms/frame                    2.22
  ovl_region ms/frame                      0.12
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           12.20
  cloak ms (outside frames)               37.00
  session_begin ms (outside frames)        2.90
  input->anim start ms                     0.28
  input->first frame ms                  119.23
| nopeeks #2 | float | 102.17 | 114.74 | 59.00 | 3.68 | 9.39 | 3.00 | 5.72 | 10.18 | 0.00 | 0.06 | 2.22 | 0.12 | 0.00 | 12.20 | 37.00 | 2.90 | 0.28 | 119.23 |
label=nopeeks #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          62.12
  command p90 ms                          74.85
  frames                                 135.00
  tick ms/frame                            1.75
  frame p90 ms                             5.66
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          5.98
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.72
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            9.20
  cloak ms (outside frames)               29.80
  session_begin ms (outside frames)        2.90
  input->anim start ms                     3.88
  input->first frame ms                  117.13
| nopeeks #2 | minimize | 62.12 | 74.85 | 135.00 | 1.75 | 5.66 | 5.00 | 5.72 | 5.98 | 0.00 | 0.02 | 0.72 | 0.05 | 1.00 | 9.20 | 29.80 | 2.90 | 3.88 | 117.13 |
label=nopeeks #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=13.1s
  command p50 ms                         103.26
  command p90 ms                         112.07
  frames                                  58.00
  tick ms/frame                            4.15
  frame p90 ms                             9.83
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                         10.66
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.06
  batch_commit ms/frame                    2.37
  ovl_region ms/frame                      0.11
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           10.70
  cloak ms (outside frames)               34.30
  session_begin ms (outside frames)        2.80
  input->anim start ms                     0.30
  input->first frame ms                  114.07
| nopeeks #2 | resize | 103.26 | 112.07 | 58.00 | 4.15 | 9.83 | 4.00 | 5.72 | 10.66 | 0.00 | 0.06 | 2.37 | 0.11 | 0.00 | 10.70 | 34.30 | 2.80 | 0.30 | 114.07 |
Moved 3 window(s) to workspace '2' to switch to.
label=nopeeks #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          66.86
  command p90 ms                          81.01
  frames                                  99.00
  tick ms/frame                            2.05
  frame p90 ms                             1.84
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.79
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            7.40
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| nopeeks #2 | switch | 66.86 | 81.01 | 99.00 | 2.05 | 1.84 | 3.00 | 5.71 | 5.79 | 0.00 | 0.00 | 0.00 | 0.02 | 0.00 | 7.40 | 0.00 | 0.00 | 0.00 | 0.00 |
label=default #2 scenario=move target=chrome bursts=10 load=0 reports=9 wall=12.5s
  command p50 ms                          40.78
  command p90 ms                          45.08
  frames                                  64.00
  tick ms/frame                            1.52
  frame p90 ms                             1.50
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.61
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           11.60
  cloak ms (outside frames)               19.20
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #2 | move | 40.78 | 45.08 | 64.00 | 1.52 | 1.50 | 2.00 | 5.71 | 5.77 | 0.61 | 0.00 | 0.00 | 0.00 | 0.00 | 11.60 | 19.20 | 0.00 | 0.00 | 0.00 |
label=default #2 scenario=float target=chrome bursts=10 load=0 reports=10 wall=13.1s
  command p50 ms                         111.12
  command p90 ms                         124.28
  frames                                  59.00
  tick ms/frame                            4.94
  frame p90 ms                            10.56
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                         10.86
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.05
  batch_commit ms/frame                    2.28
  ovl_region ms/frame                      0.15
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           11.80
  cloak ms (outside frames)               37.80
  session_begin ms (outside frames)        2.70
  input->anim start ms                     0.31
  input->first frame ms                  120.47
| default #2 | float | 111.12 | 124.28 | 59.00 | 4.94 | 10.56 | 5.00 | 5.72 | 10.86 | 0.00 | 0.05 | 2.28 | 0.15 | 0.00 | 11.80 | 37.80 | 2.70 | 0.31 | 120.47 |
label=default #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          66.23
  command p90 ms                          68.32
  frames                                 135.00
  tick ms/frame                            1.73
  frame p90 ms                             4.02
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          5.77
  peek_frame ms/frame                      1.20
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.56
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)           10.40
  cloak ms (outside frames)               30.80
  session_begin ms (outside frames)        2.40
  input->anim start ms                    37.42
  input->first frame ms                  202.57
| default #2 | minimize | 66.23 | 68.32 | 135.00 | 1.73 | 4.02 | 5.00 | 5.72 | 5.77 | 1.20 | 0.02 | 0.56 | 0.05 | 1.00 | 10.40 | 30.80 | 2.40 | 37.42 | 202.57 |
label=default #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=13.4s
  command p50 ms                         133.39
  command p90 ms                         139.00
  frames                                  57.00
  tick ms/frame                            5.17
  frame p90 ms                             4.40
  frames >2x budget                        5.00
  interval p50 ms                          5.71
  interval p90 ms                          5.81
  peek_frame ms/frame                      3.42
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.09
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            7.80
  cloak ms (outside frames)               35.10
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #2 | resize | 133.39 | 139.00 | 57.00 | 5.17 | 4.40 | 5.00 | 5.71 | 5.81 | 3.42 | 0.00 | 0.00 | 0.09 | 2.00 | 7.80 | 35.10 | 0.00 | 0.00 | 0.00 |
Moved 3 window(s) to workspace '2' to switch to.
label=default #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          68.97
  command p90 ms                          78.45
  frames                                  99.00
  tick ms/frame                            1.68
  frame p90 ms                             1.69
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            9.10
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #2 | switch | 68.97 | 78.45 | 99.00 | 1.68 | 1.69 | 3.00 | 5.71 | 5.77 | 0.00 | 0.00 | 0.00 | 0.02 | 0.00 | 9.10 | 0.00 | 0.00 | 0.00 | 0.00 |
label=morepeeks #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          23.48
  command p90 ms                          24.68
  frames                                  63.00
  tick ms/frame                            0.88
  frame p90 ms                             0.90
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  peek_frame ms/frame                      0.38
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            7.50
  cloak ms (outside frames)               10.70
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #2 | move | 23.48 | 24.68 | 63.00 | 0.88 | 0.90 | 0.00 | 5.71 | 5.76 | 0.38 | 0.00 | 0.00 | 0.00 | 0.00 | 7.50 | 10.70 | 0.00 | 0.00 | 0.00 |
label=morepeeks #2 scenario=float target=chrome bursts=10 load=0 reports=10 wall=13.3s
  command p50 ms                         133.00
  command p90 ms                         144.12
  frames                                  58.00
  tick ms/frame                            4.95
  frame p90 ms                             4.16
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          6.48
  peek_frame ms/frame                      3.28
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.07
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            8.60
  cloak ms (outside frames)               30.40
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #2 | float | 133.00 | 144.12 | 58.00 | 4.95 | 4.16 | 5.00 | 5.72 | 6.48 | 3.28 | 0.00 | 0.00 | 0.07 | 2.00 | 8.60 | 30.40 | 0.00 | 0.00 | 0.00 |
label=morepeeks #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          68.08
  command p90 ms                          78.80
  frames                                 135.00
  tick ms/frame                            2.21
  frame p90 ms                             3.76
  frames >2x budget                        6.00
  interval p50 ms                          5.71
  interval p90 ms                          5.79
  peek_frame ms/frame                      1.19
  session_overlays ms/frame                0.03
  batch_commit ms/frame                    0.54
  ovl_region ms/frame                      0.06
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)           13.60
  cloak ms (outside frames)               35.50
  session_begin ms (outside frames)        2.60
  input->anim start ms                     4.04
  input->first frame ms                  170.93
| morepeeks #2 | minimize | 68.08 | 78.80 | 135.00 | 2.21 | 3.76 | 6.00 | 5.71 | 5.79 | 1.19 | 0.03 | 0.54 | 0.06 | 1.00 | 13.60 | 35.50 | 2.60 | 4.04 | 170.93 |
label=morepeeks #2 scenario=resize target=chrome bursts=10 load=0 reports=9 wall=13.4s
  command p50 ms                         141.68
  command p90 ms                         156.55
  frames                                  57.00
  tick ms/frame                            5.06
  frame p90 ms                             4.24
  frames >2x budget                        5.00
  interval p50 ms                          5.71
  interval p90 ms                          5.98
  peek_frame ms/frame                      3.36
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.07
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            9.30
  cloak ms (outside frames)               43.40
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #2 | resize | 141.68 | 156.55 | 57.00 | 5.06 | 4.24 | 5.00 | 5.71 | 5.98 | 3.36 | 0.00 | 0.00 | 0.07 | 2.00 | 9.30 | 43.40 | 0.00 | 0.00 | 0.00 |
Moved 3 window(s) to workspace '2' to switch to.
label=morepeeks #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          71.60
  command p90 ms                          89.36
  frames                                 100.00
  tick ms/frame                            2.17
  frame p90 ms                             1.94
  frames >2x budget                        4.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            7.80
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #2 | switch | 71.60 | 89.36 | 100.00 | 2.17 | 1.94 | 4.00 | 5.71 | 5.76 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 7.80 | 0.00 | 0.00 | 0.00 | 0.00 |
label=nopeeks #3 scenario=move target=chrome bursts=10 load=0 reports=9 wall=12.5s
  command p50 ms                          42.35
  command p90 ms                          44.45
  frames                                  63.00
  tick ms/frame                            1.09
  frame p90 ms                             1.00
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.03
  batch_commit ms/frame                    0.39
  ovl_region ms/frame                      0.07
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            9.60
  cloak ms (outside frames)               16.40
  session_begin ms (outside frames)        1.00
  input->anim start ms                     0.25
  input->first frame ms                   50.15
| nopeeks #3 | move | 42.35 | 44.45 | 63.00 | 1.09 | 1.00 | 1.00 | 5.71 | 5.76 | 0.00 | 0.03 | 0.39 | 0.07 | 0.00 | 9.60 | 16.40 | 1.00 | 0.25 | 50.15 |
label=nopeeks #3 scenario=float target=chrome bursts=10 load=0 reports=10 wall=13.0s
  command p50 ms                          99.40
  command p90 ms                         105.35
  frames                                  59.00
  tick ms/frame                            3.61
  frame p90 ms                             8.93
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          9.01
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.05
  batch_commit ms/frame                    2.21
  ovl_region ms/frame                      0.13
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            8.30
  cloak ms (outside frames)               33.20
  session_begin ms (outside frames)        2.70
  input->anim start ms                     0.30
  input->first frame ms                  119.26
| nopeeks #3 | float | 99.40 | 105.35 | 59.00 | 3.61 | 8.93 | 4.00 | 5.72 | 9.01 | 0.00 | 0.05 | 2.21 | 0.13 | 0.00 | 8.30 | 33.20 | 2.70 | 0.30 | 119.26 |
label=nopeeks #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          59.82
  command p90 ms                          69.78
  frames                                 137.00
  tick ms/frame                            1.56
  frame p90 ms                             5.73
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          6.37
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.86
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)           12.00
  cloak ms (outside frames)               39.80
  session_begin ms (outside frames)        3.90
  input->anim start ms                     3.97
  input->first frame ms                  114.08
| nopeeks #3 | minimize | 59.82 | 69.78 | 137.00 | 1.56 | 5.73 | 5.00 | 5.72 | 6.37 | 0.00 | 0.02 | 0.86 | 0.04 | 1.00 | 12.00 | 39.80 | 3.90 | 3.97 | 114.08 |
label=nopeeks #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=13.0s
  command p50 ms                          97.10
  command p90 ms                         106.71
  frames                                  58.00
  tick ms/frame                            3.36
  frame p90 ms                             8.14
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          9.24
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.05
  batch_commit ms/frame                    2.19
  ovl_region ms/frame                      0.08
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           11.60
  cloak ms (outside frames)               29.70
  session_begin ms (outside frames)        2.60
  input->anim start ms                     0.28
  input->first frame ms                  117.57
| nopeeks #3 | resize | 97.10 | 106.71 | 58.00 | 3.36 | 8.14 | 4.00 | 5.72 | 9.24 | 0.00 | 0.05 | 2.19 | 0.08 | 0.00 | 11.60 | 29.70 | 2.60 | 0.28 | 117.57 |
Moved 3 window(s) to workspace '2' to switch to.
label=nopeeks #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          77.90
  command p90 ms                          83.28
  frames                                  99.00
  tick ms/frame                            1.93
  frame p90 ms                             1.90
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            9.60
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| nopeeks #3 | switch | 77.90 | 83.28 | 99.00 | 1.93 | 1.90 | 3.00 | 5.71 | 5.76 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 9.60 | 0.00 | 0.00 | 0.00 | 0.00 |
label=default #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          22.79
  command p90 ms                          25.27
  frames                                  63.00
  tick ms/frame                            0.86
  frame p90 ms                             0.91
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  peek_frame ms/frame                      0.38
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            6.30
  cloak ms (outside frames)                9.50
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #3 | move | 22.79 | 25.27 | 63.00 | 0.86 | 0.91 | 1.00 | 5.71 | 5.76 | 0.38 | 0.00 | 0.00 | 0.00 | 0.00 | 6.30 | 9.50 | 0.00 | 0.00 | 0.00 |
label=default #3 scenario=float target=chrome bursts=10 load=0 reports=10 wall=13.0s
  command p50 ms                          99.28
  command p90 ms                         110.94
  frames                                  59.00
  tick ms/frame                            3.90
  frame p90 ms                             9.44
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          9.52
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.05
  batch_commit ms/frame                    1.34
  ovl_region ms/frame                      0.14
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           13.70
  cloak ms (outside frames)               28.90
  session_begin ms (outside frames)        2.50
  input->anim start ms                     0.34
  input->first frame ms                  110.12
| default #3 | float | 99.28 | 110.94 | 59.00 | 3.90 | 9.44 | 4.00 | 5.72 | 9.52 | 0.00 | 0.05 | 1.34 | 0.14 | 0.00 | 13.70 | 28.90 | 2.50 | 0.34 | 110.12 |
label=default #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          62.36
  command p90 ms                          70.25
  frames                                 134.00
  tick ms/frame                            2.26
  frame p90 ms                             4.05
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          5.77
  peek_frame ms/frame                      1.20
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.55
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)           10.60
  cloak ms (outside frames)               30.00
  session_begin ms (outside frames)        2.60
  input->anim start ms                     4.20
  input->first frame ms                  196.96
| default #3 | minimize | 62.36 | 70.25 | 134.00 | 2.26 | 4.05 | 5.00 | 5.72 | 5.77 | 1.20 | 0.02 | 0.55 | 0.04 | 1.00 | 10.60 | 30.00 | 2.60 | 4.20 | 196.96 |
label=default #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=13.4s
  command p50 ms                         134.21
  command p90 ms                         136.97
  frames                                  56.00
  tick ms/frame                            5.93
  frame p90 ms                             3.92
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          6.77
  peek_frame ms/frame                      3.37
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.09
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            8.00
  cloak ms (outside frames)               38.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #3 | resize | 134.21 | 136.97 | 56.00 | 5.93 | 3.92 | 5.00 | 5.72 | 6.77 | 3.37 | 0.00 | 0.00 | 0.09 | 2.00 | 8.00 | 38.00 | 0.00 | 0.00 | 0.00 |
Moved 3 window(s) to workspace '2' to switch to.
label=default #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          51.18
  command p90 ms                          79.89
  frames                                  99.00
  tick ms/frame                            1.98
  frame p90 ms                             1.75
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            7.40
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #3 | switch | 51.18 | 79.89 | 99.00 | 1.98 | 1.75 | 3.00 | 5.72 | 5.77 | 0.00 | 0.00 | 0.00 | 0.02 | 0.00 | 7.40 | 0.00 | 0.00 | 0.00 | 0.00 |
label=morepeeks #3 scenario=move target=chrome bursts=10 load=0 reports=9 wall=12.4s
  command p50 ms                          38.14
  command p90 ms                          40.02
  frames                                  64.00
  tick ms/frame                            1.22
  frame p90 ms                             1.27
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.52
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           12.20
  cloak ms (outside frames)               18.70
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #3 | move | 38.14 | 40.02 | 64.00 | 1.22 | 1.27 | 1.00 | 5.71 | 5.77 | 0.52 | 0.00 | 0.00 | 0.00 | 0.00 | 12.20 | 18.70 | 0.00 | 0.00 | 0.00 |
label=morepeeks #3 scenario=float target=chrome bursts=10 load=0 reports=10 wall=13.3s
  command p50 ms                         133.88
  command p90 ms                         137.14
  frames                                  59.00
  tick ms/frame                            5.06
  frame p90 ms                             4.38
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          6.33
  peek_frame ms/frame                      3.51
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.07
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            7.60
  cloak ms (outside frames)               35.50
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #3 | float | 133.88 | 137.14 | 59.00 | 5.06 | 4.38 | 5.00 | 5.72 | 6.33 | 3.51 | 0.00 | 0.00 | 0.07 | 2.00 | 7.60 | 35.50 | 0.00 | 0.00 | 0.00 |
label=morepeeks #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          68.68
  command p90 ms                          76.48
  frames                                 133.00
  tick ms/frame                            1.79
  frame p90 ms                             3.56
  frames >2x budget                        6.00
  interval p50 ms                          5.71
  interval p90 ms                          5.78
  peek_frame ms/frame                      1.24
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.57
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            8.10
  cloak ms (outside frames)               31.60
  session_begin ms (outside frames)        2.60
  input->anim start ms                    27.42
  input->first frame ms                  171.30
| morepeeks #3 | minimize | 68.68 | 76.48 | 133.00 | 1.79 | 3.56 | 6.00 | 5.71 | 5.78 | 1.24 | 0.02 | 0.57 | 0.05 | 1.00 | 8.10 | 31.60 | 2.60 | 27.42 | 171.30 |
label=morepeeks #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=13.4s
  command p50 ms                         141.77
  command p90 ms                         159.62
  frames                                  55.00
  tick ms/frame                            5.18
  frame p90 ms                             4.42
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          6.34
  peek_frame ms/frame                      3.35
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.08
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           18.30
  cloak ms (outside frames)               42.10
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #3 | resize | 141.77 | 159.62 | 55.00 | 5.18 | 4.42 | 5.00 | 5.72 | 6.34 | 3.35 | 0.00 | 0.00 | 0.08 | 2.00 | 18.30 | 42.10 | 0.00 | 0.00 | 0.00 |
Moved 3 window(s) to workspace '2' to switch to.
label=morepeeks #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          70.26
  command p90 ms                          84.57
  frames                                  99.00
  tick ms/frame                            1.65
  frame p90 ms                             1.55
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            8.60
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #3 | switch | 70.26 | 84.57 | 99.00 | 1.65 | 1.55 | 3.00 | 5.71 | 5.77 | 0.00 | 0.00 | 0.00 | 0.02 | 0.00 | 8.60 | 0.00 | 0.00 | 0.00 | 0.00 |

## move

| label | scenario | command p50 ms | command p90 ms | frames | tick ms/frame | frame p90 ms | frames >2x budget | interval p50 ms | interval p90 ms | peek_frame ms/frame | session_overlays ms/frame | batch_commit ms/frame | ovl_region ms/frame | dwm_flush calls (in frames) | dwm_flush ms (outside frames) | cloak ms (outside frames) | session_begin ms (outside frames) | input->anim start ms | input->first frame ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| nopeeks #1 | move | 23.95 | 25.60 | 63.00 | 0.82 | 1.22 | 1.00 | 5.71 | 5.75 | 0.00 | 0.03 | 0.38 | 0.03 | 0.00 | 7.40 | 11.80 | 1.00 | 0.30 | 28.18 |
| nopeeks #2 | move | 18.23 | 19.26 | 62.00 | 0.61 | 0.99 | 0.00 | 5.71 | 5.76 | 0.00 | 0.02 | 0.27 | 0.02 | 0.00 | 6.80 | 11.50 | 1.00 | 0.30 | 18.76 |
| nopeeks #3 | move | 42.35 | 44.45 | 63.00 | 1.09 | 1.00 | 1.00 | 5.71 | 5.76 | 0.00 | 0.03 | 0.39 | 0.07 | 0.00 | 9.60 | 16.40 | 1.00 | 0.25 | 50.15 |
| default #1 | move | 37.30 | 39.54 | 64.00 | 1.25 | 1.25 | 1.00 | 5.71 | 5.77 | 0.51 | 0.00 | 0.00 | 0.00 | 0.00 | 9.90 | 17.20 | 0.00 | 0.00 | 0.00 |
| default #2 | move | 40.78 | 45.08 | 64.00 | 1.52 | 1.50 | 2.00 | 5.71 | 5.77 | 0.61 | 0.00 | 0.00 | 0.00 | 0.00 | 11.60 | 19.20 | 0.00 | 0.00 | 0.00 |
| default #3 | move | 22.79 | 25.27 | 63.00 | 0.86 | 0.91 | 1.00 | 5.71 | 5.76 | 0.38 | 0.00 | 0.00 | 0.00 | 0.00 | 6.30 | 9.50 | 0.00 | 0.00 | 0.00 |
| morepeeks #1 | move | 23.99 | 25.17 | 63.00 | 0.96 | 1.05 | 1.00 | 5.71 | 5.77 | 0.42 | 0.00 | 0.00 | 0.00 | 0.00 | 6.50 | 9.40 | 0.00 | 0.00 | 0.00 |
| morepeeks #2 | move | 23.48 | 24.68 | 63.00 | 0.88 | 0.90 | 0.00 | 5.71 | 5.76 | 0.38 | 0.00 | 0.00 | 0.00 | 0.00 | 7.50 | 10.70 | 0.00 | 0.00 | 0.00 |
| morepeeks #3 | move | 38.14 | 40.02 | 64.00 | 1.22 | 1.27 | 1.00 | 5.71 | 5.77 | 0.52 | 0.00 | 0.00 | 0.00 | 0.00 | 12.20 | 18.70 | 0.00 | 0.00 | 0.00 |

## float

| label | scenario | command p50 ms | command p90 ms | frames | tick ms/frame | frame p90 ms | frames >2x budget | interval p50 ms | interval p90 ms | peek_frame ms/frame | session_overlays ms/frame | batch_commit ms/frame | ovl_region ms/frame | dwm_flush calls (in frames) | dwm_flush ms (outside frames) | cloak ms (outside frames) | session_begin ms (outside frames) | input->anim start ms | input->first frame ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| nopeeks #1 | float | 113.47 | 115.82 | 60.00 | 3.40 | 8.55 | 4.00 | 5.72 | 9.18 | 0.00 | 0.07 | 1.83 | 0.10 | 0.00 | 9.50 | 41.90 | 2.80 | 0.28 | 121.69 |
| nopeeks #2 | float | 102.17 | 114.74 | 59.00 | 3.68 | 9.39 | 3.00 | 5.72 | 10.18 | 0.00 | 0.06 | 2.22 | 0.12 | 0.00 | 12.20 | 37.00 | 2.90 | 0.28 | 119.23 |
| nopeeks #3 | float | 99.40 | 105.35 | 59.00 | 3.61 | 8.93 | 4.00 | 5.72 | 9.01 | 0.00 | 0.05 | 2.21 | 0.13 | 0.00 | 8.30 | 33.20 | 2.70 | 0.30 | 119.26 |
| default #1 | float | 92.17 | 111.15 | 60.00 | 4.15 | 6.30 | 4.00 | 5.72 | 9.64 | 0.00 | 0.05 | 1.43 | 0.15 | 0.00 | 13.00 | 33.20 | 2.60 | 0.29 | 100.58 |
| default #2 | float | 111.12 | 124.28 | 59.00 | 4.94 | 10.56 | 5.00 | 5.72 | 10.86 | 0.00 | 0.05 | 2.28 | 0.15 | 0.00 | 11.80 | 37.80 | 2.70 | 0.31 | 120.47 |
| default #3 | float | 99.28 | 110.94 | 59.00 | 3.90 | 9.44 | 4.00 | 5.72 | 9.52 | 0.00 | 0.05 | 1.34 | 0.14 | 0.00 | 13.70 | 28.90 | 2.50 | 0.34 | 110.12 |
| morepeeks #1 | float | 143.97 | 150.27 | 56.00 | 5.66 | 4.89 | 5.00 | 5.72 | 6.40 | 3.32 | 0.00 | 0.00 | 0.09 | 2.00 | 16.40 | 37.20 | 0.00 | 0.00 | 0.00 |
| morepeeks #2 | float | 133.00 | 144.12 | 58.00 | 4.95 | 4.16 | 5.00 | 5.72 | 6.48 | 3.28 | 0.00 | 0.00 | 0.07 | 2.00 | 8.60 | 30.40 | 0.00 | 0.00 | 0.00 |
| morepeeks #3 | float | 133.88 | 137.14 | 59.00 | 5.06 | 4.38 | 5.00 | 5.72 | 6.33 | 3.51 | 0.00 | 0.00 | 0.07 | 2.00 | 7.60 | 35.50 | 0.00 | 0.00 | 0.00 |

## minimize

| label | scenario | command p50 ms | command p90 ms | frames | tick ms/frame | frame p90 ms | frames >2x budget | interval p50 ms | interval p90 ms | peek_frame ms/frame | session_overlays ms/frame | batch_commit ms/frame | ovl_region ms/frame | dwm_flush calls (in frames) | dwm_flush ms (outside frames) | cloak ms (outside frames) | session_begin ms (outside frames) | input->anim start ms | input->first frame ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| nopeeks #1 | minimize | 63.57 | 70.20 | 145.00 | 1.40 | 4.65 | 5.00 | 5.72 | 6.22 | 0.00 | 0.02 | 0.75 | 0.04 | 1.00 | 12.00 | 42.00 | 3.50 | 3.68 | 128.89 |
| nopeeks #2 | minimize | 62.12 | 74.85 | 135.00 | 1.75 | 5.66 | 5.00 | 5.72 | 5.98 | 0.00 | 0.02 | 0.72 | 0.05 | 1.00 | 9.20 | 29.80 | 2.90 | 3.88 | 117.13 |
| nopeeks #3 | minimize | 59.82 | 69.78 | 137.00 | 1.56 | 5.73 | 5.00 | 5.72 | 6.37 | 0.00 | 0.02 | 0.86 | 0.04 | 1.00 | 12.00 | 39.80 | 3.90 | 3.97 | 114.08 |
| default #1 | minimize | 40.46 | 72.27 | 133.00 | 1.76 | 3.67 | 5.00 | 5.72 | 5.78 | 1.18 | 0.02 | 0.59 | 0.04 | 1.00 | 11.40 | 35.00 | 2.60 | 27.48 | 176.00 |
| default #2 | minimize | 66.23 | 68.32 | 135.00 | 1.73 | 4.02 | 5.00 | 5.72 | 5.77 | 1.20 | 0.02 | 0.56 | 0.05 | 1.00 | 10.40 | 30.80 | 2.40 | 37.42 | 202.57 |
| default #3 | minimize | 62.36 | 70.25 | 134.00 | 2.26 | 4.05 | 5.00 | 5.72 | 5.77 | 1.20 | 0.02 | 0.55 | 0.04 | 1.00 | 10.60 | 30.00 | 2.60 | 4.20 | 196.96 |
| morepeeks #1 | minimize | 67.99 | 83.31 | 128.00 | 2.53 | 3.90 | 7.00 | 5.72 | 5.80 | 1.32 | 0.02 | 0.62 | 0.04 | 1.00 | 14.00 | 34.90 | 2.70 | 3.98 | 220.01 |
| morepeeks #2 | minimize | 68.08 | 78.80 | 135.00 | 2.21 | 3.76 | 6.00 | 5.71 | 5.79 | 1.19 | 0.03 | 0.54 | 0.06 | 1.00 | 13.60 | 35.50 | 2.60 | 4.04 | 170.93 |
| morepeeks #3 | minimize | 68.68 | 76.48 | 133.00 | 1.79 | 3.56 | 6.00 | 5.71 | 5.78 | 1.24 | 0.02 | 0.57 | 0.05 | 1.00 | 8.10 | 31.60 | 2.60 | 27.42 | 171.30 |

## resize

| label | scenario | command p50 ms | command p90 ms | frames | tick ms/frame | frame p90 ms | frames >2x budget | interval p50 ms | interval p90 ms | peek_frame ms/frame | session_overlays ms/frame | batch_commit ms/frame | ovl_region ms/frame | dwm_flush calls (in frames) | dwm_flush ms (outside frames) | cloak ms (outside frames) | session_begin ms (outside frames) | input->anim start ms | input->first frame ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| nopeeks #1 | resize | 107.89 | 110.18 | 58.00 | 4.05 | 8.80 | 4.00 | 5.72 | 9.10 | 0.00 | 0.06 | 2.19 | 0.12 | 0.00 | 11.20 | 40.60 | 2.60 | 0.30 | 117.91 |
| nopeeks #2 | resize | 103.26 | 112.07 | 58.00 | 4.15 | 9.83 | 4.00 | 5.72 | 10.66 | 0.00 | 0.06 | 2.37 | 0.11 | 0.00 | 10.70 | 34.30 | 2.80 | 0.30 | 114.07 |
| nopeeks #3 | resize | 97.10 | 106.71 | 58.00 | 3.36 | 8.14 | 4.00 | 5.72 | 9.24 | 0.00 | 0.05 | 2.19 | 0.08 | 0.00 | 11.60 | 29.70 | 2.60 | 0.28 | 117.57 |
| default #1 | resize | 135.46 | 173.72 | 59.00 | 5.62 | 4.92 | 5.00 | 5.72 | 6.96 | 3.34 | 0.00 | 0.00 | 0.07 | 2.00 | 19.50 | 40.60 | 0.00 | 0.00 | 0.00 |
| default #2 | resize | 133.39 | 139.00 | 57.00 | 5.17 | 4.40 | 5.00 | 5.71 | 5.81 | 3.42 | 0.00 | 0.00 | 0.09 | 2.00 | 7.80 | 35.10 | 0.00 | 0.00 | 0.00 |
| default #3 | resize | 134.21 | 136.97 | 56.00 | 5.93 | 3.92 | 5.00 | 5.72 | 6.77 | 3.37 | 0.00 | 0.00 | 0.09 | 2.00 | 8.00 | 38.00 | 0.00 | 0.00 | 0.00 |
| morepeeks #1 | resize | 125.53 | 146.14 | 57.00 | 5.41 | 4.41 | 5.00 | 5.71 | 5.90 | 3.40 | 0.00 | 0.00 | 0.06 | 2.00 | 12.10 | 32.40 | 0.00 | 0.00 | 0.00 |
| morepeeks #2 | resize | 141.68 | 156.55 | 57.00 | 5.06 | 4.24 | 5.00 | 5.71 | 5.98 | 3.36 | 0.00 | 0.00 | 0.07 | 2.00 | 9.30 | 43.40 | 0.00 | 0.00 | 0.00 |
| morepeeks #3 | resize | 141.77 | 159.62 | 55.00 | 5.18 | 4.42 | 5.00 | 5.72 | 6.34 | 3.35 | 0.00 | 0.00 | 0.08 | 2.00 | 18.30 | 42.10 | 0.00 | 0.00 | 0.00 |

## switch

| label | scenario | command p50 ms | command p90 ms | frames | tick ms/frame | frame p90 ms | frames >2x budget | interval p50 ms | interval p90 ms | peek_frame ms/frame | session_overlays ms/frame | batch_commit ms/frame | ovl_region ms/frame | dwm_flush calls (in frames) | dwm_flush ms (outside frames) | cloak ms (outside frames) | session_begin ms (outside frames) | input->anim start ms | input->first frame ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| nopeeks #1 | switch | 55.89 | 68.00 | 99.00 | 1.95 | 1.92 | 3.00 | 5.72 | 5.76 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 8.20 | 0.00 | 0.00 | 0.00 | 0.00 |
| nopeeks #2 | switch | 66.86 | 81.01 | 99.00 | 2.05 | 1.84 | 3.00 | 5.71 | 5.79 | 0.00 | 0.00 | 0.00 | 0.02 | 0.00 | 7.40 | 0.00 | 0.00 | 0.00 | 0.00 |
| nopeeks #3 | switch | 77.90 | 83.28 | 99.00 | 1.93 | 1.90 | 3.00 | 5.71 | 5.76 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 9.60 | 0.00 | 0.00 | 0.00 | 0.00 |
| default #1 | switch | 80.76 | 102.27 | 99.00 | 2.09 | 2.03 | 4.00 | 5.72 | 5.78 | 0.00 | 0.00 | 0.00 | 0.04 | 0.00 | 7.10 | 0.00 | 0.00 | 0.00 | 0.00 |
| default #2 | switch | 68.97 | 78.45 | 99.00 | 1.68 | 1.69 | 3.00 | 5.71 | 5.77 | 0.00 | 0.00 | 0.00 | 0.02 | 0.00 | 9.10 | 0.00 | 0.00 | 0.00 | 0.00 |
| default #3 | switch | 51.18 | 79.89 | 99.00 | 1.98 | 1.75 | 3.00 | 5.72 | 5.77 | 0.00 | 0.00 | 0.00 | 0.02 | 0.00 | 7.40 | 0.00 | 0.00 | 0.00 | 0.00 |
| morepeeks #1 | switch | 58.22 | 74.68 | 99.00 | 1.83 | 2.27 | 4.00 | 5.72 | 5.77 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 6.60 | 0.00 | 0.00 | 0.00 | 0.00 |
| morepeeks #2 | switch | 71.60 | 89.36 | 100.00 | 2.17 | 1.94 | 4.00 | 5.71 | 5.76 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 7.80 | 0.00 | 0.00 | 0.00 | 0.00 |
| morepeeks #3 | switch | 70.26 | 84.57 | 99.00 | 1.65 | 1.55 | 3.00 | 5.71 | 5.77 | 0.00 | 0.00 | 0.00 | 0.02 | 0.00 | 8.60 | 0.00 | 0.00 | 0.00 | 0.00 |
```

## main, no CPU load (bench-run.ps1)

```text
Bench run, 2026-11-12T21:16:23, modes main, target chrome, 3 round(s) of 10 bursts per scenario and mode, load 0, CPU load 0 thread(s) of 16
tiles main #1 ws 1: ApplicationFrameHost, WindowsTerminal, Notepad, Everything, ApplicationFrameHost, explorer, chrome
label=main #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          15.64
  command p90 ms                          17.32
  frames                                  64.00
  tick ms/frame                            0.62
  frame p90 ms                             0.99
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.28
  ovl_region ms/frame                      0.01
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            7.10
  cloak ms (outside frames)               10.20
  session_begin ms (outside frames)        0.90
  input->anim start ms                     0.23
  input->first frame ms                   16.47
| main #1 | move | 15.64 | 17.32 | 64.00 | 0.62 | 0.99 | 0.00 | 5.71 | 5.74 | 0.00 | 0.02 | 0.28 | 0.01 | 1.00 | 7.10 | 10.20 | 0.90 | 0.23 | 16.47 |
label=main #1 scenario=float target=chrome bursts=10 load=0 reports=10 wall=12.8s
  command p50 ms                          79.30
  command p90 ms                          89.49
  frames                                  61.00
  tick ms/frame                            3.66
  frame p90 ms                            10.71
  frames >2x budget                        6.00
  interval p50 ms                          5.72
  interval p90 ms                         10.79
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.04
  batch_commit ms/frame                    2.11
  ovl_region ms/frame                      0.10
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            8.50
  cloak ms (outside frames)               26.50
  session_begin ms (outside frames)        2.50
  input->anim start ms                     0.23
  input->first frame ms                   99.84
| main #1 | float | 79.30 | 89.49 | 61.00 | 3.66 | 10.71 | 6.00 | 5.72 | 10.79 | 0.00 | 0.04 | 2.11 | 0.10 | 2.00 | 8.50 | 26.50 | 2.50 | 0.23 | 99.84 |
label=main #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          52.82
  command p90 ms                          57.52
  frames                                 139.00
  tick ms/frame                            1.61
  frame p90 ms                             4.45
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          5.80
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.67
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              4.00
  dwm_flush ms (outside frames)            6.80
  cloak ms (outside frames)               35.40
  session_begin ms (outside frames)        2.90
  input->anim start ms                     3.05
  input->first frame ms                   99.00
| main #1 | minimize | 52.82 | 57.52 | 139.00 | 1.61 | 4.45 | 5.00 | 5.72 | 5.80 | 0.00 | 0.02 | 0.67 | 0.04 | 4.00 | 6.80 | 35.40 | 2.90 | 3.05 | 99.00 |
label=main #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.8s
  command p50 ms                          76.37
  command p90 ms                          84.60
  frames                                  59.00
  tick ms/frame                            4.14
  frame p90 ms                            10.27
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                         10.44
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.04
  batch_commit ms/frame                    2.08
  ovl_region ms/frame                      0.11
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)           10.10
  cloak ms (outside frames)               32.70
  session_begin ms (outside frames)        2.60
  input->anim start ms                     0.25
  input->first frame ms                   99.55
| main #1 | resize | 76.37 | 84.60 | 59.00 | 4.14 | 10.27 | 5.00 | 5.72 | 10.44 | 0.00 | 0.04 | 2.08 | 0.11 | 3.00 | 10.10 | 32.70 | 2.60 | 0.25 | 99.55 |
Moved 3 window(s) to workspace '2' to switch to.
label=main #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          28.64
  command p90 ms                          58.13
  frames                                  97.00
  tick ms/frame                            2.50
  frame p90 ms                             3.34
  frames >2x budget                        2.00
  interval p50 ms                          5.72
  interval p90 ms                          5.75
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.77
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            9.20
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| main #1 | switch | 28.64 | 58.13 | 97.00 | 2.50 | 3.34 | 2.00 | 5.72 | 5.75 | 0.00 | 0.00 | 0.77 | 0.02 | 3.00 | 9.20 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles main #2 ws 1: chrome, ApplicationFrameHost, ApplicationFrameHost, Notepad, explorer, WindowsTerminal, Everything
label=main #2 scenario=move target=chrome bursts=10 load=0 reports=9 wall=12.4s
  command p50 ms                          35.81
  command p90 ms                          39.20
  frames                                  63.00
  tick ms/frame                            1.24
  frame p90 ms                             1.11
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.03
  batch_commit ms/frame                    0.50
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)           10.70
  cloak ms (outside frames)               16.00
  session_begin ms (outside frames)        1.00
  input->anim start ms                     0.21
  input->first frame ms                   45.78
| main #2 | move | 35.81 | 39.20 | 63.00 | 1.24 | 1.11 | 2.00 | 5.71 | 5.75 | 0.00 | 0.03 | 0.50 | 0.05 | 1.00 | 10.70 | 16.00 | 1.00 | 0.21 | 45.78 |
label=main #2 scenario=float target=chrome bursts=10 load=0 reports=10 wall=12.8s
  command p50 ms                          77.25
  command p90 ms                          83.68
  frames                                  60.00
  tick ms/frame                            3.91
  frame p90 ms                            10.12
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                         10.47
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.03
  batch_commit ms/frame                    2.33
  ovl_region ms/frame                      0.11
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            6.30
  cloak ms (outside frames)               26.10
  session_begin ms (outside frames)        2.60
  input->anim start ms                     0.23
  input->first frame ms                   97.94
| main #2 | float | 77.25 | 83.68 | 60.00 | 3.91 | 10.12 | 5.00 | 5.72 | 10.47 | 0.00 | 0.03 | 2.33 | 0.11 | 3.00 | 6.30 | 26.10 | 2.60 | 0.23 | 97.94 |
label=main #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          44.83
  command p90 ms                          53.08
  frames                                 134.00
  tick ms/frame                            1.67
  frame p90 ms                             3.86
  frames >2x budget                        6.00
  interval p50 ms                          5.72
  interval p90 ms                          5.78
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.89
  ovl_region ms/frame                      0.06
  dwm_flush calls (in frames)              5.00
  dwm_flush ms (outside frames)            8.40
  cloak ms (outside frames)               34.20
  session_begin ms (outside frames)        6.50
  input->anim start ms                     4.08
  input->first frame ms                  126.44
| main #2 | minimize | 44.83 | 53.08 | 134.00 | 1.67 | 3.86 | 6.00 | 5.72 | 5.78 | 0.00 | 0.02 | 0.89 | 0.06 | 5.00 | 8.40 | 34.20 | 6.50 | 4.08 | 126.44 |
label=main #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.9s
  command p50 ms                          75.03
  command p90 ms                          89.11
  frames                                  58.00
  tick ms/frame                            4.31
  frame p90 ms                            10.12
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                         10.22
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.03
  batch_commit ms/frame                    2.20
  ovl_region ms/frame                      0.13
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            7.60
  cloak ms (outside frames)               22.90
  session_begin ms (outside frames)        2.70
  input->anim start ms                     0.21
  input->first frame ms                   96.06
| main #2 | resize | 75.03 | 89.11 | 58.00 | 4.31 | 10.12 | 4.00 | 5.72 | 10.22 | 0.00 | 0.03 | 2.20 | 0.13 | 3.00 | 7.60 | 22.90 | 2.70 | 0.21 | 96.06 |
Moved 3 window(s) to workspace '2' to switch to.
label=main #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          50.11
  command p90 ms                          54.58
  frames                                  97.00
  tick ms/frame                            2.22
  frame p90 ms                             3.52
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.77
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)           13.80
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| main #2 | switch | 50.11 | 54.58 | 97.00 | 2.22 | 3.52 | 3.00 | 5.71 | 5.76 | 0.00 | 0.00 | 0.77 | 0.05 | 3.00 | 13.80 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles main #3 ws 1: ApplicationFrameHost, ApplicationFrameHost, explorer, Everything, WindowsTerminal, Notepad, chrome
label=main #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          16.22
  command p90 ms                          17.55
  frames                                  64.00
  tick ms/frame                            0.65
  frame p90 ms                             0.98
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.29
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            7.40
  cloak ms (outside frames)               11.10
  session_begin ms (outside frames)        1.00
  input->anim start ms                     0.23
  input->first frame ms                   16.94
| main #3 | move | 16.22 | 17.55 | 64.00 | 0.65 | 0.98 | 0.00 | 5.71 | 5.75 | 0.00 | 0.02 | 0.29 | 0.02 | 1.00 | 7.40 | 11.10 | 1.00 | 0.23 | 16.94 |
label=main #3 scenario=float target=chrome bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          81.82
  command p90 ms                          90.03
  frames                                  59.00
  tick ms/frame                            3.87
  frame p90 ms                            10.18
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                         10.25
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.05
  batch_commit ms/frame                    2.17
  ovl_region ms/frame                      0.12
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            7.10
  cloak ms (outside frames)               24.50
  session_begin ms (outside frames)        2.60
  input->anim start ms                     0.23
  input->first frame ms                   97.35
| main #3 | float | 81.82 | 90.03 | 59.00 | 3.87 | 10.18 | 5.00 | 5.72 | 10.25 | 0.00 | 0.05 | 2.17 | 0.12 | 2.00 | 7.10 | 24.50 | 2.60 | 0.23 | 97.35 |
label=main #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          46.38
  command p90 ms                          54.24
  frames                                 138.00
  tick ms/frame                            1.71
  frame p90 ms                             3.05
  frames >2x budget                        6.00
  interval p50 ms                          5.72
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.73
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              4.00
  dwm_flush ms (outside frames)            7.20
  cloak ms (outside frames)               26.60
  session_begin ms (outside frames)        3.40
  input->anim start ms                     3.79
  input->first frame ms                   97.50
| main #3 | minimize | 46.38 | 54.24 | 138.00 | 1.71 | 3.05 | 6.00 | 5.72 | 5.77 | 0.00 | 0.02 | 0.73 | 0.04 | 4.00 | 7.20 | 26.60 | 3.40 | 3.79 | 97.50 |
label=main #3 scenario=resize target=chrome bursts=10 load=0 reports=11 wall=12.8s
  command p50 ms                          73.29
  command p90 ms                          79.15
  frames                                  57.00
  tick ms/frame                            3.93
  frame p90 ms                             9.10
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          9.24
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.04
  batch_commit ms/frame                    2.28
  ovl_region ms/frame                      0.08
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            6.50
  cloak ms (outside frames)               25.40
  session_begin ms (outside frames)        2.70
  input->anim start ms                     0.23
  input->first frame ms                   82.04
| main #3 | resize | 73.29 | 79.15 | 57.00 | 3.93 | 9.10 | 5.00 | 5.72 | 9.24 | 0.00 | 0.04 | 2.28 | 0.08 | 3.00 | 6.50 | 25.40 | 2.70 | 0.23 | 82.04 |
Moved 3 window(s) to workspace '2' to switch to.
label=main #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          51.16
  command p90 ms                          62.51
  frames                                  96.00
  tick ms/frame                            2.51
  frame p90 ms                             3.54
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.75
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.72
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)           13.70
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| main #3 | switch | 51.16 | 62.51 | 96.00 | 2.51 | 3.54 | 3.00 | 5.72 | 5.75 | 0.00 | 0.00 | 0.72 | 0.03 | 3.00 | 13.70 | 0.00 | 0.00 | 0.00 | 0.00 |

## move

| label | scenario | command p50 ms | command p90 ms | frames | tick ms/frame | frame p90 ms | frames >2x budget | interval p50 ms | interval p90 ms | peek_frame ms/frame | session_overlays ms/frame | batch_commit ms/frame | ovl_region ms/frame | dwm_flush calls (in frames) | dwm_flush ms (outside frames) | cloak ms (outside frames) | session_begin ms (outside frames) | input->anim start ms | input->first frame ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| main #1 | move | 15.64 | 17.32 | 64.00 | 0.62 | 0.99 | 0.00 | 5.71 | 5.74 | 0.00 | 0.02 | 0.28 | 0.01 | 1.00 | 7.10 | 10.20 | 0.90 | 0.23 | 16.47 |
| main #2 | move | 35.81 | 39.20 | 63.00 | 1.24 | 1.11 | 2.00 | 5.71 | 5.75 | 0.00 | 0.03 | 0.50 | 0.05 | 1.00 | 10.70 | 16.00 | 1.00 | 0.21 | 45.78 |
| main #3 | move | 16.22 | 17.55 | 64.00 | 0.65 | 0.98 | 0.00 | 5.71 | 5.75 | 0.00 | 0.02 | 0.29 | 0.02 | 1.00 | 7.40 | 11.10 | 1.00 | 0.23 | 16.94 |

## float

| label | scenario | command p50 ms | command p90 ms | frames | tick ms/frame | frame p90 ms | frames >2x budget | interval p50 ms | interval p90 ms | peek_frame ms/frame | session_overlays ms/frame | batch_commit ms/frame | ovl_region ms/frame | dwm_flush calls (in frames) | dwm_flush ms (outside frames) | cloak ms (outside frames) | session_begin ms (outside frames) | input->anim start ms | input->first frame ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| main #1 | float | 79.30 | 89.49 | 61.00 | 3.66 | 10.71 | 6.00 | 5.72 | 10.79 | 0.00 | 0.04 | 2.11 | 0.10 | 2.00 | 8.50 | 26.50 | 2.50 | 0.23 | 99.84 |
| main #2 | float | 77.25 | 83.68 | 60.00 | 3.91 | 10.12 | 5.00 | 5.72 | 10.47 | 0.00 | 0.03 | 2.33 | 0.11 | 3.00 | 6.30 | 26.10 | 2.60 | 0.23 | 97.94 |
| main #3 | float | 81.82 | 90.03 | 59.00 | 3.87 | 10.18 | 5.00 | 5.72 | 10.25 | 0.00 | 0.05 | 2.17 | 0.12 | 2.00 | 7.10 | 24.50 | 2.60 | 0.23 | 97.35 |

## minimize

| label | scenario | command p50 ms | command p90 ms | frames | tick ms/frame | frame p90 ms | frames >2x budget | interval p50 ms | interval p90 ms | peek_frame ms/frame | session_overlays ms/frame | batch_commit ms/frame | ovl_region ms/frame | dwm_flush calls (in frames) | dwm_flush ms (outside frames) | cloak ms (outside frames) | session_begin ms (outside frames) | input->anim start ms | input->first frame ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| main #1 | minimize | 52.82 | 57.52 | 139.00 | 1.61 | 4.45 | 5.00 | 5.72 | 5.80 | 0.00 | 0.02 | 0.67 | 0.04 | 4.00 | 6.80 | 35.40 | 2.90 | 3.05 | 99.00 |
| main #2 | minimize | 44.83 | 53.08 | 134.00 | 1.67 | 3.86 | 6.00 | 5.72 | 5.78 | 0.00 | 0.02 | 0.89 | 0.06 | 5.00 | 8.40 | 34.20 | 6.50 | 4.08 | 126.44 |
| main #3 | minimize | 46.38 | 54.24 | 138.00 | 1.71 | 3.05 | 6.00 | 5.72 | 5.77 | 0.00 | 0.02 | 0.73 | 0.04 | 4.00 | 7.20 | 26.60 | 3.40 | 3.79 | 97.50 |

## resize

| label | scenario | command p50 ms | command p90 ms | frames | tick ms/frame | frame p90 ms | frames >2x budget | interval p50 ms | interval p90 ms | peek_frame ms/frame | session_overlays ms/frame | batch_commit ms/frame | ovl_region ms/frame | dwm_flush calls (in frames) | dwm_flush ms (outside frames) | cloak ms (outside frames) | session_begin ms (outside frames) | input->anim start ms | input->first frame ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| main #1 | resize | 76.37 | 84.60 | 59.00 | 4.14 | 10.27 | 5.00 | 5.72 | 10.44 | 0.00 | 0.04 | 2.08 | 0.11 | 3.00 | 10.10 | 32.70 | 2.60 | 0.25 | 99.55 |
| main #2 | resize | 75.03 | 89.11 | 58.00 | 4.31 | 10.12 | 4.00 | 5.72 | 10.22 | 0.00 | 0.03 | 2.20 | 0.13 | 3.00 | 7.60 | 22.90 | 2.70 | 0.21 | 96.06 |
| main #3 | resize | 73.29 | 79.15 | 57.00 | 3.93 | 9.10 | 5.00 | 5.72 | 9.24 | 0.00 | 0.04 | 2.28 | 0.08 | 3.00 | 6.50 | 25.40 | 2.70 | 0.23 | 82.04 |

## switch

| label | scenario | command p50 ms | command p90 ms | frames | tick ms/frame | frame p90 ms | frames >2x budget | interval p50 ms | interval p90 ms | peek_frame ms/frame | session_overlays ms/frame | batch_commit ms/frame | ovl_region ms/frame | dwm_flush calls (in frames) | dwm_flush ms (outside frames) | cloak ms (outside frames) | session_begin ms (outside frames) | input->anim start ms | input->first frame ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| main #1 | switch | 28.64 | 58.13 | 97.00 | 2.50 | 3.34 | 2.00 | 5.72 | 5.75 | 0.00 | 0.00 | 0.77 | 0.02 | 3.00 | 9.20 | 0.00 | 0.00 | 0.00 | 0.00 |
| main #2 | switch | 50.11 | 54.58 | 97.00 | 2.22 | 3.52 | 3.00 | 5.71 | 5.76 | 0.00 | 0.00 | 0.77 | 0.05 | 3.00 | 13.80 | 0.00 | 0.00 | 0.00 | 0.00 |
| main #3 | switch | 51.16 | 62.51 | 96.00 | 2.51 | 3.54 | 3.00 | 5.72 | 5.75 | 0.00 | 0.00 | 0.72 | 0.03 | 3.00 | 13.70 | 0.00 | 0.00 | 0.00 | 0.00 |
```

## isolate-bench.txt, -CpuLoad 8 (21:19)

```text
Isolate benchmark, 2026-11-12T21:19:58, target chrome, 3 round(s) of 10 bursts per scenario and mode, load 0, CPU load 8 thread(s) of 16
label=nopeeks #1 scenario=move target=chrome bursts=10 load=0 reports=9 wall=12.5s
  command p50 ms                          38.58
  command p90 ms                          47.73
  frames                                  63.00
  tick ms/frame                            1.23
  frame p90 ms                             1.19
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.03
  batch_commit ms/frame                    0.45
  ovl_region ms/frame                      0.06
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            8.70
  cloak ms (outside frames)               14.50
  session_begin ms (outside frames)        1.10
  input->anim start ms                     0.30
  input->first frame ms                   50.14
| nopeeks #1 | move | 38.58 | 47.73 | 63.00 | 1.23 | 1.19 | 1.00 | 5.71 | 5.76 | 0.00 | 0.03 | 0.45 | 0.06 | 0.00 | 8.70 | 14.50 | 1.10 | 0.30 | 50.14 |
label=nopeeks #1 scenario=float target=chrome bursts=10 load=0 reports=10 wall=13.1s
  command p50 ms                         111.66
  command p90 ms                         124.58
  frames                                  59.00
  tick ms/frame                            3.43
  frame p90 ms                             9.57
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                         10.17
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.04
  batch_commit ms/frame                    2.35
  ovl_region ms/frame                      0.12
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           11.10
  cloak ms (outside frames)               38.70
  session_begin ms (outside frames)        3.30
  input->anim start ms                     0.33
  input->first frame ms                  125.91
| nopeeks #1 | float | 111.66 | 124.58 | 59.00 | 3.43 | 9.57 | 4.00 | 5.72 | 10.17 | 0.00 | 0.04 | 2.35 | 0.12 | 0.00 | 11.10 | 38.70 | 3.30 | 0.33 | 125.91 |
label=nopeeks #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          73.04
  command p90 ms                          79.05
  frames                                 134.00
  tick ms/frame                            1.65
  frame p90 ms                             5.56
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          6.10
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.82
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)           11.40
  cloak ms (outside frames)               40.10
  session_begin ms (outside frames)        5.00
  input->anim start ms                     4.74
  input->first frame ms                  126.39
| nopeeks #1 | minimize | 73.04 | 79.05 | 134.00 | 1.65 | 5.56 | 5.00 | 5.72 | 6.10 | 0.00 | 0.02 | 0.82 | 0.05 | 1.00 | 11.40 | 40.10 | 5.00 | 4.74 | 126.39 |
label=nopeeks #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=13.1s
  command p50 ms                         108.71
  command p90 ms                         122.69
  frames                                  58.00
  tick ms/frame                            4.55
  frame p90 ms                            10.00
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                         10.56
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.05
  batch_commit ms/frame                    2.17
  ovl_region ms/frame                      0.14
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           11.40
  cloak ms (outside frames)               34.90
  session_begin ms (outside frames)        3.30
  input->anim start ms                     0.32
  input->first frame ms                  117.83
| nopeeks #1 | resize | 108.71 | 122.69 | 58.00 | 4.55 | 10.00 | 4.00 | 5.72 | 10.56 | 0.00 | 0.05 | 2.17 | 0.14 | 0.00 | 11.40 | 34.90 | 3.30 | 0.32 | 117.83 |
Moved 3 window(s) to workspace '2' to switch to.
label=nopeeks #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.8s
  command p50 ms                          85.57
  command p90 ms                         101.24
  frames                                  99.00
  tick ms/frame                            1.89
  frame p90 ms                             2.16
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            8.50
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| nopeeks #1 | switch | 85.57 | 101.24 | 99.00 | 1.89 | 2.16 | 3.00 | 5.71 | 5.77 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 8.50 | 0.00 | 0.00 | 0.00 | 0.00 |
label=default #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          24.84
  command p90 ms                          27.03
  frames                                  63.00
  tick ms/frame                            0.97
  frame p90 ms                             0.99
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  peek_frame ms/frame                      0.41
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            7.20
  cloak ms (outside frames)               10.40
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #1 | move | 24.84 | 27.03 | 63.00 | 0.97 | 0.99 | 1.00 | 5.71 | 5.75 | 0.41 | 0.00 | 0.00 | 0.00 | 0.00 | 7.20 | 10.40 | 0.00 | 0.00 | 0.00 |
label=default #1 scenario=float target=chrome bursts=10 load=0 reports=11 wall=13.1s
  command p50 ms                          96.66
  command p90 ms                         123.12
  frames                                  57.00
  tick ms/frame                            4.26
  frame p90 ms                             8.77
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          9.62
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.05
  batch_commit ms/frame                    1.48
  ovl_region ms/frame                      0.10
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           11.00
  cloak ms (outside frames)               33.10
  session_begin ms (outside frames)        3.00
  input->anim start ms                     0.37
  input->first frame ms                  102.32
| default #1 | float | 96.66 | 123.12 | 57.00 | 4.26 | 8.77 | 5.00 | 5.72 | 9.62 | 0.00 | 0.05 | 1.48 | 0.10 | 0.00 | 11.00 | 33.10 | 3.00 | 0.37 | 102.32 |
label=default #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          75.62
  command p90 ms                          87.37
  frames                                 132.00
  tick ms/frame                            2.51
  frame p90 ms                             4.55
  frames >2x budget                        7.00
  interval p50 ms                          5.71
  interval p90 ms                          5.79
  peek_frame ms/frame                      1.34
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.62
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)           14.30
  cloak ms (outside frames)               38.90
  session_begin ms (outside frames)        2.60
  input->anim start ms                     4.34
  input->first frame ms                  184.23
| default #1 | minimize | 75.62 | 87.37 | 132.00 | 2.51 | 4.55 | 7.00 | 5.71 | 5.79 | 1.34 | 0.02 | 0.62 | 0.05 | 1.00 | 14.30 | 38.90 | 2.60 | 4.34 | 184.23 |
label=default #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=13.5s
  command p50 ms                         147.60
  command p90 ms                         164.01
  frames                                  57.00
  tick ms/frame                            6.07
  frame p90 ms                             5.43
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          7.08
  peek_frame ms/frame                      3.70
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.08
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           15.50
  cloak ms (outside frames)               43.70
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #1 | resize | 147.60 | 164.01 | 57.00 | 6.07 | 5.43 | 5.00 | 5.72 | 7.08 | 3.70 | 0.00 | 0.00 | 0.08 | 2.00 | 15.50 | 43.70 | 0.00 | 0.00 | 0.00 |
Moved 3 window(s) to workspace '2' to switch to.
label=default #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          74.70
  command p90 ms                          97.47
  frames                                  99.00
  tick ms/frame                            2.44
  frame p90 ms                             2.22
  frames >2x budget                        4.00
  interval p50 ms                          5.71
  interval p90 ms                          5.78
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            6.90
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #1 | switch | 74.70 | 97.47 | 99.00 | 2.44 | 2.22 | 4.00 | 5.71 | 5.78 | 0.00 | 0.00 | 0.00 | 0.04 | 0.00 | 6.90 | 0.00 | 0.00 | 0.00 | 0.00 |
label=morepeeks #1 scenario=move target=chrome bursts=10 load=0 reports=9 wall=12.4s
  command p50 ms                          28.82
  command p90 ms                          31.27
  frames                                  64.00
  tick ms/frame                            1.16
  frame p90 ms                             1.53
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.55
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            8.10
  cloak ms (outside frames)               12.30
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #1 | move | 28.82 | 31.27 | 64.00 | 1.16 | 1.53 | 1.00 | 5.71 | 5.77 | 0.55 | 0.00 | 0.00 | 0.00 | 0.00 | 8.10 | 12.30 | 0.00 | 0.00 | 0.00 |
label=morepeeks #1 scenario=float target=chrome bursts=10 load=0 reports=10 wall=13.5s
  command p50 ms                         152.90
  command p90 ms                         166.09
  frames                                  57.00
  tick ms/frame                            5.90
  frame p90 ms                             4.84
  frames >2x budget                        5.00
  interval p50 ms                          5.71
  interval p90 ms                          7.03
  peek_frame ms/frame                      3.68
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.06
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           18.80
  cloak ms (outside frames)               51.80
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #1 | float | 152.90 | 166.09 | 57.00 | 5.90 | 4.84 | 5.00 | 5.71 | 7.03 | 3.68 | 0.00 | 0.00 | 0.06 | 2.00 | 18.80 | 51.80 | 0.00 | 0.00 | 0.00 |
label=morepeeks #1 scenario=minimize target=chrome bursts=10 load=0 reports=9 wall=12.5s
  command p50 ms                          78.72
  command p90 ms                          88.86
  frames                                 128.00
  tick ms/frame                            2.14
  frame p90 ms                             4.04
  frames >2x budget                        7.00
  interval p50 ms                          5.72
  interval p90 ms                          5.84
  peek_frame ms/frame                      0.76
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.45
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)           14.30
  cloak ms (outside frames)               42.70
  session_begin ms (outside frames)        2.90
  input->anim start ms                     0.42
  input->first frame ms                  101.44
| morepeeks #1 | minimize | 78.72 | 88.86 | 128.00 | 2.14 | 4.04 | 7.00 | 5.72 | 5.84 | 0.76 | 0.02 | 0.45 | 0.05 | 1.00 | 14.30 | 42.70 | 2.90 | 0.42 | 101.44 |
label=morepeeks #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=13.5s
  command p50 ms                         149.08
  command p90 ms                         164.10
  frames                                  54.00
  tick ms/frame                            6.02
  frame p90 ms                             5.22
  frames >2x budget                        5.00
  interval p50 ms                          5.71
  interval p90 ms                          7.18
  peek_frame ms/frame                      3.61
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.06
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           16.80
  cloak ms (outside frames)               37.90
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #1 | resize | 149.08 | 164.10 | 54.00 | 6.02 | 5.22 | 5.00 | 5.71 | 7.18 | 3.61 | 0.00 | 0.00 | 0.06 | 2.00 | 16.80 | 37.90 | 0.00 | 0.00 | 0.00 |
Moved 3 window(s) to workspace '2' to switch to.
label=morepeeks #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.8s
  command p50 ms                          86.08
  command p90 ms                         102.42
  frames                                  99.00
  tick ms/frame                            2.20
  frame p90 ms                             2.12
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            7.00
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #1 | switch | 86.08 | 102.42 | 99.00 | 2.20 | 2.12 | 3.00 | 5.72 | 5.77 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 7.00 | 0.00 | 0.00 | 0.00 | 0.00 |
label=nopeeks #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          18.99
  command p90 ms                          19.92
  frames                                  62.00
  tick ms/frame                            0.61
  frame p90 ms                             1.08
  frames >2x budget                        1.00
  interval p50 ms                          5.72
  interval p90 ms                          5.74
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.28
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            6.20
  cloak ms (outside frames)               11.40
  session_begin ms (outside frames)        1.10
  input->anim start ms                     0.33
  input->first frame ms                   19.69
| nopeeks #2 | move | 18.99 | 19.92 | 62.00 | 0.61 | 1.08 | 1.00 | 5.72 | 5.74 | 0.00 | 0.02 | 0.28 | 0.02 | 0.00 | 6.20 | 11.40 | 1.10 | 0.33 | 19.69 |
label=nopeeks #2 scenario=float target=chrome bursts=10 load=0 reports=10 wall=13.1s
  command p50 ms                         117.42
  command p90 ms                         121.67
  frames                                  59.00
  tick ms/frame                            4.34
  frame p90 ms                            10.49
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                         10.58
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.05
  batch_commit ms/frame                    2.53
  ovl_region ms/frame                      0.14
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           12.20
  cloak ms (outside frames)               30.10
  session_begin ms (outside frames)        3.10
  input->anim start ms                     0.28
  input->first frame ms                  135.11
| nopeeks #2 | float | 117.42 | 121.67 | 59.00 | 4.34 | 10.49 | 5.00 | 5.72 | 10.58 | 0.00 | 0.05 | 2.53 | 0.14 | 0.00 | 12.20 | 30.10 | 3.10 | 0.28 | 135.11 |
label=nopeeks #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          68.87
  command p90 ms                          78.01
  frames                                 138.00
  tick ms/frame                            1.84
  frame p90 ms                             3.22
  frames >2x budget                        7.00
  interval p50 ms                          5.72
  interval p90 ms                          6.50
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.78
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)           12.60
  cloak ms (outside frames)               38.90
  session_begin ms (outside frames)        4.60
  input->anim start ms                     3.89
  input->first frame ms                  112.38
| nopeeks #2 | minimize | 68.87 | 78.01 | 138.00 | 1.84 | 3.22 | 7.00 | 5.72 | 6.50 | 0.00 | 0.02 | 0.78 | 0.05 | 1.00 | 12.60 | 38.90 | 4.60 | 3.89 | 112.38 |
label=nopeeks #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=13.1s
  command p50 ms                         110.68
  command p90 ms                         123.92
  frames                                  57.00
  tick ms/frame                            3.95
  frame p90 ms                            10.14
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                         10.24
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.05
  batch_commit ms/frame                    2.40
  ovl_region ms/frame                      0.12
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           12.20
  cloak ms (outside frames)               39.20
  session_begin ms (outside frames)        2.90
  input->anim start ms                     0.35
  input->first frame ms                  133.20
| nopeeks #2 | resize | 110.68 | 123.92 | 57.00 | 3.95 | 10.14 | 5.00 | 5.72 | 10.24 | 0.00 | 0.05 | 2.40 | 0.12 | 0.00 | 12.20 | 39.20 | 2.90 | 0.35 | 133.20 |
Moved 3 window(s) to workspace '2' to switch to.
label=nopeeks #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.8s
  command p50 ms                          73.45
  command p90 ms                          89.52
  frames                                  99.00
  tick ms/frame                            2.22
  frame p90 ms                             2.01
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            9.40
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| nopeeks #2 | switch | 73.45 | 89.52 | 99.00 | 2.22 | 2.01 | 3.00 | 5.71 | 5.76 | 0.00 | 0.00 | 0.00 | 0.04 | 0.00 | 9.40 | 0.00 | 0.00 | 0.00 | 0.00 |
label=default #2 scenario=move target=chrome bursts=10 load=0 reports=9 wall=12.5s
  command p50 ms                          41.74
  command p90 ms                          43.24
  frames                                  64.00
  tick ms/frame                            1.38
  frame p90 ms                             1.53
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.61
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           12.40
  cloak ms (outside frames)               19.20
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #2 | move | 41.74 | 43.24 | 64.00 | 1.38 | 1.53 | 1.00 | 5.71 | 5.77 | 0.61 | 0.00 | 0.00 | 0.00 | 0.00 | 12.40 | 19.20 | 0.00 | 0.00 | 0.00 |
label=default #2 scenario=float target=chrome bursts=10 load=0 reports=10 wall=13.1s
  command p50 ms                         122.85
  command p90 ms                         129.14
  frames                                  60.00
  tick ms/frame                            5.08
  frame p90 ms                            10.92
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                         11.18
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.04
  batch_commit ms/frame                    2.65
  ovl_region ms/frame                      0.17
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           12.00
  cloak ms (outside frames)               41.70
  session_begin ms (outside frames)        3.00
  input->anim start ms                     0.37
  input->first frame ms                  152.67
| default #2 | float | 122.85 | 129.14 | 60.00 | 5.08 | 10.92 | 5.00 | 5.72 | 11.18 | 0.00 | 0.04 | 2.65 | 0.17 | 0.00 | 12.00 | 41.70 | 3.00 | 0.37 | 152.67 |
label=default #2 scenario=minimize target=chrome bursts=10 load=0 reports=9 wall=12.6s
  command p50 ms                          68.57
  command p90 ms                          91.50
  frames                                 128.00
  tick ms/frame                            2.09
  frame p90 ms                             4.48
  frames >2x budget                        7.00
  interval p50 ms                          5.72
  interval p90 ms                          5.78
  peek_frame ms/frame                      1.24
  session_overlays ms/frame                0.01
  batch_commit ms/frame                    0.46
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)           10.70
  cloak ms (outside frames)               34.00
  session_begin ms (outside frames)        2.90
  input->anim start ms                     0.62
  input->first frame ms                  105.94
| default #2 | minimize | 68.57 | 91.50 | 128.00 | 2.09 | 4.48 | 7.00 | 5.72 | 5.78 | 1.24 | 0.01 | 0.46 | 0.04 | 1.00 | 10.70 | 34.00 | 2.90 | 0.62 | 105.94 |
label=default #2 scenario=resize target=chrome bursts=10 load=0 reports=9 wall=13.6s
  command p50 ms                         151.08
  command p90 ms                         155.79
  frames                                  56.00
  tick ms/frame                            6.10
  frame p90 ms                             5.38
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          6.64
  peek_frame ms/frame                      3.56
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.07
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           15.30
  cloak ms (outside frames)               41.50
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #2 | resize | 151.08 | 155.79 | 56.00 | 6.10 | 5.38 | 4.00 | 5.72 | 6.64 | 3.56 | 0.00 | 0.00 | 0.07 | 2.00 | 15.30 | 41.50 | 0.00 | 0.00 | 0.00 |
Moved 3 window(s) to workspace '2' to switch to.
label=default #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.8s
  command p50 ms                          84.62
  command p90 ms                          93.89
  frames                                  99.00
  tick ms/frame                            2.03
  frame p90 ms                             2.35
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           11.10
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #2 | switch | 84.62 | 93.89 | 99.00 | 2.03 | 2.35 | 3.00 | 5.72 | 5.77 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 11.10 | 0.00 | 0.00 | 0.00 | 0.00 |
label=morepeeks #2 scenario=move target=chrome bursts=10 load=0 reports=9 wall=12.3s
  command p50 ms                          24.62
  command p90 ms                          27.28
  frames                                  64.00
  tick ms/frame                            0.92
  frame p90 ms                             0.63
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  peek_frame ms/frame                      0.46
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            7.60
  cloak ms (outside frames)               11.30
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #2 | move | 24.62 | 27.28 | 64.00 | 0.92 | 0.63 | 1.00 | 5.71 | 5.76 | 0.46 | 0.00 | 0.00 | 0.00 | 0.00 | 7.60 | 11.30 | 0.00 | 0.00 | 0.00 |
label=morepeeks #2 scenario=float target=chrome bursts=10 load=0 reports=10 wall=13.6s
  command p50 ms                         152.81
  command p90 ms                         171.10
  frames                                  57.00
  tick ms/frame                            5.69
  frame p90 ms                             5.99
  frames >2x budget                        5.00
  interval p50 ms                          5.71
  interval p90 ms                          8.27
  peek_frame ms/frame                      3.73
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           21.10
  cloak ms (outside frames)               61.40
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #2 | float | 152.81 | 171.10 | 57.00 | 5.69 | 5.99 | 5.00 | 5.71 | 8.27 | 3.73 | 0.00 | 0.00 | 0.05 | 2.00 | 21.10 | 61.40 | 0.00 | 0.00 | 0.00 |
label=morepeeks #2 scenario=minimize target=chrome bursts=10 load=0 reports=9 wall=12.5s
  command p50 ms                          77.28
  command p90 ms                          90.46
  frames                                 131.00
  tick ms/frame                            2.09
  frame p90 ms                             4.36
  frames >2x budget                        6.00
  interval p50 ms                          5.72
  interval p90 ms                          5.78
  peek_frame ms/frame                      0.80
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.41
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)           13.90
  cloak ms (outside frames)               44.70
  session_begin ms (outside frames)        3.00
  input->anim start ms                     0.45
  input->first frame ms                  106.79
| morepeeks #2 | minimize | 77.28 | 90.46 | 131.00 | 2.09 | 4.36 | 6.00 | 5.72 | 5.78 | 0.80 | 0.02 | 0.41 | 0.03 | 1.00 | 13.90 | 44.70 | 3.00 | 0.45 | 106.79 |
label=morepeeks #2 scenario=resize target=chrome bursts=10 load=0 reports=11 wall=13.4s
  command p50 ms                         133.31
  command p90 ms                         159.77
  frames                                  58.00
  tick ms/frame                            4.54
  frame p90 ms                             4.98
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          6.13
  peek_frame ms/frame                      1.74
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           10.50
  cloak ms (outside frames)               32.60
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #2 | resize | 133.31 | 159.77 | 58.00 | 4.54 | 4.98 | 5.00 | 5.72 | 6.13 | 1.74 | 0.00 | 0.00 | 0.05 | 2.00 | 10.50 | 32.60 | 0.00 | 0.00 | 0.00 |
Moved 3 window(s) to workspace '2' to switch to.
label=morepeeks #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          55.72
  command p90 ms                          62.43
  frames                                  98.00
  tick ms/frame                            2.01
  frame p90 ms                             1.95
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.78
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            9.00
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #2 | switch | 55.72 | 62.43 | 98.00 | 2.01 | 1.95 | 3.00 | 5.72 | 5.78 | 0.00 | 0.00 | 0.00 | 0.02 | 2.00 | 9.00 | 0.00 | 0.00 | 0.00 | 0.00 |
label=nopeeks #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          19.73
  command p90 ms                          20.71
  frames                                  64.00
  tick ms/frame                            0.64
  frame p90 ms                             1.18
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.03
  batch_commit ms/frame                    0.31
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            6.80
  cloak ms (outside frames)               11.80
  session_begin ms (outside frames)        1.10
  input->anim start ms                     0.33
  input->first frame ms                   20.44
| nopeeks #3 | move | 19.73 | 20.71 | 64.00 | 0.64 | 1.18 | 0.00 | 5.71 | 5.75 | 0.00 | 0.03 | 0.31 | 0.02 | 0.00 | 6.80 | 11.80 | 1.10 | 0.33 | 20.44 |
label=nopeeks #3 scenario=float target=chrome bursts=10 load=0 reports=10 wall=13.1s
  command p50 ms                         111.36
  command p90 ms                         125.15
  frames                                  59.00
  tick ms/frame                            3.82
  frame p90 ms                             9.92
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                         10.02
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.05
  batch_commit ms/frame                    2.26
  ovl_region ms/frame                      0.11
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           11.00
  cloak ms (outside frames)               31.20
  session_begin ms (outside frames)        2.70
  input->anim start ms                     0.33
  input->first frame ms                  129.00
| nopeeks #3 | float | 111.36 | 125.15 | 59.00 | 3.82 | 9.92 | 4.00 | 5.72 | 10.02 | 0.00 | 0.05 | 2.26 | 0.11 | 0.00 | 11.00 | 31.20 | 2.70 | 0.33 | 129.00 |
label=nopeeks #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          69.12
  command p90 ms                          75.20
  frames                                 135.00
  tick ms/frame                            1.72
  frame p90 ms                             5.09
  frames >2x budget                        6.00
  interval p50 ms                          5.72
  interval p90 ms                          5.82
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.78
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            8.10
  cloak ms (outside frames)               36.10
  session_begin ms (outside frames)        3.10
  input->anim start ms                     3.67
  input->first frame ms                  138.99
| nopeeks #3 | minimize | 69.12 | 75.20 | 135.00 | 1.72 | 5.09 | 6.00 | 5.72 | 5.82 | 0.00 | 0.02 | 0.78 | 0.05 | 1.00 | 8.10 | 36.10 | 3.10 | 3.67 | 138.99 |
label=nopeeks #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=13.1s
  command p50 ms                         107.93
  command p90 ms                         113.97
  frames                                  57.00
  tick ms/frame                            4.27
  frame p90 ms                             8.81
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          8.92
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.05
  batch_commit ms/frame                    2.42
  ovl_region ms/frame                      0.09
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           12.60
  cloak ms (outside frames)               38.60
  session_begin ms (outside frames)        3.20
  input->anim start ms                     0.34
  input->first frame ms                  129.74
| nopeeks #3 | resize | 107.93 | 113.97 | 57.00 | 4.27 | 8.81 | 5.00 | 5.72 | 8.92 | 0.00 | 0.05 | 2.42 | 0.09 | 0.00 | 12.60 | 38.60 | 3.20 | 0.34 | 129.74 |
Moved 3 window(s) to workspace '2' to switch to.
label=nopeeks #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.8s
  command p50 ms                          80.86
  command p90 ms                          90.15
  frames                                  99.00
  tick ms/frame                            1.94
  frame p90 ms                             2.18
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            8.60
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| nopeeks #3 | switch | 80.86 | 90.15 | 99.00 | 1.94 | 2.18 | 3.00 | 5.72 | 5.77 | 0.00 | 0.00 | 0.00 | 0.04 | 0.00 | 8.60 | 0.00 | 0.00 | 0.00 | 0.00 |
label=default #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          24.47
  command p90 ms                          25.99
  frames                                  64.00
  tick ms/frame                            1.01
  frame p90 ms                             1.01
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  peek_frame ms/frame                      0.47
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            8.10
  cloak ms (outside frames)               11.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #3 | move | 24.47 | 25.99 | 64.00 | 1.01 | 1.01 | 1.00 | 5.71 | 5.75 | 0.47 | 0.00 | 0.00 | 0.00 | 0.00 | 8.10 | 11.00 | 0.00 | 0.00 | 0.00 |
label=default #3 scenario=float target=chrome bursts=10 load=0 reports=10 wall=13.2s
  command p50 ms                         108.57
  command p90 ms                         132.00
  frames                                  60.00
  tick ms/frame                            4.47
  frame p90 ms                            10.02
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                         10.78
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.05
  batch_commit ms/frame                    1.72
  ovl_region ms/frame                      0.09
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           10.90
  cloak ms (outside frames)               50.50
  session_begin ms (outside frames)        3.10
  input->anim start ms                     0.41
  input->first frame ms                  122.18
| default #3 | float | 108.57 | 132.00 | 60.00 | 4.47 | 10.02 | 5.00 | 5.72 | 10.78 | 0.00 | 0.05 | 1.72 | 0.09 | 0.00 | 10.90 | 50.50 | 3.10 | 0.41 | 122.18 |
label=default #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          82.54
  command p90 ms                          91.50
  frames                                 128.00
  tick ms/frame                            2.58
  frame p90 ms                             4.81
  frames >2x budget                        7.00
  interval p50 ms                          5.72
  interval p90 ms                          5.81
  peek_frame ms/frame                      1.31
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.65
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)           17.60
  cloak ms (outside frames)               59.50
  session_begin ms (outside frames)        2.80
  input->anim start ms                    15.24
  input->first frame ms                  225.90
| default #3 | minimize | 82.54 | 91.50 | 128.00 | 2.58 | 4.81 | 7.00 | 5.72 | 5.81 | 1.31 | 0.02 | 0.65 | 0.03 | 1.00 | 17.60 | 59.50 | 2.80 | 15.24 | 225.90 |
label=default #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=13.6s
  command p50 ms                         159.33
  command p90 ms                         171.42
  frames                                  56.00
  tick ms/frame                            6.57
  frame p90 ms                             5.36
  frames >2x budget                        5.00
  interval p50 ms                          5.71
  interval p90 ms                          6.53
  peek_frame ms/frame                      3.40
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.06
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           16.10
  cloak ms (outside frames)               67.10
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #3 | resize | 159.33 | 171.42 | 56.00 | 6.57 | 5.36 | 5.00 | 5.71 | 6.53 | 3.40 | 0.00 | 0.00 | 0.06 | 2.00 | 16.10 | 67.10 | 0.00 | 0.00 | 0.00 |
Moved 3 window(s) to workspace '2' to switch to.
label=default #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          61.29
  command p90 ms                          67.61
  frames                                  99.00
  tick ms/frame                            2.20
  frame p90 ms                             2.13
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          5.78
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           13.30
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #3 | switch | 61.29 | 67.61 | 99.00 | 2.20 | 2.13 | 4.00 | 5.72 | 5.78 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 13.30 | 0.00 | 0.00 | 0.00 | 0.00 |
label=morepeeks #3 scenario=move target=chrome bursts=10 load=0 reports=9 wall=12.4s
  command p50 ms                          28.59
  command p90 ms                          30.20
  frames                                  64.00
  tick ms/frame                            1.18
  frame p90 ms                             1.37
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  peek_frame ms/frame                      0.58
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            9.50
  cloak ms (outside frames)               13.50
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #3 | move | 28.59 | 30.20 | 64.00 | 1.18 | 1.37 | 1.00 | 5.71 | 5.76 | 0.58 | 0.00 | 0.00 | 0.00 | 0.00 | 9.50 | 13.50 | 0.00 | 0.00 | 0.00 |
label=morepeeks #3 scenario=float target=chrome bursts=10 load=0 reports=10 wall=13.5s
  command p50 ms                         145.73
  command p90 ms                         155.47
  frames                                  58.00
  tick ms/frame                            5.83
  frame p90 ms                             5.52
  frames >2x budget                        5.00
  interval p50 ms                          5.71
  interval p90 ms                          6.92
  peek_frame ms/frame                      3.39
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.06
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           10.30
  cloak ms (outside frames)               50.60
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #3 | float | 145.73 | 155.47 | 58.00 | 5.83 | 5.52 | 5.00 | 5.71 | 6.92 | 3.39 | 0.00 | 0.00 | 0.06 | 2.00 | 10.30 | 50.60 | 0.00 | 0.00 | 0.00 |
label=morepeeks #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          61.09
  command p90 ms                          85.80
  frames                                 130.00
  tick ms/frame                            2.15
  frame p90 ms                             4.81
  frames >2x budget                        7.00
  interval p50 ms                          5.72
  interval p90 ms                          5.79
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.67
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)           14.90
  cloak ms (outside frames)               37.10
  session_begin ms (outside frames)        3.00
  input->anim start ms                     5.15
  input->first frame ms                  161.28
| morepeeks #3 | minimize | 61.09 | 85.80 | 130.00 | 2.15 | 4.81 | 7.00 | 5.72 | 5.79 | 0.00 | 0.02 | 0.67 | 0.05 | 1.00 | 14.90 | 37.10 | 3.00 | 5.15 | 161.28 |
label=morepeeks #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=13.5s
  command p50 ms                         146.15
  command p90 ms                         163.13
  frames                                  56.00
  tick ms/frame                            6.00
  frame p90 ms                             5.15
  frames >2x budget                        5.00
  interval p50 ms                          5.71
  interval p90 ms                          5.85
  peek_frame ms/frame                      3.57
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.06
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           15.80
  cloak ms (outside frames)               36.80
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #3 | resize | 146.15 | 163.13 | 56.00 | 6.00 | 5.15 | 5.00 | 5.71 | 5.85 | 3.57 | 0.00 | 0.00 | 0.06 | 2.00 | 15.80 | 36.80 | 0.00 | 0.00 | 0.00 |
Moved 3 window(s) to workspace '2' to switch to.
label=morepeeks #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.8s
  command p50 ms                          86.06
  command p90 ms                         101.23
  frames                                  99.00
  tick ms/frame                            2.06
  frame p90 ms                             2.03
  frames >2x budget                        4.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            6.70
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #3 | switch | 86.06 | 101.23 | 99.00 | 2.06 | 2.03 | 4.00 | 5.71 | 5.77 | 0.00 | 0.00 | 0.00 | 0.04 | 0.00 | 6.70 | 0.00 | 0.00 | 0.00 | 0.00 |

## move

| label | scenario | command p50 ms | command p90 ms | frames | tick ms/frame | frame p90 ms | frames >2x budget | interval p50 ms | interval p90 ms | peek_frame ms/frame | session_overlays ms/frame | batch_commit ms/frame | ovl_region ms/frame | dwm_flush calls (in frames) | dwm_flush ms (outside frames) | cloak ms (outside frames) | session_begin ms (outside frames) | input->anim start ms | input->first frame ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| nopeeks #1 | move | 38.58 | 47.73 | 63.00 | 1.23 | 1.19 | 1.00 | 5.71 | 5.76 | 0.00 | 0.03 | 0.45 | 0.06 | 0.00 | 8.70 | 14.50 | 1.10 | 0.30 | 50.14 |
| nopeeks #2 | move | 18.99 | 19.92 | 62.00 | 0.61 | 1.08 | 1.00 | 5.72 | 5.74 | 0.00 | 0.02 | 0.28 | 0.02 | 0.00 | 6.20 | 11.40 | 1.10 | 0.33 | 19.69 |
| nopeeks #3 | move | 19.73 | 20.71 | 64.00 | 0.64 | 1.18 | 0.00 | 5.71 | 5.75 | 0.00 | 0.03 | 0.31 | 0.02 | 0.00 | 6.80 | 11.80 | 1.10 | 0.33 | 20.44 |
| default #1 | move | 24.84 | 27.03 | 63.00 | 0.97 | 0.99 | 1.00 | 5.71 | 5.75 | 0.41 | 0.00 | 0.00 | 0.00 | 0.00 | 7.20 | 10.40 | 0.00 | 0.00 | 0.00 |
| default #2 | move | 41.74 | 43.24 | 64.00 | 1.38 | 1.53 | 1.00 | 5.71 | 5.77 | 0.61 | 0.00 | 0.00 | 0.00 | 0.00 | 12.40 | 19.20 | 0.00 | 0.00 | 0.00 |
| default #3 | move | 24.47 | 25.99 | 64.00 | 1.01 | 1.01 | 1.00 | 5.71 | 5.75 | 0.47 | 0.00 | 0.00 | 0.00 | 0.00 | 8.10 | 11.00 | 0.00 | 0.00 | 0.00 |
| morepeeks #1 | move | 28.82 | 31.27 | 64.00 | 1.16 | 1.53 | 1.00 | 5.71 | 5.77 | 0.55 | 0.00 | 0.00 | 0.00 | 0.00 | 8.10 | 12.30 | 0.00 | 0.00 | 0.00 |
| morepeeks #2 | move | 24.62 | 27.28 | 64.00 | 0.92 | 0.63 | 1.00 | 5.71 | 5.76 | 0.46 | 0.00 | 0.00 | 0.00 | 0.00 | 7.60 | 11.30 | 0.00 | 0.00 | 0.00 |
| morepeeks #3 | move | 28.59 | 30.20 | 64.00 | 1.18 | 1.37 | 1.00 | 5.71 | 5.76 | 0.58 | 0.00 | 0.00 | 0.00 | 0.00 | 9.50 | 13.50 | 0.00 | 0.00 | 0.00 |

## float

| label | scenario | command p50 ms | command p90 ms | frames | tick ms/frame | frame p90 ms | frames >2x budget | interval p50 ms | interval p90 ms | peek_frame ms/frame | session_overlays ms/frame | batch_commit ms/frame | ovl_region ms/frame | dwm_flush calls (in frames) | dwm_flush ms (outside frames) | cloak ms (outside frames) | session_begin ms (outside frames) | input->anim start ms | input->first frame ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| nopeeks #1 | float | 111.66 | 124.58 | 59.00 | 3.43 | 9.57 | 4.00 | 5.72 | 10.17 | 0.00 | 0.04 | 2.35 | 0.12 | 0.00 | 11.10 | 38.70 | 3.30 | 0.33 | 125.91 |
| nopeeks #2 | float | 117.42 | 121.67 | 59.00 | 4.34 | 10.49 | 5.00 | 5.72 | 10.58 | 0.00 | 0.05 | 2.53 | 0.14 | 0.00 | 12.20 | 30.10 | 3.10 | 0.28 | 135.11 |
| nopeeks #3 | float | 111.36 | 125.15 | 59.00 | 3.82 | 9.92 | 4.00 | 5.72 | 10.02 | 0.00 | 0.05 | 2.26 | 0.11 | 0.00 | 11.00 | 31.20 | 2.70 | 0.33 | 129.00 |
| default #1 | float | 96.66 | 123.12 | 57.00 | 4.26 | 8.77 | 5.00 | 5.72 | 9.62 | 0.00 | 0.05 | 1.48 | 0.10 | 0.00 | 11.00 | 33.10 | 3.00 | 0.37 | 102.32 |
| default #2 | float | 122.85 | 129.14 | 60.00 | 5.08 | 10.92 | 5.00 | 5.72 | 11.18 | 0.00 | 0.04 | 2.65 | 0.17 | 0.00 | 12.00 | 41.70 | 3.00 | 0.37 | 152.67 |
| default #3 | float | 108.57 | 132.00 | 60.00 | 4.47 | 10.02 | 5.00 | 5.72 | 10.78 | 0.00 | 0.05 | 1.72 | 0.09 | 0.00 | 10.90 | 50.50 | 3.10 | 0.41 | 122.18 |
| morepeeks #1 | float | 152.90 | 166.09 | 57.00 | 5.90 | 4.84 | 5.00 | 5.71 | 7.03 | 3.68 | 0.00 | 0.00 | 0.06 | 2.00 | 18.80 | 51.80 | 0.00 | 0.00 | 0.00 |
| morepeeks #2 | float | 152.81 | 171.10 | 57.00 | 5.69 | 5.99 | 5.00 | 5.71 | 8.27 | 3.73 | 0.00 | 0.00 | 0.05 | 2.00 | 21.10 | 61.40 | 0.00 | 0.00 | 0.00 |
| morepeeks #3 | float | 145.73 | 155.47 | 58.00 | 5.83 | 5.52 | 5.00 | 5.71 | 6.92 | 3.39 | 0.00 | 0.00 | 0.06 | 2.00 | 10.30 | 50.60 | 0.00 | 0.00 | 0.00 |

## minimize

| label | scenario | command p50 ms | command p90 ms | frames | tick ms/frame | frame p90 ms | frames >2x budget | interval p50 ms | interval p90 ms | peek_frame ms/frame | session_overlays ms/frame | batch_commit ms/frame | ovl_region ms/frame | dwm_flush calls (in frames) | dwm_flush ms (outside frames) | cloak ms (outside frames) | session_begin ms (outside frames) | input->anim start ms | input->first frame ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| nopeeks #1 | minimize | 73.04 | 79.05 | 134.00 | 1.65 | 5.56 | 5.00 | 5.72 | 6.10 | 0.00 | 0.02 | 0.82 | 0.05 | 1.00 | 11.40 | 40.10 | 5.00 | 4.74 | 126.39 |
| nopeeks #2 | minimize | 68.87 | 78.01 | 138.00 | 1.84 | 3.22 | 7.00 | 5.72 | 6.50 | 0.00 | 0.02 | 0.78 | 0.05 | 1.00 | 12.60 | 38.90 | 4.60 | 3.89 | 112.38 |
| nopeeks #3 | minimize | 69.12 | 75.20 | 135.00 | 1.72 | 5.09 | 6.00 | 5.72 | 5.82 | 0.00 | 0.02 | 0.78 | 0.05 | 1.00 | 8.10 | 36.10 | 3.10 | 3.67 | 138.99 |
| default #1 | minimize | 75.62 | 87.37 | 132.00 | 2.51 | 4.55 | 7.00 | 5.71 | 5.79 | 1.34 | 0.02 | 0.62 | 0.05 | 1.00 | 14.30 | 38.90 | 2.60 | 4.34 | 184.23 |
| default #2 | minimize | 68.57 | 91.50 | 128.00 | 2.09 | 4.48 | 7.00 | 5.72 | 5.78 | 1.24 | 0.01 | 0.46 | 0.04 | 1.00 | 10.70 | 34.00 | 2.90 | 0.62 | 105.94 |
| default #3 | minimize | 82.54 | 91.50 | 128.00 | 2.58 | 4.81 | 7.00 | 5.72 | 5.81 | 1.31 | 0.02 | 0.65 | 0.03 | 1.00 | 17.60 | 59.50 | 2.80 | 15.24 | 225.90 |
| morepeeks #1 | minimize | 78.72 | 88.86 | 128.00 | 2.14 | 4.04 | 7.00 | 5.72 | 5.84 | 0.76 | 0.02 | 0.45 | 0.05 | 1.00 | 14.30 | 42.70 | 2.90 | 0.42 | 101.44 |
| morepeeks #2 | minimize | 77.28 | 90.46 | 131.00 | 2.09 | 4.36 | 6.00 | 5.72 | 5.78 | 0.80 | 0.02 | 0.41 | 0.03 | 1.00 | 13.90 | 44.70 | 3.00 | 0.45 | 106.79 |
| morepeeks #3 | minimize | 61.09 | 85.80 | 130.00 | 2.15 | 4.81 | 7.00 | 5.72 | 5.79 | 0.00 | 0.02 | 0.67 | 0.05 | 1.00 | 14.90 | 37.10 | 3.00 | 5.15 | 161.28 |

## resize

| label | scenario | command p50 ms | command p90 ms | frames | tick ms/frame | frame p90 ms | frames >2x budget | interval p50 ms | interval p90 ms | peek_frame ms/frame | session_overlays ms/frame | batch_commit ms/frame | ovl_region ms/frame | dwm_flush calls (in frames) | dwm_flush ms (outside frames) | cloak ms (outside frames) | session_begin ms (outside frames) | input->anim start ms | input->first frame ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| nopeeks #1 | resize | 108.71 | 122.69 | 58.00 | 4.55 | 10.00 | 4.00 | 5.72 | 10.56 | 0.00 | 0.05 | 2.17 | 0.14 | 0.00 | 11.40 | 34.90 | 3.30 | 0.32 | 117.83 |
| nopeeks #2 | resize | 110.68 | 123.92 | 57.00 | 3.95 | 10.14 | 5.00 | 5.72 | 10.24 | 0.00 | 0.05 | 2.40 | 0.12 | 0.00 | 12.20 | 39.20 | 2.90 | 0.35 | 133.20 |
| nopeeks #3 | resize | 107.93 | 113.97 | 57.00 | 4.27 | 8.81 | 5.00 | 5.72 | 8.92 | 0.00 | 0.05 | 2.42 | 0.09 | 0.00 | 12.60 | 38.60 | 3.20 | 0.34 | 129.74 |
| default #1 | resize | 147.60 | 164.01 | 57.00 | 6.07 | 5.43 | 5.00 | 5.72 | 7.08 | 3.70 | 0.00 | 0.00 | 0.08 | 2.00 | 15.50 | 43.70 | 0.00 | 0.00 | 0.00 |
| default #2 | resize | 151.08 | 155.79 | 56.00 | 6.10 | 5.38 | 4.00 | 5.72 | 6.64 | 3.56 | 0.00 | 0.00 | 0.07 | 2.00 | 15.30 | 41.50 | 0.00 | 0.00 | 0.00 |
| default #3 | resize | 159.33 | 171.42 | 56.00 | 6.57 | 5.36 | 5.00 | 5.71 | 6.53 | 3.40 | 0.00 | 0.00 | 0.06 | 2.00 | 16.10 | 67.10 | 0.00 | 0.00 | 0.00 |
| morepeeks #1 | resize | 149.08 | 164.10 | 54.00 | 6.02 | 5.22 | 5.00 | 5.71 | 7.18 | 3.61 | 0.00 | 0.00 | 0.06 | 2.00 | 16.80 | 37.90 | 0.00 | 0.00 | 0.00 |
| morepeeks #2 | resize | 133.31 | 159.77 | 58.00 | 4.54 | 4.98 | 5.00 | 5.72 | 6.13 | 1.74 | 0.00 | 0.00 | 0.05 | 2.00 | 10.50 | 32.60 | 0.00 | 0.00 | 0.00 |
| morepeeks #3 | resize | 146.15 | 163.13 | 56.00 | 6.00 | 5.15 | 5.00 | 5.71 | 5.85 | 3.57 | 0.00 | 0.00 | 0.06 | 2.00 | 15.80 | 36.80 | 0.00 | 0.00 | 0.00 |

## switch

| label | scenario | command p50 ms | command p90 ms | frames | tick ms/frame | frame p90 ms | frames >2x budget | interval p50 ms | interval p90 ms | peek_frame ms/frame | session_overlays ms/frame | batch_commit ms/frame | ovl_region ms/frame | dwm_flush calls (in frames) | dwm_flush ms (outside frames) | cloak ms (outside frames) | session_begin ms (outside frames) | input->anim start ms | input->first frame ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| nopeeks #1 | switch | 85.57 | 101.24 | 99.00 | 1.89 | 2.16 | 3.00 | 5.71 | 5.77 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 8.50 | 0.00 | 0.00 | 0.00 | 0.00 |
| nopeeks #2 | switch | 73.45 | 89.52 | 99.00 | 2.22 | 2.01 | 3.00 | 5.71 | 5.76 | 0.00 | 0.00 | 0.00 | 0.04 | 0.00 | 9.40 | 0.00 | 0.00 | 0.00 | 0.00 |
| nopeeks #3 | switch | 80.86 | 90.15 | 99.00 | 1.94 | 2.18 | 3.00 | 5.72 | 5.77 | 0.00 | 0.00 | 0.00 | 0.04 | 0.00 | 8.60 | 0.00 | 0.00 | 0.00 | 0.00 |
| default #1 | switch | 74.70 | 97.47 | 99.00 | 2.44 | 2.22 | 4.00 | 5.71 | 5.78 | 0.00 | 0.00 | 0.00 | 0.04 | 0.00 | 6.90 | 0.00 | 0.00 | 0.00 | 0.00 |
| default #2 | switch | 84.62 | 93.89 | 99.00 | 2.03 | 2.35 | 3.00 | 5.72 | 5.77 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 11.10 | 0.00 | 0.00 | 0.00 | 0.00 |
| default #3 | switch | 61.29 | 67.61 | 99.00 | 2.20 | 2.13 | 4.00 | 5.72 | 5.78 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 13.30 | 0.00 | 0.00 | 0.00 | 0.00 |
| morepeeks #1 | switch | 86.08 | 102.42 | 99.00 | 2.20 | 2.12 | 3.00 | 5.72 | 5.77 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 7.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| morepeeks #2 | switch | 55.72 | 62.43 | 98.00 | 2.01 | 1.95 | 3.00 | 5.72 | 5.78 | 0.00 | 0.00 | 0.00 | 0.02 | 2.00 | 9.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| morepeeks #3 | switch | 86.06 | 101.23 | 99.00 | 2.06 | 2.03 | 4.00 | 5.71 | 5.77 | 0.00 | 0.00 | 0.00 | 0.04 | 0.00 | 6.70 | 0.00 | 0.00 | 0.00 | 0.00 |
```

## main, -CpuLoad 8 (bench-run.ps1)

```text
Bench run, 2026-11-12T21:30:33, modes main, target chrome, 3 round(s) of 10 bursts per scenario and mode, load 0, CPU load 8 thread(s) of 16
tiles main #1 ws 1: ApplicationFrameHost, ApplicationFrameHost, WindowsTerminal, Everything, Notepad, explorer, chrome
label=main #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          16.42
  command p90 ms                          17.98
  frames                                  64.00
  tick ms/frame                            0.74
  frame p90 ms                             1.10
  frames >2x budget                        0.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.31
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            6.30
  cloak ms (outside frames)                9.70
  session_begin ms (outside frames)        1.00
  input->anim start ms                     0.26
  input->first frame ms                   17.24
| main #1 | move | 16.42 | 17.98 | 64.00 | 0.74 | 1.10 | 0.00 | 5.72 | 5.76 | 0.00 | 0.02 | 0.31 | 0.02 | 1.00 | 6.30 | 9.70 | 1.00 | 0.26 | 17.24 |
label=main #1 scenario=float target=chrome bursts=10 load=0 reports=10 wall=12.8s
  command p50 ms                          85.87
  command p90 ms                          97.80
  frames                                  59.00
  tick ms/frame                            4.34
  frame p90 ms                            11.27
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                         11.53
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.04
  batch_commit ms/frame                    2.53
  ovl_region ms/frame                      0.16
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            7.00
  cloak ms (outside frames)               23.00
  session_begin ms (outside frames)        3.00
  input->anim start ms                     0.25
  input->first frame ms                  116.91
| main #1 | float | 85.87 | 97.80 | 59.00 | 4.34 | 11.27 | 5.00 | 5.72 | 11.53 | 0.00 | 0.04 | 2.53 | 0.16 | 2.00 | 7.00 | 23.00 | 3.00 | 0.25 | 116.91 |
label=main #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          54.87
  command p90 ms                          68.36
  frames                                 135.00
  tick ms/frame                            2.04
  frame p90 ms                             3.77
  frames >2x budget                        7.00
  interval p50 ms                          5.72
  interval p90 ms                          5.82
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.83
  ovl_region ms/frame                      0.06
  dwm_flush calls (in frames)              4.00
  dwm_flush ms (outside frames)            9.20
  cloak ms (outside frames)               32.50
  session_begin ms (outside frames)        5.00
  input->anim start ms                     3.81
  input->first frame ms                   98.45
| main #1 | minimize | 54.87 | 68.36 | 135.00 | 2.04 | 3.77 | 7.00 | 5.72 | 5.82 | 0.00 | 0.02 | 0.83 | 0.06 | 4.00 | 9.20 | 32.50 | 5.00 | 3.81 | 98.45 |
label=main #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.9s
  command p50 ms                          80.87
  command p90 ms                          99.70
  frames                                  57.00
  tick ms/frame                            4.77
  frame p90 ms                            11.75
  frames >2x budget                        6.00
  interval p50 ms                          5.72
  interval p90 ms                         11.84
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.05
  batch_commit ms/frame                    2.37
  ovl_region ms/frame                      0.11
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)           15.80
  cloak ms (outside frames)               38.60
  session_begin ms (outside frames)        3.20
  input->anim start ms                     0.27
  input->first frame ms                   95.85
| main #1 | resize | 80.87 | 99.70 | 57.00 | 4.77 | 11.75 | 6.00 | 5.72 | 11.84 | 0.00 | 0.05 | 2.37 | 0.11 | 3.00 | 15.80 | 38.60 | 3.20 | 0.27 | 95.85 |
Moved 3 window(s) to workspace '2' to switch to.
label=main #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          40.88
  command p90 ms                          50.35
  frames                                  97.00
  tick ms/frame                            2.83
  frame p90 ms                             3.69
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.85
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            7.40
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| main #1 | switch | 40.88 | 50.35 | 97.00 | 2.83 | 3.69 | 3.00 | 5.71 | 5.77 | 0.00 | 0.00 | 0.85 | 0.03 | 3.00 | 7.40 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles main #2 ws 1: chrome, ApplicationFrameHost, WindowsTerminal, Notepad, explorer, ApplicationFrameHost, Everything
label=main #2 scenario=move target=chrome bursts=10 load=0 reports=9 wall=12.4s
  command p50 ms                          39.48
  command p90 ms                          42.04
  frames                                  62.00
  tick ms/frame                            1.38
  frame p90 ms                             1.22
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.03
  batch_commit ms/frame                    0.54
  ovl_region ms/frame                      0.07
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)           11.00
  cloak ms (outside frames)               17.60
  session_begin ms (outside frames)        1.10
  input->anim start ms                     0.24
  input->first frame ms                   49.80
| main #2 | move | 39.48 | 42.04 | 62.00 | 1.38 | 1.22 | 1.00 | 5.71 | 5.77 | 0.00 | 0.03 | 0.54 | 0.07 | 1.00 | 11.00 | 17.60 | 1.10 | 0.24 | 49.80 |
label=main #2 scenario=float target=chrome bursts=10 load=0 reports=9 wall=12.9s
  command p50 ms                          87.41
  command p90 ms                          93.19
  frames                                  60.00
  tick ms/frame                            4.25
  frame p90 ms                             7.96
  frames >2x budget                        6.00
  interval p50 ms                          5.72
  interval p90 ms                         11.07
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.03
  batch_commit ms/frame                    1.39
  ovl_region ms/frame                      0.11
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            8.20
  cloak ms (outside frames)               35.20
  session_begin ms (outside frames)        3.30
  input->anim start ms                     0.24
  input->first frame ms                   76.03
| main #2 | float | 87.41 | 93.19 | 60.00 | 4.25 | 7.96 | 6.00 | 5.72 | 11.07 | 0.00 | 0.03 | 1.39 | 0.11 | 3.00 | 8.20 | 35.20 | 3.30 | 0.24 | 76.03 |
label=main #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          53.55
  command p90 ms                          69.96
  frames                                 127.00
  tick ms/frame                            2.06
  frame p90 ms                             6.46
  frames >2x budget                        8.00
  interval p50 ms                          5.72
  interval p90 ms                          6.52
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.93
  ovl_region ms/frame                      0.06
  dwm_flush calls (in frames)              5.00
  dwm_flush ms (outside frames)            7.50
  cloak ms (outside frames)               39.60
  session_begin ms (outside frames)        8.70
  input->anim start ms                     3.95
  input->first frame ms                  128.63
| main #2 | minimize | 53.55 | 69.96 | 127.00 | 2.06 | 6.46 | 8.00 | 5.72 | 6.52 | 0.00 | 0.02 | 0.93 | 0.06 | 5.00 | 7.50 | 39.60 | 8.70 | 3.95 | 128.63 |
label=main #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.9s
  command p50 ms                          82.66
  command p90 ms                          84.72
  frames                                  57.00
  tick ms/frame                            4.87
  frame p90 ms                            12.50
  frames >2x budget                        6.00
  interval p50 ms                          5.72
  interval p90 ms                         12.57
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.03
  batch_commit ms/frame                    2.34
  ovl_region ms/frame                      0.12
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            5.50
  cloak ms (outside frames)               30.40
  session_begin ms (outside frames)        3.10
  input->anim start ms                     0.26
  input->first frame ms                  105.06
| main #2 | resize | 82.66 | 84.72 | 57.00 | 4.87 | 12.50 | 6.00 | 5.72 | 12.57 | 0.00 | 0.03 | 2.34 | 0.12 | 3.00 | 5.50 | 30.40 | 3.10 | 0.26 | 105.06 |
Moved 3 window(s) to workspace '2' to switch to.
label=main #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.6s
  command p50 ms                          47.70
  command p90 ms                          55.79
  frames                                  97.00
  tick ms/frame                            2.49
  frame p90 ms                             4.15
  frames >2x budget                        2.00
  interval p50 ms                          5.72
  interval p90 ms                          5.79
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.89
  ovl_region ms/frame                      0.06
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            8.90
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| main #2 | switch | 47.70 | 55.79 | 97.00 | 2.49 | 4.15 | 2.00 | 5.72 | 5.79 | 0.00 | 0.00 | 0.89 | 0.06 | 3.00 | 8.90 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles main #3 ws 1: ApplicationFrameHost, WindowsTerminal, explorer, Everything, ApplicationFrameHost, Notepad, chrome
label=main #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          17.08
  command p90 ms                          18.10
  frames                                  63.00
  tick ms/frame                            0.79
  frame p90 ms                             1.16
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.33
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            7.40
  cloak ms (outside frames)               11.30
  session_begin ms (outside frames)        1.10
  input->anim start ms                     0.26
  input->first frame ms                   17.91
| main #3 | move | 17.08 | 18.10 | 63.00 | 0.79 | 1.16 | 1.00 | 5.71 | 5.75 | 0.00 | 0.02 | 0.33 | 0.02 | 1.00 | 7.40 | 11.30 | 1.10 | 0.26 | 17.91 |
label=main #3 scenario=float target=chrome bursts=10 load=0 reports=10 wall=12.9s
  command p50 ms                          88.50
  command p90 ms                         100.67
  frames                                  60.00
  tick ms/frame                            3.99
  frame p90 ms                             8.87
  frames >2x budget                        6.00
  interval p50 ms                          5.72
  interval p90 ms                         11.06
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.05
  batch_commit ms/frame                    1.81
  ovl_region ms/frame                      0.08
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            9.20
  cloak ms (outside frames)               30.40
  session_begin ms (outside frames)        2.80
  input->anim start ms                     0.23
  input->first frame ms                  109.78
| main #3 | float | 88.50 | 100.67 | 60.00 | 3.99 | 8.87 | 6.00 | 5.72 | 11.06 | 0.00 | 0.05 | 1.81 | 0.08 | 2.00 | 9.20 | 30.40 | 2.80 | 0.23 | 109.78 |
label=main #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          27.24
  command p90 ms                          62.77
  frames                                 134.00
  tick ms/frame                            1.88
  frame p90 ms                             5.13
  frames >2x budget                        8.00
  interval p50 ms                          5.72
  interval p90 ms                          5.83
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    1.01
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              4.00
  dwm_flush ms (outside frames)            8.70
  cloak ms (outside frames)               38.20
  session_begin ms (outside frames)        3.80
  input->anim start ms                     3.40
  input->first frame ms                  127.90
| main #3 | minimize | 27.24 | 62.77 | 134.00 | 1.88 | 5.13 | 8.00 | 5.72 | 5.83 | 0.00 | 0.02 | 1.01 | 0.04 | 4.00 | 8.70 | 38.20 | 3.80 | 3.40 | 127.90 |
label=main #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.9s
  command p50 ms                          84.12
  command p90 ms                          92.50
  frames                                  57.00
  tick ms/frame                            4.68
  frame p90 ms                            10.91
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                         11.16
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.04
  batch_commit ms/frame                    2.42
  ovl_region ms/frame                      0.14
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            9.90
  cloak ms (outside frames)               34.20
  session_begin ms (outside frames)        3.00
  input->anim start ms                     0.27
  input->first frame ms                  104.99
| main #3 | resize | 84.12 | 92.50 | 57.00 | 4.68 | 10.91 | 5.00 | 5.72 | 11.16 | 0.00 | 0.04 | 2.42 | 0.14 | 3.00 | 9.90 | 34.20 | 3.00 | 0.27 | 104.99 |
Moved 3 window(s) to workspace '2' to switch to.
label=main #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          31.17
  command p90 ms                          51.49
  frames                                  98.00
  tick ms/frame                            2.85
  frame p90 ms                             4.08
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.78
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.86
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            6.80
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| main #3 | switch | 31.17 | 51.49 | 98.00 | 2.85 | 4.08 | 2.00 | 5.71 | 5.78 | 0.00 | 0.00 | 0.86 | 0.05 | 3.00 | 6.80 | 0.00 | 0.00 | 0.00 | 0.00 |

## move

| label | scenario | command p50 ms | command p90 ms | frames | tick ms/frame | frame p90 ms | frames >2x budget | interval p50 ms | interval p90 ms | peek_frame ms/frame | session_overlays ms/frame | batch_commit ms/frame | ovl_region ms/frame | dwm_flush calls (in frames) | dwm_flush ms (outside frames) | cloak ms (outside frames) | session_begin ms (outside frames) | input->anim start ms | input->first frame ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| main #1 | move | 16.42 | 17.98 | 64.00 | 0.74 | 1.10 | 0.00 | 5.72 | 5.76 | 0.00 | 0.02 | 0.31 | 0.02 | 1.00 | 6.30 | 9.70 | 1.00 | 0.26 | 17.24 |
| main #2 | move | 39.48 | 42.04 | 62.00 | 1.38 | 1.22 | 1.00 | 5.71 | 5.77 | 0.00 | 0.03 | 0.54 | 0.07 | 1.00 | 11.00 | 17.60 | 1.10 | 0.24 | 49.80 |
| main #3 | move | 17.08 | 18.10 | 63.00 | 0.79 | 1.16 | 1.00 | 5.71 | 5.75 | 0.00 | 0.02 | 0.33 | 0.02 | 1.00 | 7.40 | 11.30 | 1.10 | 0.26 | 17.91 |

## float

| label | scenario | command p50 ms | command p90 ms | frames | tick ms/frame | frame p90 ms | frames >2x budget | interval p50 ms | interval p90 ms | peek_frame ms/frame | session_overlays ms/frame | batch_commit ms/frame | ovl_region ms/frame | dwm_flush calls (in frames) | dwm_flush ms (outside frames) | cloak ms (outside frames) | session_begin ms (outside frames) | input->anim start ms | input->first frame ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| main #1 | float | 85.87 | 97.80 | 59.00 | 4.34 | 11.27 | 5.00 | 5.72 | 11.53 | 0.00 | 0.04 | 2.53 | 0.16 | 2.00 | 7.00 | 23.00 | 3.00 | 0.25 | 116.91 |
| main #2 | float | 87.41 | 93.19 | 60.00 | 4.25 | 7.96 | 6.00 | 5.72 | 11.07 | 0.00 | 0.03 | 1.39 | 0.11 | 3.00 | 8.20 | 35.20 | 3.30 | 0.24 | 76.03 |
| main #3 | float | 88.50 | 100.67 | 60.00 | 3.99 | 8.87 | 6.00 | 5.72 | 11.06 | 0.00 | 0.05 | 1.81 | 0.08 | 2.00 | 9.20 | 30.40 | 2.80 | 0.23 | 109.78 |

## minimize

| label | scenario | command p50 ms | command p90 ms | frames | tick ms/frame | frame p90 ms | frames >2x budget | interval p50 ms | interval p90 ms | peek_frame ms/frame | session_overlays ms/frame | batch_commit ms/frame | ovl_region ms/frame | dwm_flush calls (in frames) | dwm_flush ms (outside frames) | cloak ms (outside frames) | session_begin ms (outside frames) | input->anim start ms | input->first frame ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| main #1 | minimize | 54.87 | 68.36 | 135.00 | 2.04 | 3.77 | 7.00 | 5.72 | 5.82 | 0.00 | 0.02 | 0.83 | 0.06 | 4.00 | 9.20 | 32.50 | 5.00 | 3.81 | 98.45 |
| main #2 | minimize | 53.55 | 69.96 | 127.00 | 2.06 | 6.46 | 8.00 | 5.72 | 6.52 | 0.00 | 0.02 | 0.93 | 0.06 | 5.00 | 7.50 | 39.60 | 8.70 | 3.95 | 128.63 |
| main #3 | minimize | 27.24 | 62.77 | 134.00 | 1.88 | 5.13 | 8.00 | 5.72 | 5.83 | 0.00 | 0.02 | 1.01 | 0.04 | 4.00 | 8.70 | 38.20 | 3.80 | 3.40 | 127.90 |

## resize

| label | scenario | command p50 ms | command p90 ms | frames | tick ms/frame | frame p90 ms | frames >2x budget | interval p50 ms | interval p90 ms | peek_frame ms/frame | session_overlays ms/frame | batch_commit ms/frame | ovl_region ms/frame | dwm_flush calls (in frames) | dwm_flush ms (outside frames) | cloak ms (outside frames) | session_begin ms (outside frames) | input->anim start ms | input->first frame ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| main #1 | resize | 80.87 | 99.70 | 57.00 | 4.77 | 11.75 | 6.00 | 5.72 | 11.84 | 0.00 | 0.05 | 2.37 | 0.11 | 3.00 | 15.80 | 38.60 | 3.20 | 0.27 | 95.85 |
| main #2 | resize | 82.66 | 84.72 | 57.00 | 4.87 | 12.50 | 6.00 | 5.72 | 12.57 | 0.00 | 0.03 | 2.34 | 0.12 | 3.00 | 5.50 | 30.40 | 3.10 | 0.26 | 105.06 |
| main #3 | resize | 84.12 | 92.50 | 57.00 | 4.68 | 10.91 | 5.00 | 5.72 | 11.16 | 0.00 | 0.04 | 2.42 | 0.14 | 3.00 | 9.90 | 34.20 | 3.00 | 0.27 | 104.99 |

## switch

| label | scenario | command p50 ms | command p90 ms | frames | tick ms/frame | frame p90 ms | frames >2x budget | interval p50 ms | interval p90 ms | peek_frame ms/frame | session_overlays ms/frame | batch_commit ms/frame | ovl_region ms/frame | dwm_flush calls (in frames) | dwm_flush ms (outside frames) | cloak ms (outside frames) | session_begin ms (outside frames) | input->anim start ms | input->first frame ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| main #1 | switch | 40.88 | 50.35 | 97.00 | 2.83 | 3.69 | 3.00 | 5.71 | 5.77 | 0.00 | 0.00 | 0.85 | 0.03 | 3.00 | 7.40 | 0.00 | 0.00 | 0.00 | 0.00 |
| main #2 | switch | 47.70 | 55.79 | 97.00 | 2.49 | 4.15 | 2.00 | 5.72 | 5.79 | 0.00 | 0.00 | 0.89 | 0.06 | 3.00 | 8.90 | 0.00 | 0.00 | 0.00 | 0.00 |
| main #3 | switch | 31.17 | 51.49 | 98.00 | 2.85 | 4.08 | 2.00 | 5.71 | 5.78 | 0.00 | 0.00 | 0.86 | 0.05 | 3.00 | 6.80 | 0.00 | 0.00 | 0.00 | 0.00 |
```

## Fixed tile order, no CPU load (bench-run.ps1)

```text
Bench run, 2026-11-12T21:56:06, modes main,nopeeks,default,morepeeks, target chrome, 4 round(s) of 10 bursts per scenario and mode, load 0, CPU load 0 thread(s) of 16
tiles main #1 ws 1: WindowsTerminal, Notepad, chrome, Everything, explorer, ApplicationFrameHost, ApplicationFrameHost
label=main #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          14.64
  command p90 ms                          16.51
  frames                                  63.00
  tick ms/frame                            0.59
  frame p90 ms                             0.94
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.28
  ovl_region ms/frame                      0.01
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            6.50
  cloak ms (outside frames)                9.80
  session_begin ms (outside frames)        0.90
  input->anim start ms                     0.21
  input->first frame ms                   15.20
| main #1 | move | 14.64 | 16.51 | 63.00 | 0.59 | 0.94 | 0.00 | 5.71 | 5.75 | 0.00 | 0.02 | 0.28 | 0.01 | 1.00 | 6.50 | 9.80 | 0.90 | 0.21 | 15.20 |
label=main #1 scenario=float target=chrome bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          77.49
  command p90 ms                          84.86
  frames                                  61.00
  tick ms/frame                            3.52
  frame p90 ms                             7.71
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          8.51
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.04
  batch_commit ms/frame                    1.84
  ovl_region ms/frame                      0.13
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            6.50
  cloak ms (outside frames)               22.30
  session_begin ms (outside frames)        2.70
  input->anim start ms                     0.23
  input->first frame ms                  100.04
| main #1 | float | 77.49 | 84.86 | 61.00 | 3.52 | 7.71 | 4.00 | 5.72 | 8.51 | 0.00 | 0.04 | 1.84 | 0.13 | 3.00 | 6.50 | 22.30 | 2.70 | 0.23 | 100.04 |
label=main #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          47.49
  command p90 ms                          51.50
  frames                                 133.00
  tick ms/frame                            1.76
  frame p90 ms                             3.10
  frames >2x budget                        7.00
  interval p50 ms                          5.72
  interval p90 ms                          5.79
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.81
  ovl_region ms/frame                      0.06
  dwm_flush calls (in frames)              4.00
  dwm_flush ms (outside frames)            7.00
  cloak ms (outside frames)               31.20
  session_begin ms (outside frames)        3.50
  input->anim start ms                     3.41
  input->first frame ms                  100.10
| main #1 | minimize | 47.49 | 51.50 | 133.00 | 1.76 | 3.10 | 7.00 | 5.72 | 5.79 | 0.00 | 0.02 | 0.81 | 0.06 | 4.00 | 7.00 | 31.20 | 3.50 | 3.41 | 100.10 |
label=main #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.8s
  command p50 ms                          77.09
  command p90 ms                          93.81
  frames                                  58.00
  tick ms/frame                            4.48
  frame p90 ms                             8.86
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          9.01
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.04
  batch_commit ms/frame                    2.09
  ovl_region ms/frame                      0.08
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            5.80
  cloak ms (outside frames)               27.70
  session_begin ms (outside frames)        2.90
  input->anim start ms                     0.22
  input->first frame ms                   98.42
| main #1 | resize | 77.09 | 93.81 | 58.00 | 4.48 | 8.86 | 5.00 | 5.72 | 9.01 | 0.00 | 0.04 | 2.09 | 0.08 | 3.00 | 5.80 | 27.70 | 2.90 | 0.22 | 98.42 |
Moved 3 window(s) to workspace '2' to switch to.
label=main #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          36.05
  command p90 ms                          41.55
  frames                                  97.00
  tick ms/frame                            2.40
  frame p90 ms                             3.65
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.77
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            7.00
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| main #1 | switch | 36.05 | 41.55 | 97.00 | 2.40 | 3.65 | 2.00 | 5.71 | 5.76 | 0.00 | 0.00 | 0.77 | 0.02 | 3.00 | 7.00 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles nopeeks #1 ws 1: WindowsTerminal, Notepad, chrome, Everything, explorer, ApplicationFrameHost, ApplicationFrameHost
label=nopeeks #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          17.99
  command p90 ms                          18.88
  frames                                  64.00
  tick ms/frame                            0.54
  frame p90 ms                             0.89
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.25
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            5.90
  cloak ms (outside frames)               10.60
  session_begin ms (outside frames)        0.90
  input->anim start ms                     0.28
  input->first frame ms                   18.47
| nopeeks #1 | move | 17.99 | 18.88 | 64.00 | 0.54 | 0.89 | 0.00 | 5.71 | 5.76 | 0.00 | 0.02 | 0.25 | 0.00 | 0.00 | 5.90 | 10.60 | 0.90 | 0.28 | 18.47 |
label=nopeeks #1 scenario=float target=chrome bursts=10 load=0 reports=10 wall=13.0s
  command p50 ms                         104.39
  command p90 ms                         112.40
  frames                                  59.00
  tick ms/frame                            3.60
  frame p90 ms                             8.50
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          8.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.06
  batch_commit ms/frame                    2.04
  ovl_region ms/frame                      0.14
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            9.10
  cloak ms (outside frames)               33.50
  session_begin ms (outside frames)        2.60
  input->anim start ms                     0.29
  input->first frame ms                  111.76
| nopeeks #1 | float | 104.39 | 112.40 | 59.00 | 3.60 | 8.50 | 4.00 | 5.72 | 8.77 | 0.00 | 0.06 | 2.04 | 0.14 | 0.00 | 9.10 | 33.50 | 2.60 | 0.29 | 111.76 |
label=nopeeks #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          36.34
  command p90 ms                          63.36
  frames                                 139.00
  tick ms/frame                            1.47
  frame p90 ms                             5.71
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          6.17
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.73
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            5.30
  cloak ms (outside frames)               33.70
  session_begin ms (outside frames)        4.20
  input->anim start ms                     4.30
  input->first frame ms                  109.65
| nopeeks #1 | minimize | 36.34 | 63.36 | 139.00 | 1.47 | 5.71 | 5.00 | 5.72 | 6.17 | 0.00 | 0.02 | 0.73 | 0.04 | 1.00 | 5.30 | 33.70 | 4.20 | 4.30 | 109.65 |
label=nopeeks #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=13.0s
  command p50 ms                          87.26
  command p90 ms                         103.17
  frames                                  57.00
  tick ms/frame                            3.27
  frame p90 ms                             8.69
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          9.18
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.05
  batch_commit ms/frame                    2.21
  ovl_region ms/frame                      0.07
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            6.30
  cloak ms (outside frames)               33.80
  session_begin ms (outside frames)        2.50
  input->anim start ms                     0.29
  input->first frame ms                   98.08
| nopeeks #1 | resize | 87.26 | 103.17 | 57.00 | 3.27 | 8.69 | 4.00 | 5.72 | 9.18 | 0.00 | 0.05 | 2.21 | 0.07 | 0.00 | 6.30 | 33.80 | 2.50 | 0.29 | 98.08 |
Moved 3 window(s) to workspace '2' to switch to.
label=nopeeks #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          53.65
  command p90 ms                          57.20
  frames                                  98.00
  tick ms/frame                            1.75
  frame p90 ms                             1.63
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            9.40
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| nopeeks #1 | switch | 53.65 | 57.20 | 98.00 | 1.75 | 1.63 | 3.00 | 5.72 | 5.77 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 9.40 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles default #1 ws 1: WindowsTerminal, Notepad, chrome, Everything, explorer, ApplicationFrameHost, ApplicationFrameHost
label=default #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          24.21
  command p90 ms                          25.99
  frames                                  63.00
  tick ms/frame                            0.84
  frame p90 ms                             0.51
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  peek_frame ms/frame                      0.44
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            8.90
  cloak ms (outside frames)               12.10
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #1 | move | 24.21 | 25.99 | 63.00 | 0.84 | 0.51 | 2.00 | 5.71 | 5.75 | 0.44 | 0.00 | 0.00 | 0.00 | 0.00 | 8.90 | 12.10 | 0.00 | 0.00 | 0.00 |
label=default #1 scenario=float target=chrome bursts=10 load=0 reports=10 wall=13.0s
  command p50 ms                         104.92
  command p90 ms                         115.05
  frames                                  60.00
  tick ms/frame                            3.97
  frame p90 ms                             7.40
  frames >2x budget                        4.00
  interval p50 ms                          5.71
  interval p90 ms                          9.39
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.05
  batch_commit ms/frame                    2.00
  ovl_region ms/frame                      0.13
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            6.40
  cloak ms (outside frames)               34.50
  session_begin ms (outside frames)        2.60
  input->anim start ms                     0.31
  input->first frame ms                  111.44
| default #1 | float | 104.92 | 115.05 | 60.00 | 3.97 | 7.40 | 4.00 | 5.71 | 9.39 | 0.00 | 0.05 | 2.00 | 0.13 | 0.00 | 6.40 | 34.50 | 2.60 | 0.31 | 111.44 |
label=default #1 scenario=minimize target=chrome bursts=10 load=0 reports=9 wall=14.1s
  command p50 ms                          68.91
  command p90 ms                          76.01
  frames                                 132.00
  tick ms/frame                            1.80
  frame p90 ms                             3.87
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          5.78
  peek_frame ms/frame                      0.71
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.45
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)           10.70
  cloak ms (outside frames)               31.10
  session_begin ms (outside frames)        2.60
  input->anim start ms                     2.69
  input->first frame ms                   91.64
| default #1 | minimize | 68.91 | 76.01 | 132.00 | 1.80 | 3.87 | 5.00 | 5.72 | 5.78 | 0.71 | 0.02 | 0.45 | 0.04 | 1.00 | 10.70 | 31.10 | 2.60 | 2.69 | 91.64 |
label=default #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=13.3s
  command p50 ms                         133.14
  command p90 ms                         136.06
  frames                                  57.00
  tick ms/frame                            5.46
  frame p90 ms                             4.00
  frames >2x budget                        5.00
  interval p50 ms                          5.71
  interval p90 ms                          6.77
  peek_frame ms/frame                      3.46
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.10
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           11.30
  cloak ms (outside frames)               37.80
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #1 | resize | 133.14 | 136.06 | 57.00 | 5.46 | 4.00 | 5.00 | 5.71 | 6.77 | 3.46 | 0.00 | 0.00 | 0.10 | 2.00 | 11.30 | 37.80 | 0.00 | 0.00 | 0.00 |
Moved 3 window(s) to workspace '2' to switch to.
label=default #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          56.00
  command p90 ms                          70.07
  frames                                  99.00
  tick ms/frame                            1.93
  frame p90 ms                             1.66
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            9.20
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #1 | switch | 56.00 | 70.07 | 99.00 | 1.93 | 1.66 | 3.00 | 5.72 | 5.77 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 9.20 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles morepeeks #1 ws 1: WindowsTerminal, Notepad, chrome, Everything, explorer, ApplicationFrameHost, ApplicationFrameHost
label=morepeeks #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          24.37
  command p90 ms                          26.08
  frames                                  63.00
  tick ms/frame                            0.85
  frame p90 ms                             0.55
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  peek_frame ms/frame                      0.41
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            8.70
  cloak ms (outside frames)               12.10
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #1 | move | 24.37 | 26.08 | 63.00 | 0.85 | 0.55 | 2.00 | 5.71 | 5.76 | 0.41 | 0.00 | 0.00 | 0.00 | 0.00 | 8.70 | 12.10 | 0.00 | 0.00 | 0.00 |
label=morepeeks #1 scenario=float target=chrome bursts=10 load=0 reports=10 wall=13.3s
  command p50 ms                         130.28
  command p90 ms                         144.37
  frames                                  59.00
  tick ms/frame                            4.98
  frame p90 ms                             3.91
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          6.69
  peek_frame ms/frame                      3.16
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.08
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            8.60
  cloak ms (outside frames)               37.50
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #1 | float | 130.28 | 144.37 | 59.00 | 4.98 | 3.91 | 5.00 | 5.72 | 6.69 | 3.16 | 0.00 | 0.00 | 0.08 | 2.00 | 8.60 | 37.50 | 0.00 | 0.00 | 0.00 |
label=morepeeks #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          62.97
  command p90 ms                          71.03
  frames                                 134.00
  tick ms/frame                            1.77
  frame p90 ms                             3.49
  frames >2x budget                        6.00
  interval p50 ms                          5.72
  interval p90 ms                          5.77
  peek_frame ms/frame                      1.12
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.51
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)           10.60
  cloak ms (outside frames)               42.80
  session_begin ms (outside frames)        2.40
  input->anim start ms                    37.66
  input->first frame ms                  217.07
| morepeeks #1 | minimize | 62.97 | 71.03 | 134.00 | 1.77 | 3.49 | 6.00 | 5.72 | 5.77 | 1.12 | 0.02 | 0.51 | 0.05 | 1.00 | 10.60 | 42.80 | 2.40 | 37.66 | 217.07 |
label=morepeeks #1 scenario=resize target=chrome bursts=10 load=0 reports=11 wall=13.4s
  command p50 ms                         132.65
  command p90 ms                         158.91
  frames                                  56.00
  tick ms/frame                            4.90
  frame p90 ms                             4.32
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          6.89
  peek_frame ms/frame                      3.40
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.09
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           16.10
  cloak ms (outside frames)               43.20
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #1 | resize | 132.65 | 158.91 | 56.00 | 4.90 | 4.32 | 5.00 | 5.72 | 6.89 | 3.40 | 0.00 | 0.00 | 0.09 | 2.00 | 16.10 | 43.20 | 0.00 | 0.00 | 0.00 |
Moved 3 window(s) to workspace '2' to switch to.
label=morepeeks #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          54.36
  command p90 ms                          59.16
  frames                                  99.00
  tick ms/frame                            1.97
  frame p90 ms                             1.84
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            7.00
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #1 | switch | 54.36 | 59.16 | 99.00 | 1.97 | 1.84 | 3.00 | 5.72 | 5.77 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 7.00 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles main #2 ws 1: WindowsTerminal, Notepad, chrome, Everything, explorer, ApplicationFrameHost, ApplicationFrameHost
label=main #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          15.39
  command p90 ms                          17.27
  frames                                  63.00
  tick ms/frame                            0.66
  frame p90 ms                             1.06
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.31
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            5.90
  cloak ms (outside frames)                9.30
  session_begin ms (outside frames)        0.90
  input->anim start ms                     0.21
  input->first frame ms                   16.38
| main #2 | move | 15.39 | 17.27 | 63.00 | 0.66 | 1.06 | 0.00 | 5.71 | 5.74 | 0.00 | 0.02 | 0.31 | 0.00 | 1.00 | 5.90 | 9.30 | 0.90 | 0.21 | 16.38 |
label=main #2 scenario=float target=chrome bursts=10 load=0 reports=10 wall=12.9s
  command p50 ms                          84.73
  command p90 ms                          89.01
  frames                                  60.00
  tick ms/frame                            3.86
  frame p90 ms                             9.83
  frames >2x budget                        6.00
  interval p50 ms                          5.72
  interval p90 ms                         10.83
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.04
  batch_commit ms/frame                    2.08
  ovl_region ms/frame                      0.14
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            7.50
  cloak ms (outside frames)               31.70
  session_begin ms (outside frames)        2.70
  input->anim start ms                     0.24
  input->first frame ms                  106.79
| main #2 | float | 84.73 | 89.01 | 60.00 | 3.86 | 9.83 | 6.00 | 5.72 | 10.83 | 0.00 | 0.04 | 2.08 | 0.14 | 3.00 | 7.50 | 31.70 | 2.70 | 0.24 | 106.79 |
label=main #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          28.04
  command p90 ms                          67.25
  frames                                 132.00
  tick ms/frame                            1.87
  frame p90 ms                             3.50
  frames >2x budget                        5.00
  interval p50 ms                          5.71
  interval p90 ms                          5.79
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.01
  batch_commit ms/frame                    0.76
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              5.00
  dwm_flush ms (outside frames)            7.80
  cloak ms (outside frames)               39.20
  session_begin ms (outside frames)        5.50
  input->anim start ms                     3.84
  input->first frame ms                  117.15
| main #2 | minimize | 28.04 | 67.25 | 132.00 | 1.87 | 3.50 | 5.00 | 5.71 | 5.79 | 0.00 | 0.01 | 0.76 | 0.04 | 5.00 | 7.80 | 39.20 | 5.50 | 3.84 | 117.15 |
label=main #2 scenario=resize target=chrome bursts=10 load=0 reports=11 wall=12.8s
  command p50 ms                          75.33
  command p90 ms                          89.74
  frames                                  58.00
  tick ms/frame                            3.98
  frame p90 ms                             6.41
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          9.08
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.03
  batch_commit ms/frame                    1.96
  ovl_region ms/frame                      0.07
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            8.70
  cloak ms (outside frames)               32.60
  session_begin ms (outside frames)        2.60
  input->anim start ms                     0.21
  input->first frame ms                   84.03
| main #2 | resize | 75.33 | 89.74 | 58.00 | 3.98 | 6.41 | 4.00 | 5.72 | 9.08 | 0.00 | 0.03 | 1.96 | 0.07 | 2.00 | 8.70 | 32.60 | 2.60 | 0.21 | 84.03 |
Moved 3 window(s) to workspace '2' to switch to.
label=main #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          31.07
  command p90 ms                          53.04
  frames                                  97.00
  tick ms/frame                            2.48
  frame p90 ms                             3.76
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.78
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            7.80
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| main #2 | switch | 31.07 | 53.04 | 97.00 | 2.48 | 3.76 | 3.00 | 5.71 | 5.76 | 0.00 | 0.00 | 0.78 | 0.02 | 3.00 | 7.80 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles nopeeks #2 ws 1: WindowsTerminal, Notepad, chrome, Everything, explorer, ApplicationFrameHost, ApplicationFrameHost
label=nopeeks #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          18.38
  command p90 ms                          19.37
  frames                                  62.00
  tick ms/frame                            0.56
  frame p90 ms                             0.96
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.27
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            6.70
  cloak ms (outside frames)               11.00
  session_begin ms (outside frames)        0.90
  input->anim start ms                     0.26
  input->first frame ms                   19.18
| nopeeks #2 | move | 18.38 | 19.37 | 62.00 | 0.56 | 0.96 | 0.00 | 5.71 | 5.75 | 0.00 | 0.02 | 0.27 | 0.00 | 0.00 | 6.70 | 11.00 | 0.90 | 0.26 | 19.18 |
label=nopeeks #2 scenario=float target=chrome bursts=10 load=0 reports=10 wall=13.0s
  command p50 ms                         107.02
  command p90 ms                         119.88
  frames                                  60.00
  tick ms/frame                            3.63
  frame p90 ms                             7.51
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          7.94
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.06
  batch_commit ms/frame                    2.14
  ovl_region ms/frame                      0.14
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            7.20
  cloak ms (outside frames)               38.50
  session_begin ms (outside frames)        2.60
  input->anim start ms                     0.28
  input->first frame ms                  114.45
| nopeeks #2 | float | 107.02 | 119.88 | 60.00 | 3.63 | 7.51 | 4.00 | 5.72 | 7.94 | 0.00 | 0.06 | 2.14 | 0.14 | 0.00 | 7.20 | 38.50 | 2.60 | 0.28 | 114.45 |
label=nopeeks #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          59.19
  command p90 ms                          65.90
  frames                                 138.00
  tick ms/frame                            1.53
  frame p90 ms                             5.06
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          5.92
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.80
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            7.40
  cloak ms (outside frames)               43.10
  session_begin ms (outside frames)        6.30
  input->anim start ms                     4.20
  input->first frame ms                  124.39
| nopeeks #2 | minimize | 59.19 | 65.90 | 138.00 | 1.53 | 5.06 | 4.00 | 5.72 | 5.92 | 0.00 | 0.02 | 0.80 | 0.03 | 1.00 | 7.40 | 43.10 | 6.30 | 4.20 | 124.39 |
label=nopeeks #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=13.1s
  command p50 ms                          92.81
  command p90 ms                         102.84
  frames                                  57.00
  tick ms/frame                            3.97
  frame p90 ms                             6.53
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          6.69
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.07
  batch_commit ms/frame                    2.24
  ovl_region ms/frame                      0.08
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            9.40
  cloak ms (outside frames)               39.60
  session_begin ms (outside frames)        2.90
  input->anim start ms                     0.28
  input->first frame ms                  105.54
| nopeeks #2 | resize | 92.81 | 102.84 | 57.00 | 3.97 | 6.53 | 4.00 | 5.72 | 6.69 | 0.00 | 0.07 | 2.24 | 0.08 | 0.00 | 9.40 | 39.60 | 2.90 | 0.28 | 105.54 |
Moved 3 window(s) to workspace '2' to switch to.
label=nopeeks #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          64.58
  command p90 ms                          78.21
  frames                                  99.00
  tick ms/frame                            1.75
  frame p90 ms                             1.77
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            8.10
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| nopeeks #2 | switch | 64.58 | 78.21 | 99.00 | 1.75 | 1.77 | 3.00 | 5.71 | 5.76 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 8.10 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles default #2 ws 1: WindowsTerminal, Notepad, chrome, Everything, explorer, ApplicationFrameHost, ApplicationFrameHost
label=default #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          25.08
  command p90 ms                          27.61
  frames                                  63.00
  tick ms/frame                            0.86
  frame p90 ms                             0.76
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  peek_frame ms/frame                      0.43
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            9.00
  cloak ms (outside frames)               12.40
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #2 | move | 25.08 | 27.61 | 63.00 | 0.86 | 0.76 | 2.00 | 5.71 | 5.76 | 0.43 | 0.00 | 0.00 | 0.00 | 0.00 | 9.00 | 12.40 | 0.00 | 0.00 | 0.00 |
label=default #2 scenario=float target=chrome bursts=10 load=0 reports=10 wall=13.1s
  command p50 ms                          97.55
  command p90 ms                         123.35
  frames                                  60.00
  tick ms/frame                            4.21
  frame p90 ms                             9.25
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          9.54
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.04
  batch_commit ms/frame                    2.01
  ovl_region ms/frame                      0.09
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            8.50
  cloak ms (outside frames)               46.90
  session_begin ms (outside frames)        2.70
  input->anim start ms                     0.30
  input->first frame ms                  125.50
| default #2 | float | 97.55 | 123.35 | 60.00 | 4.21 | 9.25 | 4.00 | 5.72 | 9.54 | 0.00 | 0.04 | 2.01 | 0.09 | 0.00 | 8.50 | 46.90 | 2.70 | 0.30 | 125.50 |
label=default #2 scenario=minimize target=chrome bursts=10 load=0 reports=9 wall=13.2s
  command p50 ms                          67.03
  command p90 ms                          85.03
  frames                                 133.00
  tick ms/frame                            1.79
  frame p90 ms                             3.34
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.89
  session_overlays ms/frame                0.01
  batch_commit ms/frame                    0.50
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)           14.10
  cloak ms (outside frames)               44.20
  session_begin ms (outside frames)        2.60
  input->anim start ms                     3.21
  input->first frame ms                   96.79
| default #2 | minimize | 67.03 | 85.03 | 133.00 | 1.79 | 3.34 | 5.00 | 5.72 | 5.77 | 0.89 | 0.01 | 0.50 | 0.03 | 1.00 | 14.10 | 44.20 | 2.60 | 3.21 | 96.79 |
label=default #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=13.4s
  command p50 ms                         138.25
  command p90 ms                         150.18
  frames                                  59.00
  tick ms/frame                            4.75
  frame p90 ms                             4.47
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          5.97
  peek_frame ms/frame                      3.16
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.08
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            9.30
  cloak ms (outside frames)               40.90
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #2 | resize | 138.25 | 150.18 | 59.00 | 4.75 | 4.47 | 5.00 | 5.72 | 5.97 | 3.16 | 0.00 | 0.00 | 0.08 | 2.00 | 9.30 | 40.90 | 0.00 | 0.00 | 0.00 |
Moved 3 window(s) to workspace '2' to switch to.
label=default #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          52.23
  command p90 ms                          64.55
  frames                                  99.00
  tick ms/frame                            1.86
  frame p90 ms                             1.86
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            7.50
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #2 | switch | 52.23 | 64.55 | 99.00 | 1.86 | 1.86 | 3.00 | 5.72 | 5.77 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 7.50 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles morepeeks #2 ws 1: WindowsTerminal, Notepad, chrome, Everything, explorer, ApplicationFrameHost, ApplicationFrameHost
label=morepeeks #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          25.15
  command p90 ms                          26.49
  frames                                  63.00
  tick ms/frame                            0.83
  frame p90 ms                             0.56
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  peek_frame ms/frame                      0.47
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            8.70
  cloak ms (outside frames)               12.20
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #2 | move | 25.15 | 26.49 | 63.00 | 0.83 | 0.56 | 2.00 | 5.71 | 5.76 | 0.47 | 0.00 | 0.00 | 0.00 | 0.00 | 8.70 | 12.20 | 0.00 | 0.00 | 0.00 |
label=morepeeks #2 scenario=float target=chrome bursts=10 load=0 reports=10 wall=13.4s
  command p50 ms                         135.89
  command p90 ms                         149.80
  frames                                  59.00
  tick ms/frame                            4.97
  frame p90 ms                             4.43
  frames >2x budget                        5.00
  interval p50 ms                          5.71
  interval p90 ms                          6.51
  peek_frame ms/frame                      3.11
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.08
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           15.80
  cloak ms (outside frames)               41.20
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #2 | float | 135.89 | 149.80 | 59.00 | 4.97 | 4.43 | 5.00 | 5.71 | 6.51 | 3.11 | 0.00 | 0.00 | 0.08 | 2.00 | 15.80 | 41.20 | 0.00 | 0.00 | 0.00 |
label=morepeeks #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          66.33
  command p90 ms                          70.48
  frames                                 135.00
  tick ms/frame                            1.76
  frame p90 ms                             3.74
  frames >2x budget                        6.00
  interval p50 ms                          5.72
  interval p90 ms                          5.78
  peek_frame ms/frame                      1.19
  session_overlays ms/frame                0.03
  batch_commit ms/frame                    0.54
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            7.30
  cloak ms (outside frames)               30.70
  session_begin ms (outside frames)        2.50
  input->anim start ms                    27.88
  input->first frame ms                  170.12
| morepeeks #2 | minimize | 66.33 | 70.48 | 135.00 | 1.76 | 3.74 | 6.00 | 5.72 | 5.78 | 1.19 | 0.03 | 0.54 | 0.04 | 1.00 | 7.30 | 30.70 | 2.50 | 27.88 | 170.12 |
label=morepeeks #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=13.3s
  command p50 ms                         128.07
  command p90 ms                         143.06
  frames                                  58.00
  tick ms/frame                            5.48
  frame p90 ms                             4.37
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          6.96
  peek_frame ms/frame                      3.25
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.08
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            7.60
  cloak ms (outside frames)               38.50
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #2 | resize | 128.07 | 143.06 | 58.00 | 5.48 | 4.37 | 5.00 | 5.72 | 6.96 | 3.25 | 0.00 | 0.00 | 0.08 | 2.00 | 7.60 | 38.50 | 0.00 | 0.00 | 0.00 |
Moved 3 window(s) to workspace '2' to switch to.
label=morepeeks #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          60.92
  command p90 ms                          72.72
  frames                                  99.00
  tick ms/frame                            1.93
  frame p90 ms                             1.74
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            9.20
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #2 | switch | 60.92 | 72.72 | 99.00 | 1.93 | 1.74 | 3.00 | 5.72 | 5.77 | 0.00 | 0.00 | 0.00 | 0.02 | 0.00 | 9.20 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles main #3 ws 1: WindowsTerminal, Notepad, chrome, Everything, explorer, ApplicationFrameHost, ApplicationFrameHost
label=main #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          16.17
  command p90 ms                          16.49
  frames                                  63.00
  tick ms/frame                            0.66
  frame p90 ms                             0.95
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.28
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            6.70
  cloak ms (outside frames)                9.90
  session_begin ms (outside frames)        0.90
  input->anim start ms                     0.22
  input->first frame ms                   16.92
| main #3 | move | 16.17 | 16.49 | 63.00 | 0.66 | 0.95 | 0.00 | 5.71 | 5.75 | 0.00 | 0.02 | 0.28 | 0.00 | 1.00 | 6.70 | 9.90 | 0.90 | 0.22 | 16.92 |
label=main #3 scenario=float target=chrome bursts=10 load=0 reports=9 wall=12.8s
  command p50 ms                          71.34
  command p90 ms                          88.66
  frames                                  62.00
  tick ms/frame                            3.81
  frame p90 ms                             6.50
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          8.31
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.03
  batch_commit ms/frame                    1.16
  ovl_region ms/frame                      0.08
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            8.10
  cloak ms (outside frames)               30.70
  session_begin ms (outside frames)        2.70
  input->anim start ms                     0.23
  input->first frame ms                   76.70
| main #3 | float | 71.34 | 88.66 | 62.00 | 3.81 | 6.50 | 5.00 | 5.72 | 8.31 | 0.00 | 0.03 | 1.16 | 0.08 | 3.00 | 8.10 | 30.70 | 2.70 | 0.23 | 76.70 |
label=main #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          49.52
  command p90 ms                          60.52
  frames                                 134.00
  tick ms/frame                            1.77
  frame p90 ms                             3.51
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.01
  batch_commit ms/frame                    0.75
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              4.00
  dwm_flush ms (outside frames)            8.40
  cloak ms (outside frames)               28.40
  session_begin ms (outside frames)        6.20
  input->anim start ms                     4.30
  input->first frame ms                   97.46
| main #3 | minimize | 49.52 | 60.52 | 134.00 | 1.77 | 3.51 | 5.00 | 5.72 | 5.77 | 0.00 | 0.01 | 0.75 | 0.05 | 4.00 | 8.40 | 28.40 | 6.20 | 4.30 | 97.46 |
label=main #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.9s
  command p50 ms                          78.32
  command p90 ms                          80.00
  frames                                  59.00
  tick ms/frame                            4.59
  frame p90 ms                             8.48
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          9.23
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.04
  batch_commit ms/frame                    2.05
  ovl_region ms/frame                      0.07
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            5.20
  cloak ms (outside frames)               28.60
  session_begin ms (outside frames)        2.60
  input->anim start ms                     0.21
  input->first frame ms                   93.07
| main #3 | resize | 78.32 | 80.00 | 59.00 | 4.59 | 8.48 | 5.00 | 5.72 | 9.23 | 0.00 | 0.04 | 2.05 | 0.07 | 3.00 | 5.20 | 28.60 | 2.60 | 0.21 | 93.07 |
Moved 3 window(s) to workspace '2' to switch to.
label=main #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          37.31
  command p90 ms                          38.28
  frames                                  97.00
  tick ms/frame                            2.40
  frame p90 ms                             3.50
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.74
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            9.30
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| main #3 | switch | 37.31 | 38.28 | 97.00 | 2.40 | 3.50 | 2.00 | 5.71 | 5.77 | 0.00 | 0.00 | 0.74 | 0.02 | 3.00 | 9.30 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles nopeeks #3 ws 1: WindowsTerminal, Notepad, chrome, Everything, explorer, ApplicationFrameHost, ApplicationFrameHost
label=nopeeks #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          18.80
  command p90 ms                          19.17
  frames                                  62.00
  tick ms/frame                            0.55
  frame p90 ms                             0.93
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.26
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            6.60
  cloak ms (outside frames)               11.20
  session_begin ms (outside frames)        0.90
  input->anim start ms                     0.27
  input->first frame ms                   19.40
| nopeeks #3 | move | 18.80 | 19.17 | 62.00 | 0.55 | 0.93 | 0.00 | 5.71 | 5.75 | 0.00 | 0.02 | 0.26 | 0.00 | 0.00 | 6.60 | 11.20 | 0.90 | 0.27 | 19.40 |
label=nopeeks #3 scenario=float target=chrome bursts=10 load=0 reports=11 wall=13.0s
  command p50 ms                         107.21
  command p90 ms                         116.98
  frames                                  61.00
  tick ms/frame                            3.90
  frame p90 ms                             4.25
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          6.12
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.05
  batch_commit ms/frame                    1.29
  ovl_region ms/frame                      0.13
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           11.40
  cloak ms (outside frames)               32.60
  session_begin ms (outside frames)        2.60
  input->anim start ms                     0.28
  input->first frame ms                   84.27
| nopeeks #3 | float | 107.21 | 116.98 | 61.00 | 3.90 | 4.25 | 4.00 | 5.72 | 6.12 | 0.00 | 0.05 | 1.29 | 0.13 | 0.00 | 11.40 | 32.60 | 2.60 | 0.28 | 84.27 |
label=nopeeks #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          55.97
  command p90 ms                          69.39
  frames                                 138.00
  tick ms/frame                            1.60
  frame p90 ms                             5.15
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          5.79
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.64
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)           11.10
  cloak ms (outside frames)               41.60
  session_begin ms (outside frames)        7.10
  input->anim start ms                     4.27
  input->first frame ms                  121.08
| nopeeks #3 | minimize | 55.97 | 69.39 | 138.00 | 1.60 | 5.15 | 5.00 | 5.72 | 5.79 | 0.00 | 0.02 | 0.64 | 0.04 | 1.00 | 11.10 | 41.60 | 7.10 | 4.27 | 121.08 |
label=nopeeks #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=13.0s
  command p50 ms                          94.09
  command p90 ms                         107.41
  frames                                  59.00
  tick ms/frame                            3.92
  frame p90 ms                             8.38
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          8.89
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.06
  batch_commit ms/frame                    2.05
  ovl_region ms/frame                      0.07
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           10.00
  cloak ms (outside frames)               33.90
  session_begin ms (outside frames)        2.50
  input->anim start ms                     0.28
  input->first frame ms                  114.85
| nopeeks #3 | resize | 94.09 | 107.41 | 59.00 | 3.92 | 8.38 | 4.00 | 5.72 | 8.89 | 0.00 | 0.06 | 2.05 | 0.07 | 0.00 | 10.00 | 33.90 | 2.50 | 0.28 | 114.85 |
Moved 3 window(s) to workspace '2' to switch to.
label=nopeeks #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          66.29
  command p90 ms                          69.24
  frames                                  99.00
  tick ms/frame                            1.84
  frame p90 ms                             1.72
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            7.80
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| nopeeks #3 | switch | 66.29 | 69.24 | 99.00 | 1.84 | 1.72 | 3.00 | 5.72 | 5.77 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 7.80 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles default #3 ws 1: WindowsTerminal, Notepad, chrome, Everything, explorer, ApplicationFrameHost, ApplicationFrameHost
label=default #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          23.73
  command p90 ms                          25.52
  frames                                  63.00
  tick ms/frame                            0.81
  frame p90 ms                             0.56
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  peek_frame ms/frame                      0.44
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            9.00
  cloak ms (outside frames)               12.40
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #3 | move | 23.73 | 25.52 | 63.00 | 0.81 | 0.56 | 2.00 | 5.71 | 5.76 | 0.44 | 0.00 | 0.00 | 0.00 | 0.00 | 9.00 | 12.40 | 0.00 | 0.00 | 0.00 |
label=default #3 scenario=float target=chrome bursts=10 load=0 reports=10 wall=13.0s
  command p50 ms                         100.82
  command p90 ms                         117.96
  frames                                  60.00
  tick ms/frame                            4.07
  frame p90 ms                             8.32
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          8.47
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.05
  batch_commit ms/frame                    1.99
  ovl_region ms/frame                      0.15
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            6.10
  cloak ms (outside frames)               34.60
  session_begin ms (outside frames)        2.70
  input->anim start ms                     0.33
  input->first frame ms                  107.00
| default #3 | float | 100.82 | 117.96 | 60.00 | 4.07 | 8.32 | 4.00 | 5.72 | 8.47 | 0.00 | 0.05 | 1.99 | 0.15 | 0.00 | 6.10 | 34.60 | 2.70 | 0.33 | 107.00 |
label=default #3 scenario=minimize target=chrome bursts=10 load=0 reports=9 wall=13.3s
  command p50 ms                          69.52
  command p90 ms                          81.97
  frames                                 133.00
  tick ms/frame                            1.74
  frame p90 ms                             3.68
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.68
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.44
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)           16.80
  cloak ms (outside frames)               36.50
  session_begin ms (outside frames)        2.50
  input->anim start ms                     3.45
  input->first frame ms                   96.72
| default #3 | minimize | 69.52 | 81.97 | 133.00 | 1.74 | 3.68 | 5.00 | 5.72 | 5.77 | 0.68 | 0.02 | 0.44 | 0.03 | 1.00 | 16.80 | 36.50 | 2.50 | 3.45 | 96.72 |
label=default #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=13.4s
  command p50 ms                         133.16
  command p90 ms                         136.90
  frames                                  59.00
  tick ms/frame                            5.39
  frame p90 ms                             4.31
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          6.33
  peek_frame ms/frame                      3.37
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.09
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            9.80
  cloak ms (outside frames)               41.90
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #3 | resize | 133.16 | 136.90 | 59.00 | 5.39 | 4.31 | 5.00 | 5.72 | 6.33 | 3.37 | 0.00 | 0.00 | 0.09 | 2.00 | 9.80 | 41.90 | 0.00 | 0.00 | 0.00 |
Moved 3 window(s) to workspace '2' to switch to.
label=default #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          54.29
  command p90 ms                          67.46
  frames                                  99.00
  tick ms/frame                            1.97
  frame p90 ms                             1.89
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            8.00
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #3 | switch | 54.29 | 67.46 | 99.00 | 1.97 | 1.89 | 3.00 | 5.72 | 5.77 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 8.00 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles morepeeks #3 ws 1: WindowsTerminal, Notepad, chrome, Everything, explorer, ApplicationFrameHost, ApplicationFrameHost
label=morepeeks #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          24.12
  command p90 ms                          26.50
  frames                                  63.00
  tick ms/frame                            0.82
  frame p90 ms                             0.55
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  peek_frame ms/frame                      0.43
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            8.10
  cloak ms (outside frames)               11.70
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #3 | move | 24.12 | 26.50 | 63.00 | 0.82 | 0.55 | 2.00 | 5.71 | 5.76 | 0.43 | 0.00 | 0.00 | 0.00 | 0.00 | 8.10 | 11.70 | 0.00 | 0.00 | 0.00 |
label=morepeeks #3 scenario=float target=chrome bursts=10 load=0 reports=10 wall=13.3s
  command p50 ms                         139.85
  command p90 ms                         142.71
  frames                                  58.00
  tick ms/frame                            5.11
  frame p90 ms                             4.06
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          6.88
  peek_frame ms/frame                      3.08
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.09
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            6.60
  cloak ms (outside frames)               37.30
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #3 | float | 139.85 | 142.71 | 58.00 | 5.11 | 4.06 | 5.00 | 5.72 | 6.88 | 3.08 | 0.00 | 0.00 | 0.09 | 2.00 | 6.60 | 37.30 | 0.00 | 0.00 | 0.00 |
label=morepeeks #3 scenario=minimize target=chrome bursts=10 load=0 reports=9 wall=13.5s
  command p50 ms                          68.52
  command p90 ms                          83.56
  frames                                 133.00
  tick ms/frame                            1.78
  frame p90 ms                             3.35
  frames >2x budget                        6.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.60
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.51
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)           11.50
  cloak ms (outside frames)               31.70
  session_begin ms (outside frames)        2.50
  input->anim start ms                     0.71
  input->first frame ms                   97.70
| morepeeks #3 | minimize | 68.52 | 83.56 | 133.00 | 1.78 | 3.35 | 6.00 | 5.71 | 5.77 | 0.60 | 0.02 | 0.51 | 0.04 | 1.00 | 11.50 | 31.70 | 2.50 | 0.71 | 97.70 |
label=morepeeks #3 scenario=resize target=chrome bursts=10 load=0 reports=11 wall=13.4s
  command p50 ms                         125.26
  command p90 ms                         148.01
  frames                                  56.00
  tick ms/frame                            4.94
  frame p90 ms                             4.27
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          5.85
  peek_frame ms/frame                      3.25
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.08
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)           20.60
  cloak ms (outside frames)               40.70
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #3 | resize | 125.26 | 148.01 | 56.00 | 4.94 | 4.27 | 5.00 | 5.72 | 5.85 | 3.25 | 0.00 | 0.00 | 0.08 | 1.00 | 20.60 | 40.70 | 0.00 | 0.00 | 0.00 |
Moved 3 window(s) to workspace '2' to switch to.
label=morepeeks #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          52.98
  command p90 ms                          62.55
  frames                                  99.00
  tick ms/frame                            1.92
  frame p90 ms                             1.74
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            8.20
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #3 | switch | 52.98 | 62.55 | 99.00 | 1.92 | 1.74 | 3.00 | 5.71 | 5.77 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 8.20 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles main #4 ws 1: WindowsTerminal, Notepad, chrome, Everything, explorer, ApplicationFrameHost, ApplicationFrameHost
label=main #4 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          15.97
  command p90 ms                          16.94
  frames                                  63.00
  tick ms/frame                            0.69
  frame p90 ms                             0.96
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.29
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            6.80
  cloak ms (outside frames)                9.70
  session_begin ms (outside frames)        0.90
  input->anim start ms                     0.21
  input->first frame ms                   16.59
| main #4 | move | 15.97 | 16.94 | 63.00 | 0.69 | 0.96 | 0.00 | 5.71 | 5.75 | 0.00 | 0.02 | 0.29 | 0.00 | 1.00 | 6.80 | 9.70 | 0.90 | 0.21 | 16.59 |
label=main #4 scenario=float target=chrome bursts=10 load=0 reports=9 wall=12.8s
  command p50 ms                          66.83
  command p90 ms                          95.67
  frames                                  61.00
  tick ms/frame                            3.66
  frame p90 ms                             6.40
  frames >2x budget                        6.00
  interval p50 ms                          5.72
  interval p90 ms                          6.51
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.03
  batch_commit ms/frame                    1.43
  ovl_region ms/frame                      0.10
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)           10.30
  cloak ms (outside frames)               32.70
  session_begin ms (outside frames)        2.70
  input->anim start ms                     0.24
  input->first frame ms                   74.08
| main #4 | float | 66.83 | 95.67 | 61.00 | 3.66 | 6.40 | 6.00 | 5.72 | 6.51 | 0.00 | 0.03 | 1.43 | 0.10 | 3.00 | 10.30 | 32.70 | 2.70 | 0.24 | 74.08 |
label=main #4 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          48.06
  command p90 ms                          53.46
  frames                                 134.00
  tick ms/frame                            1.56
  frame p90 ms                             4.33
  frames >2x budget                        5.00
  interval p50 ms                          5.71
  interval p90 ms                          5.79
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.01
  batch_commit ms/frame                    0.71
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              4.00
  dwm_flush ms (outside frames)            7.50
  cloak ms (outside frames)               32.40
  session_begin ms (outside frames)        4.20
  input->anim start ms                     4.10
  input->first frame ms                  102.14
| main #4 | minimize | 48.06 | 53.46 | 134.00 | 1.56 | 4.33 | 5.00 | 5.71 | 5.79 | 0.00 | 0.01 | 0.71 | 0.04 | 4.00 | 7.50 | 32.40 | 4.20 | 4.10 | 102.14 |
label=main #4 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.8s
  command p50 ms                          70.57
  command p90 ms                          82.60
  frames                                  59.00
  tick ms/frame                            4.22
  frame p90 ms                            10.15
  frames >2x budget                        5.00
  interval p50 ms                          5.71
  interval p90 ms                         10.21
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.03
  batch_commit ms/frame                    2.05
  ovl_region ms/frame                      0.08
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            6.70
  cloak ms (outside frames)               24.70
  session_begin ms (outside frames)        2.60
  input->anim start ms                     0.21
  input->first frame ms                   81.09
| main #4 | resize | 70.57 | 82.60 | 59.00 | 4.22 | 10.15 | 5.00 | 5.71 | 10.21 | 0.00 | 0.03 | 2.05 | 0.08 | 3.00 | 6.70 | 24.70 | 2.60 | 0.21 | 81.09 |
Moved 3 window(s) to workspace '2' to switch to.
label=main #4 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          37.93
  command p90 ms                          44.59
  frames                                  97.00
  tick ms/frame                            2.34
  frame p90 ms                             3.65
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.73
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)           12.70
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| main #4 | switch | 37.93 | 44.59 | 97.00 | 2.34 | 3.65 | 2.00 | 5.71 | 5.77 | 0.00 | 0.00 | 0.73 | 0.02 | 3.00 | 12.70 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles nopeeks #4 ws 1: WindowsTerminal, Notepad, chrome, Everything, explorer, ApplicationFrameHost, ApplicationFrameHost
label=nopeeks #4 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          18.34
  command p90 ms                          20.94
  frames                                  63.00
  tick ms/frame                            0.56
  frame p90 ms                             0.96
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.26
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            6.70
  cloak ms (outside frames)               10.80
  session_begin ms (outside frames)        0.90
  input->anim start ms                     0.26
  input->first frame ms                   18.81
| nopeeks #4 | move | 18.34 | 20.94 | 63.00 | 0.56 | 0.96 | 0.00 | 5.71 | 5.76 | 0.00 | 0.02 | 0.26 | 0.00 | 0.00 | 6.70 | 10.80 | 0.90 | 0.26 | 18.81 |
label=nopeeks #4 scenario=float target=chrome bursts=10 load=0 reports=10 wall=13.0s
  command p50 ms                         102.04
  command p90 ms                         118.09
  frames                                  59.00
  tick ms/frame                            3.61
  frame p90 ms                             9.50
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          9.60
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.06
  batch_commit ms/frame                    2.04
  ovl_region ms/frame                      0.14
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            6.90
  cloak ms (outside frames)               33.90
  session_begin ms (outside frames)        2.50
  input->anim start ms                     0.31
  input->first frame ms                  108.96
| nopeeks #4 | float | 102.04 | 118.09 | 59.00 | 3.61 | 9.50 | 4.00 | 5.72 | 9.60 | 0.00 | 0.06 | 2.04 | 0.14 | 0.00 | 6.90 | 33.90 | 2.50 | 0.31 | 108.96 |
label=nopeeks #4 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          59.02
  command p90 ms                          71.00
  frames                                 136.00
  tick ms/frame                            1.60
  frame p90 ms                             6.00
  frames >2x budget                        4.00
  interval p50 ms                          5.71
  interval p90 ms                          6.07
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.80
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)           13.50
  cloak ms (outside frames)               39.00
  session_begin ms (outside frames)        7.20
  input->anim start ms                     4.06
  input->first frame ms                  130.30
| nopeeks #4 | minimize | 59.02 | 71.00 | 136.00 | 1.60 | 6.00 | 4.00 | 5.71 | 6.07 | 0.00 | 0.02 | 0.80 | 0.04 | 1.00 | 13.50 | 39.00 | 7.20 | 4.06 | 130.30 |
label=nopeeks #4 scenario=resize target=chrome bursts=10 load=0 reports=9 wall=12.9s
  command p50 ms                          87.33
  command p90 ms                          98.32
  frames                                  57.00
  tick ms/frame                            3.62
  frame p90 ms                             6.95
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          7.16
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.05
  batch_commit ms/frame                    2.23
  ovl_region ms/frame                      0.08
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            4.70
  cloak ms (outside frames)               29.50
  session_begin ms (outside frames)        2.70
  input->anim start ms                     0.30
  input->first frame ms                   94.27
| nopeeks #4 | resize | 87.33 | 98.32 | 57.00 | 3.62 | 6.95 | 4.00 | 5.72 | 7.16 | 0.00 | 0.05 | 2.23 | 0.08 | 0.00 | 4.70 | 29.50 | 2.70 | 0.30 | 94.27 |
Moved 3 window(s) to workspace '2' to switch to.
label=nopeeks #4 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          63.83
  command p90 ms                          78.13
  frames                                  99.00
  tick ms/frame                            1.70
  frame p90 ms                             1.73
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            8.80
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| nopeeks #4 | switch | 63.83 | 78.13 | 99.00 | 1.70 | 1.73 | 3.00 | 5.72 | 5.76 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 8.80 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles default #4 ws 1: WindowsTerminal, Notepad, chrome, Everything, explorer, ApplicationFrameHost, ApplicationFrameHost
label=default #4 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          24.69
  command p90 ms                          25.26
  frames                                  62.00
  tick ms/frame                            0.79
  frame p90 ms                             0.75
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.44
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            8.70
  cloak ms (outside frames)               11.90
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #4 | move | 24.69 | 25.26 | 62.00 | 0.79 | 0.75 | 1.00 | 5.71 | 5.77 | 0.44 | 0.00 | 0.00 | 0.00 | 0.00 | 8.70 | 11.90 | 0.00 | 0.00 | 0.00 |
label=default #4 scenario=float target=chrome bursts=10 load=0 reports=10 wall=13.0s
  command p50 ms                          98.26
  command p90 ms                         121.43
  frames                                  59.00
  tick ms/frame                            3.99
  frame p90 ms                             8.73
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          8.99
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.06
  batch_commit ms/frame                    2.20
  ovl_region ms/frame                      0.09
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            8.30
  cloak ms (outside frames)               46.90
  session_begin ms (outside frames)        2.50
  input->anim start ms                     0.31
  input->first frame ms                  108.52
| default #4 | float | 98.26 | 121.43 | 59.00 | 3.99 | 8.73 | 4.00 | 5.72 | 8.99 | 0.00 | 0.06 | 2.20 | 0.09 | 0.00 | 8.30 | 46.90 | 2.50 | 0.31 | 108.52 |
label=default #4 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          33.94
  command p90 ms                          66.35
  frames                                 135.00
  tick ms/frame                            1.75
  frame p90 ms                             3.59
  frames >2x budget                        5.00
  interval p50 ms                          5.71
  interval p90 ms                          5.78
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.58
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            7.90
  cloak ms (outside frames)               34.40
  session_begin ms (outside frames)        2.40
  input->anim start ms                     4.75
  input->first frame ms                  102.67
| default #4 | minimize | 33.94 | 66.35 | 135.00 | 1.75 | 3.59 | 5.00 | 5.71 | 5.78 | 0.00 | 0.02 | 0.58 | 0.04 | 1.00 | 7.90 | 34.40 | 2.40 | 4.75 | 102.67 |
label=default #4 scenario=resize target=chrome bursts=10 load=0 reports=9 wall=15.1s
  command p50 ms                         137.53
  command p90 ms                         150.70
  frames                                  58.00
  tick ms/frame                            5.56
  frame p90 ms                             4.13
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          6.96
  peek_frame ms/frame                      3.40
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.07
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           14.40
  cloak ms (outside frames)               43.50
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #4 | resize | 137.53 | 150.70 | 58.00 | 5.56 | 4.13 | 5.00 | 5.72 | 6.96 | 3.40 | 0.00 | 0.00 | 0.07 | 2.00 | 14.40 | 43.50 | 0.00 | 0.00 | 0.00 |
Moved 3 window(s) to workspace '2' to switch to.
label=default #4 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          54.39
  command p90 ms                          69.80
  frames                                  99.00
  tick ms/frame                            1.89
  frame p90 ms                             1.71
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            9.00
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #4 | switch | 54.39 | 69.80 | 99.00 | 1.89 | 1.71 | 3.00 | 5.71 | 5.77 | 0.00 | 0.00 | 0.00 | 0.02 | 0.00 | 9.00 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles morepeeks #4 ws 1: WindowsTerminal, Notepad, chrome, Everything, explorer, ApplicationFrameHost, ApplicationFrameHost
label=morepeeks #4 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          23.83
  command p90 ms                          28.24
  frames                                  63.00
  tick ms/frame                            0.86
  frame p90 ms                             0.61
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.42
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            8.10
  cloak ms (outside frames)               11.80
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #4 | move | 23.83 | 28.24 | 63.00 | 0.86 | 0.61 | 2.00 | 5.71 | 5.77 | 0.42 | 0.00 | 0.00 | 0.00 | 0.00 | 8.10 | 11.80 | 0.00 | 0.00 | 0.00 |
label=morepeeks #4 scenario=float target=chrome bursts=10 load=0 reports=9 wall=14.2s
  command p50 ms                         140.15
  command p90 ms                         146.00
  frames                                  59.00
  tick ms/frame                            5.09
  frame p90 ms                             3.70
  frames >2x budget                        5.00
  interval p50 ms                          5.71
  interval p90 ms                          7.30
  peek_frame ms/frame                      3.20
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.09
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            8.80
  cloak ms (outside frames)               39.90
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #4 | float | 140.15 | 146.00 | 59.00 | 5.09 | 3.70 | 5.00 | 5.71 | 7.30 | 3.20 | 0.00 | 0.00 | 0.09 | 2.00 | 8.80 | 39.90 | 0.00 | 0.00 | 0.00 |
label=morepeeks #4 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          32.72
  command p90 ms                          68.80
  frames                                 137.00
  tick ms/frame                            1.76
  frame p90 ms                             3.67
  frames >2x budget                        6.00
  interval p50 ms                          5.71
  interval p90 ms                          5.78
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.52
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)           17.80
  cloak ms (outside frames)               38.10
  session_begin ms (outside frames)        2.70
  input->anim start ms                     4.80
  input->first frame ms                  113.24
| morepeeks #4 | minimize | 32.72 | 68.80 | 137.00 | 1.76 | 3.67 | 6.00 | 5.71 | 5.78 | 0.00 | 0.02 | 0.52 | 0.03 | 1.00 | 17.80 | 38.10 | 2.70 | 4.80 | 113.24 |
label=morepeeks #4 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=13.3s
  command p50 ms                         133.10
  command p90 ms                         147.05
  frames                                  57.00
  tick ms/frame                            5.26
  frame p90 ms                             4.28
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          6.75
  peek_frame ms/frame                      3.32
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.08
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            6.50
  cloak ms (outside frames)               40.30
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #4 | resize | 133.10 | 147.05 | 57.00 | 5.26 | 4.28 | 5.00 | 5.72 | 6.75 | 3.32 | 0.00 | 0.00 | 0.08 | 2.00 | 6.50 | 40.30 | 0.00 | 0.00 | 0.00 |
Moved 3 window(s) to workspace '2' to switch to.
label=morepeeks #4 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          54.37
  command p90 ms                          67.15
  frames                                  99.00
  tick ms/frame                            1.93
  frame p90 ms                             1.78
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            7.90
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #4 | switch | 54.37 | 67.15 | 99.00 | 1.93 | 1.78 | 3.00 | 5.72 | 5.77 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 7.90 | 0.00 | 0.00 | 0.00 | 0.00 |

## move

| label | scenario | command p50 ms | command p90 ms | frames | tick ms/frame | frame p90 ms | frames >2x budget | interval p50 ms | interval p90 ms | peek_frame ms/frame | session_overlays ms/frame | batch_commit ms/frame | ovl_region ms/frame | dwm_flush calls (in frames) | dwm_flush ms (outside frames) | cloak ms (outside frames) | session_begin ms (outside frames) | input->anim start ms | input->first frame ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| main #1 | move | 14.64 | 16.51 | 63.00 | 0.59 | 0.94 | 0.00 | 5.71 | 5.75 | 0.00 | 0.02 | 0.28 | 0.01 | 1.00 | 6.50 | 9.80 | 0.90 | 0.21 | 15.20 |
| main #2 | move | 15.39 | 17.27 | 63.00 | 0.66 | 1.06 | 0.00 | 5.71 | 5.74 | 0.00 | 0.02 | 0.31 | 0.00 | 1.00 | 5.90 | 9.30 | 0.90 | 0.21 | 16.38 |
| main #3 | move | 16.17 | 16.49 | 63.00 | 0.66 | 0.95 | 0.00 | 5.71 | 5.75 | 0.00 | 0.02 | 0.28 | 0.00 | 1.00 | 6.70 | 9.90 | 0.90 | 0.22 | 16.92 |
| main #4 | move | 15.97 | 16.94 | 63.00 | 0.69 | 0.96 | 0.00 | 5.71 | 5.75 | 0.00 | 0.02 | 0.29 | 0.00 | 1.00 | 6.80 | 9.70 | 0.90 | 0.21 | 16.59 |
| nopeeks #1 | move | 17.99 | 18.88 | 64.00 | 0.54 | 0.89 | 0.00 | 5.71 | 5.76 | 0.00 | 0.02 | 0.25 | 0.00 | 0.00 | 5.90 | 10.60 | 0.90 | 0.28 | 18.47 |
| nopeeks #2 | move | 18.38 | 19.37 | 62.00 | 0.56 | 0.96 | 0.00 | 5.71 | 5.75 | 0.00 | 0.02 | 0.27 | 0.00 | 0.00 | 6.70 | 11.00 | 0.90 | 0.26 | 19.18 |
| nopeeks #3 | move | 18.80 | 19.17 | 62.00 | 0.55 | 0.93 | 0.00 | 5.71 | 5.75 | 0.00 | 0.02 | 0.26 | 0.00 | 0.00 | 6.60 | 11.20 | 0.90 | 0.27 | 19.40 |
| nopeeks #4 | move | 18.34 | 20.94 | 63.00 | 0.56 | 0.96 | 0.00 | 5.71 | 5.76 | 0.00 | 0.02 | 0.26 | 0.00 | 0.00 | 6.70 | 10.80 | 0.90 | 0.26 | 18.81 |
| default #1 | move | 24.21 | 25.99 | 63.00 | 0.84 | 0.51 | 2.00 | 5.71 | 5.75 | 0.44 | 0.00 | 0.00 | 0.00 | 0.00 | 8.90 | 12.10 | 0.00 | 0.00 | 0.00 |
| default #2 | move | 25.08 | 27.61 | 63.00 | 0.86 | 0.76 | 2.00 | 5.71 | 5.76 | 0.43 | 0.00 | 0.00 | 0.00 | 0.00 | 9.00 | 12.40 | 0.00 | 0.00 | 0.00 |
| default #3 | move | 23.73 | 25.52 | 63.00 | 0.81 | 0.56 | 2.00 | 5.71 | 5.76 | 0.44 | 0.00 | 0.00 | 0.00 | 0.00 | 9.00 | 12.40 | 0.00 | 0.00 | 0.00 |
| default #4 | move | 24.69 | 25.26 | 62.00 | 0.79 | 0.75 | 1.00 | 5.71 | 5.77 | 0.44 | 0.00 | 0.00 | 0.00 | 0.00 | 8.70 | 11.90 | 0.00 | 0.00 | 0.00 |
| morepeeks #1 | move | 24.37 | 26.08 | 63.00 | 0.85 | 0.55 | 2.00 | 5.71 | 5.76 | 0.41 | 0.00 | 0.00 | 0.00 | 0.00 | 8.70 | 12.10 | 0.00 | 0.00 | 0.00 |
| morepeeks #2 | move | 25.15 | 26.49 | 63.00 | 0.83 | 0.56 | 2.00 | 5.71 | 5.76 | 0.47 | 0.00 | 0.00 | 0.00 | 0.00 | 8.70 | 12.20 | 0.00 | 0.00 | 0.00 |
| morepeeks #3 | move | 24.12 | 26.50 | 63.00 | 0.82 | 0.55 | 2.00 | 5.71 | 5.76 | 0.43 | 0.00 | 0.00 | 0.00 | 0.00 | 8.10 | 11.70 | 0.00 | 0.00 | 0.00 |
| morepeeks #4 | move | 23.83 | 28.24 | 63.00 | 0.86 | 0.61 | 2.00 | 5.71 | 5.77 | 0.42 | 0.00 | 0.00 | 0.00 | 0.00 | 8.10 | 11.80 | 0.00 | 0.00 | 0.00 |

## float

| label | scenario | command p50 ms | command p90 ms | frames | tick ms/frame | frame p90 ms | frames >2x budget | interval p50 ms | interval p90 ms | peek_frame ms/frame | session_overlays ms/frame | batch_commit ms/frame | ovl_region ms/frame | dwm_flush calls (in frames) | dwm_flush ms (outside frames) | cloak ms (outside frames) | session_begin ms (outside frames) | input->anim start ms | input->first frame ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| main #1 | float | 77.49 | 84.86 | 61.00 | 3.52 | 7.71 | 4.00 | 5.72 | 8.51 | 0.00 | 0.04 | 1.84 | 0.13 | 3.00 | 6.50 | 22.30 | 2.70 | 0.23 | 100.04 |
| main #2 | float | 84.73 | 89.01 | 60.00 | 3.86 | 9.83 | 6.00 | 5.72 | 10.83 | 0.00 | 0.04 | 2.08 | 0.14 | 3.00 | 7.50 | 31.70 | 2.70 | 0.24 | 106.79 |
| main #3 | float | 71.34 | 88.66 | 62.00 | 3.81 | 6.50 | 5.00 | 5.72 | 8.31 | 0.00 | 0.03 | 1.16 | 0.08 | 3.00 | 8.10 | 30.70 | 2.70 | 0.23 | 76.70 |
| main #4 | float | 66.83 | 95.67 | 61.00 | 3.66 | 6.40 | 6.00 | 5.72 | 6.51 | 0.00 | 0.03 | 1.43 | 0.10 | 3.00 | 10.30 | 32.70 | 2.70 | 0.24 | 74.08 |
| nopeeks #1 | float | 104.39 | 112.40 | 59.00 | 3.60 | 8.50 | 4.00 | 5.72 | 8.77 | 0.00 | 0.06 | 2.04 | 0.14 | 0.00 | 9.10 | 33.50 | 2.60 | 0.29 | 111.76 |
| nopeeks #2 | float | 107.02 | 119.88 | 60.00 | 3.63 | 7.51 | 4.00 | 5.72 | 7.94 | 0.00 | 0.06 | 2.14 | 0.14 | 0.00 | 7.20 | 38.50 | 2.60 | 0.28 | 114.45 |
| nopeeks #3 | float | 107.21 | 116.98 | 61.00 | 3.90 | 4.25 | 4.00 | 5.72 | 6.12 | 0.00 | 0.05 | 1.29 | 0.13 | 0.00 | 11.40 | 32.60 | 2.60 | 0.28 | 84.27 |
| nopeeks #4 | float | 102.04 | 118.09 | 59.00 | 3.61 | 9.50 | 4.00 | 5.72 | 9.60 | 0.00 | 0.06 | 2.04 | 0.14 | 0.00 | 6.90 | 33.90 | 2.50 | 0.31 | 108.96 |
| default #1 | float | 104.92 | 115.05 | 60.00 | 3.97 | 7.40 | 4.00 | 5.71 | 9.39 | 0.00 | 0.05 | 2.00 | 0.13 | 0.00 | 6.40 | 34.50 | 2.60 | 0.31 | 111.44 |
| default #2 | float | 97.55 | 123.35 | 60.00 | 4.21 | 9.25 | 4.00 | 5.72 | 9.54 | 0.00 | 0.04 | 2.01 | 0.09 | 0.00 | 8.50 | 46.90 | 2.70 | 0.30 | 125.50 |
| default #3 | float | 100.82 | 117.96 | 60.00 | 4.07 | 8.32 | 4.00 | 5.72 | 8.47 | 0.00 | 0.05 | 1.99 | 0.15 | 0.00 | 6.10 | 34.60 | 2.70 | 0.33 | 107.00 |
| default #4 | float | 98.26 | 121.43 | 59.00 | 3.99 | 8.73 | 4.00 | 5.72 | 8.99 | 0.00 | 0.06 | 2.20 | 0.09 | 0.00 | 8.30 | 46.90 | 2.50 | 0.31 | 108.52 |
| morepeeks #1 | float | 130.28 | 144.37 | 59.00 | 4.98 | 3.91 | 5.00 | 5.72 | 6.69 | 3.16 | 0.00 | 0.00 | 0.08 | 2.00 | 8.60 | 37.50 | 0.00 | 0.00 | 0.00 |
| morepeeks #2 | float | 135.89 | 149.80 | 59.00 | 4.97 | 4.43 | 5.00 | 5.71 | 6.51 | 3.11 | 0.00 | 0.00 | 0.08 | 2.00 | 15.80 | 41.20 | 0.00 | 0.00 | 0.00 |
| morepeeks #3 | float | 139.85 | 142.71 | 58.00 | 5.11 | 4.06 | 5.00 | 5.72 | 6.88 | 3.08 | 0.00 | 0.00 | 0.09 | 2.00 | 6.60 | 37.30 | 0.00 | 0.00 | 0.00 |
| morepeeks #4 | float | 140.15 | 146.00 | 59.00 | 5.09 | 3.70 | 5.00 | 5.71 | 7.30 | 3.20 | 0.00 | 0.00 | 0.09 | 2.00 | 8.80 | 39.90 | 0.00 | 0.00 | 0.00 |

## minimize

| label | scenario | command p50 ms | command p90 ms | frames | tick ms/frame | frame p90 ms | frames >2x budget | interval p50 ms | interval p90 ms | peek_frame ms/frame | session_overlays ms/frame | batch_commit ms/frame | ovl_region ms/frame | dwm_flush calls (in frames) | dwm_flush ms (outside frames) | cloak ms (outside frames) | session_begin ms (outside frames) | input->anim start ms | input->first frame ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| main #1 | minimize | 47.49 | 51.50 | 133.00 | 1.76 | 3.10 | 7.00 | 5.72 | 5.79 | 0.00 | 0.02 | 0.81 | 0.06 | 4.00 | 7.00 | 31.20 | 3.50 | 3.41 | 100.10 |
| main #2 | minimize | 28.04 | 67.25 | 132.00 | 1.87 | 3.50 | 5.00 | 5.71 | 5.79 | 0.00 | 0.01 | 0.76 | 0.04 | 5.00 | 7.80 | 39.20 | 5.50 | 3.84 | 117.15 |
| main #3 | minimize | 49.52 | 60.52 | 134.00 | 1.77 | 3.51 | 5.00 | 5.72 | 5.77 | 0.00 | 0.01 | 0.75 | 0.05 | 4.00 | 8.40 | 28.40 | 6.20 | 4.30 | 97.46 |
| main #4 | minimize | 48.06 | 53.46 | 134.00 | 1.56 | 4.33 | 5.00 | 5.71 | 5.79 | 0.00 | 0.01 | 0.71 | 0.04 | 4.00 | 7.50 | 32.40 | 4.20 | 4.10 | 102.14 |
| nopeeks #1 | minimize | 36.34 | 63.36 | 139.00 | 1.47 | 5.71 | 5.00 | 5.72 | 6.17 | 0.00 | 0.02 | 0.73 | 0.04 | 1.00 | 5.30 | 33.70 | 4.20 | 4.30 | 109.65 |
| nopeeks #2 | minimize | 59.19 | 65.90 | 138.00 | 1.53 | 5.06 | 4.00 | 5.72 | 5.92 | 0.00 | 0.02 | 0.80 | 0.03 | 1.00 | 7.40 | 43.10 | 6.30 | 4.20 | 124.39 |
| nopeeks #3 | minimize | 55.97 | 69.39 | 138.00 | 1.60 | 5.15 | 5.00 | 5.72 | 5.79 | 0.00 | 0.02 | 0.64 | 0.04 | 1.00 | 11.10 | 41.60 | 7.10 | 4.27 | 121.08 |
| nopeeks #4 | minimize | 59.02 | 71.00 | 136.00 | 1.60 | 6.00 | 4.00 | 5.71 | 6.07 | 0.00 | 0.02 | 0.80 | 0.04 | 1.00 | 13.50 | 39.00 | 7.20 | 4.06 | 130.30 |
| default #1 | minimize | 68.91 | 76.01 | 132.00 | 1.80 | 3.87 | 5.00 | 5.72 | 5.78 | 0.71 | 0.02 | 0.45 | 0.04 | 1.00 | 10.70 | 31.10 | 2.60 | 2.69 | 91.64 |
| default #2 | minimize | 67.03 | 85.03 | 133.00 | 1.79 | 3.34 | 5.00 | 5.72 | 5.77 | 0.89 | 0.01 | 0.50 | 0.03 | 1.00 | 14.10 | 44.20 | 2.60 | 3.21 | 96.79 |
| default #3 | minimize | 69.52 | 81.97 | 133.00 | 1.74 | 3.68 | 5.00 | 5.72 | 5.77 | 0.68 | 0.02 | 0.44 | 0.03 | 1.00 | 16.80 | 36.50 | 2.50 | 3.45 | 96.72 |
| default #4 | minimize | 33.94 | 66.35 | 135.00 | 1.75 | 3.59 | 5.00 | 5.71 | 5.78 | 0.00 | 0.02 | 0.58 | 0.04 | 1.00 | 7.90 | 34.40 | 2.40 | 4.75 | 102.67 |
| morepeeks #1 | minimize | 62.97 | 71.03 | 134.00 | 1.77 | 3.49 | 6.00 | 5.72 | 5.77 | 1.12 | 0.02 | 0.51 | 0.05 | 1.00 | 10.60 | 42.80 | 2.40 | 37.66 | 217.07 |
| morepeeks #2 | minimize | 66.33 | 70.48 | 135.00 | 1.76 | 3.74 | 6.00 | 5.72 | 5.78 | 1.19 | 0.03 | 0.54 | 0.04 | 1.00 | 7.30 | 30.70 | 2.50 | 27.88 | 170.12 |
| morepeeks #3 | minimize | 68.52 | 83.56 | 133.00 | 1.78 | 3.35 | 6.00 | 5.71 | 5.77 | 0.60 | 0.02 | 0.51 | 0.04 | 1.00 | 11.50 | 31.70 | 2.50 | 0.71 | 97.70 |
| morepeeks #4 | minimize | 32.72 | 68.80 | 137.00 | 1.76 | 3.67 | 6.00 | 5.71 | 5.78 | 0.00 | 0.02 | 0.52 | 0.03 | 1.00 | 17.80 | 38.10 | 2.70 | 4.80 | 113.24 |

## resize

| label | scenario | command p50 ms | command p90 ms | frames | tick ms/frame | frame p90 ms | frames >2x budget | interval p50 ms | interval p90 ms | peek_frame ms/frame | session_overlays ms/frame | batch_commit ms/frame | ovl_region ms/frame | dwm_flush calls (in frames) | dwm_flush ms (outside frames) | cloak ms (outside frames) | session_begin ms (outside frames) | input->anim start ms | input->first frame ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| main #1 | resize | 77.09 | 93.81 | 58.00 | 4.48 | 8.86 | 5.00 | 5.72 | 9.01 | 0.00 | 0.04 | 2.09 | 0.08 | 3.00 | 5.80 | 27.70 | 2.90 | 0.22 | 98.42 |
| main #2 | resize | 75.33 | 89.74 | 58.00 | 3.98 | 6.41 | 4.00 | 5.72 | 9.08 | 0.00 | 0.03 | 1.96 | 0.07 | 2.00 | 8.70 | 32.60 | 2.60 | 0.21 | 84.03 |
| main #3 | resize | 78.32 | 80.00 | 59.00 | 4.59 | 8.48 | 5.00 | 5.72 | 9.23 | 0.00 | 0.04 | 2.05 | 0.07 | 3.00 | 5.20 | 28.60 | 2.60 | 0.21 | 93.07 |
| main #4 | resize | 70.57 | 82.60 | 59.00 | 4.22 | 10.15 | 5.00 | 5.71 | 10.21 | 0.00 | 0.03 | 2.05 | 0.08 | 3.00 | 6.70 | 24.70 | 2.60 | 0.21 | 81.09 |
| nopeeks #1 | resize | 87.26 | 103.17 | 57.00 | 3.27 | 8.69 | 4.00 | 5.72 | 9.18 | 0.00 | 0.05 | 2.21 | 0.07 | 0.00 | 6.30 | 33.80 | 2.50 | 0.29 | 98.08 |
| nopeeks #2 | resize | 92.81 | 102.84 | 57.00 | 3.97 | 6.53 | 4.00 | 5.72 | 6.69 | 0.00 | 0.07 | 2.24 | 0.08 | 0.00 | 9.40 | 39.60 | 2.90 | 0.28 | 105.54 |
| nopeeks #3 | resize | 94.09 | 107.41 | 59.00 | 3.92 | 8.38 | 4.00 | 5.72 | 8.89 | 0.00 | 0.06 | 2.05 | 0.07 | 0.00 | 10.00 | 33.90 | 2.50 | 0.28 | 114.85 |
| nopeeks #4 | resize | 87.33 | 98.32 | 57.00 | 3.62 | 6.95 | 4.00 | 5.72 | 7.16 | 0.00 | 0.05 | 2.23 | 0.08 | 0.00 | 4.70 | 29.50 | 2.70 | 0.30 | 94.27 |
| default #1 | resize | 133.14 | 136.06 | 57.00 | 5.46 | 4.00 | 5.00 | 5.71 | 6.77 | 3.46 | 0.00 | 0.00 | 0.10 | 2.00 | 11.30 | 37.80 | 0.00 | 0.00 | 0.00 |
| default #2 | resize | 138.25 | 150.18 | 59.00 | 4.75 | 4.47 | 5.00 | 5.72 | 5.97 | 3.16 | 0.00 | 0.00 | 0.08 | 2.00 | 9.30 | 40.90 | 0.00 | 0.00 | 0.00 |
| default #3 | resize | 133.16 | 136.90 | 59.00 | 5.39 | 4.31 | 5.00 | 5.72 | 6.33 | 3.37 | 0.00 | 0.00 | 0.09 | 2.00 | 9.80 | 41.90 | 0.00 | 0.00 | 0.00 |
| default #4 | resize | 137.53 | 150.70 | 58.00 | 5.56 | 4.13 | 5.00 | 5.72 | 6.96 | 3.40 | 0.00 | 0.00 | 0.07 | 2.00 | 14.40 | 43.50 | 0.00 | 0.00 | 0.00 |
| morepeeks #1 | resize | 132.65 | 158.91 | 56.00 | 4.90 | 4.32 | 5.00 | 5.72 | 6.89 | 3.40 | 0.00 | 0.00 | 0.09 | 2.00 | 16.10 | 43.20 | 0.00 | 0.00 | 0.00 |
| morepeeks #2 | resize | 128.07 | 143.06 | 58.00 | 5.48 | 4.37 | 5.00 | 5.72 | 6.96 | 3.25 | 0.00 | 0.00 | 0.08 | 2.00 | 7.60 | 38.50 | 0.00 | 0.00 | 0.00 |
| morepeeks #3 | resize | 125.26 | 148.01 | 56.00 | 4.94 | 4.27 | 5.00 | 5.72 | 5.85 | 3.25 | 0.00 | 0.00 | 0.08 | 1.00 | 20.60 | 40.70 | 0.00 | 0.00 | 0.00 |
| morepeeks #4 | resize | 133.10 | 147.05 | 57.00 | 5.26 | 4.28 | 5.00 | 5.72 | 6.75 | 3.32 | 0.00 | 0.00 | 0.08 | 2.00 | 6.50 | 40.30 | 0.00 | 0.00 | 0.00 |

## switch

| label | scenario | command p50 ms | command p90 ms | frames | tick ms/frame | frame p90 ms | frames >2x budget | interval p50 ms | interval p90 ms | peek_frame ms/frame | session_overlays ms/frame | batch_commit ms/frame | ovl_region ms/frame | dwm_flush calls (in frames) | dwm_flush ms (outside frames) | cloak ms (outside frames) | session_begin ms (outside frames) | input->anim start ms | input->first frame ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| main #1 | switch | 36.05 | 41.55 | 97.00 | 2.40 | 3.65 | 2.00 | 5.71 | 5.76 | 0.00 | 0.00 | 0.77 | 0.02 | 3.00 | 7.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| main #2 | switch | 31.07 | 53.04 | 97.00 | 2.48 | 3.76 | 3.00 | 5.71 | 5.76 | 0.00 | 0.00 | 0.78 | 0.02 | 3.00 | 7.80 | 0.00 | 0.00 | 0.00 | 0.00 |
| main #3 | switch | 37.31 | 38.28 | 97.00 | 2.40 | 3.50 | 2.00 | 5.71 | 5.77 | 0.00 | 0.00 | 0.74 | 0.02 | 3.00 | 9.30 | 0.00 | 0.00 | 0.00 | 0.00 |
| main #4 | switch | 37.93 | 44.59 | 97.00 | 2.34 | 3.65 | 2.00 | 5.71 | 5.77 | 0.00 | 0.00 | 0.73 | 0.02 | 3.00 | 12.70 | 0.00 | 0.00 | 0.00 | 0.00 |
| nopeeks #1 | switch | 53.65 | 57.20 | 98.00 | 1.75 | 1.63 | 3.00 | 5.72 | 5.77 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 9.40 | 0.00 | 0.00 | 0.00 | 0.00 |
| nopeeks #2 | switch | 64.58 | 78.21 | 99.00 | 1.75 | 1.77 | 3.00 | 5.71 | 5.76 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 8.10 | 0.00 | 0.00 | 0.00 | 0.00 |
| nopeeks #3 | switch | 66.29 | 69.24 | 99.00 | 1.84 | 1.72 | 3.00 | 5.72 | 5.77 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 7.80 | 0.00 | 0.00 | 0.00 | 0.00 |
| nopeeks #4 | switch | 63.83 | 78.13 | 99.00 | 1.70 | 1.73 | 3.00 | 5.72 | 5.76 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 8.80 | 0.00 | 0.00 | 0.00 | 0.00 |
| default #1 | switch | 56.00 | 70.07 | 99.00 | 1.93 | 1.66 | 3.00 | 5.72 | 5.77 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 9.20 | 0.00 | 0.00 | 0.00 | 0.00 |
| default #2 | switch | 52.23 | 64.55 | 99.00 | 1.86 | 1.86 | 3.00 | 5.72 | 5.77 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 7.50 | 0.00 | 0.00 | 0.00 | 0.00 |
| default #3 | switch | 54.29 | 67.46 | 99.00 | 1.97 | 1.89 | 3.00 | 5.72 | 5.77 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 8.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| default #4 | switch | 54.39 | 69.80 | 99.00 | 1.89 | 1.71 | 3.00 | 5.71 | 5.77 | 0.00 | 0.00 | 0.00 | 0.02 | 0.00 | 9.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| morepeeks #1 | switch | 54.36 | 59.16 | 99.00 | 1.97 | 1.84 | 3.00 | 5.72 | 5.77 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 7.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| morepeeks #2 | switch | 60.92 | 72.72 | 99.00 | 1.93 | 1.74 | 3.00 | 5.72 | 5.77 | 0.00 | 0.00 | 0.00 | 0.02 | 0.00 | 9.20 | 0.00 | 0.00 | 0.00 | 0.00 |
| morepeeks #3 | switch | 52.98 | 62.55 | 99.00 | 1.92 | 1.74 | 3.00 | 5.71 | 5.77 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 8.20 | 0.00 | 0.00 | 0.00 | 0.00 |
| morepeeks #4 | switch | 54.37 | 67.15 | 99.00 | 1.93 | 1.78 | 3.00 | 5.72 | 5.77 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 7.90 | 0.00 | 0.00 | 0.00 | 0.00 |
```

## Fixed tile order, -CpuLoad 8 (bench-run.ps1)

```text
Bench run, 2026-11-12T22:14:20, modes main,nopeeks,default,morepeeks, target chrome, 4 round(s) of 10 bursts per scenario and mode, load 0, CPU load 8 thread(s) of 16
tiles main #1 ws 1: WindowsTerminal, Notepad, chrome, Everything, explorer, ApplicationFrameHost, ApplicationFrameHost
label=main #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          16.63
  command p90 ms                          18.05
  frames                                  64.00
  tick ms/frame                            0.78
  frame p90 ms                             1.22
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.03
  batch_commit ms/frame                    0.35
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            5.10
  cloak ms (outside frames)                8.70
  session_begin ms (outside frames)        1.10
  input->anim start ms                     0.25
  input->first frame ms                   17.45
| main #1 | move | 16.63 | 18.05 | 64.00 | 0.78 | 1.22 | 0.00 | 5.71 | 5.76 | 0.00 | 0.03 | 0.35 | 0.00 | 1.00 | 5.10 | 8.70 | 1.10 | 0.25 | 17.45 |
label=main #1 scenario=float target=chrome bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          60.28
  command p90 ms                          64.86
  frames                                 136.00
  tick ms/frame                            1.89
  frame p90 ms                             4.17
  frames >2x budget                        6.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.66
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              5.00
  dwm_flush ms (outside frames)            8.20
  cloak ms (outside frames)               26.10
  session_begin ms (outside frames)        3.50
  input->anim start ms                     0.25
  input->first frame ms                   77.47
| main #1 | float | 60.28 | 64.86 | 136.00 | 1.89 | 4.17 | 6.00 | 5.72 | 5.76 | 0.00 | 0.02 | 0.66 | 0.02 | 5.00 | 8.20 | 26.10 | 3.50 | 0.25 | 77.47 |
label=main #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          35.73
  command p90 ms                          67.04
  frames                                 132.00
  tick ms/frame                            2.07
  frame p90 ms                             3.82
  frames >2x budget                        7.00
  interval p50 ms                          5.72
  interval p90 ms                          6.22
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.75
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              4.00
  dwm_flush ms (outside frames)            8.00
  cloak ms (outside frames)               32.90
  session_begin ms (outside frames)        7.60
  input->anim start ms                     4.75
  input->first frame ms                  104.28
| main #1 | minimize | 35.73 | 67.04 | 132.00 | 2.07 | 3.82 | 7.00 | 5.72 | 6.22 | 0.00 | 0.02 | 0.75 | 0.03 | 4.00 | 8.00 | 32.90 | 7.60 | 4.75 | 104.28 |
label=main #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.9s
  command p50 ms                          87.40
  command p90 ms                          90.37
  frames                                  57.00
  tick ms/frame                            5.24
  frame p90 ms                            11.35
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                         11.72
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.04
  batch_commit ms/frame                    2.31
  ovl_region ms/frame                      0.12
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            5.70
  cloak ms (outside frames)               32.10
  session_begin ms (outside frames)        3.00
  input->anim start ms                     0.24
  input->first frame ms                  118.32
| main #1 | resize | 87.40 | 90.37 | 57.00 | 5.24 | 11.35 | 5.00 | 5.72 | 11.72 | 0.00 | 0.04 | 2.31 | 0.12 | 3.00 | 5.70 | 32.10 | 3.00 | 0.24 | 118.32 |
Moved 3 window(s) to workspace '2' to switch to.
label=main #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          38.82
  command p90 ms                          42.44
  frames                                  98.00
  tick ms/frame                            2.79
  frame p90 ms                             4.06
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.87
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            9.80
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| main #1 | switch | 38.82 | 42.44 | 98.00 | 2.79 | 4.06 | 3.00 | 5.71 | 5.77 | 0.00 | 0.00 | 0.87 | 0.02 | 3.00 | 9.80 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles nopeeks #1 ws 1: WindowsTerminal, Notepad, chrome, Everything, explorer, ApplicationFrameHost, ApplicationFrameHost
label=nopeeks #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          19.08
  command p90 ms                          20.20
  frames                                  62.00
  tick ms/frame                            0.62
  frame p90 ms                             1.07
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.29
  ovl_region ms/frame                      0.01
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            5.60
  cloak ms (outside frames)               10.80
  session_begin ms (outside frames)        1.10
  input->anim start ms                     0.27
  input->first frame ms                   19.87
| nopeeks #1 | move | 19.08 | 20.20 | 62.00 | 0.62 | 1.07 | 1.00 | 5.71 | 5.76 | 0.00 | 0.02 | 0.29 | 0.01 | 0.00 | 5.60 | 10.80 | 1.10 | 0.27 | 19.87 |
label=nopeeks #1 scenario=float target=chrome bursts=10 load=0 reports=10 wall=13.2s
  command p50 ms                         120.93
  command p90 ms                         128.59
  frames                                  60.00
  tick ms/frame                            4.34
  frame p90 ms                             9.67
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          9.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.05
  batch_commit ms/frame                    2.44
  ovl_region ms/frame                      0.17
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           14.30
  cloak ms (outside frames)               37.70
  session_begin ms (outside frames)        3.10
  input->anim start ms                     0.33
  input->first frame ms                  136.61
| nopeeks #1 | float | 120.93 | 128.59 | 60.00 | 4.34 | 9.67 | 4.00 | 5.72 | 9.77 | 0.00 | 0.05 | 2.44 | 0.17 | 0.00 | 14.30 | 37.70 | 3.10 | 0.33 | 136.61 |
label=nopeeks #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          70.74
  command p90 ms                          82.93
  frames                                 135.00
  tick ms/frame                            1.74
  frame p90 ms                             5.48
  frames >2x budget                        7.00
  interval p50 ms                          5.72
  interval p90 ms                          6.05
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.88
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)           14.50
  cloak ms (outside frames)               55.20
  session_begin ms (outside frames)        7.60
  input->anim start ms                     4.45
  input->first frame ms                  153.98
| nopeeks #1 | minimize | 70.74 | 82.93 | 135.00 | 1.74 | 5.48 | 7.00 | 5.72 | 6.05 | 0.00 | 0.02 | 0.88 | 0.05 | 1.00 | 14.50 | 55.20 | 7.60 | 4.45 | 153.98 |
label=nopeeks #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=13.2s
  command p50 ms                         115.55
  command p90 ms                         122.92
  frames                                  57.00
  tick ms/frame                            4.40
  frame p90 ms                            10.27
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                         10.43
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.06
  batch_commit ms/frame                    2.35
  ovl_region ms/frame                      0.07
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           13.40
  cloak ms (outside frames)               46.00
  session_begin ms (outside frames)        3.20
  input->anim start ms                     0.34
  input->first frame ms                  132.36
| nopeeks #1 | resize | 115.55 | 122.92 | 57.00 | 4.40 | 10.27 | 5.00 | 5.72 | 10.43 | 0.00 | 0.06 | 2.35 | 0.07 | 0.00 | 13.40 | 46.00 | 3.20 | 0.34 | 132.36 |
Moved 3 window(s) to workspace '2' to switch to.
label=nopeeks #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.8s
  command p50 ms                          57.83
  command p90 ms                          70.47
  frames                                  98.00
  tick ms/frame                            2.19
  frame p90 ms                             2.24
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            8.10
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| nopeeks #1 | switch | 57.83 | 70.47 | 98.00 | 2.19 | 2.24 | 3.00 | 5.72 | 5.77 | 0.00 | 0.00 | 0.00 | 0.04 | 0.00 | 8.10 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles default #1 ws 1: WindowsTerminal, Notepad, chrome, Everything, explorer, ApplicationFrameHost, ApplicationFrameHost
label=default #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          24.97
  command p90 ms                          26.60
  frames                                  64.00
  tick ms/frame                            0.97
  frame p90 ms                             0.63
  frames >2x budget                        2.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  peek_frame ms/frame                      0.45
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            7.40
  cloak ms (outside frames)               10.80
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #1 | move | 24.97 | 26.60 | 64.00 | 0.97 | 0.63 | 2.00 | 5.72 | 5.76 | 0.45 | 0.00 | 0.00 | 0.00 | 0.00 | 7.40 | 10.80 | 0.00 | 0.00 | 0.00 |
label=default #1 scenario=float target=chrome bursts=10 load=0 reports=10 wall=13.2s
  command p50 ms                         121.90
  command p90 ms                         133.18
  frames                                  61.00
  tick ms/frame                            4.83
  frame p90 ms                             9.86
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                         10.46
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.05
  batch_commit ms/frame                    2.42
  ovl_region ms/frame                      0.16
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           15.30
  cloak ms (outside frames)               41.90
  session_begin ms (outside frames)        3.10
  input->anim start ms                     0.31
  input->first frame ms                  138.91
| default #1 | float | 121.90 | 133.18 | 61.00 | 4.83 | 9.86 | 4.00 | 5.72 | 10.46 | 0.00 | 0.05 | 2.42 | 0.16 | 0.00 | 15.30 | 41.90 | 3.10 | 0.31 | 138.91 |
label=default #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          78.85
  command p90 ms                          87.89
  frames                                 127.00
  tick ms/frame                            2.17
  frame p90 ms                             4.08
  frames >2x budget                        7.00
  interval p50 ms                          5.72
  interval p90 ms                          5.78
  peek_frame ms/frame                      1.36
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.62
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)           16.50
  cloak ms (outside frames)               33.90
  session_begin ms (outside frames)        3.00
  input->anim start ms                    31.78
  input->first frame ms                  183.20
| default #1 | minimize | 78.85 | 87.89 | 127.00 | 2.17 | 4.08 | 7.00 | 5.72 | 5.78 | 1.36 | 0.02 | 0.62 | 0.05 | 1.00 | 16.50 | 33.90 | 3.00 | 31.78 | 183.20 |
label=default #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=13.6s
  command p50 ms                         161.49
  command p90 ms                         165.37
  frames                                  55.00
  tick ms/frame                            6.49
  frame p90 ms                             5.32
  frames >2x budget                        5.00
  interval p50 ms                          5.71
  interval p90 ms                          7.45
  peek_frame ms/frame                      3.95
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.09
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           12.50
  cloak ms (outside frames)               47.50
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #1 | resize | 161.49 | 165.37 | 55.00 | 6.49 | 5.32 | 5.00 | 5.71 | 7.45 | 3.95 | 0.00 | 0.00 | 0.09 | 2.00 | 12.50 | 47.50 | 0.00 | 0.00 | 0.00 |
Moved 3 window(s) to workspace '2' to switch to.
label=default #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          51.50
  command p90 ms                          67.45
  frames                                  99.00
  tick ms/frame                            2.10
  frame p90 ms                             2.11
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            7.10
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #1 | switch | 51.50 | 67.45 | 99.00 | 2.10 | 2.11 | 3.00 | 5.71 | 5.76 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 7.10 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles morepeeks #1 ws 1: WindowsTerminal, Notepad, chrome, Everything, explorer, ApplicationFrameHost, ApplicationFrameHost
label=morepeeks #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          25.31
  command p90 ms                          29.27
  frames                                  63.00
  tick ms/frame                            0.96
  frame p90 ms                             0.76
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  peek_frame ms/frame                      0.46
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            7.50
  cloak ms (outside frames)               10.80
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #1 | move | 25.31 | 29.27 | 63.00 | 0.96 | 0.76 | 1.00 | 5.71 | 5.76 | 0.46 | 0.00 | 0.00 | 0.00 | 0.00 | 7.50 | 10.80 | 0.00 | 0.00 | 0.00 |
label=morepeeks #1 scenario=float target=chrome bursts=10 load=0 reports=10 wall=13.5s
  command p50 ms                         165.39
  command p90 ms                         168.93
  frames                                  56.00
  tick ms/frame                            5.73
  frame p90 ms                             5.10
  frames >2x budget                        5.00
  interval p50 ms                          5.71
  interval p90 ms                          7.43
  peek_frame ms/frame                      3.94
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.09
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           19.20
  cloak ms (outside frames)               52.90
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #1 | float | 165.39 | 168.93 | 56.00 | 5.73 | 5.10 | 5.00 | 5.71 | 7.43 | 3.94 | 0.00 | 0.00 | 0.09 | 2.00 | 19.20 | 52.90 | 0.00 | 0.00 | 0.00 |
label=morepeeks #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          73.31
  command p90 ms                          86.48
  frames                                 134.00
  tick ms/frame                            2.00
  frame p90 ms                             4.30
  frames >2x budget                        6.00
  interval p50 ms                          5.72
  interval p90 ms                          5.79
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.67
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)           12.80
  cloak ms (outside frames)               37.10
  session_begin ms (outside frames)        3.20
  input->anim start ms                     5.03
  input->first frame ms                  133.88
| morepeeks #1 | minimize | 73.31 | 86.48 | 134.00 | 2.00 | 4.30 | 6.00 | 5.72 | 5.79 | 0.00 | 0.02 | 0.67 | 0.05 | 1.00 | 12.80 | 37.10 | 3.20 | 5.03 | 133.88 |
label=morepeeks #1 scenario=resize target=chrome bursts=10 load=0 reports=9 wall=15.8s
  command p50 ms                         163.32
  command p90 ms                         177.40
  frames                                  57.00
  tick ms/frame                            5.88
  frame p90 ms                             5.35
  frames >2x budget                        5.00
  interval p50 ms                          5.71
  interval p90 ms                          6.54
  peek_frame ms/frame                      3.75
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.08
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           14.60
  cloak ms (outside frames)               51.80
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #1 | resize | 163.32 | 177.40 | 57.00 | 5.88 | 5.35 | 5.00 | 5.71 | 6.54 | 3.75 | 0.00 | 0.00 | 0.08 | 2.00 | 14.60 | 51.80 | 0.00 | 0.00 | 0.00 |
Moved 3 window(s) to workspace '2' to switch to.
label=morepeeks #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          55.75
  command p90 ms                          73.15
  frames                                  99.00
  tick ms/frame                            2.20
  frame p90 ms                             2.19
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            8.50
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #1 | switch | 55.75 | 73.15 | 99.00 | 2.20 | 2.19 | 3.00 | 5.71 | 5.77 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 8.50 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles main #2 ws 1: WindowsTerminal, Notepad, chrome, Everything, explorer, ApplicationFrameHost, ApplicationFrameHost
label=main #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          17.57
  command p90 ms                          18.97
  frames                                  64.00
  tick ms/frame                            0.82
  frame p90 ms                             1.15
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.33
  ovl_region ms/frame                      0.01
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            7.00
  cloak ms (outside frames)               11.00
  session_begin ms (outside frames)        1.10
  input->anim start ms                     0.23
  input->first frame ms                   18.32
| main #2 | move | 17.57 | 18.97 | 64.00 | 0.82 | 1.15 | 1.00 | 5.71 | 5.77 | 0.00 | 0.02 | 0.33 | 0.01 | 1.00 | 7.00 | 11.00 | 1.10 | 0.23 | 18.32 |
label=main #2 scenario=float target=chrome bursts=10 load=0 reports=10 wall=12.9s
  command p50 ms                          90.39
  command p90 ms                          98.59
  frames                                  61.00
  tick ms/frame                            4.79
  frame p90 ms                             8.03
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          9.70
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.03
  batch_commit ms/frame                    1.46
  ovl_region ms/frame                      0.11
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            6.10
  cloak ms (outside frames)               35.20
  session_begin ms (outside frames)        3.30
  input->anim start ms                     0.25
  input->first frame ms                  104.86
| main #2 | float | 90.39 | 98.59 | 61.00 | 4.79 | 8.03 | 5.00 | 5.72 | 9.70 | 0.00 | 0.03 | 1.46 | 0.11 | 3.00 | 6.10 | 35.20 | 3.30 | 0.25 | 104.86 |
label=main #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          56.41
  command p90 ms                          65.84
  frames                                 132.00
  tick ms/frame                            2.14
  frame p90 ms                             3.95
  frames >2x budget                        7.00
  interval p50 ms                          5.72
  interval p90 ms                          5.81
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.78
  ovl_region ms/frame                      0.09
  dwm_flush calls (in frames)              4.00
  dwm_flush ms (outside frames)            6.40
  cloak ms (outside frames)               36.50
  session_begin ms (outside frames)        5.80
  input->anim start ms                     4.85
  input->first frame ms                  115.26
| main #2 | minimize | 56.41 | 65.84 | 132.00 | 2.14 | 3.95 | 7.00 | 5.72 | 5.81 | 0.00 | 0.02 | 0.78 | 0.09 | 4.00 | 6.40 | 36.50 | 5.80 | 4.85 | 115.26 |
label=main #2 scenario=resize target=chrome bursts=10 load=0 reports=11 wall=12.9s
  command p50 ms                          81.70
  command p90 ms                          96.98
  frames                                  57.00
  tick ms/frame                            5.07
  frame p90 ms                            10.68
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                         10.90
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.04
  batch_commit ms/frame                    2.21
  ovl_region ms/frame                      0.11
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            5.10
  cloak ms (outside frames)               32.60
  session_begin ms (outside frames)        3.10
  input->anim start ms                     0.28
  input->first frame ms                   94.04
| main #2 | resize | 81.70 | 96.98 | 57.00 | 5.07 | 10.68 | 5.00 | 5.72 | 10.90 | 0.00 | 0.04 | 2.21 | 0.11 | 2.00 | 5.10 | 32.60 | 3.10 | 0.28 | 94.04 |
Moved 3 window(s) to workspace '2' to switch to.
label=main #2 scenario=switch target=1<->2 bursts=10 load=0 reports=11 wall=12.5s
  command p50 ms                          39.61
  command p90 ms                          43.22
  frames                                  97.00
  tick ms/frame                            2.65
  frame p90 ms                             4.08
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.79
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.86
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            8.90
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| main #2 | switch | 39.61 | 43.22 | 97.00 | 2.65 | 4.08 | 2.00 | 5.71 | 5.79 | 0.00 | 0.00 | 0.86 | 0.04 | 3.00 | 8.90 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles nopeeks #2 ws 1: WindowsTerminal, Notepad, chrome, Everything, explorer, ApplicationFrameHost, ApplicationFrameHost
label=nopeeks #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          19.70
  command p90 ms                          21.15
  frames                                  62.00
  tick ms/frame                            0.62
  frame p90 ms                             1.01
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.29
  ovl_region ms/frame                      0.01
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            5.70
  cloak ms (outside frames)               10.80
  session_begin ms (outside frames)        1.10
  input->anim start ms                     0.31
  input->first frame ms                   20.51
| nopeeks #2 | move | 19.70 | 21.15 | 62.00 | 0.62 | 1.01 | 1.00 | 5.71 | 5.76 | 0.00 | 0.02 | 0.29 | 0.01 | 0.00 | 5.70 | 10.80 | 1.10 | 0.31 | 20.51 |
label=nopeeks #2 scenario=float target=chrome bursts=10 load=0 reports=10 wall=13.1s
  command p50 ms                         111.28
  command p90 ms                         131.53
  frames                                  60.00
  tick ms/frame                            4.04
  frame p90 ms                             9.55
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          9.57
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.05
  batch_commit ms/frame                    1.92
  ovl_region ms/frame                      0.17
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            6.70
  cloak ms (outside frames)               35.90
  session_begin ms (outside frames)        3.10
  input->anim start ms                     0.32
  input->first frame ms                  119.08
| nopeeks #2 | float | 111.28 | 131.53 | 60.00 | 4.04 | 9.55 | 4.00 | 5.72 | 9.57 | 0.00 | 0.05 | 1.92 | 0.17 | 0.00 | 6.70 | 35.90 | 3.10 | 0.32 | 119.08 |
label=nopeeks #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          70.17
  command p90 ms                          74.65
  frames                                 133.00
  tick ms/frame                            1.85
  frame p90 ms                             4.92
  frames >2x budget                        6.00
  interval p50 ms                          5.72
  interval p90 ms                          6.44
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.89
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            7.50
  cloak ms (outside frames)               39.60
  session_begin ms (outside frames)        4.90
  input->anim start ms                     4.45
  input->first frame ms                  125.90
| nopeeks #2 | minimize | 70.17 | 74.65 | 133.00 | 1.85 | 4.92 | 6.00 | 5.72 | 6.44 | 0.00 | 0.02 | 0.89 | 0.05 | 1.00 | 7.50 | 39.60 | 4.90 | 4.45 | 125.90 |
label=nopeeks #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=13.2s
  command p50 ms                         113.08
  command p90 ms                         124.58
  frames                                  57.00
  tick ms/frame                            4.39
  frame p90 ms                             8.64
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          9.70
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.06
  batch_commit ms/frame                    2.45
  ovl_region ms/frame                      0.12
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           11.00
  cloak ms (outside frames)               42.90
  session_begin ms (outside frames)        3.10
  input->anim start ms                     0.30
  input->first frame ms                  124.65
| nopeeks #2 | resize | 113.08 | 124.58 | 57.00 | 4.39 | 8.64 | 5.00 | 5.72 | 9.70 | 0.00 | 0.06 | 2.45 | 0.12 | 0.00 | 11.00 | 42.90 | 3.10 | 0.30 | 124.65 |
Moved 3 window(s) to workspace '2' to switch to.
label=nopeeks #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          53.32
  command p90 ms                          73.32
  frames                                  99.00
  tick ms/frame                            1.97
  frame p90 ms                             1.98
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            9.00
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| nopeeks #2 | switch | 53.32 | 73.32 | 99.00 | 1.97 | 1.98 | 3.00 | 5.71 | 5.77 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 9.00 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles default #2 ws 1: WindowsTerminal, Notepad, chrome, Everything, explorer, ApplicationFrameHost, ApplicationFrameHost
label=default #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          25.96
  command p90 ms                          28.08
  frames                                  64.00
  tick ms/frame                            0.98
  frame p90 ms                             0.80
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  peek_frame ms/frame                      0.41
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            7.50
  cloak ms (outside frames)               10.90
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #2 | move | 25.96 | 28.08 | 64.00 | 0.98 | 0.80 | 1.00 | 5.71 | 5.76 | 0.41 | 0.00 | 0.00 | 0.00 | 0.00 | 7.50 | 10.90 | 0.00 | 0.00 | 0.00 |
label=default #2 scenario=float target=chrome bursts=10 load=0 reports=10 wall=13.1s
  command p50 ms                         107.83
  command p90 ms                         119.86
  frames                                  61.00
  tick ms/frame                            4.33
  frame p90 ms                             8.48
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          9.17
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.05
  batch_commit ms/frame                    1.99
  ovl_region ms/frame                      0.17
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            8.30
  cloak ms (outside frames)               36.30
  session_begin ms (outside frames)        2.90
  input->anim start ms                     0.38
  input->first frame ms                  114.32
| default #2 | float | 107.83 | 119.86 | 61.00 | 4.33 | 8.48 | 5.00 | 5.72 | 9.17 | 0.00 | 0.05 | 1.99 | 0.17 | 0.00 | 8.30 | 36.30 | 2.90 | 0.38 | 114.32 |
label=default #2 scenario=minimize target=chrome bursts=10 load=0 reports=9 wall=12.7s
  command p50 ms                          73.34
  command p90 ms                          91.72
  frames                                 132.00
  tick ms/frame                            1.92
  frame p90 ms                             3.92
  frames >2x budget                        7.00
  interval p50 ms                          5.72
  interval p90 ms                          5.79
  peek_frame ms/frame                      0.83
  session_overlays ms/frame                0.01
  batch_commit ms/frame                    0.45
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)           13.60
  cloak ms (outside frames)               40.60
  session_begin ms (outside frames)        2.90
  input->anim start ms                     0.57
  input->first frame ms                  105.64
| default #2 | minimize | 73.34 | 91.72 | 132.00 | 1.92 | 3.92 | 7.00 | 5.72 | 5.79 | 0.83 | 0.01 | 0.45 | 0.04 | 1.00 | 13.60 | 40.60 | 2.90 | 0.57 | 105.64 |
label=default #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=13.5s
  command p50 ms                         145.15
  command p90 ms                         157.86
  frames                                  57.00
  tick ms/frame                            6.13
  frame p90 ms                             5.15
  frames >2x budget                        5.00
  interval p50 ms                          5.71
  interval p90 ms                          6.78
  peek_frame ms/frame                      3.79
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.10
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           17.10
  cloak ms (outside frames)               37.80
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #2 | resize | 145.15 | 157.86 | 57.00 | 6.13 | 5.15 | 5.00 | 5.71 | 6.78 | 3.79 | 0.00 | 0.00 | 0.10 | 2.00 | 17.10 | 37.80 | 0.00 | 0.00 | 0.00 |
Moved 3 window(s) to workspace '2' to switch to.
label=default #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.9s
  command p50 ms                          90.77
  command p90 ms                          93.73
  frames                                  99.00
  tick ms/frame                            2.11
  frame p90 ms                             2.27
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.78
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           18.50
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #2 | switch | 90.77 | 93.73 | 99.00 | 2.11 | 2.27 | 3.00 | 5.72 | 5.78 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 18.50 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles morepeeks #2 ws 1: WindowsTerminal, Notepad, chrome, Everything, explorer, ApplicationFrameHost, ApplicationFrameHost
label=morepeeks #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          26.17
  command p90 ms                          28.94
  frames                                  64.00
  tick ms/frame                            0.91
  frame p90 ms                             0.64
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  peek_frame ms/frame                      0.44
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            8.80
  cloak ms (outside frames)               12.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #2 | move | 26.17 | 28.94 | 64.00 | 0.91 | 0.64 | 1.00 | 5.71 | 5.76 | 0.44 | 0.00 | 0.00 | 0.00 | 0.00 | 8.80 | 12.00 | 0.00 | 0.00 | 0.00 |
label=morepeeks #2 scenario=float target=chrome bursts=10 load=0 reports=9 wall=15.1s
  command p50 ms                         161.18
  command p90 ms                         182.12
  frames                                  56.00
  tick ms/frame                            6.07
  frame p90 ms                             4.84
  frames >2x budget                        5.00
  interval p50 ms                          5.71
  interval p90 ms                          6.87
  peek_frame ms/frame                      3.52
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           19.80
  cloak ms (outside frames)               54.50
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #2 | float | 161.18 | 182.12 | 56.00 | 6.07 | 4.84 | 5.00 | 5.71 | 6.87 | 3.52 | 0.00 | 0.00 | 0.05 | 2.00 | 19.80 | 54.50 | 0.00 | 0.00 | 0.00 |
label=morepeeks #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          70.88
  command p90 ms                          91.19
  frames                                 132.00
  tick ms/frame                            2.09
  frame p90 ms                             4.38
  frames >2x budget                        7.00
  interval p50 ms                          5.72
  interval p90 ms                          5.79
  peek_frame ms/frame                      1.35
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.56
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)           15.90
  cloak ms (outside frames)               33.70
  session_begin ms (outside frames)        3.00
  input->anim start ms                    30.59
  input->first frame ms                  187.16
| morepeeks #2 | minimize | 70.88 | 91.19 | 132.00 | 2.09 | 4.38 | 7.00 | 5.72 | 5.79 | 1.35 | 0.02 | 0.56 | 0.04 | 1.00 | 15.90 | 33.70 | 3.00 | 30.59 | 187.16 |
label=morepeeks #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=13.6s
  command p50 ms                         161.31
  command p90 ms                         173.75
  frames                                  55.00
  tick ms/frame                            6.24
  frame p90 ms                             5.25
  frames >2x budget                        5.00
  interval p50 ms                          5.71
  interval p90 ms                          6.78
  peek_frame ms/frame                      3.98
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.11
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           18.40
  cloak ms (outside frames)               50.60
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #2 | resize | 161.31 | 173.75 | 55.00 | 6.24 | 5.25 | 5.00 | 5.71 | 6.78 | 3.98 | 0.00 | 0.00 | 0.11 | 2.00 | 18.40 | 50.60 | 0.00 | 0.00 | 0.00 |
Moved 3 window(s) to workspace '2' to switch to.
label=morepeeks #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          57.31
  command p90 ms                          73.28
  frames                                  99.00
  tick ms/frame                            2.21
  frame p90 ms                             2.27
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.78
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            7.70
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #2 | switch | 57.31 | 73.28 | 99.00 | 2.21 | 2.27 | 3.00 | 5.71 | 5.78 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 7.70 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles main #3 ws 1: WindowsTerminal, Notepad, chrome, Everything, explorer, ApplicationFrameHost, ApplicationFrameHost
label=main #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          16.19
  command p90 ms                          17.74
  frames                                  63.00
  tick ms/frame                            0.76
  frame p90 ms                             1.12
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.33
  ovl_region ms/frame                      0.01
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            5.50
  cloak ms (outside frames)                8.90
  session_begin ms (outside frames)        1.00
  input->anim start ms                     0.25
  input->first frame ms                   16.94
| main #3 | move | 16.19 | 17.74 | 63.00 | 0.76 | 1.12 | 1.00 | 5.71 | 5.75 | 0.00 | 0.02 | 0.33 | 0.01 | 1.00 | 5.50 | 8.90 | 1.00 | 0.25 | 16.94 |
label=main #3 scenario=float target=chrome bursts=10 load=0 reports=10 wall=12.9s
  command p50 ms                          97.22
  command p90 ms                         107.50
  frames                                  60.00
  tick ms/frame                            4.17
  frame p90 ms                            10.23
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                         10.93
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.04
  batch_commit ms/frame                    1.94
  ovl_region ms/frame                      0.13
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)           11.00
  cloak ms (outside frames)               35.60
  session_begin ms (outside frames)        3.00
  input->anim start ms                     0.28
  input->first frame ms                  113.54
| main #3 | float | 97.22 | 107.50 | 60.00 | 4.17 | 10.23 | 5.00 | 5.72 | 10.93 | 0.00 | 0.04 | 1.94 | 0.13 | 3.00 | 11.00 | 35.60 | 3.00 | 0.28 | 113.54 |
label=main #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          58.93
  command p90 ms                          61.20
  frames                                 133.00
  tick ms/frame                            1.87
  frame p90 ms                             5.22
  frames >2x budget                        6.00
  interval p50 ms                          5.72
  interval p90 ms                          5.87
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.01
  batch_commit ms/frame                    0.80
  ovl_region ms/frame                      0.07
  dwm_flush calls (in frames)              4.00
  dwm_flush ms (outside frames)            8.00
  cloak ms (outside frames)               32.10
  session_begin ms (outside frames)        5.00
  input->anim start ms                     4.17
  input->first frame ms                  111.32
| main #3 | minimize | 58.93 | 61.20 | 133.00 | 1.87 | 5.22 | 6.00 | 5.72 | 5.87 | 0.00 | 0.01 | 0.80 | 0.07 | 4.00 | 8.00 | 32.10 | 5.00 | 4.17 | 111.32 |
label=main #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.9s
  command p50 ms                          87.96
  command p90 ms                          94.87
  frames                                  57.00
  tick ms/frame                            4.75
  frame p90 ms                            10.51
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                         10.83
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.03
  batch_commit ms/frame                    2.35
  ovl_region ms/frame                      0.15
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            7.10
  cloak ms (outside frames)               32.30
  session_begin ms (outside frames)        3.30
  input->anim start ms                     0.24
  input->first frame ms                  116.94
| main #3 | resize | 87.96 | 94.87 | 57.00 | 4.75 | 10.51 | 4.00 | 5.72 | 10.83 | 0.00 | 0.03 | 2.35 | 0.15 | 3.00 | 7.10 | 32.30 | 3.30 | 0.24 | 116.94 |
Moved 3 window(s) to workspace '2' to switch to.
label=main #3 scenario=switch target=1<->2 bursts=10 load=0 reports=9 wall=13.9s
  command p50 ms                          38.27
  command p90 ms                          50.76
  frames                                  96.00
  tick ms/frame                            2.91
  frame p90 ms                             4.52
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.78
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.88
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            8.70
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| main #3 | switch | 38.27 | 50.76 | 96.00 | 2.91 | 4.52 | 3.00 | 5.71 | 5.78 | 0.00 | 0.00 | 0.88 | 0.04 | 3.00 | 8.70 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles nopeeks #3 ws 1: WindowsTerminal, Notepad, chrome, Everything, explorer, ApplicationFrameHost, ApplicationFrameHost
label=nopeeks #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          19.18
  command p90 ms                          20.55
  frames                                  62.00
  tick ms/frame                            0.69
  frame p90 ms                             1.06
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.30
  ovl_region ms/frame                      0.01
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            5.70
  cloak ms (outside frames)               10.60
  session_begin ms (outside frames)        1.00
  input->anim start ms                     0.30
  input->first frame ms                   19.99
| nopeeks #3 | move | 19.18 | 20.55 | 62.00 | 0.69 | 1.06 | 1.00 | 5.71 | 5.75 | 0.00 | 0.02 | 0.30 | 0.01 | 0.00 | 5.70 | 10.60 | 1.00 | 0.30 | 19.99 |
label=nopeeks #3 scenario=float target=chrome bursts=10 load=0 reports=9 wall=13.1s
  command p50 ms                          88.34
  command p90 ms                         125.58
  frames                                  61.00
  tick ms/frame                            4.26
  frame p90 ms                             9.94
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                         10.03
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.05
  batch_commit ms/frame                    1.73
  ovl_region ms/frame                      0.13
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           12.10
  cloak ms (outside frames)               38.80
  session_begin ms (outside frames)        3.10
  input->anim start ms                     0.32
  input->first frame ms                   91.50
| nopeeks #3 | float | 88.34 | 125.58 | 61.00 | 4.26 | 9.94 | 5.00 | 5.72 | 10.03 | 0.00 | 0.05 | 1.73 | 0.13 | 0.00 | 12.10 | 38.80 | 3.10 | 0.32 | 91.50 |
label=nopeeks #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          40.55
  command p90 ms                          78.90
  frames                                 135.00
  tick ms/frame                            1.70
  frame p90 ms                             5.82
  frames >2x budget                        6.00
  interval p50 ms                          5.72
  interval p90 ms                          6.80
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.76
  ovl_region ms/frame                      0.06
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)           12.50
  cloak ms (outside frames)               41.60
  session_begin ms (outside frames)        3.30
  input->anim start ms                     4.14
  input->first frame ms                  115.41
| nopeeks #3 | minimize | 40.55 | 78.90 | 135.00 | 1.70 | 5.82 | 6.00 | 5.72 | 6.80 | 0.00 | 0.02 | 0.76 | 0.06 | 1.00 | 12.50 | 41.60 | 3.30 | 4.14 | 115.41 |
label=nopeeks #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=13.1s
  command p50 ms                          94.37
  command p90 ms                         125.03
  frames                                  58.00
  tick ms/frame                            4.13
  frame p90 ms                            10.01
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                         11.01
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.05
  batch_commit ms/frame                    1.99
  ovl_region ms/frame                      0.14
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           11.10
  cloak ms (outside frames)               41.90
  session_begin ms (outside frames)        3.10
  input->anim start ms                     0.32
  input->first frame ms                  103.68
| nopeeks #3 | resize | 94.37 | 125.03 | 58.00 | 4.13 | 10.01 | 5.00 | 5.72 | 11.01 | 0.00 | 0.05 | 1.99 | 0.14 | 0.00 | 11.10 | 41.90 | 3.10 | 0.32 | 103.68 |
Moved 3 window(s) to workspace '2' to switch to.
label=nopeeks #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.8s
  command p50 ms                          63.47
  command p90 ms                          83.07
  frames                                  99.00
  tick ms/frame                            1.85
  frame p90 ms                             2.09
  frames >2x budget                        4.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            9.50
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| nopeeks #3 | switch | 63.47 | 83.07 | 99.00 | 1.85 | 2.09 | 4.00 | 5.71 | 5.77 | 0.00 | 0.00 | 0.00 | 0.05 | 0.00 | 9.50 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles default #3 ws 1: WindowsTerminal, Notepad, chrome, Everything, explorer, ApplicationFrameHost, ApplicationFrameHost
label=default #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          26.61
  command p90 ms                          27.72
  frames                                  63.00
  tick ms/frame                            0.96
  frame p90 ms                             0.65
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  peek_frame ms/frame                      0.42
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            8.10
  cloak ms (outside frames)               11.60
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #3 | move | 26.61 | 27.72 | 63.00 | 0.96 | 0.65 | 1.00 | 5.71 | 5.75 | 0.42 | 0.00 | 0.00 | 0.00 | 0.00 | 8.10 | 11.60 | 0.00 | 0.00 | 0.00 |
label=default #3 scenario=float target=chrome bursts=10 load=0 reports=9 wall=13.2s
  command p50 ms                         117.04
  command p90 ms                         131.97
  frames                                  61.00
  tick ms/frame                            4.21
  frame p90 ms                             4.95
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          6.68
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.06
  batch_commit ms/frame                    1.51
  ovl_region ms/frame                      0.09
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           10.00
  cloak ms (outside frames)               52.60
  session_begin ms (outside frames)        3.20
  input->anim start ms                     0.33
  input->first frame ms                   92.38
| default #3 | float | 117.04 | 131.97 | 61.00 | 4.21 | 4.95 | 4.00 | 5.72 | 6.68 | 0.00 | 0.06 | 1.51 | 0.09 | 0.00 | 10.00 | 52.60 | 3.20 | 0.33 | 92.38 |
label=default #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          76.13
  command p90 ms                          83.15
  frames                                 131.00
  tick ms/frame                            2.00
  frame p90 ms                             4.37
  frames >2x budget                        6.00
  interval p50 ms                          5.72
  interval p90 ms                          5.78
  peek_frame ms/frame                      1.24
  session_overlays ms/frame                0.01
  batch_commit ms/frame                    0.64
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            9.00
  cloak ms (outside frames)               55.90
  session_begin ms (outside frames)        2.90
  input->anim start ms                    41.47
  input->first frame ms                  224.14
| default #3 | minimize | 76.13 | 83.15 | 131.00 | 2.00 | 4.37 | 6.00 | 5.72 | 5.78 | 1.24 | 0.01 | 0.64 | 0.03 | 1.00 | 9.00 | 55.90 | 2.90 | 41.47 | 224.14 |
label=default #3 scenario=resize target=chrome bursts=10 load=0 reports=9 wall=15.1s
  command p50 ms                         161.91
  command p90 ms                         164.45
  frames                                  57.00
  tick ms/frame                            6.06
  frame p90 ms                             5.12
  frames >2x budget                        5.00
  interval p50 ms                          5.71
  interval p90 ms                          6.47
  peek_frame ms/frame                      3.43
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            9.00
  cloak ms (outside frames)               61.60
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #3 | resize | 161.91 | 164.45 | 57.00 | 6.06 | 5.12 | 5.00 | 5.71 | 6.47 | 3.43 | 0.00 | 0.00 | 0.05 | 2.00 | 9.00 | 61.60 | 0.00 | 0.00 | 0.00 |
Moved 3 window(s) to workspace '2' to switch to.
label=default #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          62.01
  command p90 ms                          83.85
  frames                                  99.00
  tick ms/frame                            1.98
  frame p90 ms                             2.08
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            7.00
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #3 | switch | 62.01 | 83.85 | 99.00 | 1.98 | 2.08 | 3.00 | 5.71 | 5.77 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 7.00 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles morepeeks #3 ws 1: WindowsTerminal, Notepad, chrome, Everything, explorer, ApplicationFrameHost, ApplicationFrameHost
label=morepeeks #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          25.60
  command p90 ms                          26.93
  frames                                  64.00
  tick ms/frame                            0.94
  frame p90 ms                             0.64
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  peek_frame ms/frame                      0.45
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            7.40
  cloak ms (outside frames)               10.60
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #3 | move | 25.60 | 26.93 | 64.00 | 0.94 | 0.64 | 2.00 | 5.71 | 5.76 | 0.45 | 0.00 | 0.00 | 0.00 | 0.00 | 7.40 | 10.60 | 0.00 | 0.00 | 0.00 |
label=morepeeks #3 scenario=float target=chrome bursts=10 load=0 reports=9 wall=15.7s
  command p50 ms                         165.41
  command p90 ms                         187.78
  frames                                  59.00
  tick ms/frame                            5.69
  frame p90 ms                             5.05
  frames >2x budget                        5.00
  interval p50 ms                          5.71
  interval p90 ms                          7.36
  peek_frame ms/frame                      3.63
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.09
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           20.40
  cloak ms (outside frames)               50.20
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #3 | float | 165.41 | 187.78 | 59.00 | 5.69 | 5.05 | 5.00 | 5.71 | 7.36 | 3.63 | 0.00 | 0.00 | 0.09 | 2.00 | 20.40 | 50.20 | 0.00 | 0.00 | 0.00 |
label=morepeeks #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          72.00
  command p90 ms                          88.15
  frames                                 129.00
  tick ms/frame                            2.04
  frame p90 ms                             3.83
  frames >2x budget                        7.00
  interval p50 ms                          5.72
  interval p90 ms                          5.78
  peek_frame ms/frame                      1.25
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.60
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)           16.40
  cloak ms (outside frames)               33.80
  session_begin ms (outside frames)        2.70
  input->anim start ms                    45.41
  input->first frame ms                  235.66
| morepeeks #3 | minimize | 72.00 | 88.15 | 129.00 | 2.04 | 3.83 | 7.00 | 5.72 | 5.78 | 1.25 | 0.02 | 0.60 | 0.05 | 1.00 | 16.40 | 33.80 | 2.70 | 45.41 | 235.66 |
label=morepeeks #3 scenario=resize target=chrome bursts=10 load=0 reports=11 wall=13.6s
  command p50 ms                         138.30
  command p90 ms                         171.11
  frames                                  55.00
  tick ms/frame                            5.36
  frame p90 ms                             4.67
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          7.13
  peek_frame ms/frame                      3.73
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.10
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           13.60
  cloak ms (outside frames)               39.50
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #3 | resize | 138.30 | 171.11 | 55.00 | 5.36 | 4.67 | 5.00 | 5.72 | 7.13 | 3.73 | 0.00 | 0.00 | 0.10 | 2.00 | 13.60 | 39.50 | 0.00 | 0.00 | 0.00 |
Moved 3 window(s) to workspace '2' to switch to.
label=morepeeks #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          54.22
  command p90 ms                          69.49
  frames                                  99.00
  tick ms/frame                            2.09
  frame p90 ms                             2.22
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            8.00
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #3 | switch | 54.22 | 69.49 | 99.00 | 2.09 | 2.22 | 3.00 | 5.71 | 5.77 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 8.00 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles main #4 ws 1: WindowsTerminal, Notepad, chrome, Everything, explorer, ApplicationFrameHost, ApplicationFrameHost
label=main #4 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          16.48
  command p90 ms                          17.56
  frames                                  63.00
  tick ms/frame                            0.76
  frame p90 ms                             1.18
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.35
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            6.50
  cloak ms (outside frames)                9.50
  session_begin ms (outside frames)        1.00
  input->anim start ms                     0.23
  input->first frame ms                   17.59
| main #4 | move | 16.48 | 17.56 | 63.00 | 0.76 | 1.18 | 1.00 | 5.71 | 5.77 | 0.00 | 0.02 | 0.35 | 0.00 | 1.00 | 6.50 | 9.50 | 1.00 | 0.23 | 17.59 |
label=main #4 scenario=float target=chrome bursts=10 load=0 reports=10 wall=13.0s
  command p50 ms                          92.57
  command p90 ms                         116.96
  frames                                  61.00
  tick ms/frame                            4.26
  frame p90 ms                             7.65
  frames >2x budget                        6.00
  interval p50 ms                          5.72
  interval p90 ms                         10.86
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.04
  batch_commit ms/frame                    1.37
  ovl_region ms/frame                      0.10
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            7.70
  cloak ms (outside frames)               31.90
  session_begin ms (outside frames)        3.30
  input->anim start ms                     0.26
  input->first frame ms                  121.14
| main #4 | float | 92.57 | 116.96 | 61.00 | 4.26 | 7.65 | 6.00 | 5.72 | 10.86 | 0.00 | 0.04 | 1.37 | 0.10 | 3.00 | 7.70 | 31.90 | 3.30 | 0.26 | 121.14 |
label=main #4 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          56.15
  command p90 ms                          67.23
  frames                                 126.00
  tick ms/frame                            2.14
  frame p90 ms                             3.26
  frames >2x budget                        7.00
  interval p50 ms                          5.72
  interval p90 ms                          5.82
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.85
  ovl_region ms/frame                      0.07
  dwm_flush calls (in frames)              5.00
  dwm_flush ms (outside frames)            9.10
  cloak ms (outside frames)               39.30
  session_begin ms (outside frames)        6.90
  input->anim start ms                     4.15
  input->first frame ms                  132.98
| main #4 | minimize | 56.15 | 67.23 | 126.00 | 2.14 | 3.26 | 7.00 | 5.72 | 5.82 | 0.00 | 0.02 | 0.85 | 0.07 | 5.00 | 9.10 | 39.30 | 6.90 | 4.15 | 132.98 |
label=main #4 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.8s
  command p50 ms                          73.77
  command p90 ms                          95.72
  frames                                  57.00
  tick ms/frame                            4.34
  frame p90 ms                            10.60
  frames >2x budget                        6.00
  interval p50 ms                          5.72
  interval p90 ms                         10.61
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.04
  batch_commit ms/frame                    2.34
  ovl_region ms/frame                      0.09
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)           11.10
  cloak ms (outside frames)               30.20
  session_begin ms (outside frames)        3.30
  input->anim start ms                     0.24
  input->first frame ms                   83.90
| main #4 | resize | 73.77 | 95.72 | 57.00 | 4.34 | 10.60 | 6.00 | 5.72 | 10.61 | 0.00 | 0.04 | 2.34 | 0.09 | 3.00 | 11.10 | 30.20 | 3.30 | 0.24 | 83.90 |
Moved 3 window(s) to workspace '2' to switch to.
label=main #4 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          38.37
  command p90 ms                          46.03
  frames                                  96.00
  tick ms/frame                            2.76
  frame p90 ms                             3.97
  frames >2x budget                        2.00
  interval p50 ms                          5.72
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.87
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            7.30
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| main #4 | switch | 38.37 | 46.03 | 96.00 | 2.76 | 3.97 | 2.00 | 5.72 | 5.77 | 0.00 | 0.00 | 0.87 | 0.04 | 3.00 | 7.30 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles nopeeks #4 ws 1: WindowsTerminal, Notepad, chrome, Everything, explorer, ApplicationFrameHost, ApplicationFrameHost
label=nopeeks #4 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          19.54
  command p90 ms                          20.77
  frames                                  62.00
  tick ms/frame                            0.64
  frame p90 ms                             1.10
  frames >2x budget                        1.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.30
  ovl_region ms/frame                      0.01
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            5.00
  cloak ms (outside frames)               10.50
  session_begin ms (outside frames)        1.10
  input->anim start ms                     0.35
  input->first frame ms                   20.10
| nopeeks #4 | move | 19.54 | 20.77 | 62.00 | 0.64 | 1.10 | 1.00 | 5.72 | 5.76 | 0.00 | 0.02 | 0.30 | 0.01 | 0.00 | 5.00 | 10.50 | 1.10 | 0.35 | 20.10 |
label=nopeeks #4 scenario=float target=chrome bursts=10 load=0 reports=10 wall=13.0s
  command p50 ms                          88.12
  command p90 ms                         122.30
  frames                                  61.00
  tick ms/frame                            3.88
  frame p90 ms                            10.02
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                         10.04
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.05
  batch_commit ms/frame                    1.69
  ovl_region ms/frame                      0.14
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           12.00
  cloak ms (outside frames)               34.50
  session_begin ms (outside frames)        3.00
  input->anim start ms                     0.32
  input->first frame ms                   93.71
| nopeeks #4 | float | 88.12 | 122.30 | 61.00 | 3.88 | 10.02 | 5.00 | 5.72 | 10.04 | 0.00 | 0.05 | 1.69 | 0.14 | 0.00 | 12.00 | 34.50 | 3.00 | 0.32 | 93.71 |
label=nopeeks #4 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          67.73
  command p90 ms                          72.80
  frames                                 134.00
  tick ms/frame                            1.73
  frame p90 ms                             5.95
  frames >2x budget                        7.00
  interval p50 ms                          5.72
  interval p90 ms                          6.55
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.82
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)           16.90
  cloak ms (outside frames)               43.20
  session_begin ms (outside frames)        4.20
  input->anim start ms                     4.69
  input->first frame ms                  127.33
| nopeeks #4 | minimize | 67.73 | 72.80 | 134.00 | 1.73 | 5.95 | 7.00 | 5.72 | 6.55 | 0.00 | 0.02 | 0.82 | 0.05 | 1.00 | 16.90 | 43.20 | 4.20 | 4.69 | 127.33 |
label=nopeeks #4 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=13.1s
  command p50 ms                         115.39
  command p90 ms                         121.79
  frames                                  56.00
  tick ms/frame                            4.17
  frame p90 ms                            10.75
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                         10.83
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.05
  batch_commit ms/frame                    2.35
  ovl_region ms/frame                      0.11
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           12.10
  cloak ms (outside frames)               35.30
  session_begin ms (outside frames)        3.20
  input->anim start ms                     0.32
  input->first frame ms                  133.35
| nopeeks #4 | resize | 115.39 | 121.79 | 56.00 | 4.17 | 10.75 | 5.00 | 5.72 | 10.83 | 0.00 | 0.05 | 2.35 | 0.11 | 0.00 | 12.10 | 35.30 | 3.20 | 0.32 | 133.35 |
Moved 3 window(s) to workspace '2' to switch to.
label=nopeeks #4 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.8s
  command p50 ms                          66.11
  command p90 ms                          81.55
  frames                                  99.00
  tick ms/frame                            2.05
  frame p90 ms                             2.17
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            9.10
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| nopeeks #4 | switch | 66.11 | 81.55 | 99.00 | 2.05 | 2.17 | 3.00 | 5.71 | 5.77 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 9.10 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles default #4 ws 1: WindowsTerminal, Notepad, chrome, Everything, explorer, ApplicationFrameHost, ApplicationFrameHost
label=default #4 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          26.19
  command p90 ms                          29.29
  frames                                  63.00
  tick ms/frame                            0.93
  frame p90 ms                             0.84
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  peek_frame ms/frame                      0.39
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            8.10
  cloak ms (outside frames)               11.30
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #4 | move | 26.19 | 29.29 | 63.00 | 0.93 | 0.84 | 1.00 | 5.71 | 5.75 | 0.39 | 0.00 | 0.00 | 0.00 | 0.00 | 8.10 | 11.30 | 0.00 | 0.00 | 0.00 |
label=default #4 scenario=float target=chrome bursts=10 load=0 reports=11 wall=13.1s
  command p50 ms                          93.51
  command p90 ms                         132.87
  frames                                  61.00
  tick ms/frame                            4.20
  frame p90 ms                             6.29
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          7.28
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.05
  batch_commit ms/frame                    1.38
  ovl_region ms/frame                      0.15
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           15.10
  cloak ms (outside frames)               36.20
  session_begin ms (outside frames)        3.20
  input->anim start ms                     0.36
  input->first frame ms                   97.46
| default #4 | float | 93.51 | 132.87 | 61.00 | 4.20 | 6.29 | 4.00 | 5.72 | 7.28 | 0.00 | 0.05 | 1.38 | 0.15 | 0.00 | 15.10 | 36.20 | 3.20 | 0.36 | 97.46 |
label=default #4 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          78.70
  command p90 ms                          92.90
  frames                                 130.00
  tick ms/frame                            1.97
  frame p90 ms                             4.16
  frames >2x budget                        7.00
  interval p50 ms                          5.72
  interval p90 ms                          5.79
  peek_frame ms/frame                      1.34
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.63
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)           15.70
  cloak ms (outside frames)               33.80
  session_begin ms (outside frames)        2.90
  input->anim start ms                    41.73
  input->first frame ms                  238.19
| default #4 | minimize | 78.70 | 92.90 | 130.00 | 1.97 | 4.16 | 7.00 | 5.72 | 5.79 | 1.34 | 0.02 | 0.63 | 0.05 | 1.00 | 15.70 | 33.80 | 2.90 | 41.73 | 238.19 |
label=default #4 scenario=resize target=chrome bursts=10 load=0 reports=9 wall=15.4s
  command p50 ms                         157.54
  command p90 ms                         167.01
  frames                                  57.00
  tick ms/frame                            6.10
  frame p90 ms                             5.24
  frames >2x budget                        5.00
  interval p50 ms                          5.71
  interval p90 ms                          6.36
  peek_frame ms/frame                      3.83
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.10
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           19.20
  cloak ms (outside frames)               46.60
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #4 | resize | 157.54 | 167.01 | 57.00 | 6.10 | 5.24 | 5.00 | 5.71 | 6.36 | 3.83 | 0.00 | 0.00 | 0.10 | 2.00 | 19.20 | 46.60 | 0.00 | 0.00 | 0.00 |
Moved 3 window(s) to workspace '2' to switch to.
label=default #4 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.8s
  command p50 ms                          57.75
  command p90 ms                          80.78
  frames                                  99.00
  tick ms/frame                            2.14
  frame p90 ms                             2.09
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            7.50
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #4 | switch | 57.75 | 80.78 | 99.00 | 2.14 | 2.09 | 3.00 | 5.72 | 5.77 | 0.00 | 0.00 | 0.00 | 0.04 | 0.00 | 7.50 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles morepeeks #4 ws 1: WindowsTerminal, Notepad, chrome, Everything, explorer, ApplicationFrameHost, ApplicationFrameHost
label=morepeeks #4 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          25.18
  command p90 ms                          27.18
  frames                                  63.00
  tick ms/frame                            1.01
  frame p90 ms                             0.69
  frames >2x budget                        2.00
  interval p50 ms                          5.72
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.43
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            6.80
  cloak ms (outside frames)               10.30
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #4 | move | 25.18 | 27.18 | 63.00 | 1.01 | 0.69 | 2.00 | 5.72 | 5.77 | 0.43 | 0.00 | 0.00 | 0.00 | 0.00 | 6.80 | 10.30 | 0.00 | 0.00 | 0.00 |
label=morepeeks #4 scenario=float target=chrome bursts=10 load=0 reports=10 wall=13.5s
  command p50 ms                         146.91
  command p90 ms                         170.30
  frames                                  58.00
  tick ms/frame                            5.68
  frame p90 ms                             4.99
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          6.44
  peek_frame ms/frame                      3.59
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.10
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           13.90
  cloak ms (outside frames)               48.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #4 | float | 146.91 | 170.30 | 58.00 | 5.68 | 4.99 | 5.00 | 5.72 | 6.44 | 3.59 | 0.00 | 0.00 | 0.10 | 2.00 | 13.90 | 48.00 | 0.00 | 0.00 | 0.00 |
label=morepeeks #4 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          76.09
  command p90 ms                          84.01
  frames                                 128.00
  tick ms/frame                            2.14
  frame p90 ms                             3.94
  frames >2x budget                        7.00
  interval p50 ms                          5.71
  interval p90 ms                          5.78
  peek_frame ms/frame                      1.21
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.61
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)           10.80
  cloak ms (outside frames)               36.30
  session_begin ms (outside frames)        2.80
  input->anim start ms                    45.03
  input->first frame ms                  244.78
| morepeeks #4 | minimize | 76.09 | 84.01 | 128.00 | 2.14 | 3.94 | 7.00 | 5.71 | 5.78 | 1.21 | 0.02 | 0.61 | 0.05 | 1.00 | 10.80 | 36.30 | 2.80 | 45.03 | 244.78 |
label=morepeeks #4 scenario=resize target=chrome bursts=10 load=0 reports=9 wall=15.1s
  command p50 ms                         160.89
  command p90 ms                         181.31
  frames                                  53.00
  tick ms/frame                            6.37
  frame p90 ms                             5.32
  frames >2x budget                        5.00
  interval p50 ms                          5.71
  interval p90 ms                          7.36
  peek_frame ms/frame                      3.93
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.09
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           18.10
  cloak ms (outside frames)               49.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #4 | resize | 160.89 | 181.31 | 53.00 | 6.37 | 5.32 | 5.00 | 5.71 | 7.36 | 3.93 | 0.00 | 0.00 | 0.09 | 2.00 | 18.10 | 49.00 | 0.00 | 0.00 | 0.00 |
Moved 3 window(s) to workspace '2' to switch to.
label=morepeeks #4 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.8s
  command p50 ms                          57.11
  command p90 ms                          76.10
  frames                                  99.00
  tick ms/frame                            2.06
  frame p90 ms                             2.24
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.78
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            7.80
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #4 | switch | 57.11 | 76.10 | 99.00 | 2.06 | 2.24 | 3.00 | 5.72 | 5.78 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 7.80 | 0.00 | 0.00 | 0.00 | 0.00 |

## move

| label | scenario | command p50 ms | command p90 ms | frames | tick ms/frame | frame p90 ms | frames >2x budget | interval p50 ms | interval p90 ms | peek_frame ms/frame | session_overlays ms/frame | batch_commit ms/frame | ovl_region ms/frame | dwm_flush calls (in frames) | dwm_flush ms (outside frames) | cloak ms (outside frames) | session_begin ms (outside frames) | input->anim start ms | input->first frame ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| main #1 | move | 16.63 | 18.05 | 64.00 | 0.78 | 1.22 | 0.00 | 5.71 | 5.76 | 0.00 | 0.03 | 0.35 | 0.00 | 1.00 | 5.10 | 8.70 | 1.10 | 0.25 | 17.45 |
| main #2 | move | 17.57 | 18.97 | 64.00 | 0.82 | 1.15 | 1.00 | 5.71 | 5.77 | 0.00 | 0.02 | 0.33 | 0.01 | 1.00 | 7.00 | 11.00 | 1.10 | 0.23 | 18.32 |
| main #3 | move | 16.19 | 17.74 | 63.00 | 0.76 | 1.12 | 1.00 | 5.71 | 5.75 | 0.00 | 0.02 | 0.33 | 0.01 | 1.00 | 5.50 | 8.90 | 1.00 | 0.25 | 16.94 |
| main #4 | move | 16.48 | 17.56 | 63.00 | 0.76 | 1.18 | 1.00 | 5.71 | 5.77 | 0.00 | 0.02 | 0.35 | 0.00 | 1.00 | 6.50 | 9.50 | 1.00 | 0.23 | 17.59 |
| nopeeks #1 | move | 19.08 | 20.20 | 62.00 | 0.62 | 1.07 | 1.00 | 5.71 | 5.76 | 0.00 | 0.02 | 0.29 | 0.01 | 0.00 | 5.60 | 10.80 | 1.10 | 0.27 | 19.87 |
| nopeeks #2 | move | 19.70 | 21.15 | 62.00 | 0.62 | 1.01 | 1.00 | 5.71 | 5.76 | 0.00 | 0.02 | 0.29 | 0.01 | 0.00 | 5.70 | 10.80 | 1.10 | 0.31 | 20.51 |
| nopeeks #3 | move | 19.18 | 20.55 | 62.00 | 0.69 | 1.06 | 1.00 | 5.71 | 5.75 | 0.00 | 0.02 | 0.30 | 0.01 | 0.00 | 5.70 | 10.60 | 1.00 | 0.30 | 19.99 |
| nopeeks #4 | move | 19.54 | 20.77 | 62.00 | 0.64 | 1.10 | 1.00 | 5.72 | 5.76 | 0.00 | 0.02 | 0.30 | 0.01 | 0.00 | 5.00 | 10.50 | 1.10 | 0.35 | 20.10 |
| default #1 | move | 24.97 | 26.60 | 64.00 | 0.97 | 0.63 | 2.00 | 5.72 | 5.76 | 0.45 | 0.00 | 0.00 | 0.00 | 0.00 | 7.40 | 10.80 | 0.00 | 0.00 | 0.00 |
| default #2 | move | 25.96 | 28.08 | 64.00 | 0.98 | 0.80 | 1.00 | 5.71 | 5.76 | 0.41 | 0.00 | 0.00 | 0.00 | 0.00 | 7.50 | 10.90 | 0.00 | 0.00 | 0.00 |
| default #3 | move | 26.61 | 27.72 | 63.00 | 0.96 | 0.65 | 1.00 | 5.71 | 5.75 | 0.42 | 0.00 | 0.00 | 0.00 | 0.00 | 8.10 | 11.60 | 0.00 | 0.00 | 0.00 |
| default #4 | move | 26.19 | 29.29 | 63.00 | 0.93 | 0.84 | 1.00 | 5.71 | 5.75 | 0.39 | 0.00 | 0.00 | 0.00 | 0.00 | 8.10 | 11.30 | 0.00 | 0.00 | 0.00 |
| morepeeks #1 | move | 25.31 | 29.27 | 63.00 | 0.96 | 0.76 | 1.00 | 5.71 | 5.76 | 0.46 | 0.00 | 0.00 | 0.00 | 0.00 | 7.50 | 10.80 | 0.00 | 0.00 | 0.00 |
| morepeeks #2 | move | 26.17 | 28.94 | 64.00 | 0.91 | 0.64 | 1.00 | 5.71 | 5.76 | 0.44 | 0.00 | 0.00 | 0.00 | 0.00 | 8.80 | 12.00 | 0.00 | 0.00 | 0.00 |
| morepeeks #3 | move | 25.60 | 26.93 | 64.00 | 0.94 | 0.64 | 2.00 | 5.71 | 5.76 | 0.45 | 0.00 | 0.00 | 0.00 | 0.00 | 7.40 | 10.60 | 0.00 | 0.00 | 0.00 |
| morepeeks #4 | move | 25.18 | 27.18 | 63.00 | 1.01 | 0.69 | 2.00 | 5.72 | 5.77 | 0.43 | 0.00 | 0.00 | 0.00 | 0.00 | 6.80 | 10.30 | 0.00 | 0.00 | 0.00 |

## float

| label | scenario | command p50 ms | command p90 ms | frames | tick ms/frame | frame p90 ms | frames >2x budget | interval p50 ms | interval p90 ms | peek_frame ms/frame | session_overlays ms/frame | batch_commit ms/frame | ovl_region ms/frame | dwm_flush calls (in frames) | dwm_flush ms (outside frames) | cloak ms (outside frames) | session_begin ms (outside frames) | input->anim start ms | input->first frame ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| main #1 | float | 60.28 | 64.86 | 136.00 | 1.89 | 4.17 | 6.00 | 5.72 | 5.76 | 0.00 | 0.02 | 0.66 | 0.02 | 5.00 | 8.20 | 26.10 | 3.50 | 0.25 | 77.47 |
| main #2 | float | 90.39 | 98.59 | 61.00 | 4.79 | 8.03 | 5.00 | 5.72 | 9.70 | 0.00 | 0.03 | 1.46 | 0.11 | 3.00 | 6.10 | 35.20 | 3.30 | 0.25 | 104.86 |
| main #3 | float | 97.22 | 107.50 | 60.00 | 4.17 | 10.23 | 5.00 | 5.72 | 10.93 | 0.00 | 0.04 | 1.94 | 0.13 | 3.00 | 11.00 | 35.60 | 3.00 | 0.28 | 113.54 |
| main #4 | float | 92.57 | 116.96 | 61.00 | 4.26 | 7.65 | 6.00 | 5.72 | 10.86 | 0.00 | 0.04 | 1.37 | 0.10 | 3.00 | 7.70 | 31.90 | 3.30 | 0.26 | 121.14 |
| nopeeks #1 | float | 120.93 | 128.59 | 60.00 | 4.34 | 9.67 | 4.00 | 5.72 | 9.77 | 0.00 | 0.05 | 2.44 | 0.17 | 0.00 | 14.30 | 37.70 | 3.10 | 0.33 | 136.61 |
| nopeeks #2 | float | 111.28 | 131.53 | 60.00 | 4.04 | 9.55 | 4.00 | 5.72 | 9.57 | 0.00 | 0.05 | 1.92 | 0.17 | 0.00 | 6.70 | 35.90 | 3.10 | 0.32 | 119.08 |
| nopeeks #3 | float | 88.34 | 125.58 | 61.00 | 4.26 | 9.94 | 5.00 | 5.72 | 10.03 | 0.00 | 0.05 | 1.73 | 0.13 | 0.00 | 12.10 | 38.80 | 3.10 | 0.32 | 91.50 |
| nopeeks #4 | float | 88.12 | 122.30 | 61.00 | 3.88 | 10.02 | 5.00 | 5.72 | 10.04 | 0.00 | 0.05 | 1.69 | 0.14 | 0.00 | 12.00 | 34.50 | 3.00 | 0.32 | 93.71 |
| default #1 | float | 121.90 | 133.18 | 61.00 | 4.83 | 9.86 | 4.00 | 5.72 | 10.46 | 0.00 | 0.05 | 2.42 | 0.16 | 0.00 | 15.30 | 41.90 | 3.10 | 0.31 | 138.91 |
| default #2 | float | 107.83 | 119.86 | 61.00 | 4.33 | 8.48 | 5.00 | 5.72 | 9.17 | 0.00 | 0.05 | 1.99 | 0.17 | 0.00 | 8.30 | 36.30 | 2.90 | 0.38 | 114.32 |
| default #3 | float | 117.04 | 131.97 | 61.00 | 4.21 | 4.95 | 4.00 | 5.72 | 6.68 | 0.00 | 0.06 | 1.51 | 0.09 | 0.00 | 10.00 | 52.60 | 3.20 | 0.33 | 92.38 |
| default #4 | float | 93.51 | 132.87 | 61.00 | 4.20 | 6.29 | 4.00 | 5.72 | 7.28 | 0.00 | 0.05 | 1.38 | 0.15 | 0.00 | 15.10 | 36.20 | 3.20 | 0.36 | 97.46 |
| morepeeks #1 | float | 165.39 | 168.93 | 56.00 | 5.73 | 5.10 | 5.00 | 5.71 | 7.43 | 3.94 | 0.00 | 0.00 | 0.09 | 2.00 | 19.20 | 52.90 | 0.00 | 0.00 | 0.00 |
| morepeeks #2 | float | 161.18 | 182.12 | 56.00 | 6.07 | 4.84 | 5.00 | 5.71 | 6.87 | 3.52 | 0.00 | 0.00 | 0.05 | 2.00 | 19.80 | 54.50 | 0.00 | 0.00 | 0.00 |
| morepeeks #3 | float | 165.41 | 187.78 | 59.00 | 5.69 | 5.05 | 5.00 | 5.71 | 7.36 | 3.63 | 0.00 | 0.00 | 0.09 | 2.00 | 20.40 | 50.20 | 0.00 | 0.00 | 0.00 |
| morepeeks #4 | float | 146.91 | 170.30 | 58.00 | 5.68 | 4.99 | 5.00 | 5.72 | 6.44 | 3.59 | 0.00 | 0.00 | 0.10 | 2.00 | 13.90 | 48.00 | 0.00 | 0.00 | 0.00 |

## minimize

| label | scenario | command p50 ms | command p90 ms | frames | tick ms/frame | frame p90 ms | frames >2x budget | interval p50 ms | interval p90 ms | peek_frame ms/frame | session_overlays ms/frame | batch_commit ms/frame | ovl_region ms/frame | dwm_flush calls (in frames) | dwm_flush ms (outside frames) | cloak ms (outside frames) | session_begin ms (outside frames) | input->anim start ms | input->first frame ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| main #1 | minimize | 35.73 | 67.04 | 132.00 | 2.07 | 3.82 | 7.00 | 5.72 | 6.22 | 0.00 | 0.02 | 0.75 | 0.03 | 4.00 | 8.00 | 32.90 | 7.60 | 4.75 | 104.28 |
| main #2 | minimize | 56.41 | 65.84 | 132.00 | 2.14 | 3.95 | 7.00 | 5.72 | 5.81 | 0.00 | 0.02 | 0.78 | 0.09 | 4.00 | 6.40 | 36.50 | 5.80 | 4.85 | 115.26 |
| main #3 | minimize | 58.93 | 61.20 | 133.00 | 1.87 | 5.22 | 6.00 | 5.72 | 5.87 | 0.00 | 0.01 | 0.80 | 0.07 | 4.00 | 8.00 | 32.10 | 5.00 | 4.17 | 111.32 |
| main #4 | minimize | 56.15 | 67.23 | 126.00 | 2.14 | 3.26 | 7.00 | 5.72 | 5.82 | 0.00 | 0.02 | 0.85 | 0.07 | 5.00 | 9.10 | 39.30 | 6.90 | 4.15 | 132.98 |
| nopeeks #1 | minimize | 70.74 | 82.93 | 135.00 | 1.74 | 5.48 | 7.00 | 5.72 | 6.05 | 0.00 | 0.02 | 0.88 | 0.05 | 1.00 | 14.50 | 55.20 | 7.60 | 4.45 | 153.98 |
| nopeeks #2 | minimize | 70.17 | 74.65 | 133.00 | 1.85 | 4.92 | 6.00 | 5.72 | 6.44 | 0.00 | 0.02 | 0.89 | 0.05 | 1.00 | 7.50 | 39.60 | 4.90 | 4.45 | 125.90 |
| nopeeks #3 | minimize | 40.55 | 78.90 | 135.00 | 1.70 | 5.82 | 6.00 | 5.72 | 6.80 | 0.00 | 0.02 | 0.76 | 0.06 | 1.00 | 12.50 | 41.60 | 3.30 | 4.14 | 115.41 |
| nopeeks #4 | minimize | 67.73 | 72.80 | 134.00 | 1.73 | 5.95 | 7.00 | 5.72 | 6.55 | 0.00 | 0.02 | 0.82 | 0.05 | 1.00 | 16.90 | 43.20 | 4.20 | 4.69 | 127.33 |
| default #1 | minimize | 78.85 | 87.89 | 127.00 | 2.17 | 4.08 | 7.00 | 5.72 | 5.78 | 1.36 | 0.02 | 0.62 | 0.05 | 1.00 | 16.50 | 33.90 | 3.00 | 31.78 | 183.20 |
| default #2 | minimize | 73.34 | 91.72 | 132.00 | 1.92 | 3.92 | 7.00 | 5.72 | 5.79 | 0.83 | 0.01 | 0.45 | 0.04 | 1.00 | 13.60 | 40.60 | 2.90 | 0.57 | 105.64 |
| default #3 | minimize | 76.13 | 83.15 | 131.00 | 2.00 | 4.37 | 6.00 | 5.72 | 5.78 | 1.24 | 0.01 | 0.64 | 0.03 | 1.00 | 9.00 | 55.90 | 2.90 | 41.47 | 224.14 |
| default #4 | minimize | 78.70 | 92.90 | 130.00 | 1.97 | 4.16 | 7.00 | 5.72 | 5.79 | 1.34 | 0.02 | 0.63 | 0.05 | 1.00 | 15.70 | 33.80 | 2.90 | 41.73 | 238.19 |
| morepeeks #1 | minimize | 73.31 | 86.48 | 134.00 | 2.00 | 4.30 | 6.00 | 5.72 | 5.79 | 0.00 | 0.02 | 0.67 | 0.05 | 1.00 | 12.80 | 37.10 | 3.20 | 5.03 | 133.88 |
| morepeeks #2 | minimize | 70.88 | 91.19 | 132.00 | 2.09 | 4.38 | 7.00 | 5.72 | 5.79 | 1.35 | 0.02 | 0.56 | 0.04 | 1.00 | 15.90 | 33.70 | 3.00 | 30.59 | 187.16 |
| morepeeks #3 | minimize | 72.00 | 88.15 | 129.00 | 2.04 | 3.83 | 7.00 | 5.72 | 5.78 | 1.25 | 0.02 | 0.60 | 0.05 | 1.00 | 16.40 | 33.80 | 2.70 | 45.41 | 235.66 |
| morepeeks #4 | minimize | 76.09 | 84.01 | 128.00 | 2.14 | 3.94 | 7.00 | 5.71 | 5.78 | 1.21 | 0.02 | 0.61 | 0.05 | 1.00 | 10.80 | 36.30 | 2.80 | 45.03 | 244.78 |

## resize

| label | scenario | command p50 ms | command p90 ms | frames | tick ms/frame | frame p90 ms | frames >2x budget | interval p50 ms | interval p90 ms | peek_frame ms/frame | session_overlays ms/frame | batch_commit ms/frame | ovl_region ms/frame | dwm_flush calls (in frames) | dwm_flush ms (outside frames) | cloak ms (outside frames) | session_begin ms (outside frames) | input->anim start ms | input->first frame ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| main #1 | resize | 87.40 | 90.37 | 57.00 | 5.24 | 11.35 | 5.00 | 5.72 | 11.72 | 0.00 | 0.04 | 2.31 | 0.12 | 3.00 | 5.70 | 32.10 | 3.00 | 0.24 | 118.32 |
| main #2 | resize | 81.70 | 96.98 | 57.00 | 5.07 | 10.68 | 5.00 | 5.72 | 10.90 | 0.00 | 0.04 | 2.21 | 0.11 | 2.00 | 5.10 | 32.60 | 3.10 | 0.28 | 94.04 |
| main #3 | resize | 87.96 | 94.87 | 57.00 | 4.75 | 10.51 | 4.00 | 5.72 | 10.83 | 0.00 | 0.03 | 2.35 | 0.15 | 3.00 | 7.10 | 32.30 | 3.30 | 0.24 | 116.94 |
| main #4 | resize | 73.77 | 95.72 | 57.00 | 4.34 | 10.60 | 6.00 | 5.72 | 10.61 | 0.00 | 0.04 | 2.34 | 0.09 | 3.00 | 11.10 | 30.20 | 3.30 | 0.24 | 83.90 |
| nopeeks #1 | resize | 115.55 | 122.92 | 57.00 | 4.40 | 10.27 | 5.00 | 5.72 | 10.43 | 0.00 | 0.06 | 2.35 | 0.07 | 0.00 | 13.40 | 46.00 | 3.20 | 0.34 | 132.36 |
| nopeeks #2 | resize | 113.08 | 124.58 | 57.00 | 4.39 | 8.64 | 5.00 | 5.72 | 9.70 | 0.00 | 0.06 | 2.45 | 0.12 | 0.00 | 11.00 | 42.90 | 3.10 | 0.30 | 124.65 |
| nopeeks #3 | resize | 94.37 | 125.03 | 58.00 | 4.13 | 10.01 | 5.00 | 5.72 | 11.01 | 0.00 | 0.05 | 1.99 | 0.14 | 0.00 | 11.10 | 41.90 | 3.10 | 0.32 | 103.68 |
| nopeeks #4 | resize | 115.39 | 121.79 | 56.00 | 4.17 | 10.75 | 5.00 | 5.72 | 10.83 | 0.00 | 0.05 | 2.35 | 0.11 | 0.00 | 12.10 | 35.30 | 3.20 | 0.32 | 133.35 |
| default #1 | resize | 161.49 | 165.37 | 55.00 | 6.49 | 5.32 | 5.00 | 5.71 | 7.45 | 3.95 | 0.00 | 0.00 | 0.09 | 2.00 | 12.50 | 47.50 | 0.00 | 0.00 | 0.00 |
| default #2 | resize | 145.15 | 157.86 | 57.00 | 6.13 | 5.15 | 5.00 | 5.71 | 6.78 | 3.79 | 0.00 | 0.00 | 0.10 | 2.00 | 17.10 | 37.80 | 0.00 | 0.00 | 0.00 |
| default #3 | resize | 161.91 | 164.45 | 57.00 | 6.06 | 5.12 | 5.00 | 5.71 | 6.47 | 3.43 | 0.00 | 0.00 | 0.05 | 2.00 | 9.00 | 61.60 | 0.00 | 0.00 | 0.00 |
| default #4 | resize | 157.54 | 167.01 | 57.00 | 6.10 | 5.24 | 5.00 | 5.71 | 6.36 | 3.83 | 0.00 | 0.00 | 0.10 | 2.00 | 19.20 | 46.60 | 0.00 | 0.00 | 0.00 |
| morepeeks #1 | resize | 163.32 | 177.40 | 57.00 | 5.88 | 5.35 | 5.00 | 5.71 | 6.54 | 3.75 | 0.00 | 0.00 | 0.08 | 2.00 | 14.60 | 51.80 | 0.00 | 0.00 | 0.00 |
| morepeeks #2 | resize | 161.31 | 173.75 | 55.00 | 6.24 | 5.25 | 5.00 | 5.71 | 6.78 | 3.98 | 0.00 | 0.00 | 0.11 | 2.00 | 18.40 | 50.60 | 0.00 | 0.00 | 0.00 |
| morepeeks #3 | resize | 138.30 | 171.11 | 55.00 | 5.36 | 4.67 | 5.00 | 5.72 | 7.13 | 3.73 | 0.00 | 0.00 | 0.10 | 2.00 | 13.60 | 39.50 | 0.00 | 0.00 | 0.00 |
| morepeeks #4 | resize | 160.89 | 181.31 | 53.00 | 6.37 | 5.32 | 5.00 | 5.71 | 7.36 | 3.93 | 0.00 | 0.00 | 0.09 | 2.00 | 18.10 | 49.00 | 0.00 | 0.00 | 0.00 |

## switch

| label | scenario | command p50 ms | command p90 ms | frames | tick ms/frame | frame p90 ms | frames >2x budget | interval p50 ms | interval p90 ms | peek_frame ms/frame | session_overlays ms/frame | batch_commit ms/frame | ovl_region ms/frame | dwm_flush calls (in frames) | dwm_flush ms (outside frames) | cloak ms (outside frames) | session_begin ms (outside frames) | input->anim start ms | input->first frame ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| main #1 | switch | 38.82 | 42.44 | 98.00 | 2.79 | 4.06 | 3.00 | 5.71 | 5.77 | 0.00 | 0.00 | 0.87 | 0.02 | 3.00 | 9.80 | 0.00 | 0.00 | 0.00 | 0.00 |
| main #2 | switch | 39.61 | 43.22 | 97.00 | 2.65 | 4.08 | 2.00 | 5.71 | 5.79 | 0.00 | 0.00 | 0.86 | 0.04 | 3.00 | 8.90 | 0.00 | 0.00 | 0.00 | 0.00 |
| main #3 | switch | 38.27 | 50.76 | 96.00 | 2.91 | 4.52 | 3.00 | 5.71 | 5.78 | 0.00 | 0.00 | 0.88 | 0.04 | 3.00 | 8.70 | 0.00 | 0.00 | 0.00 | 0.00 |
| main #4 | switch | 38.37 | 46.03 | 96.00 | 2.76 | 3.97 | 2.00 | 5.72 | 5.77 | 0.00 | 0.00 | 0.87 | 0.04 | 3.00 | 7.30 | 0.00 | 0.00 | 0.00 | 0.00 |
| nopeeks #1 | switch | 57.83 | 70.47 | 98.00 | 2.19 | 2.24 | 3.00 | 5.72 | 5.77 | 0.00 | 0.00 | 0.00 | 0.04 | 0.00 | 8.10 | 0.00 | 0.00 | 0.00 | 0.00 |
| nopeeks #2 | switch | 53.32 | 73.32 | 99.00 | 1.97 | 1.98 | 3.00 | 5.71 | 5.77 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 9.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| nopeeks #3 | switch | 63.47 | 83.07 | 99.00 | 1.85 | 2.09 | 4.00 | 5.71 | 5.77 | 0.00 | 0.00 | 0.00 | 0.05 | 0.00 | 9.50 | 0.00 | 0.00 | 0.00 | 0.00 |
| nopeeks #4 | switch | 66.11 | 81.55 | 99.00 | 2.05 | 2.17 | 3.00 | 5.71 | 5.77 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 9.10 | 0.00 | 0.00 | 0.00 | 0.00 |
| default #1 | switch | 51.50 | 67.45 | 99.00 | 2.10 | 2.11 | 3.00 | 5.71 | 5.76 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 7.10 | 0.00 | 0.00 | 0.00 | 0.00 |
| default #2 | switch | 90.77 | 93.73 | 99.00 | 2.11 | 2.27 | 3.00 | 5.72 | 5.78 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 18.50 | 0.00 | 0.00 | 0.00 | 0.00 |
| default #3 | switch | 62.01 | 83.85 | 99.00 | 1.98 | 2.08 | 3.00 | 5.71 | 5.77 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 7.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| default #4 | switch | 57.75 | 80.78 | 99.00 | 2.14 | 2.09 | 3.00 | 5.72 | 5.77 | 0.00 | 0.00 | 0.00 | 0.04 | 0.00 | 7.50 | 0.00 | 0.00 | 0.00 | 0.00 |
| morepeeks #1 | switch | 55.75 | 73.15 | 99.00 | 2.20 | 2.19 | 3.00 | 5.71 | 5.77 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 8.50 | 0.00 | 0.00 | 0.00 | 0.00 |
| morepeeks #2 | switch | 57.31 | 73.28 | 99.00 | 2.21 | 2.27 | 3.00 | 5.71 | 5.78 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 7.70 | 0.00 | 0.00 | 0.00 | 0.00 |
| morepeeks #3 | switch | 54.22 | 69.49 | 99.00 | 2.09 | 2.22 | 3.00 | 5.71 | 5.77 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 8.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| morepeeks #4 | switch | 57.11 | 76.10 | 99.00 | 2.06 | 2.24 | 3.00 | 5.72 | 5.78 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 7.80 | 0.00 | 0.00 | 0.00 | 0.00 |
```

## Restart-order run, no CPU load (bench-run.ps1)

```text
Bench run, 2026-11-12T21:34:10, modes main,nopeeks,default,morepeeks, target chrome, 4 round(s) of 10 bursts per scenario and mode, load 0, CPU load 0 thread(s) of 16
tiles main #1 ws 1: chrome, ApplicationFrameHost, ApplicationFrameHost, explorer, Notepad, WindowsTerminal, Everything
label=main #1 scenario=move target=chrome bursts=10 load=0 reports=9 wall=12.4s
  command p50 ms                          37.24
  command p90 ms                          39.30
  frames                                  63.00
  tick ms/frame                            1.24
  frame p90 ms                             1.08
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.03
  batch_commit ms/frame                    0.48
  ovl_region ms/frame                      0.06
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)           10.40
  cloak ms (outside frames)               16.00
  session_begin ms (outside frames)        1.00
  input->anim start ms                     0.21
  input->first frame ms                   46.70
| main #1 | move | 37.24 | 39.30 | 63.00 | 1.24 | 1.08 | 1.00 | 5.71 | 5.77 | 0.00 | 0.03 | 0.48 | 0.06 | 1.00 | 10.40 | 16.00 | 1.00 | 0.21 | 46.70 |
label=main #1 scenario=float target=chrome bursts=10 load=0 reports=9 wall=12.8s
  command p50 ms                          77.37
  command p90 ms                          84.65
  frames                                  60.00
  tick ms/frame                            3.59
  frame p90 ms                             6.48
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          7.81
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.03
  batch_commit ms/frame                    1.25
  ovl_region ms/frame                      0.06
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)           10.10
  cloak ms (outside frames)               30.10
  session_begin ms (outside frames)        2.70
  input->anim start ms                     0.23
  input->first frame ms                   77.58
| main #1 | float | 77.37 | 84.65 | 60.00 | 3.59 | 6.48 | 4.00 | 5.72 | 7.81 | 0.00 | 0.03 | 1.25 | 0.06 | 3.00 | 10.10 | 30.10 | 2.70 | 0.23 | 77.58 |
label=main #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          47.05
  command p90 ms                          56.44
  frames                                 136.00
  tick ms/frame                            1.70
  frame p90 ms                             4.48
  frames >2x budget                        6.00
  interval p50 ms                          5.72
  interval p90 ms                          5.79
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.95
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              5.00
  dwm_flush ms (outside frames)            8.50
  cloak ms (outside frames)               35.30
  session_begin ms (outside frames)        5.90
  input->anim start ms                     3.92
  input->first frame ms                  131.58
| main #1 | minimize | 47.05 | 56.44 | 136.00 | 1.70 | 4.48 | 6.00 | 5.72 | 5.79 | 0.00 | 0.02 | 0.95 | 0.05 | 5.00 | 8.50 | 35.30 | 5.90 | 3.92 | 131.58 |
label=main #1 scenario=resize target=chrome bursts=10 load=0 reports=11 wall=12.8s
  command p50 ms                          75.34
  command p90 ms                          84.66
  frames                                  57.00
  tick ms/frame                            4.15
  frame p90 ms                             8.75
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          8.86
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.04
  batch_commit ms/frame                    2.15
  ovl_region ms/frame                      0.07
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            8.20
  cloak ms (outside frames)               25.10
  session_begin ms (outside frames)        2.70
  input->anim start ms                     0.22
  input->first frame ms                   79.08
| main #1 | resize | 75.34 | 84.66 | 57.00 | 4.15 | 8.75 | 4.00 | 5.72 | 8.86 | 0.00 | 0.04 | 2.15 | 0.07 | 3.00 | 8.20 | 25.10 | 2.70 | 0.22 | 79.08 |
Moved 3 window(s) to workspace '2' to switch to.
label=main #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          37.70
  command p90 ms                          44.18
  frames                                  97.00
  tick ms/frame                            2.18
  frame p90 ms                             3.39
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.70
  ovl_region ms/frame                      0.07
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)           14.80
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| main #1 | switch | 37.70 | 44.18 | 97.00 | 2.18 | 3.39 | 2.00 | 5.71 | 5.75 | 0.00 | 0.00 | 0.70 | 0.07 | 3.00 | 14.80 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles nopeeks #1 ws 1: ApplicationFrameHost, ApplicationFrameHost, Notepad, Everything, WindowsTerminal, explorer, chrome
label=nopeeks #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          17.72
  command p90 ms                          18.89
  frames                                  63.00
  tick ms/frame                            0.55
  frame p90 ms                             0.96
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.26
  ovl_region ms/frame                      0.01
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            6.60
  cloak ms (outside frames)               11.00
  session_begin ms (outside frames)        1.00
  input->anim start ms                     0.29
  input->first frame ms                   18.32
| nopeeks #1 | move | 17.72 | 18.89 | 63.00 | 0.55 | 0.96 | 0.00 | 5.71 | 5.75 | 0.00 | 0.02 | 0.26 | 0.01 | 0.00 | 6.60 | 11.00 | 1.00 | 0.29 | 18.32 |
label=nopeeks #1 scenario=float target=chrome bursts=10 load=0 reports=10 wall=12.9s
  command p50 ms                          78.51
  command p90 ms                         107.29
  frames                                  60.00
  tick ms/frame                            3.62
  frame p90 ms                             7.66
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          8.55
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.06
  batch_commit ms/frame                    1.45
  ovl_region ms/frame                      0.12
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            9.60
  cloak ms (outside frames)               29.20
  session_begin ms (outside frames)        2.60
  input->anim start ms                     0.28
  input->first frame ms                   92.11
| nopeeks #1 | float | 78.51 | 107.29 | 60.00 | 3.62 | 7.66 | 5.00 | 5.72 | 8.55 | 0.00 | 0.06 | 1.45 | 0.12 | 0.00 | 9.60 | 29.20 | 2.60 | 0.28 | 92.11 |
label=nopeeks #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          36.71
  command p90 ms                          63.66
  frames                                 136.00
  tick ms/frame                            1.52
  frame p90 ms                             4.58
  frames >2x budget                        5.00
  interval p50 ms                          5.71
  interval p90 ms                          5.81
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.03
  batch_commit ms/frame                    0.72
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            8.50
  cloak ms (outside frames)               28.70
  session_begin ms (outside frames)        2.90
  input->anim start ms                     3.28
  input->first frame ms                  109.84
| nopeeks #1 | minimize | 36.71 | 63.66 | 136.00 | 1.52 | 4.58 | 5.00 | 5.71 | 5.81 | 0.00 | 0.03 | 0.72 | 0.04 | 1.00 | 8.50 | 28.70 | 2.90 | 3.28 | 109.84 |
label=nopeeks #1 scenario=resize target=chrome bursts=10 load=0 reports=11 wall=13.0s
  command p50 ms                          86.29
  command p90 ms                         112.74
  frames                                  58.00
  tick ms/frame                            3.43
  frame p90 ms                             7.39
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          9.36
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.05
  batch_commit ms/frame                    2.10
  ovl_region ms/frame                      0.06
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           12.40
  cloak ms (outside frames)               32.00
  session_begin ms (outside frames)        2.70
  input->anim start ms                     0.30
  input->first frame ms                   93.85
| nopeeks #1 | resize | 86.29 | 112.74 | 58.00 | 3.43 | 7.39 | 4.00 | 5.72 | 9.36 | 0.00 | 0.05 | 2.10 | 0.06 | 0.00 | 12.40 | 32.00 | 2.70 | 0.30 | 93.85 |
Moved 3 window(s) to workspace '2' to switch to.
label=nopeeks #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          65.25
  command p90 ms                          78.83
  frames                                  99.00
  tick ms/frame                            1.87
  frame p90 ms                             1.83
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            7.00
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| nopeeks #1 | switch | 65.25 | 78.83 | 99.00 | 1.87 | 1.83 | 3.00 | 5.71 | 5.77 | 0.00 | 0.00 | 0.00 | 0.04 | 0.00 | 7.00 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles default #1 ws 1: chrome, ApplicationFrameHost, WindowsTerminal, Notepad, explorer, ApplicationFrameHost, Everything
label=default #1 scenario=move target=chrome bursts=10 load=0 reports=9 wall=12.5s
  command p50 ms                          37.82
  command p90 ms                          39.65
  frames                                  64.00
  tick ms/frame                            1.25
  frame p90 ms                             1.26
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  peek_frame ms/frame                      0.52
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           11.10
  cloak ms (outside frames)               17.60
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #1 | move | 37.82 | 39.65 | 64.00 | 1.25 | 1.26 | 1.00 | 5.71 | 5.76 | 0.52 | 0.00 | 0.00 | 0.00 | 0.00 | 11.10 | 17.60 | 0.00 | 0.00 | 0.00 |
label=default #1 scenario=float target=chrome bursts=10 load=0 reports=10 wall=13.1s
  command p50 ms                         107.76
  command p90 ms                         118.00
  frames                                  60.00
  tick ms/frame                            4.40
  frame p90 ms                            10.36
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                         10.53
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.05
  batch_commit ms/frame                    2.43
  ovl_region ms/frame                      0.14
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           12.30
  cloak ms (outside frames)               36.50
  session_begin ms (outside frames)        2.70
  input->anim start ms                     0.29
  input->first frame ms                  138.78
| default #1 | float | 107.76 | 118.00 | 60.00 | 4.40 | 10.36 | 5.00 | 5.72 | 10.53 | 0.00 | 0.05 | 2.43 | 0.14 | 0.00 | 12.30 | 36.50 | 2.70 | 0.29 | 138.78 |
label=default #1 scenario=minimize target=chrome bursts=10 load=0 reports=9 wall=12.6s
  command p50 ms                          69.08
  command p90 ms                          81.74
  frames                                 139.00
  tick ms/frame                            1.79
  frame p90 ms                             3.53
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          5.78
  peek_frame ms/frame                      0.85
  session_overlays ms/frame                0.01
  batch_commit ms/frame                    0.54
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)           13.50
  cloak ms (outside frames)               39.20
  session_begin ms (outside frames)        2.50
  input->anim start ms                     3.09
  input->first frame ms                   96.15
| default #1 | minimize | 69.08 | 81.74 | 139.00 | 1.79 | 3.53 | 5.00 | 5.72 | 5.78 | 0.85 | 0.01 | 0.54 | 0.03 | 1.00 | 13.50 | 39.20 | 2.50 | 3.09 | 96.15 |
label=default #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=13.4s
  command p50 ms                         135.94
  command p90 ms                         148.23
  frames                                  57.00
  tick ms/frame                            5.18
  frame p90 ms                             4.44
  frames >2x budget                        4.00
  interval p50 ms                          5.71
  interval p90 ms                          7.10
  peek_frame ms/frame                      3.25
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.08
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           17.00
  cloak ms (outside frames)               39.70
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #1 | resize | 135.94 | 148.23 | 57.00 | 5.18 | 4.44 | 4.00 | 5.71 | 7.10 | 3.25 | 0.00 | 0.00 | 0.08 | 2.00 | 17.00 | 39.70 | 0.00 | 0.00 | 0.00 |
Moved 3 window(s) to workspace '2' to switch to.
label=default #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.8s
  command p50 ms                          65.61
  command p90 ms                          76.48
  frames                                  99.00
  tick ms/frame                            1.65
  frame p90 ms                             1.69
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            7.80
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #1 | switch | 65.61 | 76.48 | 99.00 | 1.65 | 1.69 | 3.00 | 5.71 | 5.77 | 0.00 | 0.00 | 0.00 | 0.02 | 0.00 | 7.80 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles morepeeks #1 ws 1: ApplicationFrameHost, WindowsTerminal, explorer, Everything, ApplicationFrameHost, Notepad, chrome
label=morepeeks #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          23.27
  command p90 ms                          24.61
  frames                                  63.00
  tick ms/frame                            0.83
  frame p90 ms                             0.92
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  peek_frame ms/frame                      0.38
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            8.60
  cloak ms (outside frames)               11.50
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #1 | move | 23.27 | 24.61 | 63.00 | 0.83 | 0.92 | 1.00 | 5.71 | 5.76 | 0.38 | 0.00 | 0.00 | 0.00 | 0.00 | 8.60 | 11.50 | 0.00 | 0.00 | 0.00 |
label=morepeeks #1 scenario=float target=chrome bursts=10 load=0 reports=8 wall=14.1s
  command p50 ms                         129.65
  command p90 ms                         136.45
  frames                                  59.00
  tick ms/frame                            5.02
  frame p90 ms                             4.14
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          6.21
  peek_frame ms/frame                      3.02
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            7.30
  cloak ms (outside frames)               29.60
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #1 | float | 129.65 | 136.45 | 59.00 | 5.02 | 4.14 | 5.00 | 5.72 | 6.21 | 3.02 | 0.00 | 0.00 | 0.05 | 2.00 | 7.30 | 29.60 | 0.00 | 0.00 | 0.00 |
label=morepeeks #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          65.69
  command p90 ms                          73.54
  frames                                 135.00
  tick ms/frame                            2.21
  frame p90 ms                             3.92
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          5.78
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.55
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            7.00
  cloak ms (outside frames)               31.10
  session_begin ms (outside frames)        2.90
  input->anim start ms                     4.18
  input->first frame ms                  115.78
| morepeeks #1 | minimize | 65.69 | 73.54 | 135.00 | 2.21 | 3.92 | 5.00 | 5.72 | 5.78 | 0.00 | 0.02 | 0.55 | 0.04 | 1.00 | 7.00 | 31.10 | 2.90 | 4.18 | 115.78 |
label=morepeeks #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=13.4s
  command p50 ms                         133.86
  command p90 ms                         143.17
  frames                                  59.00
  tick ms/frame                            5.17
  frame p90 ms                             4.25
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          6.15
  peek_frame ms/frame                      3.24
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.06
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            6.70
  cloak ms (outside frames)               41.30
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #1 | resize | 133.86 | 143.17 | 59.00 | 5.17 | 4.25 | 5.00 | 5.72 | 6.15 | 3.24 | 0.00 | 0.00 | 0.06 | 2.00 | 6.70 | 41.30 | 0.00 | 0.00 | 0.00 |
Moved 3 window(s) to workspace '2' to switch to.
label=morepeeks #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          63.66
  command p90 ms                          82.49
  frames                                  99.00
  tick ms/frame                            2.20
  frame p90 ms                             2.02
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            6.90
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #1 | switch | 63.66 | 82.49 | 99.00 | 2.20 | 2.02 | 4.00 | 5.72 | 5.77 | 0.00 | 0.00 | 0.00 | 0.02 | 0.00 | 6.90 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles main #2 ws 1: chrome, ApplicationFrameHost, ApplicationFrameHost, explorer, Notepad, WindowsTerminal, Everything
label=main #2 scenario=move target=chrome bursts=10 load=0 reports=9 wall=12.3s
  command p50 ms                          22.30
  command p90 ms                          24.33
  frames                                  63.00
  tick ms/frame                            0.88
  frame p90 ms                             1.16
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.03
  batch_commit ms/frame                    0.39
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            7.10
  cloak ms (outside frames)               10.60
  session_begin ms (outside frames)        1.00
  input->anim start ms                     0.20
  input->first frame ms                   27.47
| main #2 | move | 22.30 | 24.33 | 63.00 | 0.88 | 1.16 | 1.00 | 5.71 | 5.75 | 0.00 | 0.03 | 0.39 | 0.00 | 1.00 | 7.10 | 10.60 | 1.00 | 0.20 | 27.47 |
label=main #2 scenario=float target=chrome bursts=10 load=0 reports=10 wall=12.8s
  command p50 ms                          75.65
  command p90 ms                          83.22
  frames                                  60.00
  tick ms/frame                            4.16
  frame p90 ms                             9.14
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          9.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.04
  batch_commit ms/frame                    1.65
  ovl_region ms/frame                      0.15
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            6.40
  cloak ms (outside frames)               32.40
  session_begin ms (outside frames)        2.90
  input->anim start ms                     0.22
  input->first frame ms                   94.29
| main #2 | float | 75.65 | 83.22 | 60.00 | 4.16 | 9.14 | 4.00 | 5.72 | 9.77 | 0.00 | 0.04 | 1.65 | 0.15 | 3.00 | 6.40 | 32.40 | 2.90 | 0.22 | 94.29 |
label=main #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          46.80
  command p90 ms                          64.43
  frames                                 131.00
  tick ms/frame                            1.95
  frame p90 ms                             3.47
  frames >2x budget                        5.00
  interval p50 ms                          5.71
  interval p90 ms                          5.79
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.82
  ovl_region ms/frame                      0.06
  dwm_flush calls (in frames)              5.00
  dwm_flush ms (outside frames)            9.30
  cloak ms (outside frames)               41.10
  session_begin ms (outside frames)        6.30
  input->anim start ms                     4.49
  input->first frame ms                  129.99
| main #2 | minimize | 46.80 | 64.43 | 131.00 | 1.95 | 3.47 | 5.00 | 5.71 | 5.79 | 0.00 | 0.02 | 0.82 | 0.06 | 5.00 | 9.30 | 41.10 | 6.30 | 4.49 | 129.99 |
label=main #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.9s
  command p50 ms                          81.82
  command p90 ms                          85.31
  frames                                  59.00
  tick ms/frame                            4.61
  frame p90 ms                             9.71
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                         10.13
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.04
  batch_commit ms/frame                    2.07
  ovl_region ms/frame                      0.09
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)           10.90
  cloak ms (outside frames)               39.10
  session_begin ms (outside frames)        2.60
  input->anim start ms                     0.22
  input->first frame ms                  102.09
| main #2 | resize | 81.82 | 85.31 | 59.00 | 4.61 | 9.71 | 4.00 | 5.72 | 10.13 | 0.00 | 0.04 | 2.07 | 0.09 | 3.00 | 10.90 | 39.10 | 2.60 | 0.22 | 102.09 |
Moved 3 window(s) to workspace '2' to switch to.
label=main #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          51.59
  command p90 ms                          61.77
  frames                                  96.00
  tick ms/frame                            2.42
  frame p90 ms                             3.68
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.76
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)           12.30
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| main #2 | switch | 51.59 | 61.77 | 96.00 | 2.42 | 3.68 | 3.00 | 5.71 | 5.76 | 0.00 | 0.00 | 0.76 | 0.03 | 3.00 | 12.30 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles nopeeks #2 ws 1: ApplicationFrameHost, ApplicationFrameHost, Notepad, Everything, WindowsTerminal, explorer, chrome
label=nopeeks #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          18.37
  command p90 ms                          18.74
  frames                                  63.00
  tick ms/frame                            0.53
  frame p90 ms                             0.93
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.25
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            7.10
  cloak ms (outside frames)               11.50
  session_begin ms (outside frames)        0.90
  input->anim start ms                     0.30
  input->first frame ms                   18.85
| nopeeks #2 | move | 18.37 | 18.74 | 63.00 | 0.53 | 0.93 | 0.00 | 5.71 | 5.75 | 0.00 | 0.02 | 0.25 | 0.02 | 0.00 | 7.10 | 11.50 | 0.90 | 0.30 | 18.85 |
label=nopeeks #2 scenario=float target=chrome bursts=10 load=0 reports=10 wall=13.0s
  command p50 ms                          99.07
  command p90 ms                         113.86
  frames                                  59.00
  tick ms/frame                            3.56
  frame p90 ms                             9.95
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                         10.17
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.06
  batch_commit ms/frame                    2.35
  ovl_region ms/frame                      0.12
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            9.00
  cloak ms (outside frames)               30.00
  session_begin ms (outside frames)        2.60
  input->anim start ms                     0.28
  input->first frame ms                  121.83
| nopeeks #2 | float | 99.07 | 113.86 | 59.00 | 3.56 | 9.95 | 4.00 | 5.72 | 10.17 | 0.00 | 0.06 | 2.35 | 0.12 | 0.00 | 9.00 | 30.00 | 2.60 | 0.28 | 121.83 |
label=nopeeks #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          56.54
  command p90 ms                          66.85
  frames                                 138.00
  tick ms/frame                            1.58
  frame p90 ms                             5.17
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          5.80
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.76
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            9.20
  cloak ms (outside frames)               40.40
  session_begin ms (outside frames)        3.20
  input->anim start ms                     3.48
  input->first frame ms                  113.22
| nopeeks #2 | minimize | 56.54 | 66.85 | 138.00 | 1.58 | 5.17 | 5.00 | 5.72 | 5.80 | 0.00 | 0.02 | 0.76 | 0.05 | 1.00 | 9.20 | 40.40 | 3.20 | 3.48 | 113.22 |
label=nopeeks #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=13.0s
  command p50 ms                          94.68
  command p90 ms                         108.04
  frames                                  58.00
  tick ms/frame                            3.76
  frame p90 ms                             8.15
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          8.22
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.06
  batch_commit ms/frame                    2.27
  ovl_region ms/frame                      0.09
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           11.30
  cloak ms (outside frames)               33.40
  session_begin ms (outside frames)        2.60
  input->anim start ms                     0.32
  input->first frame ms                  105.16
| nopeeks #2 | resize | 94.68 | 108.04 | 58.00 | 3.76 | 8.15 | 4.00 | 5.72 | 8.22 | 0.00 | 0.06 | 2.27 | 0.09 | 0.00 | 11.30 | 33.40 | 2.60 | 0.32 | 105.16 |
Moved 3 window(s) to workspace '2' to switch to.
label=nopeeks #2 scenario=switch target=1<->2 bursts=10 load=0 reports=9 wall=13.0s
  command p50 ms                          67.43
  command p90 ms                         123.14
  frames                                  99.00
  tick ms/frame                            1.73
  frame p90 ms                             1.56
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            8.80
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| nopeeks #2 | switch | 67.43 | 123.14 | 99.00 | 1.73 | 1.56 | 3.00 | 5.72 | 5.77 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 8.80 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles default #2 ws 1: chrome, ApplicationFrameHost, WindowsTerminal, Notepad, explorer, ApplicationFrameHost, Everything
label=default #2 scenario=move target=chrome bursts=10 load=0 reports=9 wall=12.3s
  command p50 ms                          26.46
  command p90 ms                          27.07
  frames                                  63.00
  tick ms/frame                            1.08
  frame p90 ms                             1.18
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.49
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            8.30
  cloak ms (outside frames)               12.30
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #2 | move | 26.46 | 27.07 | 63.00 | 1.08 | 1.18 | 2.00 | 5.71 | 5.77 | 0.49 | 0.00 | 0.00 | 0.00 | 0.00 | 8.30 | 12.30 | 0.00 | 0.00 | 0.00 |
label=default #2 scenario=float target=chrome bursts=10 load=0 reports=11 wall=13.0s
  command p50 ms                          97.29
  command p90 ms                         117.72
  frames                                  60.00
  tick ms/frame                            3.69
  frame p90 ms                             5.42
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          6.99
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.05
  batch_commit ms/frame                    1.44
  ovl_region ms/frame                      0.08
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            6.40
  cloak ms (outside frames)               33.70
  session_begin ms (outside frames)        2.80
  input->anim start ms                     0.33
  input->first frame ms                   88.23
| default #2 | float | 97.29 | 117.72 | 60.00 | 3.69 | 5.42 | 4.00 | 5.72 | 6.99 | 0.00 | 0.05 | 1.44 | 0.08 | 0.00 | 6.40 | 33.70 | 2.80 | 0.33 | 88.23 |
label=default #2 scenario=minimize target=chrome bursts=10 load=0 reports=9 wall=12.5s
  command p50 ms                          67.12
  command p90 ms                          81.22
  frames                                 132.00
  tick ms/frame                            1.83
  frame p90 ms                             4.19
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          5.79
  peek_frame ms/frame                      0.71
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.44
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)           11.70
  cloak ms (outside frames)               47.90
  session_begin ms (outside frames)        2.60
  input->anim start ms                     0.62
  input->first frame ms                   96.02
| default #2 | minimize | 67.12 | 81.22 | 132.00 | 1.83 | 4.19 | 5.00 | 5.72 | 5.79 | 0.71 | 0.02 | 0.44 | 0.03 | 1.00 | 11.70 | 47.90 | 2.60 | 0.62 | 96.02 |
label=default #2 scenario=resize target=chrome bursts=10 load=0 reports=11 wall=13.3s
  command p50 ms                         130.96
  command p90 ms                         138.06
  frames                                  56.00
  tick ms/frame                            4.90
  frame p90 ms                             4.38
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          5.94
  peek_frame ms/frame                      3.02
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.07
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            8.50
  cloak ms (outside frames)               36.70
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #2 | resize | 130.96 | 138.06 | 56.00 | 4.90 | 4.38 | 5.00 | 5.72 | 5.94 | 3.02 | 0.00 | 0.00 | 0.07 | 1.00 | 8.50 | 36.70 | 0.00 | 0.00 | 0.00 |
Moved 3 window(s) to workspace '2' to switch to.
label=default #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.8s
  command p50 ms                          74.29
  command p90 ms                          79.91
  frames                                  99.00
  tick ms/frame                            1.69
  frame p90 ms                             1.83
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           16.70
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #2 | switch | 74.29 | 79.91 | 99.00 | 1.69 | 1.83 | 3.00 | 5.72 | 5.77 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 16.70 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles morepeeks #2 ws 1: ApplicationFrameHost, WindowsTerminal, explorer, Everything, ApplicationFrameHost, Notepad, chrome
label=morepeeks #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          22.63
  command p90 ms                          25.77
  frames                                  63.00
  tick ms/frame                            0.84
  frame p90 ms                             0.91
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  peek_frame ms/frame                      0.37
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            8.00
  cloak ms (outside frames)               10.80
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #2 | move | 22.63 | 25.77 | 63.00 | 0.84 | 0.91 | 1.00 | 5.71 | 5.75 | 0.37 | 0.00 | 0.00 | 0.00 | 0.00 | 8.00 | 10.80 | 0.00 | 0.00 | 0.00 |
label=morepeeks #2 scenario=float target=chrome bursts=10 load=0 reports=10 wall=13.3s
  command p50 ms                         127.48
  command p90 ms                         138.51
  frames                                  57.00
  tick ms/frame                            5.42
  frame p90 ms                             3.96
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          7.08
  peek_frame ms/frame                      3.13
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            8.20
  cloak ms (outside frames)               45.40
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #2 | float | 127.48 | 138.51 | 57.00 | 5.42 | 3.96 | 5.00 | 5.72 | 7.08 | 3.13 | 0.00 | 0.00 | 0.05 | 2.00 | 8.20 | 45.40 | 0.00 | 0.00 | 0.00 |
label=morepeeks #2 scenario=minimize target=chrome bursts=10 load=0 reports=9 wall=12.6s
  command p50 ms                          68.77
  command p90 ms                          77.94
  frames                                 132.00
  tick ms/frame                            2.31
  frame p90 ms                             4.14
  frames >2x budget                        6.00
  interval p50 ms                          5.71
  interval p90 ms                          5.78
  peek_frame ms/frame                      0.71
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.45
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)           13.90
  cloak ms (outside frames)               36.30
  session_begin ms (outside frames)        2.70
  input->anim start ms                     2.50
  input->first frame ms                   88.87
| morepeeks #2 | minimize | 68.77 | 77.94 | 132.00 | 2.31 | 4.14 | 6.00 | 5.71 | 5.78 | 0.71 | 0.02 | 0.45 | 0.03 | 1.00 | 13.90 | 36.30 | 2.70 | 2.50 | 88.87 |
label=morepeeks #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=13.4s
  command p50 ms                         131.04
  command p90 ms                         140.74
  frames                                  56.00
  tick ms/frame                            5.39
  frame p90 ms                             4.39
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          6.47
  peek_frame ms/frame                      3.25
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.07
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           11.40
  cloak ms (outside frames)               39.20
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #2 | resize | 131.04 | 140.74 | 56.00 | 5.39 | 4.39 | 5.00 | 5.72 | 6.47 | 3.25 | 0.00 | 0.00 | 0.07 | 2.00 | 11.40 | 39.20 | 0.00 | 0.00 | 0.00 |
Moved 3 window(s) to workspace '2' to switch to.
label=morepeeks #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.8s
  command p50 ms                          67.37
  command p90 ms                          80.72
  frames                                  99.00
  tick ms/frame                            2.21
  frame p90 ms                             2.03
  frames >2x budget                        4.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            6.80
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #2 | switch | 67.37 | 80.72 | 99.00 | 2.21 | 2.03 | 4.00 | 5.71 | 5.76 | 0.00 | 0.00 | 0.00 | 0.02 | 0.00 | 6.80 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles main #3 ws 1: chrome, ApplicationFrameHost, ApplicationFrameHost, explorer, Notepad, WindowsTerminal, Everything
label=main #3 scenario=move target=chrome bursts=10 load=0 reports=9 wall=12.3s
  command p50 ms                          22.99
  command p90 ms                          24.84
  frames                                  64.00
  tick ms/frame                            0.94
  frame p90 ms                             1.05
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.37
  ovl_region ms/frame                      0.01
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            8.10
  cloak ms (outside frames)               11.80
  session_begin ms (outside frames)        1.00
  input->anim start ms                     0.20
  input->first frame ms                   25.11
| main #3 | move | 22.99 | 24.84 | 64.00 | 0.94 | 1.05 | 1.00 | 5.71 | 5.75 | 0.00 | 0.02 | 0.37 | 0.01 | 1.00 | 8.10 | 11.80 | 1.00 | 0.20 | 25.11 |
label=main #3 scenario=float target=chrome bursts=10 load=0 reports=10 wall=12.8s
  command p50 ms                          75.95
  command p90 ms                          83.82
  frames                                  63.00
  tick ms/frame                            4.14
  frame p90 ms                             9.45
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          9.78
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.04
  batch_commit ms/frame                    1.92
  ovl_region ms/frame                      0.14
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            6.10
  cloak ms (outside frames)               30.10
  session_begin ms (outside frames)        2.60
  input->anim start ms                     0.23
  input->first frame ms                  109.29
| main #3 | float | 75.95 | 83.82 | 63.00 | 4.14 | 9.45 | 4.00 | 5.72 | 9.78 | 0.00 | 0.04 | 1.92 | 0.14 | 3.00 | 6.10 | 30.10 | 2.60 | 0.23 | 109.29 |
label=main #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          26.50
  command p90 ms                          58.52
  frames                                 137.00
  tick ms/frame                            1.91
  frame p90 ms                             4.43
  frames >2x budget                        6.00
  interval p50 ms                          5.72
  interval p90 ms                          5.80
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.77
  ovl_region ms/frame                      0.06
  dwm_flush calls (in frames)              5.00
  dwm_flush ms (outside frames)            9.70
  cloak ms (outside frames)               34.90
  session_begin ms (outside frames)        5.40
  input->anim start ms                     4.30
  input->first frame ms                  102.40
| main #3 | minimize | 26.50 | 58.52 | 137.00 | 1.91 | 4.43 | 6.00 | 5.72 | 5.80 | 0.00 | 0.02 | 0.77 | 0.06 | 5.00 | 9.70 | 34.90 | 5.40 | 4.30 | 102.40 |
label=main #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.6s
  command p50 ms                          52.68
  command p90 ms                          55.38
  frames                                 134.00
  tick ms/frame                            3.15
  frame p90 ms                             5.45
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          5.78
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.04
  batch_commit ms/frame                    1.22
  ovl_region ms/frame                      0.06
  dwm_flush calls (in frames)              4.00
  dwm_flush ms (outside frames)            6.80
  cloak ms (outside frames)               23.20
  session_begin ms (outside frames)        2.70
  input->anim start ms                     0.21
  input->first frame ms                   73.81
| main #3 | resize | 52.68 | 55.38 | 134.00 | 3.15 | 5.45 | 5.00 | 5.72 | 5.78 | 0.00 | 0.04 | 1.22 | 0.06 | 4.00 | 6.80 | 23.20 | 2.70 | 0.21 | 73.81 |
Moved 3 window(s) to workspace '2' to switch to.
label=main #3 scenario=switch target=1<->2 bursts=10 load=0 reports=9 wall=24.2s
  command p50 ms                          43.70
  command p90 ms                         115.59
  frames                                  97.00
  tick ms/frame                            2.33
  frame p90 ms                             3.62
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.75
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.69
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)           12.10
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| main #3 | switch | 43.70 | 115.59 | 97.00 | 2.33 | 3.62 | 3.00 | 5.72 | 5.75 | 0.00 | 0.00 | 0.69 | 0.05 | 3.00 | 12.10 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles nopeeks #3 ws 1: ApplicationFrameHost, ApplicationFrameHost, Notepad, Everything, WindowsTerminal, explorer, chrome
label=nopeeks #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          17.50
  command p90 ms                          18.89
  frames                                  63.00
  tick ms/frame                            0.53
  frame p90 ms                             0.94
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.25
  ovl_region ms/frame                      0.01
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            6.00
  cloak ms (outside frames)               10.20
  session_begin ms (outside frames)        0.90
  input->anim start ms                     0.27
  input->first frame ms                   18.04
| nopeeks #3 | move | 17.50 | 18.89 | 63.00 | 0.53 | 0.94 | 0.00 | 5.71 | 5.75 | 0.00 | 0.02 | 0.25 | 0.01 | 0.00 | 6.00 | 10.20 | 0.90 | 0.27 | 18.04 |
label=nopeeks #3 scenario=float target=chrome bursts=10 load=0 reports=10 wall=13.0s
  command p50 ms                          99.70
  command p90 ms                         109.27
  frames                                  59.00
  tick ms/frame                            3.74
  frame p90 ms                             9.77
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                         10.05
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.05
  batch_commit ms/frame                    2.25
  ovl_region ms/frame                      0.12
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           14.40
  cloak ms (outside frames)               28.80
  session_begin ms (outside frames)        2.50
  input->anim start ms                     0.29
  input->first frame ms                  113.44
| nopeeks #3 | float | 99.70 | 109.27 | 59.00 | 3.74 | 9.77 | 4.00 | 5.72 | 10.05 | 0.00 | 0.05 | 2.25 | 0.12 | 0.00 | 14.40 | 28.80 | 2.50 | 0.29 | 113.44 |
label=nopeeks #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          60.25
  command p90 ms                          71.04
  frames                                 135.00
  tick ms/frame                            1.60
  frame p90 ms                             6.07
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          6.38
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.91
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)           14.50
  cloak ms (outside frames)               37.20
  session_begin ms (outside frames)        3.10
  input->anim start ms                     3.56
  input->first frame ms                  128.40
| nopeeks #3 | minimize | 60.25 | 71.04 | 135.00 | 1.60 | 6.07 | 5.00 | 5.72 | 6.38 | 0.00 | 0.02 | 0.91 | 0.04 | 1.00 | 14.50 | 37.20 | 3.10 | 3.56 | 128.40 |
label=nopeeks #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=13.1s
  command p50 ms                          96.54
  command p90 ms                         109.71
  frames                                  56.00
  tick ms/frame                            3.89
  frame p90 ms                             9.55
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          9.62
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.06
  batch_commit ms/frame                    2.39
  ovl_region ms/frame                      0.10
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           12.70
  cloak ms (outside frames)               32.20
  session_begin ms (outside frames)        2.70
  input->anim start ms                     0.31
  input->first frame ms                  116.18
| nopeeks #3 | resize | 96.54 | 109.71 | 56.00 | 3.89 | 9.55 | 5.00 | 5.72 | 9.62 | 0.00 | 0.06 | 2.39 | 0.10 | 0.00 | 12.70 | 32.20 | 2.70 | 0.31 | 116.18 |
Moved 3 window(s) to workspace '2' to switch to.
label=nopeeks #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          78.94
  command p90 ms                          80.98
  frames                                  99.00
  tick ms/frame                            1.97
  frame p90 ms                             1.91
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            9.70
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| nopeeks #3 | switch | 78.94 | 80.98 | 99.00 | 1.97 | 1.91 | 3.00 | 5.71 | 5.77 | 0.00 | 0.00 | 0.00 | 0.04 | 0.00 | 9.70 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles default #3 ws 1: chrome, ApplicationFrameHost, WindowsTerminal, Notepad, explorer, ApplicationFrameHost, Everything
label=default #3 scenario=move target=chrome bursts=10 load=0 reports=9 wall=12.4s
  command p50 ms                          37.75
  command p90 ms                          40.24
  frames                                  63.00
  tick ms/frame                            1.25
  frame p90 ms                             1.18
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.51
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           11.10
  cloak ms (outside frames)               17.60
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #3 | move | 37.75 | 40.24 | 63.00 | 1.25 | 1.18 | 1.00 | 5.71 | 5.77 | 0.51 | 0.00 | 0.00 | 0.00 | 0.00 | 11.10 | 17.60 | 0.00 | 0.00 | 0.00 |
label=default #3 scenario=float target=chrome bursts=10 load=0 reports=10 wall=13.0s
  command p50 ms                         106.07
  command p90 ms                         119.69
  frames                                  60.00
  tick ms/frame                            4.82
  frame p90 ms                            10.64
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                         10.75
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.04
  batch_commit ms/frame                    2.50
  ovl_region ms/frame                      0.16
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           13.20
  cloak ms (outside frames)               33.60
  session_begin ms (outside frames)        2.70
  input->anim start ms                     0.30
  input->first frame ms                  131.12
| default #3 | float | 106.07 | 119.69 | 60.00 | 4.82 | 10.64 | 5.00 | 5.72 | 10.75 | 0.00 | 0.04 | 2.50 | 0.16 | 0.00 | 13.20 | 33.60 | 2.70 | 0.30 | 131.12 |
label=default #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          67.41
  command p90 ms                          77.98
  frames                                 135.00
  tick ms/frame                            1.79
  frame p90 ms                             3.88
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          5.78
  peek_frame ms/frame                      1.27
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.62
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)           17.20
  cloak ms (outside frames)               40.20
  session_begin ms (outside frames)        2.40
  input->anim start ms                    37.15
  input->first frame ms                  201.66
| default #3 | minimize | 67.41 | 77.98 | 135.00 | 1.79 | 3.88 | 5.00 | 5.72 | 5.78 | 1.27 | 0.02 | 0.62 | 0.05 | 1.00 | 17.20 | 40.20 | 2.40 | 37.15 | 201.66 |
label=default #3 scenario=resize target=chrome bursts=10 load=0 reports=9 wall=13.4s
  command p50 ms                         116.75
  command p90 ms                         138.02
  frames                                  59.00
  tick ms/frame                            4.99
  frame p90 ms                             4.51
  frames >2x budget                        5.00
  interval p50 ms                          5.71
  interval p90 ms                          6.16
  peek_frame ms/frame                      3.15
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.07
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            7.30
  cloak ms (outside frames)               28.10
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #3 | resize | 116.75 | 138.02 | 59.00 | 4.99 | 4.51 | 5.00 | 5.71 | 6.16 | 3.15 | 0.00 | 0.00 | 0.07 | 2.00 | 7.30 | 28.10 | 0.00 | 0.00 | 0.00 |
Moved 3 window(s) to workspace '2' to switch to.
label=default #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          72.80
  command p90 ms                          77.25
  frames                                  99.00
  tick ms/frame                            1.66
  frame p90 ms                             1.66
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.78
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           11.40
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #3 | switch | 72.80 | 77.25 | 99.00 | 1.66 | 1.66 | 3.00 | 5.72 | 5.78 | 0.00 | 0.00 | 0.00 | 0.02 | 0.00 | 11.40 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles morepeeks #3 ws 1: ApplicationFrameHost, WindowsTerminal, explorer, Everything, ApplicationFrameHost, Notepad, chrome
label=morepeeks #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          23.88
  command p90 ms                          25.42
  frames                                  63.00
  tick ms/frame                            0.84
  frame p90 ms                             0.93
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  peek_frame ms/frame                      0.38
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            8.10
  cloak ms (outside frames)               11.10
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #3 | move | 23.88 | 25.42 | 63.00 | 0.84 | 0.93 | 1.00 | 5.71 | 5.75 | 0.38 | 0.00 | 0.00 | 0.00 | 0.00 | 8.10 | 11.10 | 0.00 | 0.00 | 0.00 |
label=morepeeks #3 scenario=float target=chrome bursts=10 load=0 reports=8 wall=13.9s
  command p50 ms                         130.37
  command p90 ms                         136.58
  frames                                  57.00
  tick ms/frame                            4.95
  frame p90 ms                             4.51
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          5.98
  peek_frame ms/frame                      3.11
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            9.40
  cloak ms (outside frames)               30.70
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #3 | float | 130.37 | 136.58 | 57.00 | 4.95 | 4.51 | 5.00 | 5.72 | 5.98 | 3.11 | 0.00 | 0.00 | 0.05 | 2.00 | 9.40 | 30.70 | 0.00 | 0.00 | 0.00 |
label=morepeeks #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          73.31
  command p90 ms                          77.16
  frames                                 135.00
  tick ms/frame                            2.17
  frame p90 ms                             4.03
  frames >2x budget                        6.00
  interval p50 ms                          5.72
  interval p90 ms                          5.77
  peek_frame ms/frame                      1.09
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.55
  ovl_region ms/frame                      0.06
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)           13.80
  cloak ms (outside frames)               35.90
  session_begin ms (outside frames)        2.50
  input->anim start ms                     4.27
  input->first frame ms                  170.02
| morepeeks #3 | minimize | 73.31 | 77.16 | 135.00 | 2.17 | 4.03 | 6.00 | 5.72 | 5.77 | 1.09 | 0.02 | 0.55 | 0.06 | 1.00 | 13.80 | 35.90 | 2.50 | 4.27 | 170.02 |
label=morepeeks #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=13.3s
  command p50 ms                         111.52
  command p90 ms                         139.77
  frames                                  57.00
  tick ms/frame                            5.26
  frame p90 ms                             4.40
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          6.38
  peek_frame ms/frame                      3.16
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.07
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           12.30
  cloak ms (outside frames)               39.60
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #3 | resize | 111.52 | 139.77 | 57.00 | 5.26 | 4.40 | 5.00 | 5.72 | 6.38 | 3.16 | 0.00 | 0.00 | 0.07 | 2.00 | 12.30 | 39.60 | 0.00 | 0.00 | 0.00 |
Moved 3 window(s) to workspace '2' to switch to.
label=morepeeks #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          68.87
  command p90 ms                          78.33
  frames                                  99.00
  tick ms/frame                            2.15
  frame p90 ms                             2.03
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            7.00
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #3 | switch | 68.87 | 78.33 | 99.00 | 2.15 | 2.03 | 4.00 | 5.72 | 5.77 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 7.00 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles main #4 ws 1: chrome, ApplicationFrameHost, ApplicationFrameHost, explorer, Notepad, WindowsTerminal, Everything
label=main #4 scenario=move target=chrome bursts=10 load=0 reports=9 wall=12.3s
  command p50 ms                          22.51
  command p90 ms                          25.56
  frames                                  63.00
  tick ms/frame                            0.94
  frame p90 ms                             1.32
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.41
  ovl_region ms/frame                      0.01
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            7.70
  cloak ms (outside frames)               11.50
  session_begin ms (outside frames)        1.00
  input->anim start ms                     0.20
  input->first frame ms                   28.08
| main #4 | move | 22.51 | 25.56 | 63.00 | 0.94 | 1.32 | 1.00 | 5.71 | 5.75 | 0.00 | 0.02 | 0.41 | 0.01 | 1.00 | 7.70 | 11.50 | 1.00 | 0.20 | 28.08 |
label=main #4 scenario=float target=chrome bursts=10 load=0 reports=9 wall=12.9s
  command p50 ms                          77.56
  command p90 ms                          89.45
  frames                                  62.00
  tick ms/frame                            4.11
  frame p90 ms                             7.58
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          9.22
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.03
  batch_commit ms/frame                    1.45
  ovl_region ms/frame                      0.15
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            8.70
  cloak ms (outside frames)               33.30
  session_begin ms (outside frames)        3.00
  input->anim start ms                     0.22
  input->first frame ms                   77.65
| main #4 | float | 77.56 | 89.45 | 62.00 | 4.11 | 7.58 | 4.00 | 5.72 | 9.22 | 0.00 | 0.03 | 1.45 | 0.15 | 3.00 | 8.70 | 33.30 | 3.00 | 0.22 | 77.65 |
label=main #4 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          48.38
  command p90 ms                          58.96
  frames                                 136.00
  tick ms/frame                            1.83
  frame p90 ms                             4.64
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          5.78
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.76
  ovl_region ms/frame                      0.06
  dwm_flush calls (in frames)              5.00
  dwm_flush ms (outside frames)           12.00
  cloak ms (outside frames)               37.90
  session_begin ms (outside frames)        5.00
  input->anim start ms                     4.18
  input->first frame ms                  105.46
| main #4 | minimize | 48.38 | 58.96 | 136.00 | 1.83 | 4.64 | 5.00 | 5.72 | 5.78 | 0.00 | 0.02 | 0.76 | 0.06 | 5.00 | 12.00 | 37.90 | 5.00 | 4.18 | 105.46 |
label=main #4 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.8s
  command p50 ms                          68.83
  command p90 ms                          95.62
  frames                                  60.00
  tick ms/frame                            4.92
  frame p90 ms                            10.64
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                         11.45
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.04
  batch_commit ms/frame                    2.07
  ovl_region ms/frame                      0.15
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            6.40
  cloak ms (outside frames)               34.60
  session_begin ms (outside frames)        2.90
  input->anim start ms                     0.23
  input->first frame ms                   81.54
| main #4 | resize | 68.83 | 95.62 | 60.00 | 4.92 | 10.64 | 5.00 | 5.72 | 11.45 | 0.00 | 0.04 | 2.07 | 0.15 | 3.00 | 6.40 | 34.60 | 2.90 | 0.23 | 81.54 |
Moved 3 window(s) to workspace '2' to switch to.
label=main #4 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          48.63
  command p90 ms                          53.65
  frames                                  97.00
  tick ms/frame                            2.28
  frame p90 ms                             3.49
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.76
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)           15.60
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| main #4 | switch | 48.63 | 53.65 | 97.00 | 2.28 | 3.49 | 3.00 | 5.71 | 5.76 | 0.00 | 0.00 | 0.76 | 0.04 | 3.00 | 15.60 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles nopeeks #4 ws 1: ApplicationFrameHost, ApplicationFrameHost, Notepad, Everything, WindowsTerminal, explorer, chrome
label=nopeeks #4 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          17.95
  command p90 ms                          18.83
  frames                                  63.00
  tick ms/frame                            0.54
  frame p90 ms                             0.95
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.25
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            6.80
  cloak ms (outside frames)               10.90
  session_begin ms (outside frames)        1.00
  input->anim start ms                     0.30
  input->first frame ms                   18.81
| nopeeks #4 | move | 17.95 | 18.83 | 63.00 | 0.54 | 0.95 | 0.00 | 5.71 | 5.75 | 0.00 | 0.02 | 0.25 | 0.02 | 0.00 | 6.80 | 10.90 | 1.00 | 0.30 | 18.81 |
label=nopeeks #4 scenario=float target=chrome bursts=10 load=0 reports=10 wall=13.0s
  command p50 ms                         101.31
  command p90 ms                         109.65
  frames                                  60.00
  tick ms/frame                            3.46
  frame p90 ms                             8.13
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          9.24
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.06
  batch_commit ms/frame                    2.27
  ovl_region ms/frame                      0.12
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           11.40
  cloak ms (outside frames)               31.60
  session_begin ms (outside frames)        2.50
  input->anim start ms                     0.29
  input->first frame ms                  111.06
| nopeeks #4 | float | 101.31 | 109.65 | 60.00 | 3.46 | 8.13 | 4.00 | 5.72 | 9.24 | 0.00 | 0.06 | 2.27 | 0.12 | 0.00 | 11.40 | 31.60 | 2.50 | 0.29 | 111.06 |
label=nopeeks #4 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          52.74
  command p90 ms                          67.05
  frames                                 137.00
  tick ms/frame                            1.60
  frame p90 ms                             5.26
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          5.82
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.76
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            8.20
  cloak ms (outside frames)               28.80
  session_begin ms (outside frames)        2.80
  input->anim start ms                     3.51
  input->first frame ms                  122.36
| nopeeks #4 | minimize | 52.74 | 67.05 | 137.00 | 1.60 | 5.26 | 5.00 | 5.72 | 5.82 | 0.00 | 0.02 | 0.76 | 0.04 | 1.00 | 8.20 | 28.80 | 2.80 | 3.51 | 122.36 |
label=nopeeks #4 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=13.0s
  command p50 ms                         106.40
  command p90 ms                         112.81
  frames                                  59.00
  tick ms/frame                            3.63
  frame p90 ms                             9.58
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          9.66
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.06
  batch_commit ms/frame                    2.33
  ovl_region ms/frame                      0.08
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           11.90
  cloak ms (outside frames)               36.40
  session_begin ms (outside frames)        2.60
  input->anim start ms                     0.30
  input->first frame ms                  117.72
| nopeeks #4 | resize | 106.40 | 112.81 | 59.00 | 3.63 | 9.58 | 4.00 | 5.72 | 9.66 | 0.00 | 0.06 | 2.33 | 0.08 | 0.00 | 11.90 | 36.40 | 2.60 | 0.30 | 117.72 |
Moved 3 window(s) to workspace '2' to switch to.
label=nopeeks #4 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          60.20
  command p90 ms                          67.66
  frames                                  99.00
  tick ms/frame                            1.74
  frame p90 ms                             1.67
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            9.90
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| nopeeks #4 | switch | 60.20 | 67.66 | 99.00 | 1.74 | 1.67 | 3.00 | 5.71 | 5.77 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 9.90 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles default #4 ws 1: chrome, ApplicationFrameHost, WindowsTerminal, Notepad, explorer, ApplicationFrameHost, Everything
label=default #4 scenario=move target=chrome bursts=10 load=0 reports=9 wall=12.3s
  command p50 ms                          26.13
  command p90 ms                          29.14
  frames                                  63.00
  tick ms/frame                            1.03
  frame p90 ms                             1.14
  frames >2x budget                        1.00
  interval p50 ms                          5.72
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.48
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            9.10
  cloak ms (outside frames)               13.10
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #4 | move | 26.13 | 29.14 | 63.00 | 1.03 | 1.14 | 1.00 | 5.72 | 5.77 | 0.48 | 0.00 | 0.00 | 0.00 | 0.00 | 9.10 | 13.10 | 0.00 | 0.00 | 0.00 |
label=default #4 scenario=float target=chrome bursts=10 load=0 reports=10 wall=13.0s
  command p50 ms                         100.26
  command p90 ms                         118.15
  frames                                  59.00
  tick ms/frame                            4.49
  frame p90 ms                             9.24
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          9.53
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.05
  batch_commit ms/frame                    1.91
  ovl_region ms/frame                      0.12
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            7.70
  cloak ms (outside frames)               49.60
  session_begin ms (outside frames)        2.60
  input->anim start ms                     0.35
  input->first frame ms                  114.76
| default #4 | float | 100.26 | 118.15 | 59.00 | 4.49 | 9.24 | 4.00 | 5.72 | 9.53 | 0.00 | 0.05 | 1.91 | 0.12 | 0.00 | 7.70 | 49.60 | 2.60 | 0.35 | 114.76 |
label=default #4 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          63.11
  command p90 ms                          69.55
  frames                                 134.00
  tick ms/frame                            1.86
  frame p90 ms                             3.83
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          5.79
  peek_frame ms/frame                      1.29
  session_overlays ms/frame                0.02
  batch_commit ms/frame                    0.63
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)           12.20
  cloak ms (outside frames)               32.10
  session_begin ms (outside frames)        2.50
  input->anim start ms                    38.96
  input->first frame ms                  206.17
| default #4 | minimize | 63.11 | 69.55 | 134.00 | 1.86 | 3.83 | 5.00 | 5.72 | 5.79 | 1.29 | 0.02 | 0.63 | 0.05 | 1.00 | 12.20 | 32.10 | 2.50 | 38.96 | 206.17 |
label=default #4 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=13.4s
  command p50 ms                         136.18
  command p90 ms                         143.41
  frames                                  57.00
  tick ms/frame                            5.43
  frame p90 ms                             4.52
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          6.33
  peek_frame ms/frame                      3.08
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.06
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           11.80
  cloak ms (outside frames)               37.20
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #4 | resize | 136.18 | 143.41 | 57.00 | 5.43 | 4.52 | 5.00 | 5.72 | 6.33 | 3.08 | 0.00 | 0.00 | 0.06 | 2.00 | 11.80 | 37.20 | 0.00 | 0.00 | 0.00 |
Moved 3 window(s) to workspace '2' to switch to.
label=default #4 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          72.16
  command p90 ms                          75.86
  frames                                  99.00
  tick ms/frame                            1.73
  frame p90 ms                             1.60
  frames >2x budget                        4.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            9.80
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| default #4 | switch | 72.16 | 75.86 | 99.00 | 1.73 | 1.60 | 4.00 | 5.71 | 5.76 | 0.00 | 0.00 | 0.00 | 0.04 | 0.00 | 9.80 | 0.00 | 0.00 | 0.00 | 0.00 |
tiles morepeeks #4 ws 1: ApplicationFrameHost, WindowsTerminal, explorer, Everything, ApplicationFrameHost, chrome, Notepad
label=morepeeks #4 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          32.15
  command p90 ms                          34.78
  frames                                  64.00
  tick ms/frame                            1.44
  frame p90 ms                             2.93
  frames >2x budget                        1.00
  interval p50 ms                          5.72
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.54
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            9.20
  cloak ms (outside frames)               12.80
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #4 | move | 32.15 | 34.78 | 64.00 | 1.44 | 2.93 | 1.00 | 5.72 | 5.77 | 0.54 | 0.00 | 0.00 | 0.00 | 0.00 | 9.20 | 12.80 | 0.00 | 0.00 | 0.00 |
label=morepeeks #4 scenario=float target=chrome bursts=10 load=0 reports=9 wall=14.0s
  command p50 ms                         126.46
  command p90 ms                         140.28
  frames                                  56.00
  tick ms/frame                            5.14
  frame p90 ms                             3.86
  frames >2x budget                        5.00
  interval p50 ms                          5.71
  interval p90 ms                          5.80
  peek_frame ms/frame                      3.05
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            9.90
  cloak ms (outside frames)               45.70
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #4 | float | 126.46 | 140.28 | 56.00 | 5.14 | 3.86 | 5.00 | 5.71 | 5.80 | 3.05 | 0.00 | 0.00 | 0.04 | 2.00 | 9.90 | 45.70 | 0.00 | 0.00 | 0.00 |
label=morepeeks #4 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.8s
  command p50 ms                         119.25
  command p90 ms                         132.91
  frames                                  58.00
  tick ms/frame                            4.85
  frame p90 ms                             3.82
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          5.99
  peek_frame ms/frame                      2.90
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           25.90
  cloak ms (outside frames)               46.20
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #4 | minimize | 119.25 | 132.91 | 58.00 | 4.85 | 3.82 | 5.00 | 5.72 | 5.99 | 2.90 | 0.00 | 0.00 | 0.04 | 2.00 | 25.90 | 46.20 | 0.00 | 0.00 | 0.00 |
label=morepeeks #4 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=13.3s
  command p50 ms                         120.53
  command p90 ms                         134.82
  frames                                  57.00
  tick ms/frame                            5.21
  frame p90 ms                             3.88
  frames >2x budget                        5.00
  interval p50 ms                          5.71
  interval p90 ms                          6.83
  peek_frame ms/frame                      3.19
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.06
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            8.60
  cloak ms (outside frames)               33.30
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #4 | resize | 120.53 | 134.82 | 57.00 | 5.21 | 3.88 | 5.00 | 5.71 | 6.83 | 3.19 | 0.00 | 0.00 | 0.06 | 2.00 | 8.60 | 33.30 | 0.00 | 0.00 | 0.00 |
Moved 3 window(s) to workspace '2' to switch to.
label=morepeeks #4 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.6s
  command p50 ms                          53.59
  command p90 ms                          57.73
  frames                                  99.00
  tick ms/frame                            1.87
  frame p90 ms                             1.78
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.77
  peek_frame ms/frame                      0.00
  session_overlays ms/frame                0.00
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)            8.20
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| morepeeks #4 | switch | 53.59 | 57.73 | 99.00 | 1.87 | 1.78 | 3.00 | 5.72 | 5.77 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 8.20 | 0.00 | 0.00 | 0.00 | 0.00 |

## move

| label | scenario | command p50 ms | command p90 ms | frames | tick ms/frame | frame p90 ms | frames >2x budget | interval p50 ms | interval p90 ms | peek_frame ms/frame | session_overlays ms/frame | batch_commit ms/frame | ovl_region ms/frame | dwm_flush calls (in frames) | dwm_flush ms (outside frames) | cloak ms (outside frames) | session_begin ms (outside frames) | input->anim start ms | input->first frame ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| main #1 | move | 37.24 | 39.30 | 63.00 | 1.24 | 1.08 | 1.00 | 5.71 | 5.77 | 0.00 | 0.03 | 0.48 | 0.06 | 1.00 | 10.40 | 16.00 | 1.00 | 0.21 | 46.70 |
| main #2 | move | 22.30 | 24.33 | 63.00 | 0.88 | 1.16 | 1.00 | 5.71 | 5.75 | 0.00 | 0.03 | 0.39 | 0.00 | 1.00 | 7.10 | 10.60 | 1.00 | 0.20 | 27.47 |
| main #3 | move | 22.99 | 24.84 | 64.00 | 0.94 | 1.05 | 1.00 | 5.71 | 5.75 | 0.00 | 0.02 | 0.37 | 0.01 | 1.00 | 8.10 | 11.80 | 1.00 | 0.20 | 25.11 |
| main #4 | move | 22.51 | 25.56 | 63.00 | 0.94 | 1.32 | 1.00 | 5.71 | 5.75 | 0.00 | 0.02 | 0.41 | 0.01 | 1.00 | 7.70 | 11.50 | 1.00 | 0.20 | 28.08 |
| nopeeks #1 | move | 17.72 | 18.89 | 63.00 | 0.55 | 0.96 | 0.00 | 5.71 | 5.75 | 0.00 | 0.02 | 0.26 | 0.01 | 0.00 | 6.60 | 11.00 | 1.00 | 0.29 | 18.32 |
| nopeeks #2 | move | 18.37 | 18.74 | 63.00 | 0.53 | 0.93 | 0.00 | 5.71 | 5.75 | 0.00 | 0.02 | 0.25 | 0.02 | 0.00 | 7.10 | 11.50 | 0.90 | 0.30 | 18.85 |
| nopeeks #3 | move | 17.50 | 18.89 | 63.00 | 0.53 | 0.94 | 0.00 | 5.71 | 5.75 | 0.00 | 0.02 | 0.25 | 0.01 | 0.00 | 6.00 | 10.20 | 0.90 | 0.27 | 18.04 |
| nopeeks #4 | move | 17.95 | 18.83 | 63.00 | 0.54 | 0.95 | 0.00 | 5.71 | 5.75 | 0.00 | 0.02 | 0.25 | 0.02 | 0.00 | 6.80 | 10.90 | 1.00 | 0.30 | 18.81 |
| default #1 | move | 37.82 | 39.65 | 64.00 | 1.25 | 1.26 | 1.00 | 5.71 | 5.76 | 0.52 | 0.00 | 0.00 | 0.00 | 0.00 | 11.10 | 17.60 | 0.00 | 0.00 | 0.00 |
| default #2 | move | 26.46 | 27.07 | 63.00 | 1.08 | 1.18 | 2.00 | 5.71 | 5.77 | 0.49 | 0.00 | 0.00 | 0.00 | 0.00 | 8.30 | 12.30 | 0.00 | 0.00 | 0.00 |
| default #3 | move | 37.75 | 40.24 | 63.00 | 1.25 | 1.18 | 1.00 | 5.71 | 5.77 | 0.51 | 0.00 | 0.00 | 0.00 | 0.00 | 11.10 | 17.60 | 0.00 | 0.00 | 0.00 |
| default #4 | move | 26.13 | 29.14 | 63.00 | 1.03 | 1.14 | 1.00 | 5.72 | 5.77 | 0.48 | 0.00 | 0.00 | 0.00 | 0.00 | 9.10 | 13.10 | 0.00 | 0.00 | 0.00 |
| morepeeks #1 | move | 23.27 | 24.61 | 63.00 | 0.83 | 0.92 | 1.00 | 5.71 | 5.76 | 0.38 | 0.00 | 0.00 | 0.00 | 0.00 | 8.60 | 11.50 | 0.00 | 0.00 | 0.00 |
| morepeeks #2 | move | 22.63 | 25.77 | 63.00 | 0.84 | 0.91 | 1.00 | 5.71 | 5.75 | 0.37 | 0.00 | 0.00 | 0.00 | 0.00 | 8.00 | 10.80 | 0.00 | 0.00 | 0.00 |
| morepeeks #3 | move | 23.88 | 25.42 | 63.00 | 0.84 | 0.93 | 1.00 | 5.71 | 5.75 | 0.38 | 0.00 | 0.00 | 0.00 | 0.00 | 8.10 | 11.10 | 0.00 | 0.00 | 0.00 |
| morepeeks #4 | move | 32.15 | 34.78 | 64.00 | 1.44 | 2.93 | 1.00 | 5.72 | 5.77 | 0.54 | 0.00 | 0.00 | 0.00 | 0.00 | 9.20 | 12.80 | 0.00 | 0.00 | 0.00 |

## float

| label | scenario | command p50 ms | command p90 ms | frames | tick ms/frame | frame p90 ms | frames >2x budget | interval p50 ms | interval p90 ms | peek_frame ms/frame | session_overlays ms/frame | batch_commit ms/frame | ovl_region ms/frame | dwm_flush calls (in frames) | dwm_flush ms (outside frames) | cloak ms (outside frames) | session_begin ms (outside frames) | input->anim start ms | input->first frame ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| main #1 | float | 77.37 | 84.65 | 60.00 | 3.59 | 6.48 | 4.00 | 5.72 | 7.81 | 0.00 | 0.03 | 1.25 | 0.06 | 3.00 | 10.10 | 30.10 | 2.70 | 0.23 | 77.58 |
| main #2 | float | 75.65 | 83.22 | 60.00 | 4.16 | 9.14 | 4.00 | 5.72 | 9.77 | 0.00 | 0.04 | 1.65 | 0.15 | 3.00 | 6.40 | 32.40 | 2.90 | 0.22 | 94.29 |
| main #3 | float | 75.95 | 83.82 | 63.00 | 4.14 | 9.45 | 4.00 | 5.72 | 9.78 | 0.00 | 0.04 | 1.92 | 0.14 | 3.00 | 6.10 | 30.10 | 2.60 | 0.23 | 109.29 |
| main #4 | float | 77.56 | 89.45 | 62.00 | 4.11 | 7.58 | 4.00 | 5.72 | 9.22 | 0.00 | 0.03 | 1.45 | 0.15 | 3.00 | 8.70 | 33.30 | 3.00 | 0.22 | 77.65 |
| nopeeks #1 | float | 78.51 | 107.29 | 60.00 | 3.62 | 7.66 | 5.00 | 5.72 | 8.55 | 0.00 | 0.06 | 1.45 | 0.12 | 0.00 | 9.60 | 29.20 | 2.60 | 0.28 | 92.11 |
| nopeeks #2 | float | 99.07 | 113.86 | 59.00 | 3.56 | 9.95 | 4.00 | 5.72 | 10.17 | 0.00 | 0.06 | 2.35 | 0.12 | 0.00 | 9.00 | 30.00 | 2.60 | 0.28 | 121.83 |
| nopeeks #3 | float | 99.70 | 109.27 | 59.00 | 3.74 | 9.77 | 4.00 | 5.72 | 10.05 | 0.00 | 0.05 | 2.25 | 0.12 | 0.00 | 14.40 | 28.80 | 2.50 | 0.29 | 113.44 |
| nopeeks #4 | float | 101.31 | 109.65 | 60.00 | 3.46 | 8.13 | 4.00 | 5.72 | 9.24 | 0.00 | 0.06 | 2.27 | 0.12 | 0.00 | 11.40 | 31.60 | 2.50 | 0.29 | 111.06 |
| default #1 | float | 107.76 | 118.00 | 60.00 | 4.40 | 10.36 | 5.00 | 5.72 | 10.53 | 0.00 | 0.05 | 2.43 | 0.14 | 0.00 | 12.30 | 36.50 | 2.70 | 0.29 | 138.78 |
| default #2 | float | 97.29 | 117.72 | 60.00 | 3.69 | 5.42 | 4.00 | 5.72 | 6.99 | 0.00 | 0.05 | 1.44 | 0.08 | 0.00 | 6.40 | 33.70 | 2.80 | 0.33 | 88.23 |
| default #3 | float | 106.07 | 119.69 | 60.00 | 4.82 | 10.64 | 5.00 | 5.72 | 10.75 | 0.00 | 0.04 | 2.50 | 0.16 | 0.00 | 13.20 | 33.60 | 2.70 | 0.30 | 131.12 |
| default #4 | float | 100.26 | 118.15 | 59.00 | 4.49 | 9.24 | 4.00 | 5.72 | 9.53 | 0.00 | 0.05 | 1.91 | 0.12 | 0.00 | 7.70 | 49.60 | 2.60 | 0.35 | 114.76 |
| morepeeks #1 | float | 129.65 | 136.45 | 59.00 | 5.02 | 4.14 | 5.00 | 5.72 | 6.21 | 3.02 | 0.00 | 0.00 | 0.05 | 2.00 | 7.30 | 29.60 | 0.00 | 0.00 | 0.00 |
| morepeeks #2 | float | 127.48 | 138.51 | 57.00 | 5.42 | 3.96 | 5.00 | 5.72 | 7.08 | 3.13 | 0.00 | 0.00 | 0.05 | 2.00 | 8.20 | 45.40 | 0.00 | 0.00 | 0.00 |
| morepeeks #3 | float | 130.37 | 136.58 | 57.00 | 4.95 | 4.51 | 5.00 | 5.72 | 5.98 | 3.11 | 0.00 | 0.00 | 0.05 | 2.00 | 9.40 | 30.70 | 0.00 | 0.00 | 0.00 |
| morepeeks #4 | float | 126.46 | 140.28 | 56.00 | 5.14 | 3.86 | 5.00 | 5.71 | 5.80 | 3.05 | 0.00 | 0.00 | 0.04 | 2.00 | 9.90 | 45.70 | 0.00 | 0.00 | 0.00 |

## minimize

| label | scenario | command p50 ms | command p90 ms | frames | tick ms/frame | frame p90 ms | frames >2x budget | interval p50 ms | interval p90 ms | peek_frame ms/frame | session_overlays ms/frame | batch_commit ms/frame | ovl_region ms/frame | dwm_flush calls (in frames) | dwm_flush ms (outside frames) | cloak ms (outside frames) | session_begin ms (outside frames) | input->anim start ms | input->first frame ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| main #1 | minimize | 47.05 | 56.44 | 136.00 | 1.70 | 4.48 | 6.00 | 5.72 | 5.79 | 0.00 | 0.02 | 0.95 | 0.05 | 5.00 | 8.50 | 35.30 | 5.90 | 3.92 | 131.58 |
| main #2 | minimize | 46.80 | 64.43 | 131.00 | 1.95 | 3.47 | 5.00 | 5.71 | 5.79 | 0.00 | 0.02 | 0.82 | 0.06 | 5.00 | 9.30 | 41.10 | 6.30 | 4.49 | 129.99 |
| main #3 | minimize | 26.50 | 58.52 | 137.00 | 1.91 | 4.43 | 6.00 | 5.72 | 5.80 | 0.00 | 0.02 | 0.77 | 0.06 | 5.00 | 9.70 | 34.90 | 5.40 | 4.30 | 102.40 |
| main #4 | minimize | 48.38 | 58.96 | 136.00 | 1.83 | 4.64 | 5.00 | 5.72 | 5.78 | 0.00 | 0.02 | 0.76 | 0.06 | 5.00 | 12.00 | 37.90 | 5.00 | 4.18 | 105.46 |
| nopeeks #1 | minimize | 36.71 | 63.66 | 136.00 | 1.52 | 4.58 | 5.00 | 5.71 | 5.81 | 0.00 | 0.03 | 0.72 | 0.04 | 1.00 | 8.50 | 28.70 | 2.90 | 3.28 | 109.84 |
| nopeeks #2 | minimize | 56.54 | 66.85 | 138.00 | 1.58 | 5.17 | 5.00 | 5.72 | 5.80 | 0.00 | 0.02 | 0.76 | 0.05 | 1.00 | 9.20 | 40.40 | 3.20 | 3.48 | 113.22 |
| nopeeks #3 | minimize | 60.25 | 71.04 | 135.00 | 1.60 | 6.07 | 5.00 | 5.72 | 6.38 | 0.00 | 0.02 | 0.91 | 0.04 | 1.00 | 14.50 | 37.20 | 3.10 | 3.56 | 128.40 |
| nopeeks #4 | minimize | 52.74 | 67.05 | 137.00 | 1.60 | 5.26 | 5.00 | 5.72 | 5.82 | 0.00 | 0.02 | 0.76 | 0.04 | 1.00 | 8.20 | 28.80 | 2.80 | 3.51 | 122.36 |
| default #1 | minimize | 69.08 | 81.74 | 139.00 | 1.79 | 3.53 | 5.00 | 5.72 | 5.78 | 0.85 | 0.01 | 0.54 | 0.03 | 1.00 | 13.50 | 39.20 | 2.50 | 3.09 | 96.15 |
| default #2 | minimize | 67.12 | 81.22 | 132.00 | 1.83 | 4.19 | 5.00 | 5.72 | 5.79 | 0.71 | 0.02 | 0.44 | 0.03 | 1.00 | 11.70 | 47.90 | 2.60 | 0.62 | 96.02 |
| default #3 | minimize | 67.41 | 77.98 | 135.00 | 1.79 | 3.88 | 5.00 | 5.72 | 5.78 | 1.27 | 0.02 | 0.62 | 0.05 | 1.00 | 17.20 | 40.20 | 2.40 | 37.15 | 201.66 |
| default #4 | minimize | 63.11 | 69.55 | 134.00 | 1.86 | 3.83 | 5.00 | 5.72 | 5.79 | 1.29 | 0.02 | 0.63 | 0.05 | 1.00 | 12.20 | 32.10 | 2.50 | 38.96 | 206.17 |
| morepeeks #1 | minimize | 65.69 | 73.54 | 135.00 | 2.21 | 3.92 | 5.00 | 5.72 | 5.78 | 0.00 | 0.02 | 0.55 | 0.04 | 1.00 | 7.00 | 31.10 | 2.90 | 4.18 | 115.78 |
| morepeeks #2 | minimize | 68.77 | 77.94 | 132.00 | 2.31 | 4.14 | 6.00 | 5.71 | 5.78 | 0.71 | 0.02 | 0.45 | 0.03 | 1.00 | 13.90 | 36.30 | 2.70 | 2.50 | 88.87 |
| morepeeks #3 | minimize | 73.31 | 77.16 | 135.00 | 2.17 | 4.03 | 6.00 | 5.72 | 5.77 | 1.09 | 0.02 | 0.55 | 0.06 | 1.00 | 13.80 | 35.90 | 2.50 | 4.27 | 170.02 |
| morepeeks #4 | minimize | 119.25 | 132.91 | 58.00 | 4.85 | 3.82 | 5.00 | 5.72 | 5.99 | 2.90 | 0.00 | 0.00 | 0.04 | 2.00 | 25.90 | 46.20 | 0.00 | 0.00 | 0.00 |

## resize

| label | scenario | command p50 ms | command p90 ms | frames | tick ms/frame | frame p90 ms | frames >2x budget | interval p50 ms | interval p90 ms | peek_frame ms/frame | session_overlays ms/frame | batch_commit ms/frame | ovl_region ms/frame | dwm_flush calls (in frames) | dwm_flush ms (outside frames) | cloak ms (outside frames) | session_begin ms (outside frames) | input->anim start ms | input->first frame ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| main #1 | resize | 75.34 | 84.66 | 57.00 | 4.15 | 8.75 | 4.00 | 5.72 | 8.86 | 0.00 | 0.04 | 2.15 | 0.07 | 3.00 | 8.20 | 25.10 | 2.70 | 0.22 | 79.08 |
| main #2 | resize | 81.82 | 85.31 | 59.00 | 4.61 | 9.71 | 4.00 | 5.72 | 10.13 | 0.00 | 0.04 | 2.07 | 0.09 | 3.00 | 10.90 | 39.10 | 2.60 | 0.22 | 102.09 |
| main #3 | resize | 52.68 | 55.38 | 134.00 | 3.15 | 5.45 | 5.00 | 5.72 | 5.78 | 0.00 | 0.04 | 1.22 | 0.06 | 4.00 | 6.80 | 23.20 | 2.70 | 0.21 | 73.81 |
| main #4 | resize | 68.83 | 95.62 | 60.00 | 4.92 | 10.64 | 5.00 | 5.72 | 11.45 | 0.00 | 0.04 | 2.07 | 0.15 | 3.00 | 6.40 | 34.60 | 2.90 | 0.23 | 81.54 |
| nopeeks #1 | resize | 86.29 | 112.74 | 58.00 | 3.43 | 7.39 | 4.00 | 5.72 | 9.36 | 0.00 | 0.05 | 2.10 | 0.06 | 0.00 | 12.40 | 32.00 | 2.70 | 0.30 | 93.85 |
| nopeeks #2 | resize | 94.68 | 108.04 | 58.00 | 3.76 | 8.15 | 4.00 | 5.72 | 8.22 | 0.00 | 0.06 | 2.27 | 0.09 | 0.00 | 11.30 | 33.40 | 2.60 | 0.32 | 105.16 |
| nopeeks #3 | resize | 96.54 | 109.71 | 56.00 | 3.89 | 9.55 | 5.00 | 5.72 | 9.62 | 0.00 | 0.06 | 2.39 | 0.10 | 0.00 | 12.70 | 32.20 | 2.70 | 0.31 | 116.18 |
| nopeeks #4 | resize | 106.40 | 112.81 | 59.00 | 3.63 | 9.58 | 4.00 | 5.72 | 9.66 | 0.00 | 0.06 | 2.33 | 0.08 | 0.00 | 11.90 | 36.40 | 2.60 | 0.30 | 117.72 |
| default #1 | resize | 135.94 | 148.23 | 57.00 | 5.18 | 4.44 | 4.00 | 5.71 | 7.10 | 3.25 | 0.00 | 0.00 | 0.08 | 2.00 | 17.00 | 39.70 | 0.00 | 0.00 | 0.00 |
| default #2 | resize | 130.96 | 138.06 | 56.00 | 4.90 | 4.38 | 5.00 | 5.72 | 5.94 | 3.02 | 0.00 | 0.00 | 0.07 | 1.00 | 8.50 | 36.70 | 0.00 | 0.00 | 0.00 |
| default #3 | resize | 116.75 | 138.02 | 59.00 | 4.99 | 4.51 | 5.00 | 5.71 | 6.16 | 3.15 | 0.00 | 0.00 | 0.07 | 2.00 | 7.30 | 28.10 | 0.00 | 0.00 | 0.00 |
| default #4 | resize | 136.18 | 143.41 | 57.00 | 5.43 | 4.52 | 5.00 | 5.72 | 6.33 | 3.08 | 0.00 | 0.00 | 0.06 | 2.00 | 11.80 | 37.20 | 0.00 | 0.00 | 0.00 |
| morepeeks #1 | resize | 133.86 | 143.17 | 59.00 | 5.17 | 4.25 | 5.00 | 5.72 | 6.15 | 3.24 | 0.00 | 0.00 | 0.06 | 2.00 | 6.70 | 41.30 | 0.00 | 0.00 | 0.00 |
| morepeeks #2 | resize | 131.04 | 140.74 | 56.00 | 5.39 | 4.39 | 5.00 | 5.72 | 6.47 | 3.25 | 0.00 | 0.00 | 0.07 | 2.00 | 11.40 | 39.20 | 0.00 | 0.00 | 0.00 |
| morepeeks #3 | resize | 111.52 | 139.77 | 57.00 | 5.26 | 4.40 | 5.00 | 5.72 | 6.38 | 3.16 | 0.00 | 0.00 | 0.07 | 2.00 | 12.30 | 39.60 | 0.00 | 0.00 | 0.00 |
| morepeeks #4 | resize | 120.53 | 134.82 | 57.00 | 5.21 | 3.88 | 5.00 | 5.71 | 6.83 | 3.19 | 0.00 | 0.00 | 0.06 | 2.00 | 8.60 | 33.30 | 0.00 | 0.00 | 0.00 |

## switch

| label | scenario | command p50 ms | command p90 ms | frames | tick ms/frame | frame p90 ms | frames >2x budget | interval p50 ms | interval p90 ms | peek_frame ms/frame | session_overlays ms/frame | batch_commit ms/frame | ovl_region ms/frame | dwm_flush calls (in frames) | dwm_flush ms (outside frames) | cloak ms (outside frames) | session_begin ms (outside frames) | input->anim start ms | input->first frame ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| main #1 | switch | 37.70 | 44.18 | 97.00 | 2.18 | 3.39 | 2.00 | 5.71 | 5.75 | 0.00 | 0.00 | 0.70 | 0.07 | 3.00 | 14.80 | 0.00 | 0.00 | 0.00 | 0.00 |
| main #2 | switch | 51.59 | 61.77 | 96.00 | 2.42 | 3.68 | 3.00 | 5.71 | 5.76 | 0.00 | 0.00 | 0.76 | 0.03 | 3.00 | 12.30 | 0.00 | 0.00 | 0.00 | 0.00 |
| main #3 | switch | 43.70 | 115.59 | 97.00 | 2.33 | 3.62 | 3.00 | 5.72 | 5.75 | 0.00 | 0.00 | 0.69 | 0.05 | 3.00 | 12.10 | 0.00 | 0.00 | 0.00 | 0.00 |
| main #4 | switch | 48.63 | 53.65 | 97.00 | 2.28 | 3.49 | 3.00 | 5.71 | 5.76 | 0.00 | 0.00 | 0.76 | 0.04 | 3.00 | 15.60 | 0.00 | 0.00 | 0.00 | 0.00 |
| nopeeks #1 | switch | 65.25 | 78.83 | 99.00 | 1.87 | 1.83 | 3.00 | 5.71 | 5.77 | 0.00 | 0.00 | 0.00 | 0.04 | 0.00 | 7.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| nopeeks #2 | switch | 67.43 | 123.14 | 99.00 | 1.73 | 1.56 | 3.00 | 5.72 | 5.77 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 8.80 | 0.00 | 0.00 | 0.00 | 0.00 |
| nopeeks #3 | switch | 78.94 | 80.98 | 99.00 | 1.97 | 1.91 | 3.00 | 5.71 | 5.77 | 0.00 | 0.00 | 0.00 | 0.04 | 0.00 | 9.70 | 0.00 | 0.00 | 0.00 | 0.00 |
| nopeeks #4 | switch | 60.20 | 67.66 | 99.00 | 1.74 | 1.67 | 3.00 | 5.71 | 5.77 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 9.90 | 0.00 | 0.00 | 0.00 | 0.00 |
| default #1 | switch | 65.61 | 76.48 | 99.00 | 1.65 | 1.69 | 3.00 | 5.71 | 5.77 | 0.00 | 0.00 | 0.00 | 0.02 | 0.00 | 7.80 | 0.00 | 0.00 | 0.00 | 0.00 |
| default #2 | switch | 74.29 | 79.91 | 99.00 | 1.69 | 1.83 | 3.00 | 5.72 | 5.77 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 16.70 | 0.00 | 0.00 | 0.00 | 0.00 |
| default #3 | switch | 72.80 | 77.25 | 99.00 | 1.66 | 1.66 | 3.00 | 5.72 | 5.78 | 0.00 | 0.00 | 0.00 | 0.02 | 0.00 | 11.40 | 0.00 | 0.00 | 0.00 | 0.00 |
| default #4 | switch | 72.16 | 75.86 | 99.00 | 1.73 | 1.60 | 4.00 | 5.71 | 5.76 | 0.00 | 0.00 | 0.00 | 0.04 | 0.00 | 9.80 | 0.00 | 0.00 | 0.00 | 0.00 |
| morepeeks #1 | switch | 63.66 | 82.49 | 99.00 | 2.20 | 2.02 | 4.00 | 5.72 | 5.77 | 0.00 | 0.00 | 0.00 | 0.02 | 0.00 | 6.90 | 0.00 | 0.00 | 0.00 | 0.00 |
| morepeeks #2 | switch | 67.37 | 80.72 | 99.00 | 2.21 | 2.03 | 4.00 | 5.71 | 5.76 | 0.00 | 0.00 | 0.00 | 0.02 | 0.00 | 6.80 | 0.00 | 0.00 | 0.00 | 0.00 |
| morepeeks #3 | switch | 68.87 | 78.33 | 99.00 | 2.15 | 2.03 | 4.00 | 5.72 | 5.77 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 7.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| morepeeks #4 | switch | 53.59 | 57.73 | 99.00 | 1.87 | 1.78 | 3.00 | 5.72 | 5.77 | 0.00 | 0.00 | 0.00 | 0.03 | 0.00 | 8.20 | 0.00 | 0.00 | 0.00 | 0.00 |
```
