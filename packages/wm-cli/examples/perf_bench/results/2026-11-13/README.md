# Start-latency words bench, 2026-11-13 (desktop PC)

Start with `bench-report.md`: verdict per word, setup, the separated
numbers, the `perf.log` answers and what was seen on screen.

Builds: `tryout/start-latency` `c1470413` (words via `GLAZEWM_ISOLATE`) vs
`origin/main` `cfd3cef9` (`main`). Ryzen 7 3700X, RTX 3070, 3440×1440 @
174 Hz, 6 tiled windows.

## Files

- `bench-report.md`: the report.
- `perf-excerpts.txt`: one full profiler report per mode, `resize`, round 1,
  load 0: the burst with the longest frame interval max.
- `switch-excerpts-2026-11-12.txt`, `switch-perburst-2026-11-12.md`: `main`
  vs `nopeeks` switch reports and per-burst stage totals, read from
  2026-11-12's `perf.log` (no new run).
- `results/`
  - `peek-*.txt`, `more-*.txt`, `main-*.txt`: perf_bench output per run
    (`load0`, `cpu8`): peek words, `morepeeks` words, and `main,nopeeks,default`.
  - `log-*.txt`: timestamped console output of each run, which `reports.py`
    uses to map `perf.log` reports to runs. `log-smoke*.txt`: the two
    `isolate_modes.ps1` smoke runs.
  - `cmp-*.md`: each word vs its baseline, load 0 and CPU 8, with round
    ranges and whether they separate (`compare.py`).
  - `stage-medians.md`: per-report medians of the outside-frames stages,
    first frame, interval tails, worst `[sync]` row (`reports.py medians`).
  - `sync-rows.md`: `[sync]` rows of `rd_apply by process` per mode
    (`syncrows.py`).
  - `marks.txt`: run start/end times and errors.log offsets.
- `checks/`: the per-word manual checks (`wordcheck.ps1`): commands, their
  answer times, final window rects, new errors.log lines. `visual-nopeeks*`
  are void (taken while the post-restart reorder was still moving);
  `visual2-*` are their retakes.
- Scripts, as run:
  - `bench-chain.sh`: the six runs in order. Writes to the session
    scratchpad (`$S/bench`); copied here afterwards.
  - `bench-run.ps1`, `baseline-config.yaml`: the `main` runner from
    2026-11-12, with the `Wait-WmReady` fix (see the report).
  - `reports.py`, `syncrows.py`, `compare.py`: the analysis.
  - `wordcheck.ps1`: restarts the WM with one word, sends a keybound
    resize and a move over IPC, takes one screenshot (settled, or timed on
    a slow-motion config copy).

Not committed, on the desktop in `~/.glzr/glazewm/bench-2026-11-13/`:
`logs/perf-<run>.log` (one per run, 13 MB in all), `screens/` (the
screenshots show this desktop's windows).

Desktop only: these restart the WM and drive bursts over IPC.
