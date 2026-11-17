# Start cost 2: one surface vs nopeeks vs main (2026-11-17, desktop, with DWM timing)

`tryout/one-surface` (`this`, `51191dff`: ready-peek counts, finer start stages,
minimize kept on top) against `origin/tryout/stand-ins` `65a60de` (`nopeeks`) and
`origin/main` `cfd3cef`. Desktop PC: RTX 3070, 3440x1440 at 174 Hz (5.75 ms
refresh), 16 threads. Run 15:29–15:47 local:

```
powershell -ExecutionPolicy Bypass -File packages\wm-cli\examples\perf_bench\compare_branches.ps1 -Target chrome -CpuLoad 0,8 -Screenshots -Dwm -Launch none -TileOrder Notepad,WindowsTerminal,WindowsTerminal,chrome,chrome,FPilot -Against "origin/tryout/stand-ins:nopeeks;origin/main"
```

18 restarts (CPU 0 and 8 threads x 3 rounds x 3 builds), 10 bursts per scenario,
backdrops and borders as configured. All runs had the same tile order and the target
focused. The `PresentMon` service was running and capture worked from a
non-elevated shell. GPU clocks were not locked. The previous run is
`../2026-11-17-start-cost/` (`a3d1dff3`).

- `compare-bench.md`: every run's output, then per scenario the tables (medians over
  rounds).
- `compare-bench-rows.md`: one summary row per run.

Screenshots were checked but not committed (they showed private browser tabs).

## Headline (median of 3 rounds, CPU 0 / CPU 8; `was` = the previous run)

| scenario | metric | this | this, was | nopeeks | main |
| --- | --- | --- | --- | --- | --- |
| move | input->first frame ms | 17.9 / 19.0 | 18.7 / 19.6 | 15.0 / 16.6 | **13.1 / 14.4** |
| move | frame p90 ms | **0.56 / 0.55** | 0.54 / 0.53 | 1.14 / 1.25 | 1.22 / 1.30 |
| move | interval p99 ms | 10.7 / 12.6 | 10.5 / 12.3 | 7.8 / 9.0 | **7.7 / 8.9** |
| move | dwm missed vblanks/burst | 12 / 13 | 5 / 6 | 11 / 11 | 11 / 11 |
| move | dwm gpu busy p90 ms | 8.5 / 8.3 | 8.5 / 8.7 | 8.7 / 8.7 | **7.3 / 7.4** |
| resize | input->first frame ms | 63.2 / 74.7 | 64.4 / 78.0 | 44.2 / 50.9 | **35.8 / 37.8** |
| resize | frame p90 ms | **3.44 / 4.34** | 3.27 / 4.09 | 4.47 / 4.68 | 4.45 / 4.69 |
| resize | interval p99 ms | 39.8 / 38.8 | 36.5 / 44.4 | **20.2 / 22.2** | 30.4 / 29.5 |
| resize | dwm missed vblanks/burst | 28 / 29 | 22 / 25 | 14 / 17 | **9 / 11** |
| resize | dwm gpu busy p90 ms | 1.5 / 1.6 | 1.5 / 1.6 | 2.9 / 1.3 | 1.6 / 2.0 |
| switch | command p50 ms | 30.8 / 34.3 | 30.5 / 34.6 | 30.9 / 33.8 | **20.6 / 22.5** |
| switch | frame p90 ms | **1.09 / 1.32** | 1.09 / 1.30 | 1.27 / 1.41 | 2.53 / 3.05 |
| switch | interval p99 ms | **21.4 / 23.4** | 21.7 / 23.2 | 24.7 / 27.3 | 52.7 / 62.1 |
| switch | dwm missed vblanks/burst | 23 / 32 | 18 / 25 | 19 / 26 | **9 / 10** |
| switch | dwm gpu busy p90 ms | 2.5 / 6.2 | 5.3 / 5.3 | **2.0 / 2.1** | 4.1 / 3.6 |
| minimize | input->first frame ms | 60.1 / 68.7 | 60.9 / 72.0 | 57.1 / 60.1 | **43.8 / 50.5** |
| minimize | frame p90 ms | 3.81 / 4.54 | 3.69 / 4.47 | **3.24 / 3.59** | 4.14 / 4.74 |
| minimize | interval p99 ms | 52.1 / 53.6 | 52.1 / 61.2 | 48.9 / 51.4 | **38.5 / 42.7** |
| minimize | dwm missed vblanks/burst | 19 / 31 | 28 / 35 | **16** / 19 | 22 / **18** |
| minimize | dwm gpu busy p90 ms | **2.5 / 2.6** | 3.1 / 8.7 | 6.3 / 6.7 | 7.4 / 7.1 |

Switch has no input->first frame on any build. Missed vblanks vary between runs for
every build (move on main: 5 then 11), so read them as rough. Interval p90 is at the
refresh period (5.74–5.79 ms) on every build.

## Ready peeks (`perf_bench`, median per burst, both CPU loads)

| scenario | peek_ready hits | peek_ready misses |
| --- | --- | --- |
| move | 2 | 0 |
| resize | 6 | 0 |
| minimize | 6 | 0 |
| switch | 0 | 0 |

No misses, so no miss reasons were needed.

## Start stages of `this` (outside frames, median, CPU 0 / CPU 8, ms)

All from `perf_bench` columns, except `bd_park`, which comes from `perf.log`: all
this run's reports, split by scenario by their stage pattern, both CPU loads. `main`
and `nopeeks` start with `session_begin` only (0.7–2.8). `dwm_flush` is every build's;
`main` pays 4.4–5.4 of it.

| stage | move | resize | minimize | switch |
| --- | --- | --- | --- | --- |
| to_draw | 0.3 / 0.3 | 0.8 / 1.0 | 1.0 / 1.0 | — |
| to_alpha | 0.4 / 0.5 | 2.7 / 2.3 | 2.3 / 2.5 | — |
| to_cloak | 1.8 / 2.0 | **8.5 / 9.6** | **7.4 / 8.4** | — |
| ring_prep | 0 / 0 | 0 / 0 | 3.3 / 2.9 | — |
| ring_park | 1.2 / 1.4 | 1.0 / 1.2 | 1.2 / 1.3 | — |
| ring_copy | 0.3 / 0.3 | **6.7 / 7.1** | **7.3 / 7.9** | — |
| bd_copy | 0.1 / 0.1 | 0.5 / 0.5 | 0.9 / 1.0 | — |
| bd_park | 0.9 | 5.2 | 5.5 (with restores) | — |
| ovl_restack | 0.6 / 0.4 | 2.1 / 2.6 | 0.1 / 0.2 | — |
| peek_new | **2.1 / 2.3** | 3.4 / 4.0 | **7.4 / 8.7** | — |
| surface | 1.0 / 1.2 | 0.8 / 0.9 | 2.7 / 3.8 | 0.6 / 0.7 |
| restack | 0.8 / 0.8 | — | — | 0 |
| native_frame | 0.2 / 0.2 | 1.5 / 1.6 | 1.7 / 1.8 | 0.1 / 0.1 |
| dwm_flush (main) | **4.7 / 4.7** (4.8 / 5.4) | **8.8 / 10.3** (5.4 / 5.1) | **12.1 / 11.3** (5.0 / 4.4) | 5.7 / 6.2 (5.6 / 5.9) |
| ws_surrogates / ws_show | — | — | — | **9.8 / 11.5, 8.1 / 8.9** |

## Most expensive start steps, and the proposals that would cut them

| scenario | top 3 (ms) | proposals |
| --- | --- | --- |
| move | `dwm_flush` 4.7 (as main), `peek_new` 2.1–2.3, `to_cloak` 1.8–2.0 | 1 drops the flush (every peek ready: 2 hits, 0 misses). None cuts `peek_new` (`reclaim`) or `to_cloak`; 5 only moves `take_over` earlier. 2 takes `ring_park` (1.2–1.4) out of the first frame. |
| resize | `dwm_flush` 8.8–10.3 (main ~5), `to_cloak` 8.5–9.6, `ring_copy` 6.7–7.1 | 1 for the flush (6 hits, 0 misses). None cuts `to_cloak` or `ring_copy`. Next: `bd_park` 5.2 and `ovl_restack` 2.1–2.6, moved out of the first frame by 2; `native_frame` 1.5 cut by 3. |
| minimize | `dwm_flush` 11.3–12.1 (main ~5), `peek_new` 7.4–8.7, `to_cloak` 7.4–8.4 (`ring_copy` 7.3–7.9 close behind) | 4 shares the two flushes (one saved), 1 may drop one. None cuts `peek_new`, `to_cloak` or `ring_copy`. 2 for `bd_park` 5.5 and `ring_park`; 3 for `native_frame` 1.7. |
| switch | `ws_surrogates` 9.8–11.5, `ws_show` 8.1–8.9, `dwm_flush` 5.7–6.2 (as main) | None of the six touches the switch. |

Notable: with every motion on a ready peek, `peek_new` is still 7–9 ms on minimize
and `to_cloak` 7–10 ms on resize and minimize. Neither is in any proposal.

## Screenshots

All 48 were reviewed: borders, backdrops and gaps at rest are as before on every
build, and the target was focused in every run.

- **Resize, CPU 8, round 1, `this`, mid (66 ms):** two neighbours' rings are still
  drawn at their pre-resize place: thin grey vertical lines at x 1155/1700 (tile 3) and
  2305/2851 (tile 5), across the windows that have moved. In the CPU 0 shot (69 ms) the
  same rings sit on their moving windows (973/1428, 2487/2941). Seen once; the other
  builds don't show it.
- **Minimize mid (62–65 ms):** `this` hasn't moved a neighbour yet (later start).
  `nopeeks` and `main` already slide theirs over the fading tile. At CPU 8, `this`
  shows a thin sliver of chrome behind the shrinking copy's left edge.
- **A minimize staying over the neighbour taking its place** can't be seen this early,
  so a short `--verbose` run of `this` with `window_minimize` at 1500 ms took two
  captures. At 400 and 900 ms the minimizing chrome is not visible at all: its
  neighbours landed about 240 ms in (`Kept the landed peek of … ready`), and their real
  windows cover the surface for the rest of its 1.5 s (peek 14:49:12.293–13.798 UTC).
  This is the close's case (`Start cost 2` c, proposal 6), now seen on minimize. In
  both minimizes, a neighbour still got a `Stacking … over a tile it may cross` line.
  `Minimize of … ranked over the moves reflowing for it` never appeared.
