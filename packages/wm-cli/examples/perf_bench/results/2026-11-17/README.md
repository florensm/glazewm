# One surface vs stand-ins modes vs main (2026-11-17, desktop)

`tryout/one-surface` (`this`, WM code of `f17e50c1`) against `origin/tryout/stand-ins`
`65a60de` in `nopeeks` (old `ResizeSession` engine), default (peeks) and `morepeeks`,
and `origin/main` `cfd3cef`. Desktop PC: RTX 3070, 3440x1440 at 174 Hz (5.75 ms
refresh), 16 threads. "To measure (desktop)" run 2 in `ONE_SURFACE_PLAN.md`, with
screenshots:

```
powershell -ExecutionPolicy Bypass -File packages\wm-cli\examples\perf_bench\compare_branches.ps1 -Target chrome -CpuLoad 0,8 -Screenshots -TileOrder Notepad,WindowsTerminal,WindowsTerminal,chrome,chrome,FPilot
```

30 restarts (CPU 0 and 8 threads x 3 rounds x 5 builds/modes), 10 bursts per scenario,
backdrops and borders as configured, no `--dwm`. 10:00 to 10:29. All 30 runs had the
same tile order. The script ran from local branch `os/21-bench-fixes` (`38350415`,
not pushed): `tryout/one-surface` plus four bench-only fixes, needed before it would
run here. `-CpuLoad 0` alone was rejected, `Wait-WmReady` hung on a query sent during
WM startup, and `origin/main` couldn't parse the `focus --special-workspace` keybinding,
so config copies now have no keybindings. The fourth makes `Stop-AllWm` kill a stuck
WM.

- `compare-bench.md`: every run's output, then per scenario the tables (medians over
  rounds).
- `compare-bench-rows.md`: one summary row per run.
Screenshots were checked but not committed (they showed private browser tabs).

## Headline (median of 3 rounds, CPU 0 / CPU 8)

| scenario | metric | this | stand-ins:nopeeks | stand-ins | stand-ins:morepeeks | main |
| --- | --- | --- | --- | --- | --- | --- |
| move | input->first frame ms | 17.4 / 18.7 | 16.5 / 17.2 | — | — | 14.8 / 15.5 |
| move | command p50 ms | 16.8 / 17.5 | 16.1 / 16.6 | 19.5 / 21.0 | 20.0 / 22.1 | 13.9 / 14.9 |
| move | frame p90 ms | **0.57 / 0.56** | 0.93 / 1.04 | 0.93 / 1.08 | 0.96 / 1.08 | 0.99 / 1.15 |
| move | interval p99 ms | 10.0 / 12.2 | 7.4 / 8.3 | 10.8 / 13.1 | 11.1 / 13.0 | 7.1 / 8.1 |
| move | peek_new / session_begin ms | 2.0 / 2.2 | 0.9 / 1.1 | — | — | 0.9 / 1.0 |
| resize | input->first frame ms | 64.0 / 71.4 | 44.4 / 48.9 | — | — | 35.3 / 36.9 |
| resize | command p50 ms | 47.8 / 52.6 | 40.2 / 44.2 | 62.6 / 69.4 | 65.0 / 72.3 | 29.9 / 32.1 |
| resize | frame p90 ms | **3.31 / 4.17** | 4.19 / 4.87 | 3.54 / 4.69 | 3.44 / 4.36 | 3.98 / 4.36 |
| resize | interval p90 ms | 5.77 / 5.78 | 5.78 / 5.79 | 5.85 / 6.70 | 5.81 / 6.44 | 5.77 / 5.76 |
| resize | interval p99 ms | 37.1 / 38.4 | 23.1 / 17.0 | 73.1 / 60.5 | 67.7 / 64.6 | 24.9 / 26.9 |
| resize | peek_new / session_begin ms | 3.5 / 4.4 | 2.2 / 2.6 | — | — | 2.2 / 2.7 |
| switch | command p50 ms | 25.6 / 28.8 | 29.7 / 31.5 | 30.3 / 32.4 | 30.0 / 32.5 | 20.3 / 21.2 |
| switch | frame p90 ms | **1.06 / 1.22** | 1.19 / 1.47 | 1.23 / 1.50 | 1.21 / 1.49 | 2.41 / 2.76 |
| switch | interval p99 ms | **19.7 / 22.7** | 21.7 / 24.4 | 23.6 / 26.9 | 23.6 / 26.6 | 50.8 / 56.4 |
| minimize | input->first frame ms | 69.5 / 81.4 | 52.6 / 58.3 | — | — | 43.3 / 46.4 |
| minimize | command p50 ms | 50.5 / 58.8 | 23.2 / 26.7 | 58.0 / 66.6 | 58.5 / 65.7 | 16.3 / 18.6 |
| minimize | frame p90 ms | 3.49 / 4.15 | 2.74 / 3.28 | 3.81 / 4.96 | 4.10 / 4.82 | 3.83 / 4.30 |
| minimize | interval p90 ms | 5.77 / 5.78 | 5.76 / 5.77 | 7.22 / 7.47 | 7.12 / 7.97 | 5.74 / 5.77 |
| minimize | interval p99 ms | 58.9 / 53.6 | 45.7 / 48.0 | 72.4 / 81.0 | 73.4 / 82.2 | 34.5 / 35.5 |
| minimize | peek_new / session_begin ms | 12.9 / 8.4 | 2.2 / 2.5 | — | — | 2.3 / 2.3 |
| minimize | restack ms | 9.2 / 9.6 | — | — | — | — |

Interval p90 is at the refresh period (5.73–5.79 ms) for every build in move and switch.
Rounds agree closely: per build and setting, frame p90 stays within about 1 ms and
command p50 within about 5 ms across the three rounds (`compare-bench-rows.md`).

## Reading

- **Against stand-ins' peek modes**, one surface is faster everywhere measured: lower
  command time, frame p90 and interval p99 in all four scenarios, and no interval p90
  creep under load in resize and minimize (5.78 vs 6.4–8.0 ms).
- **Against main and the old engine (`nopeeks`)**, one surface is cheaper per frame in
  move (about 40% lower frame p90), resize and switch, and the smoothest switch of all
  (p99 19.7 vs 50.8 ms on main). It is slower to start, though: input->first frame
  +1–3 ms on move, +20–35 ms on resize and +17–35 ms on minimize. `peek_new` costs
  about 2x `session_begin`, and minimize adds `peek_new` 8–13 ms plus `restack` about
  9 ms. Interval p99 is also worse than main/`nopeeks` on move, resize and minimize,
  so the start produces longer frame gaps.

## Caveats

- `input->first frame` is not recorded on stand-ins' peek path (`—`), and on switch it
  is `—` on `this` and 0 on the others; command p50 stands in for both.
- Minimize: `nopeeks` and `main` run 144–150 frames per run against about 60 for the
  peek builds, so their per-frame numbers aren't like for like.
- CPU 0, round 1, `this` (the run's first restart): focus was still on File Pilot from
  the launch, so `move` and `resize` worked on an unfocused window. In every other run,
  every build had the target focused (checked by border color in the shots). That
  round's numbers are in line with rounds 2–3.

## Screenshots

Nothing wrong was seen in any of the 80 shots: borders stay on their windows,
backdrops are in place, nothing is left on screen, and gaps are even at rest. On
`this` the border sits inside the window (rows 20–21), while every other build draws it
outside (rows 18–19), as decided.

- `this-move-mid`: chrome sliding over its neighbor, focused border following it.
- `this-resize-mid`: chrome growing, neighbors making room.
- `this-switch-mid`: horizontal slide, the outgoing workspace leaving left with a
  40 px gap to the incoming one. Notepad's dark right part is Notepad not having
  repainted after its tile widened; stand-ins shows the same.
- `this-minimize-mid` vs `main-default-minimize-mid` (64 vs 62 ms after the command):
  main's tiles have already moved about 110 px, while one surface's haven't started.
  That is the slower start from the table, seen on screen.
