# Start-latency stages bench, 2026-11-13 afternoon (desktop PC)

Second bench of the day (the morning's is `../2026-11-13/`). A measuring
run: it reads the 16 stages of `START_LATENCY_PLAN.md`'s "Stages, second
round". Start with `bench-report.md`.

Builds: `tryout/start-latency` `479b7482` vs `origin/main` `cfd3cef9`
(`main`). Ryzen 7 3700X, RTX 3070, 3440×1440 @ 174 Hz. **Backdrops off** in
the user config (`window_effects.yaml`, both `backdrop.enabled: false`) and
in `baseline-config.yaml`, at the owner's request; the morning ran with them
on.

## Files

- `bench-report.md`: the report.
- `perf-excerpts.txt`: two full profiler reports, `default`, `resize`,
  load 0: the burst with the median interval max of all 40, and the one with
  the longest.
- `results/`
  - `<run>.txt`: perf_bench output per run: `resize-*` (`nopeeks`,
    `nopeeks+nokeepoverlays`, `default`, `noasynchandback`), `switch-*`
    (`nopeeks`, `default`, `noslidepeeks`), `main-*` (`main`, `nopeeks`,
    `default` on `resize` and `switch`); `load0`, `cpu8`.
  - `log-<run>.txt`: timestamped console output, which `stages.py`,
    `reports.py` and `errlines.py` use to map `perf.log` reports to runs.
    `log-smoke-iso.txt`, `log-smoke-main.txt`: the smoke runs;
    `log-smoke-zedlock.txt`: the first one, stopped by Zed's file handles.
  - `marks.txt`: run start/end times, errors.log and perf.log offsets.
  - `anydesk/`: the first `main-load0` and three CPU 8 runs, void: an
    AnyDesk window joined workspace 1 during them (see the report). Rerun
    with it moved to workspace 3.
  - `tables.md`, `cmp-*.md`, `stages-*.md`, `sync-rows.md`, `errors.md`:
    the analysis output the report quotes.
- Scripts, as run:
  - `bench-chain.sh`: the runs in order (all six, or those named).
  - `bench-run.ps1`, `baseline-config.yaml`: the `main` runner of the
    morning with `-Launch` added; the morning's config with backdrops off.
  - `stages.py`: the new stages: before the answer / in the first
    `peek_frame`, `peek_new`'s children, parent-vs-children gaps, switch per
    burst, full reports.
  - `tables.py`: one table per scenario from perf_bench output.
  - `errlines.py`: errors.log warnings per mode.
  - `reports.py`, `compare.py`, `syncrows.py`: the morning's, unchanged.

Not committed, on the desktop in `~/.glzr/glazewm/bench-2026-11-13-stages/`:
`logs/perf-<run>.log` (one per run).

Desktop only: these restart the WM and drive bursts over IPC.
