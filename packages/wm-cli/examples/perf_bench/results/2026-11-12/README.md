# Peek drawing bench, 2026-11-12 (desktop PC)

Start with `bench-report.md`, which has the bottom line, setup,
deviations, all tables, the verdict and the raw results.

Builds: `tryout/stand-ins` `65a60de3` (modes `nopeeks`, `default`,
`morepeeks` via `GLAZEWM_ISOLATE`) vs `origin/main` `cfd3cef9` (`main`).
Desktop: Ryzen 7 3700X, RTX 3070, 3440×1440 @ 174 Hz, 7 tiled windows.

## Files

- `bench-report.md`: the full report.
- `perf-excerpts.txt`: full profiler reports from `perf.log` for one
  `resize` burst per build (round 1, fixed tile order, no CPU load), the
  burst with the longest frame gap. The whole `perf.log` (19 MB) and
  `errors.log` aren't committed; they're on the desktop in
  `~/.glzr/glazewm/bench-2026-11-12/logs/`.
- `results/`
  - `load0-isolate.txt`, `cpu8-isolate.txt`: `isolate_modes.ps1` output,
    tryout modes, 3 rounds.
  - `load0-main.txt`, `cpu8-main.txt`: `main`, 3 rounds, same scenarios.
  - `fixed-load0.txt`, `fixed-cpu8.txt`: the controlled runs. All four
    builds interleaved, fixed tile order, 4 rounds. The verdict leans on
    these.
  - `parity-load0.txt`: the first interleaved run. Its tile order flipped
    per restart, which locked each mode to one order.
  - `*-summary.md`: the tables built from the above.
  - `log-*.txt`: timestamped console output of each run. `tails.py` uses
    them to map `perf.log` reports to runs.
  - `isolate-bench.prev-2026-11-12T2048.txt`: an earlier, shorter
    `nopeeks`/`default` run.
- Scripts, as they ran that night. They're written for that desktop and
  hardcode `C:\Users\googlers\glazewm` and `..\glazewm-baseline`.
  - `bench-run.ps1`: `isolate_modes.ps1`'s loop with `main` as a mode,
    failures recorded and skipped, tile order logged, and `-TileOrder`
    to fix the order after each restart.
  - `summarize.py`: result files to per-scenario median tables and
    comparisons.
  - `tails.py`: interval p99/max per mode and scenario from `perf.log`
    (`logs/perf.log` next to it, else the live one).
  - `build-report.sh`: assembles `bench-report.md` from `report-head.md`,
    `fixed-intro.md`, `report-verdict.md`, `parity-intro.md` and the
    results.
  - `baseline-config.yaml`: the user config translated to a single file
    for `origin/main`, which has no `include:`.
- Prompts for the follow-up:
  - `cloud-prompt.md`: plan and implement the start-latency fixes.
  - `desktop-bench-prompt.md`: measure them on the desktop.
  - `tomorrow-prompt.md`: a single-session version of both.

## Re-running `main`

The `main` mode needs a baseline worktree:

1. `git worktree add --detach ..\glazewm-baseline origin/main`.
2. Copy this branch's `packages\wm-cli\examples\perf_bench\*.rs` into it.
3. Add `serde_yaml = "0.9"` under `wm-cli`'s `[dev-dependencies]`.
4. `cargo build --release -p wm -p wm-watcher` and
   `cargo build --release -p wm-cli --example perf_bench` in it.

Desktop only: the bench restarts the WM and drives bursts over IPC.
