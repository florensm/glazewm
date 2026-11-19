# Round 3 on main: one surface at start cost 2 vs main vs restructure (2026-11-19, desktop, with DWM timing)

`this` is `tryout/one-surface` at `092d63ad`, the same code as the previous run's `this`
(`51191dff`, start cost 2), against `origin/main` `ffba51c` (one surface with start cost 3,
pop-in zoom, borders inside) and `tryout/restructure` `dee4693` (main with its files moved,
nothing else). Desktop PC: RTX 3070, 3440x1440 at 174 Hz (5.75 ms refresh), 16 threads.
Run 22:10–22:28 local:

```
powershell -ExecutionPolicy Bypass -File packages\wm-cli\examples\perf_bench\compare_branches.ps1 -Target chrome -CpuLoad 0,8 -Dwm -Launch none -TileOrder Notepad,WindowsTerminal,WindowsTerminal,chrome,chrome,FPilot -Against "origin/main;tryout/restructure" -ResultsFile <file>
```

18 restarts (CPU 0 and 8 threads x 3 rounds x 3 builds), 10 bursts per scenario,
backdrops and borders as configured. Same tiles and order as the previous run in every run,
the target (the chrome window with the lower handle) focused. No screenshots. The
`PresentMon` service captured from a non-elevated shell; GPU clocks were not locked. The
previous run is `../2026-11-17-start-cost-2/`.

- `compare-bench.md`: every run's output, then per scenario the tables (medians over
  rounds).
- `compare-bench-rows.md`: one summary row per run.

## Headline (median of 3 rounds, CPU 0 / CPU 8, ms; `was` = the previous run)

| scenario | metric | this | this, was | main | restructure | main, was (`cfd3cef`) |
| --- | --- | --- | --- | --- | --- | --- |
| move | input->first frame | 18.7 / 20.2 | 17.9 / 19.0 | 18.9 / 19.4 | 18.4 / 19.5 | 13.1 / 14.4 |
| move | frame p90 | 0.52 / 0.53 | 0.56 / 0.55 | 0.52 / 0.51 | 0.52 / 0.52 | 1.22 / 1.30 |
| move | interval p99 | 10.6 / 12.5 | 10.7 / 12.6 | 10.6 / 12.6 | 10.6 / 12.4 | 7.7 / 8.9 |
| move | dwm missed vblanks/burst | 5 / 5 | 12 / 13 | 4 / 4 | 4 / 5 | 11 / 11 |
| resize | input->first frame | 67.4 / 78.9 | 63.2 / 74.7 | 55.6 / 65.3 | 54.2 / 60.8 | 35.8 / 37.8 |
| resize | frame p90 | 3.25 / 4.19 | 3.44 / 4.34 | 3.26 / 4.26 | 3.23 / 4.20 | 4.45 / 4.69 |
| resize | interval p99 | 35.1 / 42.1 | 39.8 / 38.8 | 40.4 / 42.7 | 37.5 / 38.7 | 30.4 / 29.5 |
| resize | dwm missed vblanks/burst | 15 / 22 | 28 / 29 | 12 / 24 | 22 / 2 | 9 / 11 |
| switch | command p50 | 30.6 / 34.9 | 30.8 / 34.3 | 31.2 / 32.9 | 30.4 / 34.2 | 20.6 / 22.5 |
| switch | frame p90 | 1.19 / 1.25 | 1.09 / 1.32 | 1.16 / 1.26 | 1.19 / 1.32 | 2.53 / 3.05 |
| switch | interval p99 | 22.1 / 22.7 | 21.4 / 23.4 | 21.8 / 24.3 | 21.3 / 24.5 | 52.7 / 62.1 |
| minimize | input->first frame | 60.6 / 66.7 | 60.1 / 68.7 | 55.6 / 65.7 | 54.0 / 60.9 | 43.8 / 50.5 |
| minimize | frame p90 | 3.80 / 4.34 | 3.81 / 4.54 | 3.83 / 4.30 | 3.81 / 4.30 | 4.14 / 4.74 |
| minimize | interval p99 | 46.0 / 51.7 | 52.1 / 53.6 | 40.3 / 45.0 | 39.1 / 43.0 | 38.5 / 42.7 |
| minimize | dwm gpu busy p90 | 9.5 / 9.4 | 2.5 / 2.6 | 9.1 / 9.7 | 9.4 / 9.0 | 7.4 / 7.1 |

- `this` measured ~1 ms later on move and ~4 ms later on resize than in the previous run:
  the day's drift. Compare `main` with `this` of the same run.
- `restructure` is `main` within noise in every scenario and stage.
- Plan item "does one surface now start as fast as `main` (resize ~36, minimize ~44)": not
  yet. `main` (one surface) starts resizes 12–14 ms sooner than start cost 2, but ~20–28 ms
  after the old engine's 35.8 / 37.8; minimizes ~12–15 ms after its 43.8 / 50.5.
- Minimize's dwm gpu busy p90 is ~9.5 on every build today (2.5 for `this` before), and
  missed vblanks vary between runs (resize `restructure`: 22 at CPU 0, 2 at CPU 8). Read
  both as rough.

## Start stages, `this` -> `main` (outside frames, median, CPU 0 / CPU 8, ms)

| stage | resize | minimize |
| --- | --- | --- |
| command p50 | 54.6 -> 40.4 / 57.3 -> 44.8 | 45.5 -> 39.7 / 47.5 -> 44.5 |
| dwm_flush | 10.5 -> — / 10.0 -> — | 9.7 -> 5.8 / 13.7 -> 4.0 |
| cloak | 24.0 -> 13.1 / 23.0 -> 14.6 | 16.5 -> 13.0 / 21.6 -> 14.1 |
| peek_cover | 11.2 -> 7.9 / 12.5 -> 9.0 | 17.8 -> 13.4 / 17.3 -> 14.9 |
| ring_copy | 6.7 -> 3.5 / 7.1 -> 4.0 | 7.4 -> 4.4 / 8.2 -> 4.5 |
| native_frame | 1.6 -> 0.9 / 2.0 -> 1.4 | 1.7 -> 0.9 / 2.1 -> 1.3 |
| peek_new | 3.3 -> 3.3 / 4.1 -> 3.9 | 8.5 -> 7.7 / 10.4 -> 8.9 |
| take_over | 13.0 -> 12.6 / 13.5 -> 14.2 | 10.2 -> 12.6 / 12.2 -> 13.7 |
| to_cloak | 8.6 -> 8.7 / 9.4 -> 9.4 | 7.2 -> 9.5 / 8.7 -> 11.2 |
| peek_handover | 12.6 -> 13.3 / 14.6 -> 14.1 | 14.4 -> 14.4 / 14.9 -> 15.1 |
| ovl_restack | 2.1 -> 2.0 / 2.6 -> 2.6 | 0.0 -> 0.1 / 0.1 -> 0.2 |

Ready peeks: 6 hits, 0 misses per burst on resize and minimize, 2 and 0 on move, in every
build. Round 3's proposal 1 (no flush for a pass of ready peeks) shows on resize; minimize
keeps one flush (its neighbours', proposal 4). Move keeps a ~5 ms `dwm_flush` on every build,
as the old `main` did (4.8 / 5.4). Switch starts with `ws_surrogates` ~9.8 and `ws_show`
~8 on every build, as before.

What is left of the start, by size: `take_over` 12.6–14.2 (of it `to_cloak` 8.7–11.2),
`peek_handover` 13–15, `peek_cover` 8–15, `peek_new` 7.7–8.9 on minimize, ring and backdrop
parking (`ring_park` ~1.2, `ovl_restack` 2.0–2.6 on resize: proposal 2).
