# One surface vs start-latency vs stand-ins modes vs main (2026-11-17, desktop, second run)

`tryout/one-surface` (`this`, `c6068c12`, the bench fixes merged) against
`origin/tryout/start-latency` `ffb2d88` (default), `origin/tryout/stand-ins` `65a60de`
(`nopeeks`, default, `morepeeks`) and `origin/main` `cfd3cef`. Desktop PC: RTX 3070,
3440x1440 at 174 Hz (5.75 ms refresh), 16 threads. Run 11:03–11:38 local:

```
powershell -ExecutionPolicy Bypass -File packages\wm-cli\examples\perf_bench\compare_branches.ps1 -Target chrome -CpuLoad 0,8 -Screenshots -TileOrder Notepad,WindowsTerminal,WindowsTerminal,chrome,chrome,FPilot -Launch none -Against "origin/tryout/start-latency:default;origin/tryout/stand-ins:nopeeks,default,morepeeks;origin/main"
```

36 restarts (CPU 0 and 8 threads x 3 rounds x 6 builds/modes), 10 bursts per scenario,
backdrops and borders as configured, no `--dwm`. All 36 runs had the same tile order.
The four windows were opened by hand before the run (`-Launch none`) and the target
chrome was focused. In the first run File Pilot held focus at the first restart; here
every build had the target focused in every screenshot.

- `compare-bench.md`: every run's output, then per scenario the tables (medians over
  rounds).
- `compare-bench-rows.md`: one summary row per run.

Screenshots were checked but not committed (they showed private browser tabs).

## Headline (median of 3 rounds, CPU 0 / CPU 8, ms)

| scenario | metric | this | start-latency | stand-ins:nopeeks | stand-ins | stand-ins:morepeeks | main |
| --- | --- | --- | --- | --- | --- | --- | --- |
| move | input->first frame | 17.4 / 19.1 | 21.6 / 22.7 | 16.7 / 17.2 | — | — | **15.1 / 16.8** |
| move | command p50 | 16.1 / 17.8 | 19.6 / 20.5 | 16.2 / 16.7 | 19.2 / 21.0 | 19.5 / 22.0 | **14.4 / 15.9** |
| move | frame p90 | **0.50 / 0.53** | 0.87 / 0.92 | 1.09 / 1.19 | 0.89 / 0.97 | 0.92 / 1.04 | 1.16 / 1.26 |
| move | interval p90 | 5.74 / 5.75 | 5.74 / 5.76 | 5.74 / 5.75 | 5.75 / 5.75 | 5.75 / 5.76 | 5.74 / 5.74 |
| move | interval p99 | 10.3 / 12.4 | 11.0 / 13.1 | 7.7 / 8.5 | 10.8 / 12.9 | 10.9 / 12.9 | **7.5 / 8.0** |
| resize | input->first frame | 67.5 / 73.0 | 89.0 / 102.0 | 42.9 / 48.8 | — | — | **35.1 / 35.9** |
| resize | command p50 | 46.6 / 50.1 | 64.6 / 71.9 | 39.2 / 44.5 | 64.8 / 71.1 | 62.7 / 71.0 | **30.5 / 31.8** |
| resize | frame p90 | 3.53 / **4.01** | **3.27** / 4.55 | 4.39 / 4.77 | 3.80 / 4.24 | 3.30 / 4.38 | 3.99 / 4.30 |
| resize | interval p90 | 5.77 / 5.79 | 5.95 / 6.79 | 5.77 / 5.78 | 5.90 / 6.64 | 5.86 / 5.84 | 5.76 / 5.77 |
| resize | interval p99 | 40.3 / 36.3 | 50.5 / 50.5 | **25.6 / 17.9** | 52.3 / 65.4 | 51.0 / 64.1 | 34.4 / 29.2 |
| switch | command p50 | 26.4 / 30.1 | 30.2 / 33.1 | 30.2 / 34.1 | 30.2 / 33.4 | 30.5 / 34.3 | **20.2 / 20.5** |
| switch | frame p90 | **1.08 / 1.21** | 1.21 / 1.45 | 1.19 / 1.51 | 1.17 / 1.34 | 1.20 / 1.37 | 2.44 / 2.76 |
| switch | interval p90 | 5.76 / 5.76 | 5.76 / 5.76 | 5.76 / 5.76 | 5.75 / 5.75 | 5.75 / 5.76 | 5.75 / 5.75 |
| switch | interval p99 | **19.0 / 23.3** | 24.3 / 27.9 | 22.9 / 27.0 | 24.3 / 28.6 | 25.1 / 28.2 | 53.1 / 59.3 |
| minimize | input->first frame | 71.0 / 76.4 | 90.1 / 97.7 | 55.4 / 64.1 | — | — | **49.4 / 45.9** |
| minimize | command p50 | 51.2 / 56.8 | 60.9 / 66.7 | 23.2 / 26.8 | 61.7 / 64.0 | 57.4 / 64.7 | **17.0 / 19.0** |
| minimize | frame p90 | 3.59 / 4.15 | 3.76 / 5.09 | **3.16 / 3.17** | 3.82 / 4.56 | 3.90 / 4.75 | 3.89 / 4.35 |
| minimize | interval p90 | 5.78 / 5.78 | 7.68 / 8.23 | 5.76 / 5.77 | 7.00 / 8.91 | 7.04 / 7.72 | 5.76 / 5.77 |
| minimize | interval p99 | 57.0 / 68.5 | 74.2 / 83.2 | 48.0 / 52.8 | 77.5 / 84.9 | 72.4 / 83.1 | **37.7 / 35.8** |

`—`: not recorded (stand-ins' peek path has no input->first frame; switch has none on
any build, so its start is read from command p50).

## Start stages (outside frames, median of 3 rounds, CPU 0 / CPU 8, ms)

`perf_bench` columns: `peek_new`, `peek_place`, `peek_cover`, `surface`, `restack`,
`session_begin`, `cloak`, `ws_*`. Taken from `perf.log` (each run's 10 measured
reports): `take_over` and `peek_new`'s parts `peek_styles` (backdrop copies) and
`sg_window` (a surrogate's own window). For `this` and start-latency, the CPU 0 values
from `perf.log` use rounds 2–3 only, because round 1's timestamps were too coarse to
split runs. Stand-ins and main have none of the peek stages.

| scenario | stage | this | start-latency | stand-ins:nopeeks | main |
| --- | --- | --- | --- | --- | --- |
| move | take_over | 2.5 / 2.7 | 2.7 / 3.1 | — | — |
| move | peek_new | **2.0 / 2.3** | 5.4 / 5.8 | — | — |
| move | └ peek_styles / sg_window | 0 / 0, — | 3.7 / 4.1, 0.9 / 1.0 | — | — |
| move | peek_place / peek_cover | 0.9 / 1.0, 1.2 / 1.3 | 1.2 / 1.3, 1.1 / 1.2 | — | — |
| move | surface | 1.0 / 1.3 | — | — | — |
| move | session_begin | — | — | 0.7 / 0.7 | 0.9 / 1.1 |
| resize | take_over | 9.0 / 10.6 | 9.4 / 10.3 | — | — |
| resize | peek_new | **3.3 / 3.7** | 16.2 / 19.1 | — | — |
| resize | └ peek_styles / sg_window | 0 / 0, — | 11.3 / 13.0, 3.1 / 3.7 | — | — |
| resize | peek_place / peek_cover | 1.9 / 2.1, 11.2 / 12.0 | 4.1 / 5.1, 9.9 / 11.1 | — | — |
| resize | surface | 0.7 / 0.9 | — | — | — |
| resize | session_begin | — | — | 1.7 / 1.9 | 2.1 / 2.6 |
| switch | ws_surrogates / ws_show | **10.2 / 12.6, 8.0 / 8.0** | 12.8 / 14.8, 9.2 / 9.4 | — | — |
| switch | ws_park / ws_start | 2.0 / 2.2, 0.4 / 0.4 | 1.9 / 2.3, 0.4 / 0.4 | — | — |
| switch | restack | 0 / 0 | — | — | — |
| minimize | take_over | 10.1 / 10.8 | 9.4 / 10.8 | — | — |
| minimize | peek_new | **8.5 / 9.0** | 25.5 / 29.5 | — | — |
| minimize | └ peek_styles / sg_window | 2.7 / 3.3, — | 15.8 / 17.8, 4.2 / 5.3 | — | — |
| minimize | peek_place / peek_cover | 1.7 / 1.8, 23.0 / 23.7 | 5.9 / 6.3, 18.4 / 21.3 | — | — |
| minimize | restack / surface | 8.6 / 9.7, 2.6 / 2.4 | — | — | — |
| minimize | session_begin | — | — | 2.2 / 3.2 | 2.1 / 2.6 |

`cloak` runs on every build at about the same cost (move 7–9, resize 16–22, minimize
15–26 ms) and doesn't separate them. `ws_*` are scopes only `this` and start-latency
have.

## Reading

| scenario | best start | best smooth frames | what makes the difference |
| --- | --- | --- | --- |
| move | main (15.1 / 16.8), then nopeeks; `this` +2 ms, start-latency +6 ms | `this` per frame (frame p90 0.50 vs ≥0.87); main/nopeeks fewest long gaps (p99 7.5–8.5 vs 10–13) | `this` vs start-latency: `peek_new` −3.4 ms (no `peek_styles`, no `sg_window`). Both pay `take_over` + `peek_cover` (~4 ms), which main doesn't. |
| resize | main (35 / 36), nopeeks (43 / 49); `this` 67 / 73; start-latency 89 / 102 | p90 at refresh: `this`, nopeeks, main (start-latency and stand-ins slip to 6.6–6.8 at CPU 8); fewest gaps: nopeeks (p99 26 / 18), then main; frame p90 about even (3.3–4.8) | `this` vs start-latency: `peek_new` −13/−15 ms (`peek_styles` 11–13 ms, `sg_window` 3–4 ms gone), `peek_place` −2/−3 ms. vs main: `take_over` (9–11) + `peek_cover` (10–12), together 20–23 ms of the 32–37 ms gap. |
| switch | main (command 20 / 20); `this` 26 / 30; others 30–34 | `this` (frame p90 1.08 / 1.21, p99 19 / 23); main the worst (p99 53 / 59) | `this` vs start-latency: `ws_surrogates` −2.6/−2.2 ms, `ws_show` −1.2/−1.4 ms. |
| minimize | main (49 / 46), nopeeks (55 / 64); `this` 71 / 76; start-latency 90 / 98 | p90 at refresh: `this`, nopeeks, main (start-latency and both stand-ins peek modes 7–9 ms); fewest gaps: main (p99 38 / 36); frame p90: nopeeks (but 142–150 frames against ~60, so not like for like) | `this` vs start-latency: `peek_new` −17/−20 ms and `peek_place` −4 ms, partly given back by `restack` (9–10 ms), `surface` (2.5 ms) and `peek_cover` (+2–5 ms). vs main: `take_over` ~10 + `peek_cover` ~23 + `peek_new` ~9 + `restack` ~9. |

One surface is the fastest peek build to start in every scenario, and has the fewest
long gaps of the peek builds everywhere. It is also the cheapest per frame, except
resize at CPU 0, where start-latency and `morepeeks` are 0.3 ms lower. Start-latency
starts about as fast as stand-ins default (command p50 within 3 ms); its `peek_new` is
3–5x `this`'s. Main starts sooner everywhere, and `nopeeks` everywhere but switch.
Both have fewer long gaps on move, resize and minimize. Main has the slowest frames
and the most gaps on switch.

## Screenshots

All 96 were reviewed (contact sheets per scenario, full-resolution top strips of
`this` and start-latency, exact border colors per tile). At rest, every tile on every
build has its border, the focused one on the target, and the layouts are identical.
`this` draws the border inside the window (rows 20–21), the others outside (18–19),
as decided. Mid-animation, the builds differ only in how far they have got
(main/nopeeks furthest, the peek builds later), as the start times say. Two things:

- Minimize mid (all builds that had started the zoom-out): the shrinking window shows
  over a faded full-size copy of itself. The same on main, so not a regression.
- Start-latency, CPU 8, switch mid: Notepad drawn about 165 px narrower than its tile,
  leaving a wallpaper gap before PowerShell. `perf_bench`'s switch setup moves tiles and
  sends the first (shot) switch straight away, so that switch started mid-reflow,
  before Notepad had its new width. A timing artifact of the shot pass, not of the
  measured bursts; not seen on any other build or at CPU 0.
