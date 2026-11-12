You're on my desktop PC in the GlazeWM repo (C:\Users\googlers\glazewm). Read CLAUDE.md first and follow it. This is the tryout machine: benchmarks are OK, but no input synthesis, no screenshot loops, no dummy windows. I'm away; run unattended.

A cloud session pushed `tryout/start-latency`. Its changes cut the start delay of the peek drawing, each behind its own off-by-default `GLAZEWM_ISOLATE` word. Your job is to measure them against last night's baseline, then cut the laptop test build.

1. Read `C:\Users\googlers\.glzr\glazewm\bench-2026-11-12\README.md` and `bench-report.md` ("Bottom line", "Verdict"). Then read `START_LATENCY_PLAN.md` on the branch, especially what it says to measure and which `GLAZEWM_ISOLATE` words it added.
2. Make sure the working tree is clean. If it isn't, stop and tell me. Then `git fetch origin`, `git checkout tryout/start-latency`, `git pull`, `cargo build --release -p wm`, `cargo build --release -p wm-cli --example perf_bench` and `cargo test --workspace`. If something fails, report it and stop.
3. If GlazeWM isn't running, start `target\release\glazewm.exe start` (no env vars). List the tiled windows. Last night's set was WindowsTerminal, chrome, Notepad, Everything, explorer and 2× ApplicationFrameHost (Settings, Store). Note any difference in the report.
4. Probe first: one round, `move` only, 2 bursts, with the bench folder's `bench-run.ps1`. Check that every mode produces reports. Never TaskStop a running bench, because that kills the WM.
5. Bench with `bench-run.ps1` from the bench folder:
   - Use `-TileOrder WindowsTerminal,Notepad,chrome,Everything,explorer,ApplicationFrameHost,ApplicationFrameHost`.
   - Modes: `nopeeks`, `default`, then `default+<word>` for each new word, plus any combination the plan suggests. Add `morepeeks` and `morepeeks+<word>` if a change targets `float`.
   - 4 rounds, all scenarios, once at load 0 and once with `-CpuLoad 8`.
   - Use an odd number of modes, or check the logged tile order: it must be identical on every restart.
6. Compare the new runs against last night's `results/fixed-load0.txt` and `results/fixed-cpu8.txt` with `summarize.py` and `tails.py`. Same-day comparisons (`default` vs `default+word` in one run) count most; cross-day ones only as a sanity check.
7. Write a dated report folder next to last night's (`bench-YYYY-MM-DD`) with the same layout:
   - Per word: does it cut command p50, and does it keep interval p90, frame p90 and the longest gap?
   - Beyond noise or not.
   - Which words to keep.
   - Include `perf.log`/`errors.log` copies and the raw results.
8. Restart GlazeWM as it was (no `GLAZEWM_PERF`, no `GLAZEWM_ISOLATE`) and restore the layout.
9. Cut the laptop test build as CLAUDE.md describes:
   - `test/start-latency` from `origin/tryout/start-latency`, with the strip commit.
   - `cargo build --release && cargo test --workspace`.
   - Push it with `--force-with-lease`.
   - List any new Windows API calls the branch added, so a Defender quarantine can be traced.

No other commits or pushes. If anything fails, note it in the report, skip that part, and still restore my WM at the end.
