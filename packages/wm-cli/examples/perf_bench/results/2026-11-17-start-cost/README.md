# Start cost: one surface vs nopeeks vs main (2026-11-17, desktop, with DWM timing)

`tryout/one-surface` (`this`, `a3d1dff3`: transits on top, ready peeks for every
resting tile, one zoom per open) against `origin/tryout/stand-ins` `65a60de`
(`nopeeks`) and `origin/main` `cfd3cef`. Desktop PC: RTX 3070, 3440x1440 at 174 Hz
(5.75 ms refresh), 16 threads. Run 12:41–12:59 local:

```
powershell -ExecutionPolicy Bypass -File packages\wm-cli\examples\perf_bench\compare_branches.ps1 -Target chrome -CpuLoad 0,8 -Screenshots -Dwm -TileOrder Notepad,WindowsTerminal,WindowsTerminal,chrome,chrome,FPilot -Launch none -Against "origin/tryout/stand-ins:nopeeks;origin/main"
```

18 restarts (CPU 0 and 8 threads x 3 rounds x 3 builds), 10 bursts per scenario,
backdrops and borders as configured. All runs had the same tile order and the
target focused. The `PresentMon` service was running, and the shell wasn't elevated,
but DWM capture worked (a 2-burst probe first). GPU clocks were not locked. The
previous run is `../2026-11-17-start-latency/` (same machine, same day, no `--dwm`).

- `compare-bench.md`: every run's output, then per scenario the tables (medians over
  rounds).
- `compare-bench-rows.md`: one summary row per run.

Screenshots were checked but not committed (they showed private browser tabs).

## Headline (median of 3 rounds, CPU 0 / CPU 8, ms)

`was` is `this` in the previous run.

| scenario | metric | this | was | stand-ins:nopeeks | main |
| --- | --- | --- | --- | --- | --- |
| move | input->first frame | 18.7 / 19.6 | 17.4 / 19.1 | 16.5 / 16.8 | **14.4 / 14.6** |
| move | command p50 | 18.1 / 18.7 | 16.1 / 17.8 | 15.5 / 15.7 | **13.7 / 13.9** |
| move | frame p90 | **0.54 / 0.53** | 0.50 / 0.53 | 1.31 / 1.50 | 1.36 / 1.58 |
| move | interval p90 / p99 | 5.74 / 10.5, 5.76 / 12.3 | 10.3 / 12.4 | 5.74 / 8.0, 5.74 / 9.2 | 5.74 / **7.8**, 5.75 / **8.8** |
| resize | input->first frame | 64.4 / 78.0 | 67.5 / 73.0 | 44.5 / 50.6 | **34.1 / 36.8** |
| resize | command p50 | 49.8 / 53.7 | 46.6 / 50.1 | 39.4 / 45.8 | **29.1 / 31.9** |
| resize | frame p90 | **3.27 / 4.09** | 3.53 / 4.01 | 4.09 / 4.62 | 4.37 / 4.82 |
| resize | interval p90 / p99 | 5.78 / 36.5, 5.77 / 44.4 | 40.3 / 36.3 | 5.78 / **20.4**, 5.79 / **19.7** | 5.78 / 25.8, 5.76 / 30.4 |
| switch | command p50 | 30.5 / 34.6 | 26.4 / 30.1 | 31.1 / 34.2 | **20.2 / 22.1** |
| switch | frame p90 | **1.09 / 1.30** | 1.08 / 1.21 | 1.24 / 1.38 | 2.64 / 3.22 |
| switch | interval p90 / p99 | 5.76 / **21.7**, 5.76 / **23.2** | 19.0 / 23.3 | 5.76 / 24.3, 5.76 / 26.0 | 5.74 / 54.3, 5.75 / 59.2 |
| minimize | input->first frame | 60.9 / 72.0 | 71.0 / 76.4 | 57.0 / 62.8 | **45.8 / 46.7** |
| minimize | command p50 | 49.6 / 53.1 | 51.2 / 56.8 | 25.2 / 30.6 | **17.9 / 21.1** |
| minimize | frame p90 | 3.69 / 4.47 | 3.59 / 4.15 | **3.25 / 3.56** | 3.98 / 4.38 |
| minimize | interval p90 / p99 | 5.77 / 52.1, 5.79 / 61.2 | 57.0 / 68.5 | 5.76 / 49.0, 5.77 / 54.3 | 5.76 / **35.0**, 5.77 / **36.7** |

## Start stages of `this` (outside frames, median, CPU 0 / CPU 8, ms)

`peek_new`, `peek_cover`, `restack`, `surface` and `session_begin` are `perf_bench`
columns. `take_over` comes from `perf.log`: each run's 10 measured reports, matched by
time. `nopeeks` and `main` start with `session_begin` only: 0.7 / 0.9 (move),
1.8–2.8 (resize), 2.2–2.6 (minimize).

| scenario | stage | now | was |
| --- | --- | --- | --- |
| move | take_over / peek_new / peek_cover | 2.6 / 2.9, 2.1 / 2.2, 1.2 / 1.6 | 2.5 / 2.7, 2.0 / 2.3, 1.2 / 1.3 |
| move | restack / surface | 0.8 / 0.9, 1.0 / 1.2 | — , 1.0 / 1.3 |
| resize | take_over / peek_new / peek_cover | 9.4 / 10.1, 3.3 / 4.1, 11.0 / 12.4 | 9.0 / 10.6, 3.3 / 3.7, 11.0 / 11.9 |
| resize | restack / surface | — , 0.7 / 0.9 | — , 0.7 / 0.9 |
| switch | ws_surrogates / ws_show | 9.9 / 11.8, 8.1 / 8.6 | 10.2 / 12.6, 8.0 / 8.0 |
| minimize | take_over / peek_new / peek_cover | 10.3 / 12.5, 7.2 / 9.2, 16.0 / 18.8 | 10.1 / 10.8, 8.5 / 9.0, 17.9 / 19.4 |
| minimize | restack / surface | **— (gone)**, 2.4 / 2.4 | 8.6 / 9.7, 2.0 / 2.3 |

The only start stage that moved is minimize's `restack` (now never run). The plan
expected ready peeks to cut `peek_new` to a reclaim on resize and minimize, and
`peek_cover` by the 8-cell ring registration on resize. Neither shows: both are within
noise of the previous run. A move now runs `restack` (0.8–0.9 ms) where it ran
none before.

## DWM (`--dwm`, per burst, median, CPU 0 / CPU 8)

| scenario | metric | this | stand-ins:nopeeks | main |
| --- | --- | --- | --- | --- |
| move | gpu busy p90 ms | 8.5 / 8.7 | 8.6 / 8.8 | **7.3 / 7.3** |
| move | missed vblanks/burst | 5 / 6 | 10 / 5 | **5 / 4** |
| resize | gpu busy p90 ms | 1.5 / 1.6 | 4.8 / 4.3 | **1.4 / 1.6** |
| resize | missed vblanks/burst | 22 / 25 | 11 / 7 | **7 / 7** |
| switch | gpu busy p90 ms | 5.3 / 5.3 | **2.4 / 2.7** | 3.1 / 3.8 |
| switch | missed vblanks/burst | 18 / 25 | 19 / 30 | **8 / 11** |
| minimize | gpu busy p90 ms | **3.1** / 8.7 | 6.6 / 7.1 | 6.6 / **6.6** |
| minimize | missed vblanks/burst | 28 / 35 | 15 / 17 | **13 / 13** |

`dwm display interval p90` is about 11.4 ms on every build, and higher for `this` on
resize (17.2) and minimize (22.9). The GPU clock wasn't locked, so GPU busy shifts
with clock boost. Rest cost can't be told: `perf_bench` measures bursts only, and the
bench has no idle window between restarts. The previous run had no `--dwm`, so there
is no before for these rows.

## Reading

One surface does not start as fast as main anywhere. Its gaps to main:

- **move:** +4 / +5 ms (input->first frame). `take_over` + `peek_new` + `peek_cover`
  is about 6 ms against main's 1 ms `session_begin`.
- **resize:** +30 / +41 ms. `take_over` and `peek_cover`, ~10 + 11–12 ms, are
  unchanged. Missed vblanks are 3x main's.
- **switch:** command +10 / +12 ms. `ws_surrogates` + `ws_show` is about 18–20 ms.
  Still the fewest long gaps (p99 22 vs main 54–59).
- **minimize:** +15 / +25 ms, down from +22 / +30 (main's own figure moved too).
  `restack` is gone; `take_over` 10–12 and `peek_cover` 16–19 remain. Missed vblanks
  are 2–3x main's.

Per frame it stays the cheapest on move, resize and switch (frame p90).

## Checks on screen and in the log

All 48 bench screenshots were reviewed. Borders, backdrops and gaps at rest are as
before, the same on every build, and the target was focused in every run. On resize,
mid-motion at 61–67 ms, the first frame is drawn with ring and backdrop on all three
builds: no blank or ring-less frame. On minimize, at 62–64 ms the chrome tile on
`this` is still full size and on top (its start is later). None of it is drawn under
a neighbour.

A verbose run of `this` (`--verbose`, the bench config) drove: idle, a resize and
back, two minimize/restores, a Windows Terminal open and close, then a config reload,
and the same with opens, closes and minimizes at 1500 ms. It took single captures
mid-restore, mid-open and mid-close. Counts from `debug.log`:

- `Opening of … started`: one per open (the 6 tiles on start, each restore, each
  terminal open). No `retargeted`, no `not started again`.
- `Made a ready peek of …`: none in 10 s idle after start, none after the motions. Six
  (one per tile, within 50 ms) right after the config reload, then none again.
- `Stacking … over a tile it may cross`:
  - Minimize 2: once for the neighbour that took focus (your Windows Terminal), besides
    the minimizing tile's own. The plan says this should not happen.
  - Resize-back and the 1500 ms minimize/restore pair: also once each for a neighbour
    (File Pilot).
  - Every open, restore and close: once for the transit tile itself.
- Captures: the open zooms once inside its slot, with its ring, while its neighbours
  make room. The restore zooms in fading, the same way. The close, 450 ms into a 1.5 s
  close, shows no trace of the closing terminal, although the log has its
  `Leaving(Close)` peek alive from 20.853 to 22.403. Not compared with main; possibly
  Windows Terminal hides its own window when closing, which would leave its thumbnail
  empty.
