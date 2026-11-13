# Start-latency stages, desktop bench, 2026-11-13 afternoon

`tryout/start-latency` `479b7482` (`keepoverlays`, `asynchandback`,
`slidepeeks` on by default) vs `origin/main` `cfd3cef9` (`main`). Ryzen 7
3700X, RTX 3070, 3440×1440 @ 174 Hz. A measuring run: numbers only.

## Setup

- **Backdrops off** (owner's request): `backdrop.enabled: false` for focused
  and other windows in the user config and in `main`'s `baseline-config.yaml`.
  The morning ran with them on, so `peek_styles` and `peek_backdrop` read 0
  here, and `default` numbers don't compare with the morning's.
- 6 tiles on workspace 1 in every measured mode-round (tile line logged after
  each restart): Notepad, WindowsTerminal (this session's), WindowsTerminal
  (new), chrome (Helium, the target, lowest handle), chrome (new), FPilot.
  `-Launch` default list per run, `-TileOrder
  Notepad,WindowsTerminal,WindowsTerminal,chrome,chrome,FPilot`. Microsoft
  Store, Calculator and Clock moved to workspace 3 for the session. `switch`
  goes between workspace 1 and workspace 2 (Zed, 1 window).
- 4 rounds × 10 bursts, modes interleaved per round, WM restarted per mode.
  CPU 8 = 8 busy-loop jobs of 16 threads.
- `main`: `bench-run.ps1 -Modes main,nopeeks,default -Scenarios resize,switch`
  in `..\glazewm-baseline`, perf_bench built there from this branch's source.
  Each mode-round runs `resize` then `switch` in one WM session; its
  `nopeeks`/`default` rows are marked `(main run)`.
- `cargo test --workspace`: 788 passed, 1 ignored, 0 failed.
- One report per burst on both scenarios (40 per mode), so "per report" =
  "per burst" below. Stage values are medians over a mode's 40 reports.

## 1. Per scenario (median over rounds, load 0 / CPU 8, ms)

### resize

| mode | command p50 | input→first frame | frame p90 | interval p90 | interval p99 | interval max (median burst) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `nopeeks` | 30.52 / 42.85 | 34.98 / 51.93 | 3.79 / 5.82 | 5.78 / 6.42 | 14.51 / 27.35 | 14.51 / 27.35 |
| `nopeeks+nokeepoverlays` | 35.88 / 49.10 | 39.69 / 54.75 | 3.78 / 4.92 | 5.78 / 5.97 | 14.09 / 27.35 | 14.09 / 27.35 |
| `default` | 47.73 / 67.63 | 82.22 / 103.50 | 2.96 / 3.35 | 5.80 / 5.88 | 35.62 / 53.16 | 35.62 / 53.16 |
| `noasynchandback` | 45.49 / 66.91 | 77.58 / 96.44 | 2.79 / 3.39 | 5.80 / 5.90 | 47.69 / 59.59 | 47.69 / 59.59 |
| `main` | 33.25 / 33.17* | 36.78 / 39.06* | 4.33 / 4.62* | 5.79 / 5.77* | 37.25 / 30.07* | 37.25 / 30.07* |
| `nopeeks` (main run) | 35.49 / 33.89* | 39.75 / 41.84* | 4.70 / 4.26* | 5.80 / 5.78* | 18.52 / 17.08* | 18.52 / 17.08* |
| `default` (main run) | 54.98 / 53.53* | 87.85 / 85.42* | 3.25 / 3.51* | 5.84 / 5.83* | 44.17 / 36.82* | 44.17 / 36.82* |

### switch

input→first frame is not recorded on `switch` (perf_bench prints 0.00).

| mode | command p50 | frame p90 | interval p90 | interval p99 | interval max (median burst) |
| --- | ---: | ---: | ---: | ---: | ---: |
| `nopeeks` | 27.56 / 34.79 | 1.25 / 1.57 | 5.75 / 5.77 | 17.63 / 22.57 | 17.63 / 23.66 |
| `default` | 27.66 / 34.42 | 1.23 / 1.46 | 5.75 / 5.77 | 16.23 / 23.29 | 16.23 / 23.29 |
| `noslidepeeks` | 27.87 / 35.11 | 1.31 / 1.40 | 5.76 / 5.77 | 16.42 / 22.87 | 16.42 / 23.40 |
| `main` | 42.45 / 37.50* | 0.94 / 0.76* | 5.76 / 5.75* | 79.09 / 71.70* | 79.09 / 71.70* |
| `nopeeks` (main run) | 31.50 / 26.52* | 1.25 / 1.21* | 5.76 / 5.75* | 20.62 / 18.06* | 20.62 / 20.36* |
| `default` (main run) | 32.67 / 27.65* | 1.23 / 1.32* | 5.78 / 5.75* | 20.91 / 20.76* | 20.91 / 20.82* |

\* **The main run's CPU 8 is not a loaded reference.** The load shows only in
round 1: resize `main #1` 44.06 ms, `#2`–`#4` 32.34–33.65 (load 0:
32.30–34.58); `nopeeks` 30.17–34.95 (load 0: 33.03–35.69). The voided first
main CPU 8 run did the same after round 2 (`main` 41.1, 40.6, then 32.9,
33.1). The isolate runs stayed loaded all 4 rounds (`nopeeks` resize
42.24–44.98 vs 30.07–31.07). Cause not checked. Compare `main` at load 0.

`main` vs this branch, resize, load 0 (main run, `results/cmp-main.md`; "sep" =
rounds don't overlap): `default` command +65% sep, first frame +139% sep,
frame p90 −25% sep; `nopeeks` command +7% ~, first frame +8% sep, interval
p99 −50% sep. Switch, load 0: `nopeeks` command −26% sep, interval p99 −74% sep,
frame p90 +34% sep.

## 2. Opt-outs against the morning's result

**`nokeepoverlays`: confirmed, smaller.** vs `nopeeks`, resize:

| metric | `nopeeks` | `+nokeepoverlays` load 0 | CPU 8 |
| --- | ---: | --- | --- |
| command p50 ms | 30.52 / 42.85 | 35.88 (+18%, rounds overlap: one round 30.54) | 49.10 (+15%, sep) |
| command ms (WM side) | 30.15 / 42.45 | 35.55 (+18%, ~) | 48.70 (+15%, sep) |
| input→first frame ms | 34.98 / 51.93 | 39.69 (+13%, ~) | 54.75 (+5%, ~) |
| session_overlays ms (outside) | 7.10 / 9.85 | 10.80 (+52%, sep) | 15.15 (+54%, sep) |
| cloak ms (outside) | 18.25 / 25.75 | 21.15 (+16%, sep) | 26.90 (+4%, ~) |

Frame p90, interval p90/p99/max: no separation. Morning, backdrops on:
42.4 → 29.5 ms (−30%) the other way.

**`noasynchandback`: confirmed.** vs `default`, resize (`results/sync-rows.md`):

| | `default` load 0 | `noasynchandback` | `default` CPU 8 | `noasynchandback` |
| --- | ---: | ---: | ---: | ---: |
| reports with `[sync]` rows | 0 / 40 | 40 / 40 | 0 / 40 | 40 / 40 |
| `[sync]` rows, worst ms by app | none | WT 16, FPilot 16, chrome 8 (120 rows) | none | FPilot 25, chrome 23, WT 19 (120 rows) |
| interval max, median burst (perf_bench) | 35.62 | 47.69 (+34%, sep) | 53.16 | 59.59 (+12%, sep) |
| interval max, worst report | 44.1 | 66.4 | 69.7 | 99.3 |

Command, first frame, frame p90, interval p90: no separation. Notepad's
synchronous reposition (56–70 ms, `noasynchandback` only) came once per
restart during the tile reorder, before measuring. Notepad then stays async
for 30 s, so no measured burst has it (morning: first burst, 122 ms).

`noslidepeeks` vs `default`, switch: nothing separates (command +1% / +2%).
`nopeeks` vs `default`, switch: same.

## 3. Resize, `default`, from perf.log

Full reports, load 0, all 40 bursts (`perf-excerpts.txt`; appendix below):
`default #4` 12:11:10Z, the median interval max (35.13 ms), and `default #4`
12:11:12Z, the longest (44.14 ms). In both, the longest frame isn't the first
peek frame. It's a later tick whose `platform_sync` (worst 25.31 / 35.41 ms:
`rd_apply` 15.74 / 16.86, `rp_visible` 11.42 / 12.91; in-frame `dwm_flush`
worst 18.22 / 16.02) gives the frame time max 33.70 / 41.72 ms. The first frame's
tick took 25.58 / 27.32 ms.

### Second-round stages per burst (medians of 40; load 0 / CPU 8)

"Before answer" and "first `peek_frame`" are read from the slowest input's
timeline, which drops entries under 0.3 ms, so their calls count only
entries ≥ 0.3 ms. "Outside frames" is exact but also holds the syncs after
the command (7 `platform_sync` calls per burst, 1 `command`). `ws_*` don't
run on resize; see section 4.

| stage | before answer ms (calls ≥0.3) | in first `peek_frame` ms (calls ≥0.3) | outside frames ms (calls) |
| --- | --- | --- | --- |
| `peek_draw` | 0 / 0 | 29.01 (6) / 29.92 (6) | 0 / 0 |
| `peek_track` | 0 / 0 | ≤ 0.08 / ≤ 0.11 (0)¹ | 0 / 0 |
| `peek_ring` | 9.26 (8) / 12.25 (9) | 0 / 0 | 10.55 (40) / 13.60 (40) |
| `peek_backdrop` | 0 / 0 | 0 / 0 | 0.00 (40) / 0.00 (40) |
| `peek_styles` | 0 / 0 | 0 / 0 | 0.00 (6) / 0.00 (6) |
| `peek_warm` | 0 / 0 | 0 / 0 | 0.10 (6) / 0.10 (6) |
| `sg_window` | 3.60 (6) / 4.56 (6) | 0 / 0 | 3.60 (6) / 4.60 (6) |
| `sg_thumb` | 0 / 0 | 0 / 0 | 0.55 (6) / 0.60 (6) |
| `sync_focus` | 0 / 0 | 0 / 0 | 0.00 (7) / 0.00 (7) |
| `tab_bars` | 0 / 0 | 0 / 0 | 0.10 (7) / 0.10 (7) |
| `overview` | 0 / 0 | 0 / 0 | 0.00 (7) / 0.00 (7) |
| `guards` | 0.40 (1) / 0.52 (1) | 0 / 0 | 0.45 (7) / 0.60 (7) |

¹ Under the timeline's cut-off; bounded by the time from the last
`peek_draw`'s end to the frame's end. In the frame tree, per burst: `peek_track`
8.25 / 12.95 ms over 22–23 frames, `peek_ring` 7.9 of it at load 0.

In the frames (whole burst): `peek_draw` 69.8 ms / 118 calls at load 0, of
which the first frame's 6 draws are 29.01 ms (4.83 ms each, min 2.91, max
8.47) and the other 112 are 40.80 ms (0.36 ms each).

### First `peek_frame` (medians of 40, ms)

| | load 0 | CPU 8 | `default` (main run) load 0 |
| --- | ---: | ---: | ---: |
| command | 47.70 | 66.75 | 56.62 |
| input → first frame | 77.40 | 100.08 | 85.25 |
| `peek_frame` | 31.73 | 33.05 | 30.89 |
| wait: timer wake → WM thread's tick | 2.46 | 2.78 | 3.25 |
| `peek_draw` (6 draws) | 29.01 | 29.92 | 27.00 |
| `peek_track` | ≤ 0.08 | ≤ 0.11 | ≤ 0.09 |
| `peek_ring` / `peek_backdrop` in it | < 0.3 / < 0.3 | < 0.3 / < 0.3 | < 0.3 / < 0.3 |
| left over (`peek_frame` − draw − track) | 2.57 | 2.93 | 3.39 |

The timer wakes inside the command (typical burst: `peek_frame` starts at
36.80 ms of a 42.25 ms command), so the wait is the command's tail. The
left-over is the wait plus ≤ 0.1 ms.

### `peek_new` (outside frames, per burst, ms (calls))

| | `peek_new` | `peek_styles` | `peek_warm` | `sg_window` | `sg_thumb` | left over |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| load 0 | 5.65 (6) | 0.00 (6) | 0.10 (6) | 3.60 (6) | 0.55 (6) | 1.30 (23%) |
| CPU 8 | 7.15 (6) | 0.00 (6) | 0.10 (6) | 4.60 (6) | 0.60 (6) | 1.70 (24%) |
| main run, load 0 | 6.45 (6) | 0.00 (6) | 0.10 (6) | 4.20 (6) | 0.60 (6) | 1.45 (23%) |

`sg_window` calls = `peek_new` calls: no other surrogate was created in
these bursts. Morning, backdrops on, same 6 windows: `peek_new` 17.70 /
19.80 ms.

## 4. Switch, `nopeeks` vs `main` (outside frames, ms per burst)

Sums over each run ÷ 10 bursts, median over rounds; main run, load 0
(`results/stages-switch.md`). `main` has none of the new stages.

| stage | `main` | `nopeeks` | Δ |
| --- | ---: | ---: | ---: |
| command p50 (perf_bench) | 42.45 | 31.50 | −10.95 |
| `platform_sync` | 37.0 | 26.9 | −10.1 |
| `redraw` | 34.7 | 20.3 | −14.4 |
| `redraw_prep` | 14.5 | 16.3 | +1.8 |
| ↳ `ws_surrogates` | – | 3.3 | |
| ↳ `ws_park` | – | 2.3 | |
| ↳ `ws_show` | – | 10.2 | |
| ↳ `ws_start` | – | 0.4 | |
| `redraw_loop` | 19.9 | 4.1 | −15.8 |
| `platform_sync` − `redraw` | 2.3 | 6.6 | +4.3 |
| `sync_tail` | – | 6.6 | |
| ↳ `sync_focus` / `tab_bars` / `overview` / `guards` | – | 0.0 / 0.0 / 0.0 / 0.1 | |
| ↳ `sync_tail` − children − `backdrop_sync`/`border_sync` | – | 4.4 | |
| `border_sync` (cross-cutting) | 1.9 | 2.1 | +0.2 |
| `dwm_flush` (cross-cutting) | 6.9 | 7.1 | +0.2 |

`nopeeks` now answers `switch` 11 ms sooner than `main`: `main`'s
`redraw_loop` is 15.8 ms longer. Against that, `nopeeks`' extra over `main`
is in `platform_sync` outside `redraw`, i.e. `sync_tail` (+4.3). That 4.4 ms
is outside its four children and the overlay syncs. The rest is
`redraw_prep` (+1.8, of which `ws_show` 10.2). The main run at CPU 8 (loaded
in round 1 only) gives the same split: `redraw_prep` 13.1 vs 16.5,
`platform_sync` − `redraw` 2.5 vs 6.4, `redraw_loop` 16.6 vs 4.2.

Switch run (isolate, no `main`), per burst, load 0 / CPU 8:

| stage | `nopeeks` | `default` | `noslidepeeks` |
| --- | --- | --- | --- |
| `redraw_prep` | 17.4 / 21.3 | 17.0 / 21.9 | 17.6 / 21.9 |
| ↳ `ws_surrogates` | 3.8 / 4.8 | 3.9 / 4.8 | 3.8 / 4.8 |
| ↳ `ws_park` | 2.4 / 3.0 | 2.4 / 2.9 | 2.5 / 3.0 |
| ↳ `ws_show` | 10.8 / 12.9 | 10.3 / 13.6 | 10.8 / 13.5 |
| ↳ `ws_start` | 0.4 / 0.5 | 0.4 / 0.5 | 0.4 / 0.5 |
| `redraw_loop` | 3.9 / 6.4 | 3.9 / 6.2 | 3.9 / 5.3 |
| `sync_tail` | 5.6 / 7.8 | 5.7 / 7.9 | 5.6 / 7.5 |
| ↳ unscoped (− children − overlay syncs) | 5.5 / 5.0 | 5.5 / 4.9 | 5.4 / 5.0 |
| `dwm_flush` (1 call per burst) | 6.9 / 8.3 | 6.6 / 9.1 | 7.0 / 8.9 |
| `sg_window` (7 calls) | 2.3 / 2.9 | 2.3 / 2.9 | 2.2 / 2.9 |
| `sg_thumb` (7 calls) | 0.6 / 0.7 | 0.6 / 0.7 | 0.6 / 0.7 |

Each burst has one `redraw_prep` call, in the command.

## 5. Stages whose children fall clearly short (medians per burst, ms)

| stage | scenario, mode | total | children | short by |
| --- | --- | ---: | ---: | ---: |
| `anim_step` (outside) | resize `default` | 11.30 / 13.65 | `peek_new` 5.65 / 7.15 | 5.55 (49%) / 6.50 (48%) |
| `peek_new` (outside) | resize `default` | 5.65 / 7.15 | 4.35 / 5.45 | 1.30 (23%) / 1.70 (24%) |
| `tick` (frames) | resize `default` | 142.40 / 173.55 | `platform_sync`+`cleanup`+`peek_draw`+`peek_track` 120.90 / 149.30 | 21.30 (15%) / 23.10 (14%) |
| `peek_frame` (frames) | resize `default` | 108.35 / 125.55 | `peek_draw`+`peek_track` 80.80 / 93.25 | 27.80 (26%) / 32.05 (25%): the wait, by design |
| `tick` (frames) | switch `nopeeks` | 69.30 | `platform_sync`+`cleanup` 27.25 | 42.00 (63%), ~98 frames |
| `tick` (frames) | switch `default` | 69.25 | 27.15 | 41.45 (63%) |
| `sync_tail` (outside) | switch `nopeeks` | 5.50 | children + overlay syncs 0.10 | 5.35 (98%) |
| `cloak` (outside) | resize `nopeeks` | 18.25 | `dwm_flush`+`take_over`+`preposition`+`session_begin` 15.50 | 2.45 (13%) |
| `rd_apply` (outside) | switch `nopeeks` | 3.80 | `rp_*`+`uncloak` 3.25 | 0.55 (14%) |

The tree also lists these as short. A cross-cutting stage called directly in
them covers the gap:

- `redraw` outside frames, resize `default`: 50.00 against 29.10, short by
  20.95. `cloak` is 20.95.
- `redraw` in frames: `default` 36.25 against 23.65, short by 12.75, which
  is the in-frame `dwm_flush`. `nopeeks` 74.25 against 19.65, short by
  54.15, which is `batch_commit` (53.3).
- `sync_tail` in frames and outside, on resize: `backdrop_sync` and
  `border_sync` fill it.
- `peek_track` against `peek_ring` + `peek_backdrop`: 8.25 against 7.90,
  short by only 0.50 (6%).

## 6. Backdrop or border over a window

Not seen. I didn't watch the screen and took no screenshots. errors.log has
no `Overlay z-settle ... capped` warning in any run, voided ones included.
Before the bench, at 11:50:49Z (outside these runs, mode unknown), it has four:
`topmost=true, expected false`, for chrome and three ApplicationFrameHost
windows.

## 7. What failed and what was changed

- `cargo build` failed: the running WM locks `target\release\glazewm.exe`.
  Fix: saved the layout, `wm-exit`, built, restarted the WM plainly and
  restored the layout.
- **Zed** holds the files in `~/.glzr/glazewm` open, so PowerShell's
  `Set-Content`/`Add-Content` on `isolate-bench.txt` failed ("being used by
  another process"). The first smoke run stopped at its header line.
  Changes:
  - `isolate_modes.ps1` got `-ResultsFile` (default unchanged); runs wrote
    to the session scratchpad.
  - `perf.log` is no longer moved; `bench-chain.sh` copies out each run's
    part by byte offset. Rust's writes (`perf.log`, the layout file) coexist
    with Zed.
- `bench-run.ps1` (the copy in this folder) got `-Launch`, using
  `isolate_modes.ps1`'s functions, so `main` measured the same windows.
- **AnyDesk:** at ~13:15 an AnyDesk window joined workspace 1. It switched
  between minimized and tiled and was repositioned in every burst. That
  affected the first `main-load0` (rounds 2–4: `main` resize 31–37 vs 29 ms
  in round 1, switch 43–48 vs 36) and all three first CPU 8 runs. Those four
  are void (`results/anydesk/`). They were rerun 13:47–14:06 with the AnyDesk
  window moved to workspace 3. In the six runs kept, all 1,040 measured
  reports have no AnyDesk row, and every tile line has the 6 windows.
- The main runs' CPU 8 load fades after round 1–2 (section 1). Not
  investigated.
- `GLAZEWM_ISOLATE`/`GLAZEWM_PERF` weren't set in the user environment.
  Nothing to restore.
- At the end, the WM was running as `glazewm.exe start` on the user config,
  without the profiler. The UWP windows were moved back to workspace 1 in
  their order. Backdrops stay off. The AnyDesk window had closed by then.

## Appendix: the two full reports

```text
=== default #4 resize (median of 40 reports by interval max: 35.13 ms)
2026-11-13T12:11:10.439221Z  INFO glazewm::perf: perf [animations idle]: 60 frames in 417.4ms, 4 slow (>5.7ms), peak 0 window(s) animating
  stage                 calls      total  per-frame      worst
  peek_frame               23    101.7ms     1.70ms    31.12ms
    peek_draw             122     66.1ms     1.10ms    25.43ms
    peek_track             23      9.0ms     0.15ms     4.01ms
  tick                     60    153.4ms     2.56ms    33.70ms
    platform_sync           4     54.6ms     0.91ms    25.31ms
      redraw                4     49.9ms     0.83ms    23.69ms
        redraw_prep         4      0.3ms     0.00ms     0.09ms
        redraw_loop         4     24.7ms     0.41ms    16.17ms
          anim_step        12      0.0ms     0.00ms     0.01ms
          rd_apply         12     23.7ms     0.40ms    15.74ms
            rp_query       12      0.0ms     0.00ms     0.00ms
            rp_swp         12      1.7ms     0.03ms     1.35ms
            rp_visible     12     17.8ms     0.30ms    11.42ms
            uncloak        12      3.0ms     0.05ms     2.13ms
        peek_raise          4      0.0ms     0.00ms     0.00ms
        session_overlays      4      0.0ms     0.00ms     0.00ms
      sync_tail             4      4.3ms     0.07ms     2.28ms
        sync_focus          4      2.0ms     0.03ms     1.65ms
        tab_bars            4      0.0ms     0.00ms     0.01ms
        overview            4      0.0ms     0.00ms     0.00ms
        guards              4      0.5ms     0.01ms     0.47ms
    cleanup                60      0.0ms     0.00ms     0.00ms
  -- called from several parents, already counted above --
    backdrop_sync           4      0.1ms     0.00ms     0.02ms
    border_sync             4      1.7ms     0.03ms     0.89ms
    dwm_flush               2     24.9ms     0.42ms    18.22ms
    native_frame          148      0.9ms     0.01ms     0.27ms
    ovl_region              6      2.7ms     0.04ms     1.42ms
    ovl_place               6      2.9ms     0.05ms     1.56ms
    ovl_blend             152      0.0ms     0.00ms     0.01ms
    peek_ring             142      8.5ms     0.14ms     4.00ms
    peek_backdrop         142      0.0ms     0.00ms     0.00ms
  -- outside frames (e.g. the relayout starting an animation) --
  stage                 calls      total
  command                   1     42.3ms
  platform_sync             7     46.6ms
  redraw                    7     45.3ms
  redraw_prep               7      0.2ms
  redraw_loop               7     15.4ms
  anim_step                12     11.4ms
  rd_frozen                 6      0.0ms
  rd_apply                  6      3.5ms
  rp_query                  6      0.0ms
  rp_swp                    6      0.0ms
  rp_visible                6      3.5ms
  uncloak                   6      0.0ms
  peek_cover                1      7.6ms
  peek_handover             1      0.7ms
  peek_raise                7      1.1ms
  backdrop_sync             7      0.1ms
  border_sync               7      0.4ms
  dwm_flush                 1     11.5ms
  cloak                     7     20.3ms
  session_overlays          7      0.0ms
  sync_tail                 7      0.8ms
  sync_focus                7      0.0ms
  tab_bars                  7      0.1ms
  overview                  7      0.0ms
  guards                    7      0.3ms
  native_frame             44      0.2ms
  peek_new                  6      6.0ms
  take_over                 6      8.7ms
  preposition               6      0.1ms
  ovl_blend                53      0.0ms
  peek_styles               6      0.0ms
  peek_warm                 6      0.1ms
  sg_window                 6      4.0ms
  sg_thumb                  6      0.5ms
  peek_ring                32      8.3ms
  peek_backdrop            32      0.0ms
  -- frame time distribution (budget 5.7ms) --
  frame time                  p50        p90        p99        max
                           0.15ms     2.50ms    33.70ms    33.70ms
  hitches: 4 frame(s) >2x budget, 3 >4x budget, out of 60
  frame interval           5.72ms     5.79ms    35.13ms    35.13ms  (~175 fps)
  -- input -> motion latency --
                        count        p50        max
  to anim start             1     2.11ms     2.11ms
  to first frame            1    67.95ms    67.95ms
  -- slowest input timeline (67.95ms to first frame; * = called from several parents) --
  stage                        at       took
  command                  0.00ms    42.25ms
    platform_sync          0.38ms    41.86ms
      redraw               0.42ms    41.56ms
        redraw_loop        0.53ms    11.81ms
          anim_step        0.55ms     2.00ms
  * peek_new               0.55ms     0.87ms
  * sg_window              0.56ms     0.46ms
          anim_step        2.60ms     1.69ms
  * peek_new               2.60ms     0.86ms
  * sg_window              2.60ms     0.54ms
          anim_step        4.38ms     2.02ms
  * peek_new               4.39ms     1.15ms
  * sg_window              4.44ms     0.82ms
          anim_step        6.47ms     1.53ms
  * peek_new               6.47ms     0.80ms
  * sg_window              6.52ms     0.53ms
          anim_step        8.05ms     1.96ms
  * peek_new               8.05ms     1.19ms
  * sg_window              8.11ms     0.86ms
          anim_step       10.08ms     2.24ms
  * peek_new              10.09ms     1.09ms
  * sg_window             10.13ms     0.77ms
        peek_cover        12.34ms     7.55ms
  * peek_ring             12.36ms     1.24ms
  * peek_ring             13.61ms     1.27ms
  * peek_ring             14.89ms     1.30ms
  * peek_ring             16.20ms     1.18ms
  * peek_ring             17.39ms     1.22ms
  * peek_ring             18.61ms     1.28ms
  * cloak                 19.89ms    20.28ms
  * dwm_flush             19.89ms    11.48ms
  * take_over             31.38ms     1.39ms
  * take_over             32.82ms     1.98ms
  * take_over             34.81ms     1.27ms
  * take_over             36.08ms     1.58ms
  peek_frame              36.80ms    31.12ms
  * take_over             37.67ms     1.22ms
  * take_over             38.89ms     1.28ms
        peek_handover     40.17ms     0.66ms
        peek_raise        40.84ms     1.14ms
  tick                    42.37ms    25.58ms
    peek_draw             42.38ms     6.07ms
    peek_draw             48.45ms     2.02ms
    peek_draw             50.47ms     4.86ms
    peek_draw             55.34ms     4.13ms
    peek_draw             59.47ms     3.98ms
    peek_draw             63.46ms     4.38ms
  -- rd_apply by process (sync = blocked on that app's message pump)        --
  process               calls      total   per-call      worst
  chrome [async]            6      6.5ms     1.09ms     2.56ms
  Notepad [async]           3      6.2ms     2.06ms     4.80ms
  WindowsTerminal [async]      6      5.2ms     0.87ms     2.02ms
  FPilot [async]            3      5.0ms     1.67ms     3.93ms
  -- event queue wait (main-loop starvation) --
  event                 count      total       mean      worst
  window                   74   2086.8ms    28.20ms    63.02ms  (1 unpaired -- SUSPECT)
  -- animations started (move vs resize) --
  gesture               count      share
  move                      0       0.0%
  resize                    6     100.0%
```

```text
=== default #4 resize (max of 40 reports by interval max: 44.14 ms)
2026-11-13T12:11:12.959817Z  INFO glazewm::perf: perf [animations idle]: 60 frames in 421.6ms, 4 slow (>5.7ms), peak 0 window(s) animating
  stage                 calls      total  per-frame      worst
  peek_frame               23    100.8ms     1.68ms    28.71ms
    peek_draw             122     67.3ms     1.12ms    27.14ms
    peek_track             23      7.9ms     0.13ms     3.31ms
  tick                     60    155.2ms     2.59ms    41.72ms
    platform_sync           4     59.9ms     1.00ms    35.41ms
      redraw                4     54.7ms     0.91ms    33.45ms
        redraw_prep         4      0.3ms     0.00ms     0.09ms
        redraw_loop         4     27.1ms     0.45ms    17.35ms
          anim_step        12      0.0ms     0.00ms     0.01ms
          rd_apply         12     25.8ms     0.43ms    16.86ms
            rp_query       12      0.0ms     0.00ms     0.00ms
            rp_swp         12      1.6ms     0.03ms     0.88ms
            rp_visible     12     19.9ms     0.33ms    12.91ms
            uncloak        12      2.9ms     0.05ms     2.02ms
        peek_raise          4      0.0ms     0.00ms     0.00ms
        session_overlays      4      0.0ms     0.00ms     0.00ms
      sync_tail             4      4.6ms     0.08ms     1.86ms
        sync_focus          4      1.7ms     0.03ms     1.34ms
        tab_bars            4      0.0ms     0.00ms     0.01ms
        overview            4      0.0ms     0.00ms     0.00ms
        guards              4      0.6ms     0.01ms     0.54ms
    cleanup                60      0.0ms     0.00ms     0.00ms
  -- called from several parents, already counted above --
    backdrop_sync           4      0.1ms     0.00ms     0.02ms
    border_sync             4      2.1ms     0.04ms     1.19ms
    dwm_flush               2     27.4ms     0.46ms    16.02ms
    native_frame          148      0.8ms     0.01ms     0.26ms
    ovl_region              6      2.3ms     0.04ms     1.25ms
    ovl_place               6      2.5ms     0.04ms     1.38ms
    ovl_blend             152      0.0ms     0.00ms     0.01ms
    peek_ring             142      7.4ms     0.12ms     3.31ms
    peek_backdrop         142      0.0ms     0.00ms     0.00ms
  -- outside frames (e.g. the relayout starting an animation) --
  stage                 calls      total
  command                   1     43.7ms
  platform_sync             7     48.9ms
  redraw                    7     47.0ms
  redraw_prep               7      0.2ms
  redraw_loop               7     16.0ms
  anim_step                12     11.5ms
  rd_frozen                 6      0.0ms
  rd_apply                  6      3.9ms
  rp_query                  6      0.0ms
  rp_swp                    6      0.0ms
  rp_visible                6      3.9ms
  uncloak                   6      0.0ms
  peek_cover                1      7.7ms
  peek_handover             1      0.8ms
  peek_raise                7      1.6ms
  backdrop_sync             7      0.1ms
  border_sync               7      0.7ms
  dwm_flush                 1     12.4ms
  cloak                     7     20.7ms
  session_overlays          7      0.0ms
  sync_tail                 7      1.3ms
  sync_focus                7      0.0ms
  tab_bars                  7      0.1ms
  overview                  7      0.0ms
  guards                    7      0.4ms
  native_frame             44      0.4ms
  peek_new                  6      5.9ms
  take_over                 6      8.3ms
  preposition               6      0.0ms
  ovl_blend                53      0.0ms
  peek_styles               6      0.0ms
  peek_warm                 6      0.1ms
  sg_window                 6      3.9ms
  sg_thumb                  6      0.5ms
  peek_ring                32      8.6ms
  peek_backdrop            32      0.0ms
  -- frame time distribution (budget 5.7ms) --
  frame time                  p50        p90        p99        max
                           0.16ms     2.46ms    41.72ms    41.72ms
  hitches: 4 frame(s) >2x budget, 3 >4x budget, out of 60
  frame interval           5.72ms     5.84ms    44.14ms    44.14ms  (~175 fps)
  -- input -> motion latency --
                        count        p50        max
  to anim start             1     1.85ms     1.85ms
  to first frame            1    71.10ms    71.10ms
  -- slowest input timeline (71.10ms to first frame; * = called from several parents) --
  stage                        at       took
  command                  0.00ms    43.65ms
    platform_sync          0.30ms    43.33ms
      redraw               0.33ms    42.84ms
        redraw_loop        0.41ms    11.94ms
          anim_step        0.42ms     1.93ms
  * peek_new               0.42ms     0.82ms
  * sg_window              0.43ms     0.41ms
          anim_step        2.40ms     1.53ms
  * peek_new               2.40ms     0.74ms
  * sg_window              2.41ms     0.49ms
          anim_step        4.11ms     2.07ms
  * peek_new               4.11ms     1.33ms
  * sg_window              4.18ms     0.96ms
          anim_step        6.25ms     1.81ms
  * peek_new               6.26ms     0.97ms
  * sg_window              6.31ms     0.63ms
          anim_step        8.13ms     2.51ms
  * peek_new               8.13ms     1.15ms
  * sg_window              8.18ms     0.78ms
          anim_step       10.73ms     1.61ms
  * peek_new              10.73ms     0.91ms
  * sg_window             10.78ms     0.59ms
        peek_cover        12.35ms     7.69ms
  * peek_ring             12.36ms     1.27ms
  * peek_ring             13.64ms     1.39ms
  * peek_ring             15.04ms     1.33ms
  * peek_ring             16.38ms     1.17ms
  * peek_ring             17.56ms     1.19ms
  * peek_ring             18.76ms     1.28ms
  * cloak                 20.04ms    20.68ms
  * dwm_flush             20.04ms    12.38ms
  * take_over             32.43ms     1.67ms
  * take_over             34.12ms     1.70ms
  * take_over             35.82ms     1.29ms
  * take_over             37.11ms     1.30ms
  * take_over             38.41ms     1.21ms
  * take_over             39.61ms     1.11ms
        peek_handover     40.72ms     0.84ms
        peek_raise        41.56ms     1.61ms
  peek_frame              42.36ms    28.71ms
      sync_tail           43.18ms     0.45ms
        guards            43.28ms     0.35ms
  tick                    43.78ms    27.32ms
    peek_draw             43.80ms     6.86ms
    peek_draw             50.67ms     2.16ms
    peek_draw             52.83ms     5.05ms
    peek_draw             57.89ms     5.03ms
    peek_draw             62.92ms     4.11ms
    peek_draw             67.04ms     3.93ms
  -- rd_apply by process (sync = blocked on that app's message pump)        --
  process               calls      total   per-call      worst
  FPilot [async]            3      7.5ms     2.50ms     6.18ms
  WindowsTerminal [async]      6      6.7ms     1.12ms     2.63ms
  chrome [async]            6      5.8ms     0.97ms     1.84ms
  Notepad [async]           3      5.4ms     1.80ms     3.57ms
  -- event queue wait (main-loop starvation) --
  event                 count      total       mean      worst
  window                   73   2124.5ms    29.10ms    66.40ms  (1 unpaired -- SUSPECT)
  -- animations started (move vs resize) --
  gesture               count      share
  move                      0       0.0%
  resize                    6     100.0%
```
